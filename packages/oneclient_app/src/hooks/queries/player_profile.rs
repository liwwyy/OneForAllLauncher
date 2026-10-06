use std::time::Duration;

use freya::query::{Query, QueryCapability, UseQuery, use_query};
use oneclient_core::LauncherError;
use oneclient_mc::{self as minecraft, PlayerProfileView};

const PROFILE_STALE: Duration = Duration::from_secs(30 * 60);
const PROFILE_CLEAN: Duration = Duration::from_secs(2 * 60 * 60);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FetchPlayerProfileQuery;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PlayerProfileQueryKeys {
    pub uuid: String,
    pub access_token: Option<String>,
}

impl QueryCapability for FetchPlayerProfileQuery {
    type Ok = PlayerProfileView;
    type Err = LauncherError;
    type Keys = PlayerProfileQueryKeys;

    async fn run(&self, keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let access_token = keys.access_token.clone();

        let state = crate::launcher::state()?;
        let client = &state.services.requester;

        if let Some(username) = keys.uuid.strip_prefix("elyby:") {
            let mut url = url::Url::parse("https://skinsystem.ely.by/textures/")?;
            url.path_segments_mut()
                .map_err(|_| LauncherError::Minecraft("Invalid Ely.by skin URL".into()))?
                .push(username);
            let response = client
                .http()
                .get(url)
                .send()
                .await
                .map_err(oneclient_net::RequestError::from)?;
            if response.status().as_u16() == 204 {
                return Ok(PlayerProfileView::placeholder());
            }
            let value = response
                .error_for_status()
                .map_err(oneclient_net::RequestError::from)?
                .json::<serde_json::Value>()
                .await
                .map_err(oneclient_net::RequestError::from)?;
            return Ok(PlayerProfileView {
                uuid: keys.uuid.clone(),
                username: username.into(),
                skin_url: value
                    .pointer("/SKIN/url")
                    .and_then(|v| v.as_str())
                    .map(str::to_owned),
                cape_url: value
                    .pointer("/CAPE/url")
                    .and_then(|v| v.as_str())
                    .map(str::to_owned),
                is_slim: value
                    .pointer("/SKIN/metadata/model")
                    .and_then(|v| v.as_str())
                    == Some("slim"),
                ..Default::default()
            });
        }
        Ok(
            minecraft::fetch_player_profile_view(client, &keys.uuid, access_token.as_deref())
                .await?,
        )
    }

    // TODO Cache
    // fn matches(&self, keys: &Self::Keys) -> bool {
    //     keys.uuid.as_deref().is_some_and(|id| !id.is_empty())
    // }
}

pub fn use_player_profile(
    uuid: impl Into<String>,
    access_token: Option<impl Into<String>>,
) -> UseQuery<FetchPlayerProfileQuery> {
    use_query(
        Query::new(
            PlayerProfileQueryKeys {
                uuid: uuid.into(),
                access_token: access_token.map(|s| s.into()),
            },
            FetchPlayerProfileQuery,
        )
        .stale_time(PROFILE_STALE)
        .clean_time(PROFILE_CLEAN),
    )
}
