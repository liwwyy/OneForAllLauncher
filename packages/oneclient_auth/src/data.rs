use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountKind {
    Microsoft,
    Offline,
    Elyby,
    Custom,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MinecraftAccount {
    pub id: Uuid,
    pub username: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires: DateTime<Utc>,
    #[serde(default = "default_account_kind")]
    pub kind: AccountKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elyby_client_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom: Option<crate::custom::CustomAccountData>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub signed_out: bool,
}

impl std::fmt::Debug for MinecraftAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MinecraftAccount")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("kind", &self.kind)
            .field("expires", &self.expires)
            .field("signed_out", &self.signed_out)
            .field("custom_server", &self.custom.as_ref().map(|c| &c.server))
            .finish_non_exhaustive()
    }
}

fn default_account_kind() -> AccountKind {
    AccountKind::Microsoft
}

impl MinecraftAccount {
    pub fn is_microsoft(&self) -> bool {
        self.kind == AccountKind::Microsoft
    }

    pub fn is_offline(&self) -> bool {
        self.kind == AccountKind::Offline
    }

    /// Public profile key; Ely.by skins use their own username namespace.
    pub fn skin_profile_key(&self) -> String {
        if self.is_elyby() {
            format!("elyby:{}", self.username)
        } else if let Some(custom) = &self.custom {
            format!("custom:{}:{}", self.id, custom.server.api_url)
        } else {
            self.id.to_string()
        }
    }

    pub fn is_elyby(&self) -> bool {
        self.kind == AccountKind::Elyby
    }

    pub fn is_custom(&self) -> bool {
        self.kind == AccountKind::Custom
    }

    pub fn authlib_url(&self) -> Option<&str> {
        if self.is_elyby() {
            Some(crate::ELYBY_AUTHLIB_URL)
        } else if self.is_custom() {
            self.custom.as_ref().map(|c| c.server.api_url.as_str())
        } else {
            None
        }
    }

    pub fn needs_sign_in(&self) -> bool {
        self.is_microsoft() && self.signed_out
    }

    pub fn is_expired(&self) -> bool {
        self.expires <= Utc::now() + chrono::TimeDelta::seconds(60)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ElybyLoginSession {
    pub client_id: String,
    pub device: DeviceCodeLogin,
}

impl std::fmt::Debug for ElybyLoginSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ElybyLoginSession")
            .field("client_id", &self.client_id)
            .field("expires_in", &self.device.expires_in)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceCodeLogin {
    pub user_code: String,
    pub device_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BrowserLogin {
    pub auth_url: String,
    pub state: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MicrosoftLoginSession {
    pub browser: BrowserLogin,
    pub device: DeviceCodeLogin,
}

impl MicrosoftLoginSession {
    pub fn dedupe_key(&self) -> &str {
        &self.browser.state
    }

    pub fn auth_url(&self) -> &str {
        &self.browser.auth_url
    }

    pub fn user_code(&self) -> &str {
        &self.device.user_code
    }

    pub fn verification_uri(&self) -> &str {
        &self.device.verification_uri
    }
}
