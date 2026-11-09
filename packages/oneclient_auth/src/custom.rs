//! Yggdrasil accounts, following FreesmLauncher's CustomAuth/RefreshStep protocol.
use crate::{AccountKind, MinecraftAccount};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

/// Freesm fills empty paths on a server-address edit, without replacing user overrides.
pub fn custom_auth_paths_on_server_change(
    login: &str,
    refresh: &str,
) -> (Option<String>, Option<String>) {
    (
        login.is_empty().then(|| "/authserver/authenticate".into()),
        refresh.is_empty().then(|| "/authserver/refresh".into()),
    )
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomServer {
    pub api_url: String,
    pub login_path: String,
    pub refresh_path: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CustomAccountData {
    pub server: CustomServer,
    pub client_token: String,
}

impl std::fmt::Debug for CustomAccountData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomAccountData")
            .field("server", &self.server)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomProfile {
    pub id: Uuid,
    pub name: String,
}

// No Debug/Serialize: tokens are never displayed, logged or used as query keys.
#[derive(Clone)]
pub struct CustomLoginResponse {
    pub server: CustomServer,
    pub profiles: Vec<CustomProfile>,
    access_token: String,
    client_token: String,
    selected: Option<Uuid>,
}

#[derive(Debug, thiserror::Error)]
pub enum CustomAuthError {
    #[error("Custom authentication URL is invalid: {0}")]
    InvalidUrl(String),
    #[error("Custom authentication request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error(
        "Custom authentication server rejected the request (HTTP {0}). Check your credentials or sign in again."
    )]
    Rejected(u16),
    #[error("Custom authentication server returned an invalid response: {0}")]
    InvalidResponse(String),
}

impl CustomAuthError {
    pub fn is_transient(&self) -> bool {
        matches!(self, Self::Request(e) if e.is_connect() || e.is_timeout())
            || matches!(self, Self::Rejected(429 | 500..=599))
    }
}

fn validate_url(value: &str) -> Result<Url, CustomAuthError> {
    let url = Url::parse(value).map_err(|_| {
        CustomAuthError::InvalidUrl("Enter an http:// or https:// server URL".into())
    })?;
    if !matches!(url.scheme(), "https" | "http")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || url.query().is_some()
    {
        return Err(CustomAuthError::InvalidUrl(
            "Use HTTP(S) without embedded credentials, query or fragment".into(),
        ));
    }
    Ok(url)
}

impl CustomServer {
    pub fn new(
        api_url: &str,
        login_path: &str,
        refresh_path: &str,
    ) -> Result<Self, CustomAuthError> {
        let url = validate_url(api_url)?;
        let server = Self {
            api_url: url.as_str().trim_end_matches('/').into(),
            login_path: login_path.into(),
            refresh_path: refresh_path.into(),
        };
        server.endpoint(login_path)?;
        server.endpoint(refresh_path)?;
        Ok(server)
    }

    pub fn endpoint(&self, path: &str) -> Result<Url, CustomAuthError> {
        let base = validate_url(&self.api_url)?;
        if !path.starts_with('/')
            || path.starts_with("//")
            || path.contains('?')
            || path.contains('#')
        {
            return Err(CustomAuthError::InvalidUrl(
                "Login and refresh paths must start with a single /".into(),
            ));
        }
        let endpoint = validate_url(&format!("{}{}", self.api_url.trim_end_matches('/'), path))?;
        if endpoint.origin() != base.origin() {
            return Err(CustomAuthError::InvalidUrl(
                "Authentication paths must use the selected server".into(),
            ));
        }
        Ok(endpoint)
    }
}

pub async fn discover_custom_server(
    client: &reqwest::Client,
    api_url: &str,
    login_path: &str,
    refresh_path: &str,
) -> Result<CustomServer, CustomAuthError> {
    let original = validate_url(api_url)?;
    let response = client.get(original).send().await?;
    if !response.status().is_success() {
        return Err(CustomAuthError::Rejected(response.status().as_u16()));
    }
    let resolved =
        if let Some(header) = response.headers().get("x-authlib-injector-api-location") {
            response
                .url()
                .join(header.to_str().map_err(|_| {
                    CustomAuthError::InvalidUrl("Invalid API location header".into())
                })?)
                .map_err(|_| CustomAuthError::InvalidUrl("Invalid API location header".into()))?
        } else {
            response.url().clone()
        };
    // The UI obtains trust for the server and its declared authentication endpoint before sign-in.
    CustomServer::new(resolved.as_str(), login_path, refresh_path)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    client_token: String,
    selected_profile: Option<CustomProfile>,
    #[serde(default)]
    available_profiles: Vec<CustomProfile>,
}

async fn post(url: Url, body: serde_json::Value) -> Result<TokenResponse, CustomAuthError> {
    // Never follow a credential-bearing POST to an unreviewed redirect target.
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    let response = client.post(url).json(&body).send().await?;
    if !response.status().is_success() {
        return Err(CustomAuthError::Rejected(response.status().as_u16()));
    }
    let data: TokenResponse = response.json().await?;
    if data.access_token.is_empty() {
        return Err(CustomAuthError::InvalidResponse(
            "Missing access token".into(),
        ));
    }
    Ok(data)
}

pub async fn login(
    server: CustomServer,
    username: &str,
    password: &str,
) -> Result<CustomLoginResponse, CustomAuthError> {
    let data = post(server.endpoint(&server.login_path)?, serde_json::json!({
        "username": username, "password": password, "agent": { "name": "Minecraft", "version": 1 }
    })).await?;
    response_to_login(server, data)
}

fn response_to_login(
    server: CustomServer,
    data: TokenResponse,
) -> Result<CustomLoginResponse, CustomAuthError> {
    if data.access_token.is_empty() || data.client_token.is_empty() {
        return Err(CustomAuthError::InvalidResponse(
            "Missing authentication tokens".into(),
        ));
    }
    let selected = data.selected_profile.as_ref().map(|p| p.id);
    let profiles = if let Some(profile) = data.selected_profile {
        vec![profile]
    } else {
        data.available_profiles
    };
    if profiles.is_empty() || profiles.iter().any(|p| p.name.is_empty() || p.id.is_nil()) {
        return Err(CustomAuthError::InvalidResponse(
            "No playable Minecraft profiles".into(),
        ));
    }
    Ok(CustomLoginResponse {
        server,
        profiles,
        access_token: data.access_token,
        client_token: data.client_token,
        selected,
    })
}

pub async fn select_profile(
    response: CustomLoginResponse,
    id: Uuid,
) -> Result<MinecraftAccount, CustomAuthError> {
    let profile = response
        .profiles
        .iter()
        .find(|p| p.id == id)
        .ok_or_else(|| CustomAuthError::InvalidResponse("Unknown profile".into()))?
        .clone();
    let account = MinecraftAccount {
        id: profile.id,
        username: profile.name,
        access_token: response.access_token,
        refresh_token: String::new(),
        expires: Utc::now() + Duration::hours(12),
        kind: AccountKind::Custom,
        signed_out: false,
        elyby_client_id: None,
        custom: Some(CustomAccountData {
            server: response.server,
            client_token: response.client_token,
        }),
    };
    if response.selected == Some(id) {
        return Ok(account);
    }
    // Bind an explicitly chosen profile with a Yggdrasil refresh.
    refresh(&account).await
}

pub async fn refresh(account: &MinecraftAccount) -> Result<MinecraftAccount, CustomAuthError> {
    let custom = account.custom.as_ref().ok_or_else(|| {
        CustomAuthError::InvalidResponse("Missing server details; sign in again".into())
    })?;
    let data = post(
        custom.server.endpoint(&custom.server.refresh_path)?,
        serde_json::json!({
            "accessToken": account.access_token, "clientToken": custom.client_token,
            "selectedProfile": { "id": account.id.simple().to_string(), "name": account.username }
        }),
    )
    .await?;
    let profile = data
        .selected_profile
        .ok_or_else(|| CustomAuthError::InvalidResponse("No selected profile".into()))?;
    if profile.id != account.id || profile.name.is_empty() {
        return Err(CustomAuthError::InvalidResponse(
            "Server changed the selected profile".into(),
        ));
    }
    let mut updated = account.clone();
    updated.access_token = data.access_token;
    updated.username = profile.name;
    updated.expires = Utc::now() + Duration::hours(12);
    if !data.client_token.is_empty() {
        updated.custom.as_mut().unwrap().client_token = data.client_token;
    }
    Ok(updated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    const ID: &str = "6d7d9f2e8a9c4f809a12b99d8cc92c35";

    #[test]
    fn server_edits_fill_only_empty_authentication_paths() {
        assert_eq!(
            custom_auth_paths_on_server_change("", ""),
            (
                Some("/authserver/authenticate".into()),
                Some("/authserver/refresh".into())
            )
        );
        assert_eq!(
            custom_auth_paths_on_server_change("/custom/login", ""),
            (None, Some("/authserver/refresh".into()))
        );
        assert_eq!(
            custom_auth_paths_on_server_change("", "/custom/refresh"),
            (Some("/authserver/authenticate".into()), None)
        );
        assert_eq!(
            custom_auth_paths_on_server_change("/custom/login", "/custom/refresh"),
            (None, None)
        );
    }

    #[test]
    fn endpoint_paths_preserve_api_prefix_and_reject_unreviewed_destinations() {
        let server = CustomServer::new(
            "https://example.com/api/",
            "/authserver/authenticate",
            "/authserver/refresh",
        )
        .unwrap();
        assert_eq!(
            server.endpoint(&server.login_path).unwrap().as_str(),
            "https://example.com/api/authserver/authenticate"
        );
        for path in [
            "https://other.com/authenticate",
            "//other.com",
            "authserver/authenticate",
            "/authenticate?token=secret",
            "/authenticate#fragment",
        ] {
            assert!(server.endpoint(path).is_err());
        }
        for base in [
            "file:///tmp/auth",
            "ftp://host",
            "https://user:password@example.com",
            "https://example.com?token=secret",
        ] {
            assert!(CustomServer::new(base, "/auth", "/refresh").is_err());
        }
    }

    #[test]
    fn login_profiles_and_tokens_are_validated_and_uuid_formats_supported() {
        let server = CustomServer::new("https://example.com", "/authenticate", "/refresh").unwrap();
        let data = json!({"accessToken":"private-access", "clientToken":"private-client", "availableProfiles":[{"id":ID,"name":"First"}, {"id":Uuid::new_v4(),"name":"Second"}]});
        let response = response_to_login(
            server.clone(),
            serde_json::from_value(data.clone()).unwrap(),
        )
        .unwrap();
        assert_eq!(response.profiles.len(), 2);
        assert_eq!(response.profiles[0].id, Uuid::parse_str(ID).unwrap());
        assert!(response.selected.is_none());
        for invalid in [
            json!({"accessToken":"", "clientToken":"client"}),
            json!({"accessToken":"access", "clientToken":""}),
            json!({"accessToken":"access", "clientToken":"client", "selectedProfile":{"id":ID,"name":""}}),
        ] {
            assert!(
                response_to_login(server.clone(), serde_json::from_value(invalid).unwrap())
                    .is_err()
            );
        }
    }

    #[tokio::test]
    async fn selected_profile_persists_provider_and_never_password() {
        let server = CustomServer::new("https://example.com", "/authenticate", "/refresh").unwrap();
        let data = json!({"accessToken":"access", "clientToken":"client", "selectedProfile":{"id":ID,"name":"CustomPlayer"}});
        let response = response_to_login(server, serde_json::from_value(data).unwrap()).unwrap();
        let account = select_profile(response.clone(), Uuid::parse_str(ID).unwrap())
            .await
            .unwrap();
        assert!(account.is_custom());
        assert_eq!(account.authlib_url(), Some("https://example.com"));
        assert!(account.skin_profile_key().starts_with("custom:"));
        assert!(select_profile(response, Uuid::new_v4()).await.is_err());
        let serialized = serde_json::to_string(&account).unwrap();
        assert!(!serialized.contains("password"));
        let mut secret_account = account.clone();
        secret_account.access_token = "private-access-token".into();
        secret_account.refresh_token = "private-refresh-token".into();
        secret_account.custom.as_mut().unwrap().client_token = "private-client-token".into();
        assert!(!format!("{secret_account:?}").contains("private-"));
        assert!(!format!("{:?}", secret_account.custom).contains("private-"));
        let restored: MinecraftAccount = serde_json::from_str(&serialized).unwrap();
        assert!(restored.is_custom());
        assert_eq!(restored.custom.unwrap().client_token, "client");
    }

    #[test]
    fn transient_custom_errors_do_not_include_bad_credentials() {
        assert!(CustomAuthError::Rejected(503).is_transient());
        assert!(CustomAuthError::Rejected(429).is_transient());
        assert!(!CustomAuthError::Rejected(403).is_transient());
    }

    #[tokio::test]
    async fn cancelled_custom_login_never_commits_an_account() {
        let server = CustomServer::new("https://example.com", "/authenticate", "/refresh").unwrap();
        let data = json!({"accessToken":"access", "clientToken":"client", "selectedProfile":{"id":ID,"name":"Player"}});
        let response = response_to_login(server, serde_json::from_value(data).unwrap()).unwrap();
        let (events, _rx) = oneclient_events::EventBus::channel();
        let service = crate::AuthService::with_store(
            crate::CredentialsStore::default(),
            oneclient_net::RequestClient::new(oneclient_net::NetConfig::default()).unwrap(),
            events,
        );
        let cancel = tokio_util::sync::CancellationToken::new();
        cancel.cancel();
        assert!(matches!(
            service
                .finish_custom_login(response, Uuid::parse_str(ID).unwrap(), cancel)
                .await,
            Err(crate::AuthError::LoginCancelled)
        ));
        assert!(service.list_accounts().await.is_empty());
    }

    #[tokio::test]
    async fn discovery_login_profile_selection_and_refresh_use_configured_server() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let fixture = tokio::spawn(async move {
            for step in 0..4 {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut bytes = Vec::new();
                let (head, body) = loop {
                    let mut buffer = [0; 4096];
                    let read = stream.read(&mut buffer).await.unwrap();
                    assert!(read > 0);
                    bytes.extend_from_slice(&buffer[..read]);
                    if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                        let head = String::from_utf8_lossy(&bytes[..end]).to_string();
                        let length = head
                            .lines()
                            .find_map(|line| {
                                line.to_lowercase()
                                    .strip_prefix("content-length: ")
                                    .and_then(|v| v.parse::<usize>().ok())
                            })
                            .unwrap_or(0);
                        if bytes.len() >= end + 4 + length {
                            break (head, bytes[end + 4..end + 4 + length].to_vec());
                        }
                    }
                };
                let (headers, response) = if step == 0 {
                    assert!(head.starts_with("GET /entry HTTP/1.1"));
                    assert!(body.is_empty());
                    ("X-Authlib-Injector-API-Location: /api\r\n", json!({}))
                } else {
                    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
                    if step == 1 {
                        assert!(head.starts_with("POST /api/custom/login HTTP/1.1"));
                        assert_eq!(body["username"], "login-user");
                        assert_eq!(body["password"], "password-only-for-login");
                        assert_eq!(body["agent"]["name"], "Minecraft");
                        (
                            "",
                            json!({"accessToken":"initial-token","clientToken":"client","availableProfiles":[{"id":ID,"name":"Player"},{"id":Uuid::new_v4(),"name":"Other"}]}),
                        )
                    } else {
                        assert!(head.starts_with("POST /api/custom/refresh HTTP/1.1"));
                        assert!(body.get("password").is_none());
                        assert_eq!(body["clientToken"], "client");
                        assert_eq!(body["selectedProfile"]["id"], ID);
                        assert_eq!(
                            body["accessToken"],
                            if step == 2 {
                                "initial-token"
                            } else {
                                "selected-token"
                            }
                        );
                        (
                            "",
                            json!({"accessToken":if step == 2 { "selected-token" } else { "rotated-token" },"clientToken":"client","selectedProfile":{"id":ID,"name":"Player"}}),
                        )
                    }
                };
                let payload = response.to_string();
                stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n{headers}Content-Length: {}\r\nConnection: close\r\n\r\n{payload}", payload.len()).as_bytes()).await.unwrap();
            }
        });
        let client = reqwest::Client::new();
        let server = discover_custom_server(
            &client,
            &format!("{base}/entry"),
            "/custom/login",
            "/custom/refresh",
        )
        .await
        .unwrap();
        assert_eq!(server.api_url, format!("{base}/api"));
        let response = login(server, "login-user", "password-only-for-login")
            .await
            .unwrap();
        assert_eq!(response.profiles.len(), 2);
        let account = select_profile(response, Uuid::parse_str(ID).unwrap())
            .await
            .unwrap();
        assert_eq!(account.access_token, "selected-token");
        let updated = refresh(&account).await.unwrap();
        assert_eq!(updated.access_token, "rotated-token");
        fixture.await.unwrap();
    }
}
