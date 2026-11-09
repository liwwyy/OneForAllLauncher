//! Construct [`AuthService`] in the composition layer and pass it down nothing
//! in here reaches for a global or a database
//! Accounts persist to `auth.json`

mod custom;
mod data;
mod diagnostics;
mod elyby;
mod error;
mod msa;
mod offline;
mod online_username;
mod service;
mod store;

pub use data::{
    AccountKind, BrowserLogin, DeviceCodeLogin, ElybyLoginSession, MicrosoftLoginSession,
    MinecraftAccount,
};
pub use diagnostics::{AuthErrorGuidance, AuthErrorSample, diagnose_auth_error, preview_samples};
pub use elyby::{DEFAULT_ELYBY_CLIENT_ID, ELYBY_AUTHLIB_URL, ElybyAuthError};
pub use error::{AuthError, AuthResult, MinecraftAuthError, MinecraftAuthStep};
pub use msa::PendingBrowserLogin;
pub use offline::{
    offline_account, offline_username_input_allowed, offline_uuid, random_offline_characters,
    random_offline_username, validate_offline_username, validate_offline_username_with_override,
};
pub use online_username::lookup_online_username;
pub use service::ELYBY_LOGIN_PROGRESS;
pub use service::{AuthService, MICROSOFT_LOGIN_PROGRESS};
pub use store::CredentialsStore;

pub use custom::{
    CustomAccountData, CustomAuthError, CustomLoginResponse, CustomProfile, CustomServer,
    custom_auth_paths_on_server_change, discover_custom_server,
};
