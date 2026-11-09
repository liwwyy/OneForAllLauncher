//! Shared automatic and explicit offline launch policy.
use crate::{LauncherResult, LauncherState};

pub fn is_offline(online: bool, preference: bool, explicit: bool) -> bool {
    explicit || preference || !online
}

pub async fn preference(state: &LauncherState, cluster: i64) -> LauncherResult<bool> {
    let value = sqlx::query_scalar::<_, i64>(
        "SELECT always_offline FROM offline_launch_settings WHERE cluster_id = ?",
    )
    .bind(cluster)
    .fetch_optional(&state.services.db)
    .await?;
    Ok(value.unwrap_or(0) != 0)
}

pub async fn set_preference(
    state: &LauncherState,
    cluster: i64,
    enabled: bool,
) -> LauncherResult<()> {
    sqlx::query("INSERT INTO offline_launch_settings(cluster_id, always_offline) VALUES (?, ?) ON CONFLICT(cluster_id) DO UPDATE SET always_offline = excluded.always_offline")
        .bind(cluster).bind(i64::from(enabled)).execute(&state.services.db).await?;
    Ok(())
}

pub async fn resolve(state: &LauncherState, cluster: i64, explicit: bool) -> LauncherResult<bool> {
    let always = preference(state, cluster).await?;
    if always || explicit {
        return Ok(true);
    }
    Ok(!oneclient_net::status::online_for_launch().await)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recovering_online_only_releases_automatic_offline_mode() {
        assert!(is_offline(false, false, false));
        assert!(!is_offline(true, false, false));
        assert!(is_offline(true, true, false));
        assert!(is_offline(true, false, true));
    }
}
