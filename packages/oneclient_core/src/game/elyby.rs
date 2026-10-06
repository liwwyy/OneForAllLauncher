//! FreesmLauncher's authlib-injector fallback for Ely.by accounts.
use crate::{LauncherResult, state::LauncherState};
use oneclient_common::paths;
use sha2::{Digest, Sha256};

const VERSION: &str = "1.2.8";
const SHA256: &str = "9c7f4343e6c82034958ffb48c14a2cb0c85928be7283103ce17da00c6d5a7b10";
const URL: &str = "https://authlib-injector.yushi.moe/artifact/56/authlib-injector-1.2.8.jar";
const PREVIEW_AGENT: &[u8] = include_bytes!("../../assets/elyby-preview-compat.jar");

pub(super) async fn preview_jar(version: &str) -> LauncherResult<Option<std::path::PathBuf>> {
    if version != "1.8.9" {
        return Ok(None);
    }
    let jar = paths::libraries_dir()?.join("oneforall/elyby/preview-compat-1.jar");
    if !tokio::fs::read(&jar)
        .await
        .is_ok_and(|bytes| bytes == PREVIEW_AGENT)
    {
        polyio::write_atomic(&jar, PREVIEW_AGENT).await?;
    }
    Ok(Some(jar))
}

pub(super) async fn java_argument(state: &LauncherState) -> LauncherResult<String> {
    let jar = paths::libraries_dir()?
        .join("oneforall/elyby")
        .join(format!("authlib-injector-{VERSION}.jar"));
    let valid = tokio::fs::read(&jar).await.is_ok_and(|bytes| {
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
            == SHA256
    });
    if !valid {
        state
            .services
            .events
            .notify("Preparing Ely.by authentication")
            .body("Downloading authlib-injector for Minecraft authentication and skins.")
            .send();
        let checksum = polyio::Checksum::sha256(SHA256);
        oneclient_net::download_verified(
            &state.services.requester,
            &state.services.events,
            URL,
            &jar,
            Some(&checksum),
            0,
            None,
        )
        .await?;
    }
    Ok(format!(
        "-javaagent:{}={}",
        jar.display(),
        oneclient_auth::ELYBY_AUTHLIB_URL
    ))
}
