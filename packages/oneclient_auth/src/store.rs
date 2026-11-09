use std::collections::HashMap;

use uuid::Uuid;

use oneclient_common::paths;
use oneclient_events::EventBus;

use crate::data::{AccountKind, MinecraftAccount};
use crate::error::{AuthError, AuthResult};
use crate::offline::{offline_account, validate_offline_username_with_override};

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct CredentialsStore {
    pub users: HashMap<Uuid, MinecraftAccount>,
    pub default_user: Option<Uuid>,
}

impl CredentialsStore {
    #[tracing::instrument(level = "debug")]
    pub async fn load() -> AuthResult<Self> {
        let path = paths::auth_file()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        match polyio::read_json(&path).await {
            Ok(store) => Ok(store),
            Err(err) => {
                tracing::warn!("failed to read auth file: {err}");
                Ok(Self::default())
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn save(&self) -> AuthResult<()> {
        let path = paths::auth_file()?;

        polyio::write_json_atomic(&path, self).await?;
        Ok(())
    }

    pub fn has_microsoft_account(&self) -> bool {
        self.users
            .values()
            .any(|account| account.kind == AccountKind::Microsoft)
    }

    pub fn list_accounts(&self) -> Vec<MinecraftAccount> {
        self.users.values().cloned().collect()
    }

    pub fn get_account(&self, id: Uuid) -> Option<&MinecraftAccount> {
        self.users.get(&id)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(username = %account.username))]
    pub async fn commit_account(
        &mut self,
        account: MinecraftAccount,
        events: &EventBus,
    ) -> AuthResult<MinecraftAccount> {
        let account = self.prepare_account(account)?;
        self.users.insert(account.id, account.clone());

        if self.default_user.is_none() {
            self.default_user = Some(account.id);
        }

        self.save().await?;
        tracing::debug!("committed account to credentials store");
        events
            .notify("Account added")
            .body(format!("Signed in as {}", account.username))
            .send();
        Ok(account)
    }

    fn prepare_account(&self, mut account: MinecraftAccount) -> AuthResult<MinecraftAccount> {
        let game_id = account.game_id();
        let existing = self.users.values().find(|entry| {
            entry.kind == account.kind && entry.game_id() == game_id
                && entry.custom.as_ref().map(|c| &c.server.api_url)
                    == account.custom.as_ref().map(|c| &c.server.api_url)
        });
        if self.users.values().any(|entry| {
            entry.kind == account.kind
                && entry.username.eq_ignore_ascii_case(&account.username)
                && existing.is_none_or(|same| same.id != entry.id)
        }) {
            return Err(AuthError::DuplicateUsername { username: account.username });
        }
        if let Some(existing) = existing {
            account.id = existing.id;
            account.profile_id = existing.profile_id;
        } else if self.users.contains_key(&account.id) {
            account.profile_id = Some(game_id);
            loop {
                account.id = Uuid::new_v4();
                if !self.users.contains_key(&account.id) { break; }
            }
        }
        Ok(account)
    }

    pub fn add_offline_account(&mut self, username: String) -> AuthResult<MinecraftAccount> {
        let account = self.insert_offline_account(username, false)?;
        Ok(account)
    }

    #[tracing::instrument(level = "debug", skip(self), fields(username = %username))]
    pub async fn add_offline_account_and_save(
        &mut self,
        username: String,
    ) -> AuthResult<MinecraftAccount> {
        let account = self.insert_offline_account(username, false)?;
        self.save().await?;
        Ok(account)
    }

    pub fn add_offline_account_with_override(
        &mut self,
        username: String,
        allow_invalid: bool,
    ) -> AuthResult<MinecraftAccount> {
        self.insert_offline_account(username, allow_invalid)
    }

    pub async fn add_offline_account_with_override_and_save(
        &mut self,
        username: String,
        allow_invalid: bool,
    ) -> AuthResult<MinecraftAccount> {
        let account = self.insert_offline_account(username, allow_invalid)?;
        self.save().await?;
        Ok(account)
    }

    fn insert_offline_account(
        &mut self,
        username: String,
        allow_invalid: bool,
    ) -> AuthResult<MinecraftAccount> {
        validate_offline_username_with_override(&username, allow_invalid)?;

        if self
            .users
            .values()
            .any(|u| u.kind == AccountKind::Offline && u.username.eq_ignore_ascii_case(&username))
        {
            return Err(AuthError::DuplicateUsername { username });
        }

        let account = self.prepare_account(offline_account(username))?;
        self.users.insert(account.id, account.clone());
        if self.default_user.is_none() {
            self.default_user = Some(account.id);
        }
        Ok(account)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(%account.id))]
    pub async fn commit_refreshed_account(&mut self, account: MinecraftAccount) -> AuthResult<()> {
        self.users.insert(account.id, account);
        self.save().await?;
        tracing::debug!("stored refreshed account");
        Ok(())
    }

    /// Keeps the entry and its default slot so the UI can offer a sign-in for
    /// this exact account The refresh token is kept too Microsoft sometimes
    /// rejects a good one and the next renewal retries it
    #[tracing::instrument(level = "debug", skip(self), fields(%id))]
    pub async fn mark_signed_out(&mut self, id: Uuid) -> AuthResult<()> {
        let Some(account) = self.users.get_mut(&id) else {
            return Err(AuthError::AccountNotFound(id));
        };

        account.signed_out = true;
        account.access_token.clear();
        self.save().await?;
        Ok(())
    }

    /// Debug aid swaps in a token Microsoft will reject and backdates expiry so
    /// the next launch walks the real `invalid_grant` path
    #[tracing::instrument(level = "debug", skip(self), fields(%id))]
    pub async fn corrupt_refresh_token(&mut self, id: Uuid) -> AuthResult<()> {
        let Some(account) = self.users.get_mut(&id) else {
            return Err(AuthError::AccountNotFound(id));
        };

        account.refresh_token = "oneclient-debug-invalid-refresh-token".to_string();
        account.expires = chrono::DateTime::UNIX_EPOCH;
        self.save().await?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self), fields(%id))]
    pub async fn remove_account(&mut self, id: Uuid) -> AuthResult<Option<MinecraftAccount>> {
        let removed = self.users.remove(&id);

        if self.default_user == Some(id) {
            self.default_user = self.users.keys().copied().next();
        }

        self.save().await?;
        Ok(removed)
    }

    #[tracing::instrument(level = "debug", skip(self), fields(?id))]
    pub async fn set_default_user(&mut self, id: Option<Uuid>) -> AuthResult<()> {
        if let Some(id) = id
            && !self.users.contains_key(&id)
        {
            return Err(AuthError::AccountNotFound(id));
        }

        self.default_user = id;
        self.save().await?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn resolve_default_id(&mut self) -> AuthResult<Option<Uuid>> {
        let id = self
            .default_user
            .or_else(|| self.users.keys().copied().next());

        let Some(id) = id else {
            return Ok(None);
        };

        if !self.users.contains_key(&id) {
            return Ok(None);
        }

        if self.default_user != Some(id) {
            self.default_user = Some(id);
            self.save().await?;
        }

        Ok(Some(id))
    }

    /// Never touches the network callers that need a usable token go through
    /// [`crate::AuthService::account_for_launch`]
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn default_account(&mut self) -> AuthResult<Option<MinecraftAccount>> {
        let Some(id) = self.resolve_default_id().await? else {
            return Ok(None);
        };
        let Some(account) = self.users.get(&id).cloned() else {
            return Ok(None);
        };

        Ok(Some(account))
    }
}

/// A transient failure must keep the existing token discarding it because
/// Wi-Fi dropped would sign the user out of a working account
pub(crate) fn is_transient_auth_error(err: &AuthError) -> bool {
    if let AuthError::Custom(error) = err {
        return error.is_transient();
    }
    if let AuthError::Elyby(error) = err {
        return error.is_transient();
    }
    matches!(
        err,
        AuthError::Minecraft(crate::error::MinecraftAuthError::RequestError { source, .. })
            if source.is_connect() || source.is_timeout()
    )
}

#[cfg(test)]
mod provider_identity_tests {
    use super::*;

    #[test]
    fn same_name_requires_different_kinds_and_preserves_game_identity() {
        let mut store = CredentialsStore::default();
        let offline = store.add_offline_account("liwwyy".into()).unwrap();
        let mut elyby = offline.clone();
        elyby.kind = AccountKind::Elyby;
        let elyby = store.prepare_account(elyby).unwrap();
        assert_ne!(elyby.id, offline.id);
        assert_eq!(elyby.game_id(), offline.game_id());
        store.users.insert(elyby.id, elyby.clone());
        let mut duplicate = elyby.clone();
        duplicate.id = Uuid::new_v4();
        duplicate.profile_id = None;
        duplicate.username = "LIWWYY".into();
        assert!(matches!(store.prepare_account(duplicate), Err(AuthError::DuplicateUsername { .. })));
        let signed_in_again = store.prepare_account(elyby.clone()).unwrap();
        assert_eq!(signed_in_again.id, elyby.id);
        let raw = serde_json::to_string(&store).unwrap();
        let restored: CredentialsStore = serde_json::from_str(&raw).unwrap();
        assert_eq!(restored.users[&elyby.id].game_id(), offline.game_id());
        assert_eq!(restored.default_user, Some(offline.id));
    }

    #[test]
    fn legacy_accounts_need_no_identity_rewrite() {
        let account = offline_account("Legacy".into());
        let mut json = serde_json::to_value(&account).unwrap();
        json.as_object_mut().unwrap().remove("profile_id");
        let restored: MinecraftAccount = serde_json::from_value(json).unwrap();
        assert_eq!(restored.id, account.id);
        assert_eq!(restored.game_id(), account.id);
    }
}
