//! Local-only skins for offline accounts, independent of Minecraft version or loader.
pub mod catalog;
pub mod library;
use std::path::PathBuf;

use oneclient_auth::MinecraftAccount;
use oneclient_common::paths;
use serde::{Deserialize, Serialize};

use crate::LauncherResult;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalSkin {
    pub slim: bool,
}

fn directory(account: uuid::Uuid) -> LauncherResult<PathBuf> {
    Ok(paths::config_dir()?
        .join("local-skins")
        .join(account.to_string()))
}

pub async fn load(account: uuid::Uuid) -> LauncherResult<Option<(Vec<u8>, bool)>> {
    library::active_skin(account).await
}

pub(crate) struct LaunchAppearance {
    pub jar: PathBuf,
    pub skin: Option<PathBuf>,
    pub cape: Option<PathBuf>,
    pub slim: bool,
}

pub(crate) async fn launch_files(
    account: &MinecraftAccount,
) -> LauncherResult<Option<LaunchAppearance>> {
    if !account.is_offline() {
        return Ok(None);
    }
    let (skin, cape, slim) = library::launch_selection(account.id).await?;
    if skin.is_none() && cape.is_none() {
        return Ok(None);
    }
    let libraries = paths::libraries_dir()?.join("oneforall/local-skin");
    let assets: [(&str, &[u8]); 2] = [
        (
            "offline-skin-6.jar",
            include_bytes!(concat!(env!("OUT_DIR"), "/offline-skin.jar")),
        ),
        (
            "offline-skin-bridge-6.jar",
            include_bytes!(concat!(env!("OUT_DIR"), "/offline-skin-bridge-6.jar")),
        ),
    ];
    for (name, bytes) in assets {
        let path = libraries.join(name);
        if !tokio::fs::read(&path)
            .await
            .is_ok_and(|found| found == bytes)
        {
            polyio::write_atomic(&path, bytes).await?;
        }
    }
    let jar = libraries.join("offline-skin-6.jar");
    Ok(Some(LaunchAppearance {
        jar,
        skin,
        cape,
        slim,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn disconnected_online_accounts_never_receive_skin_files() {
        let mut account = oneclient_auth::offline_account("Player".into());
        for kind in [
            oneclient_auth::AccountKind::Microsoft,
            oneclient_auth::AccountKind::Elyby,
            oneclient_auth::AccountKind::Custom,
        ] {
            account.kind = kind;
            account.access_token.clear();
            assert!(launch_files(&account).await.unwrap().is_none());
        }
    }
    #[test]
    fn malformed_and_oversized_pngs_are_rejected() {
        assert!(library::validate_texture(b"not a PNG", library::Kind::Skin).is_err());
        assert!(library::validate_texture(&vec![0; 1024 * 1024 + 1], library::Kind::Skin).is_err());
    }
    #[test]
    fn pngs_must_have_minecraft_skin_dimensions() {
        for (edge, valid) in [(64, true), (32, false)] {
            let mut bytes = std::io::Cursor::new(Vec::new());
            image::DynamicImage::new_rgba8(edge, edge)
                .write_to(&mut bytes, image::ImageFormat::Png)
                .unwrap();
            assert_eq!(
                library::validate_texture(bytes.get_ref(), library::Kind::Skin).is_ok(),
                valid
            );
        }
    }
}
