//! Ely.by's public device-code OAuth flow, as used by FreesmLauncher.
//! Passwords and OAuth client secrets never enter the launcher.
use chrono::{TimeDelta, Utc};
use reqwest::Client;
use serde::Deserialize;
use uuid::Uuid;

use crate::data::{AccountKind, DeviceCodeLogin, ElybyLoginSession, MinecraftAccount};

pub const DEFAULT_ELYBY_CLIENT_ID: &str = match option_env!("ONEFORALL_ELYBY_CLIENT_ID") {
    Some(id) => id,
    None => "oneforalllauncher",
};
pub const ELYBY_AUTHLIB_URL: &str = "https://account.ely.by/api/authlib-injector";
const API: &str = "https://account.ely.by/api";
const SCOPE: &str = "account_info offline_access minecraft_server_session";

#[derive(Debug, thiserror::Error)]
pub enum ElybyAuthError {
    #[error("Ely.by connection failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Ely.by rejected the request ({code}, HTTP {status}).")]
    Rejected { code: String, status: u16 },
    #[error("Ely.by returned an invalid {0} response. Please try again.")]
    InvalidResponse(&'static str),
    #[error("Ely.by sign-in expired. Start sign-in again.")]
    Expired,
    #[error("Ely.by sign-in was denied in the browser.")]
    Denied,
    #[error("This Ely.by account needs to be signed in again.")]
    Reauthenticate,
}

impl ElybyAuthError {
    pub(crate) fn is_transient(&self) -> bool {
        match self {
            Self::Network(error) => error.is_timeout() || error.is_connect(),
            Self::Rejected { status, .. } => *status >= 500 || *status == 429,
            _ => false,
        }
    }
}

fn interval_default() -> u64 {
    5
}

#[derive(Deserialize)]
struct DeviceResponse {
    user_code: String,
    device_code: String,
    #[serde(alias = "verification_url")]
    verification_uri: String,
    expires_in: u64,
    #[serde(default = "interval_default")]
    interval: u64,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
}

#[derive(Deserialize)]
struct ProfileResponse {
    id: String,
    name: String,
}

async fn json(
    response: reqwest::Response,
    stage: &'static str,
) -> Result<serde_json::Value, ElybyAuthError> {
    let status = response.status();
    let body = response
        .json::<serde_json::Value>()
        .await
        .map_err(|_| ElybyAuthError::InvalidResponse(stage))?;
    if !status.is_success() {
        let code = body
            .get("error")
            .and_then(|value| value.as_str())
            .filter(|code| {
                code.len() <= 80
                    && code
                        .chars()
                        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
            })
            .unwrap_or("service_error")
            .to_string();
        return Err(ElybyAuthError::Rejected {
            code,
            status: status.as_u16(),
        });
    }
    Ok(body)
}

pub(crate) async fn begin(
    client: &Client,
    client_id: String,
) -> Result<ElybyLoginSession, ElybyAuthError> {
    begin_at(client, client_id, API).await
}

async fn begin_at(
    client: &Client,
    client_id: String,
    api: &str,
) -> Result<ElybyLoginSession, ElybyAuthError> {
    if client_id.trim().is_empty() {
        return Err(ElybyAuthError::Reauthenticate);
    }
    let response = client
        .post(format!("{api}/oauth2/v1/devicecode"))
        .form(&[("client_id", client_id.as_str()), ("scope", SCOPE)])
        .send()
        .await?;
    let device: DeviceResponse = serde_json::from_value(json(response, "device-code").await?)
        .map_err(|_| ElybyAuthError::InvalidResponse("device-code"))?;
    let mut uri = url::Url::parse(&device.verification_uri)
        .map_err(|_| ElybyAuthError::InvalidResponse("verification URL"))?;
    // Ely.by currently advertises http://account.ely.by/code; always open its HTTPS equivalent.
    if uri.scheme() == "http"
        && matches!(uri.host_str(), Some("account.ely.by" | "ely.by"))
        && uri.port().is_none()
    {
        uri.set_scheme("https")
            .map_err(|_| ElybyAuthError::InvalidResponse("verification URL"))?;
    }
    if uri.scheme() != "https"
        || !uri.username().is_empty()
        || uri.password().is_some()
        || uri.port().is_some()
        || !matches!(uri.host_str(), Some("account.ely.by" | "ely.by"))
        || device.device_code.is_empty()
        || device.user_code.is_empty()
        || device.expires_in == 0
    {
        return Err(ElybyAuthError::InvalidResponse("device-code"));
    }
    Ok(ElybyLoginSession {
        client_id,
        device: DeviceCodeLogin {
            user_code: device.user_code,
            device_code: device.device_code,
            verification_uri: uri.to_string(),
            expires_in: device.expires_in.min(3600),
            interval: device.interval.clamp(1, 60),
            message: "Sign in at Ely.by in your browser and authorize this launcher.".into(),
        },
    })
}

pub(crate) async fn finish(
    client: &Client,
    session: &ElybyLoginSession,
    progress: impl Fn(&str),
) -> Result<MinecraftAccount, ElybyAuthError> {
    finish_at(client, session, API, progress).await
}

async fn finish_at(
    client: &Client,
    session: &ElybyLoginSession,
    api: &str,
    progress: impl Fn(&str),
) -> Result<MinecraftAccount, ElybyAuthError> {
    let deadline =
        tokio::time::Instant::now() + std::time::Duration::from_secs(session.device.expires_in);
    let mut interval = session.device.interval.clamp(1, 60);
    progress("Waiting for Ely.by browser authorization");
    loop {
        tokio::time::sleep_until(
            (tokio::time::Instant::now() + std::time::Duration::from_secs(interval)).min(deadline),
        )
        .await;
        if tokio::time::Instant::now() >= deadline {
            return Err(ElybyAuthError::Expired);
        }
        let response = client
            .post(format!("{api}/oauth2/v1/token"))
            .form(&[
                ("client_id", session.client_id.as_str()),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("device_code", session.device.device_code.as_str()),
            ])
            .timeout(deadline.saturating_duration_since(tokio::time::Instant::now()))
            .send()
            .await?;
        match json(response, "token").await {
            Ok(value) => {
                let token = serde_json::from_value(value)
                    .map_err(|_| ElybyAuthError::InvalidResponse("token"))?;
                progress("Loading Ely.by Minecraft profile");
                return account(client, api, &session.client_id, token, None).await;
            }
            Err(ElybyAuthError::Rejected { code, .. }) if code == "authorization_pending" => {}
            Err(ElybyAuthError::Rejected { code, .. }) if code == "slow_down" => {
                interval = interval.saturating_add(5).min(60);
            }
            Err(ElybyAuthError::Rejected { code, .. }) if code == "access_denied" => {
                return Err(ElybyAuthError::Denied);
            }
            Err(ElybyAuthError::Rejected { code, .. }) if code == "expired_token" => {
                return Err(ElybyAuthError::Expired);
            }
            Err(error) => return Err(error),
        }
    }
}

async fn account(
    client: &Client,
    api: &str,
    client_id: &str,
    token: TokenResponse,
    old_refresh: Option<&str>,
) -> Result<MinecraftAccount, ElybyAuthError> {
    let expires_in = i64::try_from(token.expires_in)
        .ok()
        .filter(|expires| *expires > 0 && *expires <= 31_536_000)
        .ok_or(ElybyAuthError::InvalidResponse("token expiry"))?;
    if token.access_token.is_empty() {
        return Err(ElybyAuthError::InvalidResponse("token"));
    }
    let response = client
        .get(format!("{api}/mojang/services/minecraft/profile"))
        .bearer_auth(&token.access_token)
        .send()
        .await?;
    let profile: ProfileResponse =
        serde_json::from_value(json(response, "Minecraft profile").await?)
            .map_err(|_| ElybyAuthError::InvalidResponse("Minecraft profile"))?;
    let id = Uuid::parse_str(&profile.id)
        .map_err(|_| ElybyAuthError::InvalidResponse("profile UUID"))?;
    if id.is_nil() || profile.name.is_empty() {
        return Err(ElybyAuthError::InvalidResponse("Minecraft profile"));
    }
    let refresh_token = token
        .refresh_token
        .filter(|token| !token.is_empty())
        .or_else(|| old_refresh.map(str::to_owned))
        .filter(|token| !token.is_empty())
        .ok_or(ElybyAuthError::Reauthenticate)?;
    Ok(MinecraftAccount {
        id,
        username: profile.name,
        access_token: token.access_token,
        refresh_token,
        expires: Utc::now() + TimeDelta::seconds(expires_in),
        kind: AccountKind::Elyby,
        elyby_client_id: Some(client_id.to_owned()),
    })
}

pub(crate) async fn refresh(
    client: &Client,
    existing: &MinecraftAccount,
) -> Result<MinecraftAccount, ElybyAuthError> {
    refresh_at(client, existing, API).await
}

async fn refresh_at(
    client: &Client,
    existing: &MinecraftAccount,
    api: &str,
) -> Result<MinecraftAccount, ElybyAuthError> {
    let client_id = existing
        .elyby_client_id
        .as_deref()
        .filter(|id| !id.is_empty())
        .ok_or(ElybyAuthError::Reauthenticate)?;
    if existing.refresh_token.is_empty() {
        return Err(ElybyAuthError::Reauthenticate);
    }
    let response = client
        .post(format!("{api}/oauth2/v1/token"))
        .form(&[
            ("client_id", client_id),
            ("grant_type", "refresh_token"),
            ("refresh_token", existing.refresh_token.as_str()),
        ])
        .send()
        .await?;
    let token = serde_json::from_value(json(response, "refresh token").await?)
        .map_err(|_| ElybyAuthError::InvalidResponse("refresh token"))?;
    let refreshed = account(client, api, client_id, token, Some(&existing.refresh_token)).await?;
    if refreshed.id != existing.id {
        return Err(ElybyAuthError::InvalidResponse(
            "refreshed profile identity",
        ));
    }
    Ok(refreshed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    const ID: &str = "6d7d9f2e-8a9c-4f80-9a12-b99d8cc92c35";

    async fn server(
        replies: Vec<(u16, serde_json::Value, Vec<&'static str>)>,
    ) -> (String, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            for (status, body, expected) in replies {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut request = Vec::new();
                loop {
                    let mut buffer = [0; 4096];
                    let read = stream.read(&mut buffer).await.unwrap();
                    assert!(read > 0);
                    request.extend_from_slice(&buffer[..read]);
                    if let Some(header_end) =
                        request.windows(4).position(|window| window == b"\r\n\r\n")
                    {
                        let headers =
                            String::from_utf8_lossy(&request[..header_end]).to_lowercase();
                        let length = headers
                            .lines()
                            .find_map(|line| {
                                line.strip_prefix("content-length: ")
                                    .and_then(|length| length.parse::<usize>().ok())
                            })
                            .unwrap_or(0);
                        if request.len() >= header_end + 4 + length {
                            break;
                        }
                    }
                }
                let request = String::from_utf8(request).unwrap();
                for expected in expected {
                    assert!(
                        request.contains(expected),
                        "Missing expected request field {expected}"
                    );
                }
                assert!(!request.contains("client_secret"));
                let body = body.to_string();
                stream.write_all(format!("HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).as_bytes()).await.unwrap();
            }
        });
        (url, task)
    }
    fn device(uri: &str) -> serde_json::Value {
        json!({"user_code":"USER-CODE", "device_code":"private-device", "verification_uri":uri, "expires_in":600,"interval":1})
    }
    fn token(refresh: Option<&str>) -> serde_json::Value {
        json!({"access_token":"test-access", "refresh_token":refresh, "expires_in":3600})
    }
    fn profile(id: &str) -> serde_json::Value {
        json!({"id":id,"name":"ElyPlayer"})
    }

    #[tokio::test]
    async fn device_login_uses_public_client_scopes_and_ely_profile() {
        let (api, task) = server(vec![
            (
                200,
                device("http://account.ely.by/code"),
                vec![
                    "POST /oauth2/v1/devicecode",
                    "client_id=oneforalllauncher",
                    "scope=account_info+offline_access+minecraft_server_session",
                ],
            ),
            (
                400,
                json!({"error":"authorization_pending"}),
                vec![
                    "device_code=private-device",
                    "grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Adevice_code",
                ],
            ),
            (
                200,
                token(Some("test-refresh")),
                vec!["POST /oauth2/v1/token"],
            ),
            (
                200,
                profile(ID),
                vec![
                    "GET /mojang/services/minecraft/profile",
                    "Bearer test-access",
                ],
            ),
        ])
        .await;
        let client = Client::new();
        let session = begin_at(&client, "oneforalllauncher".into(), &api)
            .await
            .unwrap();
        assert_eq!(
            session.device.verification_uri,
            "https://account.ely.by/code"
        );
        assert!(!format!("{session:?}").contains("private-device"));
        let account = finish_at(&client, &session, &api, |_| {}).await.unwrap();
        assert!(account.is_elyby());
        assert_eq!(account.id, Uuid::parse_str(ID).unwrap());
        assert_eq!(
            account.elyby_client_id.as_deref(),
            Some("oneforalllauncher")
        );
        assert_eq!(account.skin_profile_key(), "elyby:ElyPlayer");
        assert!(!account.is_expired());
        task.await.unwrap();
    }
    #[tokio::test]
    async fn refresh_retains_or_rotates_refresh_token_and_rejects_identity_changes() {
        let (api, task) = server(vec![
            (
                200,
                token(None),
                vec![
                    "grant_type=refresh_token",
                    "refresh_token=old-refresh",
                    "client_id=oneforalllauncher",
                ],
            ),
            (200, profile(ID), vec!["Bearer test-access"]),
            (
                200,
                token(Some("new-refresh")),
                vec!["refresh_token=old-refresh"],
            ),
            (200, profile(ID), vec![]),
            (
                200,
                token(Some("another-refresh")),
                vec!["refresh_token=new-refresh"],
            ),
            (200, profile("dd6d4633-b78c-41d7-bc0b-f67d3e0a1d82"), vec![]),
        ])
        .await;
        let existing = MinecraftAccount {
            id: Uuid::parse_str(ID).unwrap(),
            username: "ElyPlayer".into(),
            access_token: "old-access".into(),
            refresh_token: "old-refresh".into(),
            expires: Utc::now(),
            kind: AccountKind::Elyby,
            elyby_client_id: Some("oneforalllauncher".into()),
        };
        let client = Client::new();
        let retained = refresh_at(&client, &existing, &api).await.unwrap();
        assert_eq!(retained.refresh_token, "old-refresh");
        let rotated = refresh_at(&client, &retained, &api).await.unwrap();
        assert_eq!(rotated.refresh_token, "new-refresh");
        assert!(matches!(
            refresh_at(&client, &rotated, &api).await,
            Err(ElybyAuthError::InvalidResponse(
                "refreshed profile identity"
            ))
        ));
        task.await.unwrap();
    }
    #[tokio::test]
    async fn verification_urls_and_server_errors_do_not_leak_or_redirect_credentials() {
        for uri in [
            "https://evil.example/code",
            "https://account.ely.by@evil.example/",
            "https://secret@account.ely.by/code",
        ] {
            let (api, task) = server(vec![(200, device(uri), vec![])]).await;
            assert!(matches!(
                begin_at(&Client::new(), "oneforalllauncher".into(), &api).await,
                Err(ElybyAuthError::InvalidResponse(_))
            ));
            task.await.unwrap();
        }
        let (api, task) = server(vec![(
            400,
            json!({"error":"private-device access-token","error_description":"private-device"}),
            vec![],
        )])
        .await;
        let error = begin_at(&Client::new(), "oneforalllauncher".into(), &api)
            .await
            .unwrap_err();
        assert!(!format!("{error:?}").contains("private-device"));
        task.await.unwrap();
    }
    #[tokio::test]
    async fn denied_and_expired_device_sessions_stop_polling() {
        let (api, task) = server(vec![(400, json!({"error":"access_denied"}), vec![])]).await;
        let session = ElybyLoginSession {
            client_id: "oneforalllauncher".into(),
            device: DeviceCodeLogin {
                user_code: "code".into(),
                device_code: "private-device".into(),
                verification_uri: "https://account.ely.by/code".into(),
                expires_in: 10,
                interval: 1,
                message: String::new(),
            },
        };
        assert!(matches!(
            finish_at(&Client::new(), &session, &api, |_| {}).await,
            Err(ElybyAuthError::Denied)
        ));
        task.await.unwrap();
        let mut expired = session;
        expired.device.expires_in = 0;
        assert!(matches!(
            finish_at(&Client::new(), &expired, &api, |_| {}).await,
            Err(ElybyAuthError::Expired)
        ));
    }
    #[test]
    fn older_accounts_deserialize_without_elyby_metadata() {
        let account: MinecraftAccount = serde_json::from_value(json!({"id":ID, "username":"Existing", "access_token":"", "refresh_token":"", "expires":Utc::now()})).unwrap();
        assert!(account.is_microsoft());
        assert_eq!(account.elyby_client_id, None);
    }
}
