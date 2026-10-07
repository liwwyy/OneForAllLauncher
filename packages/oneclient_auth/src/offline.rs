use chrono::{Duration, Utc};
use md5::{Digest, Md5};
use uuid::Uuid;

use super::data::{AccountKind, MinecraftAccount};
use super::error::AuthError;
use crate::error::AuthResult;

pub fn offline_uuid(username: &str) -> Uuid {
    let mut hasher = Md5::new();
    hasher.update(format!("OfflinePlayer:{username}").as_bytes());
    let digest = hasher.finalize();

    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest);
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

/// Full validity for submission; the input validator also accepts partial names.
pub fn offline_username_input_allowed(username: &str, allow_invalid: bool) -> bool {
    allow_invalid
        || (username.len() <= 16
            && username
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_'))
}

pub fn validate_offline_username_with_override(
    username: &str,
    allow_invalid: bool,
) -> AuthResult<()> {
    if allow_invalid && !username.is_empty() {
        return Ok(());
    }
    validate_offline_username(username)
}

// Behavioral reference: FreesmLauncher ChooseOfflineNameDialog.cpp (GPL-3.0-only).
pub fn random_offline_characters() -> String {
    use rand::RngExt;
    const CHARACTERS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..14)
        .map(|_| CHARACTERS[rng.random_range(0..CHARACTERS.len())] as char)
        .collect()
}

pub fn random_offline_username() -> String {
    use rand::RngExt;
    const WORDS: &[&str] = &[
        "Cookie", "Clicker", "Licker", "Lenny", "Super", "Sakupen", "Sonic", "Geometry", "Mining",
        "Chicken", "Sculpted", "Random", "Painted", "Fainted", "MadeIn", "Chinese", "Bing", "Hell",
        "Circles", "Wave", "Dash", "Crafting", "Smelting", "Jockey", "Vase", "Heaven", "Pudding",
        "Chilling",
    ];
    let mut rng = rand::rng();
    format!(
        "{}{}",
        WORDS[rng.random_range(0..WORDS.len())],
        WORDS[rng.random_range(0..WORDS.len())]
    )
}

pub fn validate_offline_username(username: &str) -> AuthResult<()> {
    let len = username.chars().count();
    if !(3..=16).contains(&len) {
        return Err(AuthError::InvalidOfflineUsername {
            reason: "username must be 3-16 characters".into(),
        });
    }

    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(AuthError::InvalidOfflineUsername {
            reason: "username may only contain letters, digits, and underscores".into(),
        });
    }

    Ok(())
}

#[tracing::instrument(level = "debug", fields(username = %username))]
pub fn offline_account(username: String) -> MinecraftAccount {
    tracing::info!("creating offline account");
    MinecraftAccount {
        id: offline_uuid(&username),
        username,
        access_token: String::new(),
        refresh_token: String::new(),
        expires: Utc::now() + Duration::days(3650),
        kind: AccountKind::Offline,
        elyby_client_id: None,
        custom: None,
    }
}
