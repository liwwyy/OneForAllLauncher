//! Optional public name lookup; it never changes an offline account's identity.
use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct OnlineProfile {
    id: Uuid,
    name: String,
}

pub async fn lookup_online_username(
    client: &Client,
    username: &str,
) -> Result<Option<String>, String> {
    lookup_at(
        client,
        username,
        "https://api.mojang.com/users/profiles/minecraft",
    )
    .await
}

async fn lookup_at(
    client: &Client,
    username: &str,
    endpoint: &str,
) -> Result<Option<String>, String> {
    crate::validate_offline_username(username).map_err(|e| e.to_string())?;
    let response = client
        .get(format!("{endpoint}/{username}"))
        .timeout(Duration::from_secs(8))
        .send()
        .await
        .map_err(|_| "Couldn't reach the online username service.".to_string())?;
    match response.status() {
        StatusCode::NO_CONTENT | StatusCode::NOT_FOUND => return Ok(None),
        status if !status.is_success() => {
            return Err(format!(
                "Couldn't check online usernames (HTTP {}).",
                status.as_u16()
            ));
        }
        _ => {}
    }
    let profile: OnlineProfile = response
        .json()
        .await
        .map_err(|_| "The online username service returned an invalid response.".to_string())?;
    if profile.id.is_nil() || !profile.name.eq_ignore_ascii_case(username) {
        return Err("The online username service returned a different or invalid profile.".into());
    }
    Ok(Some(profile.name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
    };

    async fn response(status: &str, body: &str) -> Result<Option<String>, String> {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let reply = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut bytes = [0; 2048];
            let n = socket.read(&mut bytes).unwrap();
            assert!(String::from_utf8_lossy(&bytes[..n]).starts_with("GET /Notch "));
            socket.write_all(reply.as_bytes()).unwrap();
        });
        let result = lookup_at(&Client::new(), "Notch", &endpoint).await;
        server.join().unwrap();
        result
    }

    #[tokio::test]
    async fn distinguishes_missing_names_from_service_failures() {
        assert_eq!(response("204 No Content", "").await.unwrap(), None);
        assert_eq!(response("404 Not Found", "{}").await.unwrap(), None);
        assert!(response("429 Too Many Requests", "{}").await.is_err());
        assert!(response("200 OK", "not JSON").await.is_err());
    }

    #[tokio::test]
    async fn requires_a_matching_online_profile() {
        let id = "069a79f444e94726a5befca90e38aaf5";
        assert_eq!(
            response("200 OK", &format!(r#"{{"id":"{id}","name":"notch"}}"#))
                .await
                .unwrap(),
            Some("notch".into())
        );
        assert!(
            response(
                "200 OK",
                &format!(r#"{{"id":"{id}","name":"OtherPlayer"}}"#)
            )
            .await
            .is_err()
        );
        assert!(
            lookup_at(&Client::new(), "../invalid", "http://127.0.0.1:1")
                .await
                .is_err()
        );
    }
}
