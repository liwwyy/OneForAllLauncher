//! Opt-in validation against a real launcher directory, opened strictly read-only.
//! ONEFORALL_EXPORT_SAMPLE_ROOT=/path/to/OneClient cargo test -p oneclient_core --test export_sample -- --ignored
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use oneclient_core::export::{ExportFormat, ExportSource, export_instance};
use oneclient_core::settings::LauncherSettings;
use oneclient_db::models::{ArtifactRow, ClusterArtifactRow, ClusterRow};

#[tokio::test]
#[ignore = "requires an explicitly supplied launcher data directory"]
async fn export_real_instance_read_only() {
    let root = PathBuf::from(
        std::env::var("ONEFORALL_EXPORT_SAMPLE_ROOT").expect("Set ONEFORALL_EXPORT_SAMPLE_ROOT"),
    );
    let name =
        std::env::var("ONEFORALL_EXPORT_SAMPLE_INSTANCE").unwrap_or_else(|_| "1.8.9 OC".into());
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(root.join("user_data.db"))
                .read_only(true)
                .create_if_missing(false),
        )
        .await
        .unwrap();
    let row: ClusterRow = sqlx::query_as("SELECT * FROM clusters WHERE name = ?")
        .bind(&name)
        .fetch_one(&pool)
        .await
        .unwrap();
    let cluster = oneclient_core::Cluster::try_from_row(row).unwrap();
    let dir = root.join("clusters").join(&cluster.folder_name);
    let game_dir = if cluster.is_isolated() || dir.join(".dedicated_directory").is_file() {
        dir.clone()
    } else {
        root.join(".minecraft")
    };
    let settings: LauncherSettings =
        serde_json::from_slice(&std::fs::read(root.join("settings.json")).unwrap()).unwrap();
    let profile = oneclient_cluster::profiles::resolve_cluster_profile(
        &pool,
        &settings.global_game_settings,
        cluster.setting_profile_name.as_deref(),
    )
    .await
    .unwrap();
    let mut source = ExportSource {
        name: cluster.name.clone(),
        mc_version: cluster.mc_version.clone(),
        loader: cluster.mc_loader,
        loader_version: cluster.mc_loader_version.clone(),
        mods_dir: dir.join("mods"),
        game_dir,
        settings: profile,
        cached_mods: BTreeMap::new(),
        disabled_mods: BTreeSet::new(),
    };
    let links: Vec<ClusterArtifactRow> =
        sqlx::query_as("SELECT * FROM cluster_artifacts WHERE cluster_id = ?")
            .bind(cluster.id)
            .fetch_all(&pool)
            .await
            .unwrap();
    for link in links {
        let artifact: ArtifactRow = sqlx::query_as("SELECT * FROM artifacts WHERE hash = ?")
            .bind(&link.hash)
            .fetch_one(&pool)
            .await
            .unwrap();
        if artifact.content_type != oneclient_common::domain::ContentType::Mod as i64 {
            continue;
        }
        if link.enabled == 0 {
            source.disabled_mods.insert(link.cluster_file_name);
        } else {
            source
                .cached_mods
                .insert(link.cluster_file_name, root.join(artifact.path));
        }
    }
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/validation-2.6.2");
    std::fs::create_dir_all(&output).unwrap();
    let mods = export_instance(source.clone(), ExportFormat::Mods, &output.join("mods.zip"))
        .await
        .unwrap();
    let prism = export_instance(
        source,
        ExportFormat::Prism,
        &output.join("instance-prism.zip"),
    )
    .await
    .unwrap();
    assert_eq!(mods.mods, prism.mods);
    assert!(prism.mods > 0);
    println!(
        "Exported {} mods; Prism archive contains {} files",
        prism.mods, prism.files
    );
    pool.close().await;
}
