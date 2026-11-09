use crate::LauncherResult;
use oneclient_net::RequestClient;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct ChangelogEntry {
    pub version: String,
    pub body: String,
}

/// Fork release notes are bundled locally until a fork release feed is configured.
pub async fn fetch_changelog(_net: &RequestClient) -> LauncherResult<Vec<ChangelogEntry>> {
    Ok(vec![ChangelogEntry {
        version: env!("CARGO_PKG_VERSION").to_string(),
        body: include_str!("../../../FORK_CHANGELOG.md").to_string(),
    }])
}
