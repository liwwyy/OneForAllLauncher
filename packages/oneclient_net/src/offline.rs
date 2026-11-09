//! Cache-only requests for one launch task, independent of other launcher work.
use std::future::Future;
tokio::task_local! { static CACHE_ONLY: bool; }
pub fn active() -> bool {
    CACHE_ONLY.try_with(|value| *value).unwrap_or(false)
}
pub async fn scope<T>(offline: bool, work: impl Future<Output = T>) -> T {
    CACHE_ONLY.scope(offline, work).await
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn concurrent_launch_modes_are_isolated() {
        let (offline, online) = tokio::join!(
            scope(true, async { active() }),
            scope(false, async { active() })
        );
        assert!(offline);
        assert!(!online);
        assert!(!active());
    }
}

#[cfg(test)]
mod transport_tests {
    use super::*;
    use crate::{EtagPolicy, NetConfig, RequestClient, RequestError, fetch_cached};
    #[tokio::test]
    async fn cache_only_never_connects_even_when_a_server_is_available() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}/metadata", listener.local_addr().unwrap());
        let cache = std::env::temp_dir().join(format!("oneforall-cache-{}", uuid::Uuid::new_v4()));
        tokio::fs::write(&cache, b"cached metadata").await.unwrap();
        let client = RequestClient::new(NetConfig::default()).unwrap();
        scope(true, async {
            let response = fetch_cached(&client, &url, &cache, EtagPolicy::CommitNow)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(response.bytes, b"cached metadata");
            assert!(!response.changed);
            tokio::fs::remove_file(&cache).await.unwrap();
            assert!(
                fetch_cached(&client, &url, &cache, EtagPolicy::CommitNow)
                    .await
                    .unwrap()
                    .is_none()
            );
            let request = reqwest::Request::new(reqwest::Method::GET, url.parse().unwrap());
            assert!(matches!(
                client.send(request).await,
                Err(RequestError::Offline)
            ));
        })
        .await;
        assert!(
            matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
        );
    }
}
