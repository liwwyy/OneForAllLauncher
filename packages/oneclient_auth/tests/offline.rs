use chrono::{TimeDelta, Utc};
use oneclient_auth::{AccountKind, CredentialsStore, offline_account, offline_uuid};
use uuid::Uuid;

fn isolate_launcher_dir() {
    oneclient_common::paths::set_launcher_dir(
        std::env::temp_dir().join(format!("oneclient-auth-test-{}", Uuid::new_v4())),
    );
}

fn fake_microsoft_account(username: &str) -> oneclient_auth::MinecraftAccount {
    let mut account = offline_account(username.to_string());
    account.kind = AccountKind::Microsoft;
    account.access_token = "access".into();
    account.refresh_token = "refresh".into();
    account
}

#[test]
fn offline_uuid_matches_vanilla_algorithm() {
    let id = offline_uuid("Notch");

    assert_eq!(
        id,
        Uuid::parse_str("b50ad385-829d-3141-a216-7e7d7539ba7f").unwrap()
    );
    assert_ne!(id, offline_uuid("Steve"));
}

#[tokio::test]
async fn offline_account_is_default_without_microsoft() {
    isolate_launcher_dir();

    let mut store = CredentialsStore::default();
    let account = store.add_offline_account("Steve".into()).unwrap();
    assert_eq!(account.kind, AccountKind::Offline);
    assert_eq!(account.id, offline_uuid("Steve"));
    assert_eq!(store.default_user, Some(account.id));
    assert_eq!(
        store.default_account().await.unwrap().unwrap().id,
        account.id
    );
    assert!(!store.has_microsoft_account());
}

#[tokio::test]
async fn offline_account_allowed_when_microsoft_exists() {
    isolate_launcher_dir();

    let mut store = CredentialsStore::default();
    let msa = fake_microsoft_account("MsaUser");
    store.users.insert(msa.id, msa);

    let offline = store.add_offline_account("Steve".into()).unwrap();
    assert_eq!(offline.kind, AccountKind::Offline);
    assert_eq!(offline.username, "Steve");
}

#[tokio::test]
async fn default_offline_survives_last_microsoft_removal() {
    isolate_launcher_dir();

    let mut store = CredentialsStore::default();
    let msa = fake_microsoft_account("MsaUser");
    let msa_id = msa.id;
    store.users.insert(msa_id, msa);

    let offline = store.add_offline_account("Steve".into()).unwrap();
    store.default_user = Some(offline.id);
    store.users.remove(&msa_id);

    let account = store.default_account().await.unwrap().unwrap();
    assert_eq!(account.id, offline.id);
    assert_eq!(account.kind, AccountKind::Offline);
}

#[test]
fn token_expiring_imminently_counts_as_expired() {
    let mut account = fake_microsoft_account("MsaUser");

    account.expires = Utc::now() + TimeDelta::hours(1);
    assert!(!account.is_expired(), "a token with an hour left is usable");

    account.expires = Utc::now() + TimeDelta::seconds(5);
    assert!(
        account.is_expired(),
        "a token about to lapse must be renewed before launching"
    );

    account.expires = Utc::now() - TimeDelta::hours(1);
    assert!(account.is_expired());
}

#[tokio::test]
async fn default_account_returns_expired_token_without_refreshing() {
    isolate_launcher_dir();

    let mut store = CredentialsStore::default();
    let mut msa = fake_microsoft_account("MsaUser");
    msa.expires = Utc::now() - TimeDelta::hours(5);
    let msa_id = msa.id;
    store.users.insert(msa_id, msa);

    store.default_user = Some(msa_id);

    let account = store
        .default_account()
        .await
        .expect("an expired token must not fail the read path")
        .expect("the only account should be the default");

    assert_eq!(account.id, msa_id);
    assert!(account.is_expired());
}

#[tokio::test]
async fn offline_only_service_launches_and_refreshes_without_network() {
    use oneclient_auth::AuthService;
    use oneclient_events::EventBus;
    use oneclient_net::{NetConfig, RequestClient};

    let mut store = CredentialsStore::default();
    let offline = store.add_offline_account("Alex".into()).unwrap();
    // Offline accounts must remain usable even if their timestamp is expired.
    store.users.get_mut(&offline.id).unwrap().expires = Utc::now() - TimeDelta::days(1);
    let (events, _rx) = EventBus::channel();
    let service = AuthService::with_store(
        store,
        RequestClient::new(NetConfig::default()).unwrap(),
        events,
    );
    for account in [
        service.account_for_launch(offline.id).await.unwrap(),
        service.default_account_for_launch().await.unwrap().unwrap(),
        service.refresh_account(offline.id).await.unwrap(),
    ] {
        assert_eq!(account.id, offline_uuid("Alex"));
        assert_eq!(account.kind, AccountKind::Offline);
        assert!(account.refresh_token.is_empty());
    }
}

#[test]
fn offline_only_store_round_trips_and_validates_usernames() {
    let mut store = CredentialsStore::default();
    assert!(store.add_offline_account("bad name".into()).is_err());
    let account = store.add_offline_account("Steve".into()).unwrap();
    assert!(store.add_offline_account("steve".into()).is_err());
    let saved = serde_json::to_string(&store).unwrap();
    let restored: CredentialsStore = serde_json::from_str(&saved).unwrap();
    assert_eq!(restored.default_user, Some(account.id));
    assert_eq!(restored.users[&account.id].kind, AccountKind::Offline);
    assert!(!restored.has_microsoft_account());
}

#[tokio::test]
async fn empty_store_cannot_supply_launch_account() {
    use oneclient_auth::AuthService;
    use oneclient_events::EventBus;
    use oneclient_net::{NetConfig, RequestClient};
    let (events, _rx) = EventBus::channel();
    let service = AuthService::with_store(
        CredentialsStore::default(),
        RequestClient::new(NetConfig::default()).unwrap(),
        events,
    );
    assert!(
        service
            .default_account_for_launch()
            .await
            .unwrap()
            .is_none()
    );
}
