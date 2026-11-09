use oneclient_common::paths;
use oneclient_core::{LauncherState, settings::LauncherSettings};
use oneclient_events::EventBus;

/// Run in its own test process because launcher paths are initialized once.
#[tokio::test]
async fn a_shared_library_reuses_instances_and_java_while_fork_configuration_stays_private() {
    let root = polyio::testing::ScratchDir::new("shared_library_startup");
    let private = root.join("OneForAllLauncher");
    let library = root.join("custom-games");
    let next = root.join("another-library");
    std::fs::create_dir_all(&private).unwrap();
    // Set the shared override first; set_launcher_dir supplies only the remaining
    // private override when the data override is already initialized.
    paths::set_data_dir(library.clone());
    paths::set_launcher_dir(private.clone());
    let settings = LauncherSettings {
        data_dir: Some(library.clone()),
        reused_data_dir: true,
        seen_onboarding: true,
        discord_enabled: false,
        auto_update: true,
        ..Default::default()
    };
    oneclient_core::settings::store::save_settings(&settings)
        .await
        .unwrap();
    let pool = oneclient_db::connect(library.join(paths::DATABASE_FILE))
        .await
        .unwrap();
    sqlx::query("INSERT INTO clusters (name, folder_name, mc_version, stage) VALUES ('Stock instance', 'stock-instance', '1.8.9', 3)")
        .execute(&pool).await.unwrap();
    let java = library.join("metadata/java/runtime/bin/java");
    std::fs::create_dir_all(java.parent().unwrap()).unwrap();
    std::fs::write(&java, b"installed Java fixture").unwrap();
    oneclient_db::dao::java::insert(
        &pool,
        java.to_str().unwrap(),
        8,
        "8.0",
        "mojang",
        "x86_64",
        false,
        1,
    )
    .await
    .unwrap();
    let upstream_auth = b"upstream accounts must not be loaded";
    std::fs::write(library.join("auth.json"), upstream_auth).unwrap();
    let upstream_settings = b"{\"theme\":\"light\",\"auto_update\":false}";
    std::fs::write(library.join("settings.json"), upstream_settings).unwrap();
    pool.close().await;

    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let state = LauncherState::new(EventBus::new(tx)).await.unwrap();
    let instances = state.clusters.list().await.unwrap();
    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0].name, "Stock instance");
    assert_eq!(instances[0].stage, oneclient_core::ClusterStage::Ready);
    assert_eq!(
        state.java.list_runtimes().await.unwrap()[0].absolute_path,
        java.to_str().unwrap()
    );
    for path in [
        paths::java_dir().unwrap(),
        paths::clusters_dir().unwrap(),
        paths::assets_dir().unwrap(),
        paths::libraries_dir().unwrap(),
        paths::versions_dir().unwrap(),
        paths::packages_cache_dir().unwrap(),
        paths::shared_minecraft_dir().unwrap(),
    ] {
        assert!(
            path.starts_with(&library),
            "{} must use the game library",
            path.display()
        );
    }
    for path in [
        paths::settings_file().unwrap(),
        paths::auth_file().unwrap(),
        paths::logs_dir().unwrap(),
        paths::themes_dir().unwrap(),
    ] {
        assert!(path.starts_with(&private));
    }
    assert!(!private.join(paths::DATABASE_FILE).exists());

    let plan = oneclient_core::relocate::plan_library(&state, &next)
        .await
        .unwrap();
    oneclient_core::relocate::use_library(&state, &plan)
        .await
        .unwrap();
    let saved = oneclient_core::settings::store::load_settings(None).await;
    assert_eq!(saved.data_dir, Some(next));
    assert!(saved.reused_data_dir);
    assert!(saved.previous_data_dir.is_none());
    assert!(saved.auto_update);
    assert!(saved.seen_onboarding);
    assert_eq!(saved.theme, settings.theme);
    assert_eq!(
        paths::data_dir().unwrap(),
        library,
        "switches apply only after restarting"
    );
    assert_eq!(
        std::fs::read(library.join("auth.json")).unwrap(),
        upstream_auth
    );
    assert_eq!(
        std::fs::read(library.join("settings.json")).unwrap(),
        upstream_settings
    );
    assert_eq!(std::fs::read(java).unwrap(), b"installed Java fixture");
    assert_eq!(state.clusters.list().await.unwrap().len(), 1);
    oneclient_core::shutdown(&state).await;
}
