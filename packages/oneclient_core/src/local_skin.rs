//! Local-only skins for offline accounts, independent of Minecraft version or loader.
use std::path::PathBuf;

use oneclient_auth::MinecraftAccount;
use oneclient_common::paths;
use serde::{Deserialize, Serialize};

use crate::{LauncherError, LauncherResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalSkin {
    pub slim: bool,
}

fn directory(account: uuid::Uuid) -> LauncherResult<PathBuf> {
    Ok(paths::config_dir()?
        .join("local-skins")
        .join(account.to_string()))
}

pub fn validate(bytes: &[u8]) -> LauncherResult<()> {
    if bytes.len() > 1024 * 1024 {
        return Err(LauncherError::Minecraft(
            "The skin PNG must be smaller than 1 MB.".into(),
        ));
    }
    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .map_err(|_| LauncherError::Minecraft("Choose a valid 64 × 64 PNG skin.".into()))?;
    if image.width() != 64 || image.height() != 64 {
        return Err(LauncherError::Minecraft("Use a 64 × 64 PNG skin.".into()));
    }
    Ok(())
}

pub async fn load(account: uuid::Uuid) -> LauncherResult<Option<(Vec<u8>, bool)>> {
    let dir = directory(account)?;
    if !dir.join("skin.json").is_file() {
        return Ok(None);
    }
    let skin: LocalSkin = polyio::read_json(dir.join("skin.json")).await?;
    let bytes = tokio::fs::read(dir.join("skin.png")).await?;
    validate(&bytes)?;
    Ok(Some((bytes, skin.slim)))
}

pub async fn save(account: &MinecraftAccount, bytes: &[u8], slim: bool) -> LauncherResult<()> {
    if !account.is_offline() {
        return Err(LauncherError::Minecraft(
            "Local skins are only available for offline accounts.".into(),
        ));
    }
    validate(bytes)?;
    let dir = directory(account.id)?;
    polyio::write_atomic(dir.join("skin.png"), bytes).await?;
    polyio::write_json_atomic(dir.join("skin.json"), &LocalSkin { slim }).await?;
    Ok(())
}

pub async fn restore(account: &MinecraftAccount) -> LauncherResult<()> {
    if !account.is_offline() {
        return Err(LauncherError::Minecraft(
            "Local skins are only available for offline accounts.".into(),
        ));
    }
    let dir = directory(account.id)?;
    if dir.exists() {
        tokio::fs::remove_dir_all(dir).await?;
    }
    Ok(())
}

pub(crate) async fn launch_files(
    account: &MinecraftAccount,
) -> LauncherResult<Option<(PathBuf, PathBuf, bool)>> {
    if !account.is_offline() {
        return Ok(None);
    }
    let Some((_, slim)) = load(account.id).await? else {
        return Ok(None);
    };
    let libraries = paths::libraries_dir()?.join("oneforall/local-skin");
    let assets: [(&str, &[u8]); 2] = [
        (
            "offline-skin-4.jar",
            include_bytes!(concat!(env!("OUT_DIR"), "/offline-skin.jar")),
        ),
        (
            "offline-skin-bridge-4.jar",
            include_bytes!(concat!(env!("OUT_DIR"), "/offline-skin-bridge-4.jar")),
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
    let jar = libraries.join("offline-skin-4.jar");
    Ok(Some((jar, directory(account.id)?.join("skin.png"), slim)))
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
        assert!(validate(b"not a PNG").is_err());
        assert!(validate(&vec![0; 1024 * 1024 + 1]).is_err());
    }
    #[test]
    fn pngs_must_have_minecraft_skin_dimensions() {
        for (edge, valid) in [(64, true), (32, false)] {
            let mut bytes = std::io::Cursor::new(Vec::new());
            image::DynamicImage::new_rgba8(edge, edge)
                .write_to(&mut bytes, image::ImageFormat::Png)
                .unwrap();
            assert_eq!(validate(bytes.get_ref()).is_ok(), valid);
        }
    }
}
