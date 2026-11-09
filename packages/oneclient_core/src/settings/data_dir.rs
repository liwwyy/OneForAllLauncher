use std::ffi::OsString;
use std::path::{Path, PathBuf};

use oneclient_common::paths;
use sqlx::{Connection, sqlite::SqliteConnectOptions};

use super::launcher::LauncherSettings;
use super::store::save_settings;

pub const FOLDER_NAME: &str = "OneClient";
const LOW_SPACE_BYTES: u64 = 5 * 1000 * 1000 * 1000;

const PROBE_NAME: &str = ".oneclient_write_test";

const OS_CLUTTER: &[&str] = &[
    ".DS_Store",
    ".localized",
    ".Spotlight-V100",
    ".Trashes",
    ".fseventsd",
    "desktop.ini",
    "Thumbs.db",
    "$RECYCLE.BIN",
    "System Volume Information",
];

/// Read only the upstream location preference; never import its accounts/settings.
pub async fn oneclient_folder() -> Result<PathBuf, String> {
    let config = paths::oneclient_default_dir().map_err(|error| error.to_string())?;
    oneclient_folder_from(&config).await
}

async fn oneclient_folder_from(config: &Path) -> Result<PathBuf, String> {
    let file = config.join(paths::SETTINGS_FILE);
    if !polyio::try_exists(&file)
        .await
        .map_err(|error| error.to_string())?
    {
        return Ok(config.to_path_buf());
    }
    #[derive(serde::Deserialize)]
    struct Location {
        data_dir: Option<PathBuf>,
    }
    let raw = polyio::read(&file)
        .await
        .map_err(|error| error.to_string())?;
    let location: Location = serde_json::from_slice(&raw).map_err(|error| {
        format!(
            "Couldn't read OneClient's folder from {}: {error}",
            file.display()
        )
    })?;
    match location.data_dir {
        None => Ok(config.to_path_buf()),
        Some(dir) if !dir.is_absolute() => Err(format!(
            "OneClient's game folder must be an absolute path: {}",
            dir.display()
        )),
        Some(dir)
            if !polyio::try_exists(&dir)
                .await
                .map_err(|error| error.to_string())? =>
        {
            Err(format!(
                "OneClient's game folder is unavailable: {}. Connect its drive before selecting it.",
                dir.display()
            ))
        }
        Some(dir) => Ok(dir),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDirCheck {
    pub path: PathBuf,
    pub warning: Option<String>,
}

pub async fn looks_empty(dir: &Path, ignored: &[OsString]) -> bool {
    let Ok(mut entries) = polyio::read_dir(dir).await else {
        return true;
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name();

        if ignored.contains(&name) {
            continue;
        }

        let Some(name) = name.to_str() else {
            return false;
        };

        if !OS_CLUTTER
            .iter()
            .any(|junk| junk.eq_ignore_ascii_case(name))
        {
            return false;
        }
    }

    true
}

pub async fn resolve(picked: &Path) -> Result<PathBuf, String> {
    if picked.file_name().is_some_and(|name| name == FOLDER_NAME)
        || looks_empty(picked, &[]).await
        || contains_launcher_library(picked).await?
    {
        Ok(picked.to_path_buf())
    } else {
        Ok(picked.join(FOLDER_NAME))
    }
}

/// Reuse explicitly picked existing libraries, including custom roots named "data".
/// The read-only probe must not initialize or migrate a database during browsing.
pub(crate) async fn contains_launcher_library(dir: &Path) -> Result<bool, String> {
    let file = dir.join(paths::DATABASE_FILE);
    if !file.is_file() {
        return Ok(false);
    }
    let options = SqliteConnectOptions::new().filename(&file).read_only(true);
    let mut db = sqlx::SqliteConnection::connect_with(&options)
        .await
        .map_err(|error| format!("Couldn't inspect {}: {error}", file.display()))?;
    let result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sqlite_schema WHERE type = 'table' AND name IN ('clusters', 'setting_profiles')"
    ).fetch_one(&mut db).await;
    db.close().await.map_err(|error| error.to_string())?;
    result
        .map(|count| count == 2)
        .map_err(|error| format!("Couldn't inspect {}: {error}", file.display()))
}

pub fn default_path() -> Result<PathBuf, String> {
    paths::config_dir()
        .map(Path::to_path_buf)
        .map_err(|err| format!("Couldn't work out where OneClient keeps its settings: {err}"))
}

#[must_use]
pub fn is_default(dir: &Path) -> bool {
    default_path().is_ok_and(|home| home == dir)
}

pub async fn check(picked: &Path) -> Result<DataDirCheck, String> {
    check_exact(&resolve(picked).await?).await
}

pub async fn check_exact(path: &Path) -> Result<DataDirCheck, String> {
    let path = path.to_path_buf();

    let existed = polyio::try_exists(&path).await.unwrap_or(false);
    polyio::create_dir_all(&path)
        .await
        .map_err(|err| format!("Couldn't create {}: {err}", path.display()))?;

    let probe = path.join(PROBE_NAME);
    let writable = polyio::write(&probe, b"".as_slice()).await;
    polyio::remove_file(&probe).await.ok();

    if let Err(err) = writable {
        if !existed {
            polyio::remove_dir(&path).await.ok();
        }
        return Err(format!("Can't write to {}: {err}", path.display()));
    }

    if !existed {
        polyio::remove_dir(&path).await.ok();
    }

    Ok(DataDirCheck {
        warning: low_space_warning(&path),
        path,
    })
}

pub async fn apply(picked: Option<PathBuf>) -> Result<(), String> {
    let checked = match picked {
        Some(path) => Some(check(&path).await?),
        None => None,
    };
    let reused = match &checked {
        Some(checked) => contains_launcher_library(&checked.path).await?,
        None => false,
    };
    apply_checked(checked, reused).await
}

/// OneClient's default may use an older layout whose leaf is not "OneClient".
pub async fn apply_exact(path: PathBuf) -> Result<(), String> {
    apply_checked(Some(check_exact(&path).await?), true).await
}

async fn apply_checked(checked: Option<DataDirCheck>, reused: bool) -> Result<(), String> {
    let mut settings = LauncherSettings::default();
    if let Some(checked) = checked {
        settings.reused_data_dir = reused;
        polyio::create_dir_all(&checked.path)
            .await
            .map_err(|err| format!("Couldn't create {}: {err}", checked.path.display()))?;
        settings.data_dir = Some(checked.path);
    }
    save_settings(&settings)
        .await
        .map_err(|err| format!("Couldn't save your settings: {err}"))?;
    if let Some(chosen) = settings.data_dir {
        paths::set_data_dir(chosen);
    }
    Ok(())
}

fn low_space_warning(path: &Path) -> Option<String> {
    let available = available_space(path)?;
    if available >= LOW_SPACE_BYTES {
        return None;
    }

    Some(format!(
        "Only {} free here. Minecraft, its libraries and a Java runtime need a few GB before you \
		 install anything.",
        crate::storage::format_bytes(available)
    ))
}

pub(crate) fn available_space(path: &Path) -> Option<u64> {
    let target = polyio::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

    sysinfo::Disks::new_with_refreshed_list()
        .iter()
        .filter(|disk| target.starts_with(disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().as_os_str().len())
        .map(sysinfo::Disk::available_space)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_folder_with_things_in_it_gets_a_folder_of_its_own() {
        let root = polyio::testing::ScratchDir::new("data_dir_busy");
        let dir = root.path();
        polyio::create_dir_all(dir).await.unwrap();
        polyio::write(dir.join("holiday.jpg"), b"mine".as_slice())
            .await
            .unwrap();

        assert_eq!(resolve(dir).await.unwrap(), dir.join(FOLDER_NAME));

        std::fs::remove_dir_all(dir).ok();
    }

    #[tokio::test]
    async fn an_empty_folder_is_used_as_it_stands() {
        let root = polyio::testing::ScratchDir::new("data_dir_empty");
        let dir = root.path();
        polyio::create_dir_all(dir).await.unwrap();

        assert_eq!(
            resolve(dir).await.unwrap(),
            dir.to_path_buf(),
            "a folder made for this should not get another one inside it"
        );

        std::fs::remove_dir_all(dir).ok();
    }

    #[tokio::test]
    async fn a_folder_the_finder_has_been_in_still_counts_as_empty() {
        let root = polyio::testing::ScratchDir::new("data_dir_clutter");
        let dir = root.path();
        polyio::create_dir_all(dir).await.unwrap();
        polyio::write(dir.join(".DS_Store"), b"junk".as_slice())
            .await
            .unwrap();

        assert_eq!(
            resolve(dir).await.unwrap(),
            dir.to_path_buf(),
            "the folder looks empty to the person who picked it"
        );

        std::fs::remove_dir_all(dir).ok();
    }

    #[tokio::test]
    async fn resolving_twice_does_not_nest() {
        let root = polyio::testing::ScratchDir::new("data_dir_nest");
        let dir = root.path();
        polyio::create_dir_all(dir).await.unwrap();
        polyio::write(dir.join("holiday.jpg"), b"mine".as_slice())
            .await
            .unwrap();

        let once = resolve(dir).await.unwrap();
        assert_eq!(resolve(&once).await.unwrap(), once);

        std::fs::remove_dir_all(dir).ok();
    }

    #[tokio::test]
    async fn the_settings_that_stayed_behind_do_not_make_the_default_folder_look_busy() {
        let root = polyio::testing::ScratchDir::new("data_dir_settled");
        let dir = root.path();
        polyio::create_dir_all(dir).await.unwrap();
        polyio::write(dir.join("settings.json"), b"{}".as_slice())
            .await
            .unwrap();

        let settled = [OsString::from("settings.json")];

        assert!(
            looks_empty(dir, &settled).await,
            "moving back in has to be allowed over the settings that never left"
        );

        polyio::create_dir_all(dir.join("clusters")).await.unwrap();

        assert!(
            !looks_empty(dir, &settled).await,
            "game data already sitting there is another matter"
        );

        std::fs::remove_dir_all(dir).ok();
    }

    #[tokio::test]
    async fn a_writable_folder_passes_and_is_not_left_behind() {
        let root = polyio::testing::ScratchDir::new("data_dir_check");
        polyio::create_dir_all(root.path()).await.unwrap();

        polyio::write(root.join("holiday.jpg"), b"mine".as_slice())
            .await
            .unwrap();

        let checked = check(root.path()).await.unwrap();
        assert_eq!(checked.path, root.join(FOLDER_NAME));
        assert!(
            !checked.path.exists(),
            "a folder the user has not confirmed yet is litter"
        );

        std::fs::remove_dir_all(root.path()).ok();
    }

    #[tokio::test]
    async fn an_existing_oneclient_library_is_reused_without_nesting_or_config_changes() {
        let root = polyio::testing::ScratchDir::new("oneclient_shared_library");
        let dir = root.join("custom-data");
        let pool = oneclient_db::connect(dir.join(paths::DATABASE_FILE))
            .await
            .unwrap();
        polyio::write(dir.join("settings.json"), b"{\"auto_update\":false}")
            .await
            .unwrap();
        polyio::write(dir.join("auth.json"), b"upstream accounts")
            .await
            .unwrap();
        assert_eq!(check(&dir).await.unwrap().path, dir);
        assert_eq!(check_exact(&dir).await.unwrap().path, dir);
        assert!(!dir.join(FOLDER_NAME).exists());
        assert_eq!(
            polyio::read(dir.join("settings.json")).await.unwrap(),
            b"{\"auto_update\":false}"
        );
        assert_eq!(
            polyio::read(dir.join("auth.json")).await.unwrap(),
            b"upstream accounts"
        );
        pool.close().await;
    }

    #[tokio::test]
    async fn an_unrelated_database_does_not_turn_a_folder_into_a_launcher_library() {
        let root = polyio::testing::ScratchDir::new("data_dir_other_database");
        polyio::create_dir_all(root.path()).await.unwrap();
        let options = SqliteConnectOptions::new()
            .filename(root.join(paths::DATABASE_FILE))
            .create_if_missing(true);
        let mut db = sqlx::SqliteConnection::connect_with(&options)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE unrelated (value TEXT)")
            .execute(&mut db)
            .await
            .unwrap();
        db.close().await.unwrap();
        assert_eq!(resolve(root.path()).await.unwrap(), root.join(FOLDER_NAME));
    }

    #[tokio::test]
    async fn upstream_selection_follows_its_custom_folder_without_importing_configuration() {
        let root = polyio::testing::ScratchDir::new("upstream_location");
        let config = root.join("config");
        let library = root.join("games");
        polyio::create_dir_all(&config).await.unwrap();
        polyio::create_dir_all(&library).await.unwrap();
        let raw = serde_json::to_vec(&serde_json::json!({
            "data_dir": library,
            "auto_update": false,
            "theme": "light"
        }))
        .unwrap();
        polyio::write(config.join(paths::SETTINGS_FILE), &raw)
            .await
            .unwrap();
        assert_eq!(oneclient_folder_from(&config).await.unwrap(), library);
        assert_eq!(
            polyio::read(config.join(paths::SETTINGS_FILE))
                .await
                .unwrap(),
            raw
        );
        assert!(!config.join(paths::DATABASE_FILE).exists());
        assert!(!library.join(paths::DATABASE_FILE).exists());
        polyio::remove_dir(&library).await.unwrap();
        assert!(
            oneclient_folder_from(&config)
                .await
                .unwrap_err()
                .contains("unavailable")
        );
        assert!(
            !library.exists(),
            "an unavailable drive must not become a new empty library"
        );
    }

    #[tokio::test]
    async fn upstream_without_a_custom_location_uses_its_release_or_legacy_root() {
        let root = polyio::testing::ScratchDir::new("upstream_default_location");
        assert_eq!(
            oneclient_folder_from(root.path()).await.unwrap(),
            root.path()
        );
        polyio::create_dir_all(root.path()).await.unwrap();
        for raw in [b"{}".as_slice(), b"{\"data_dir\":null}".as_slice()] {
            polyio::write(root.join(paths::SETTINGS_FILE), raw)
                .await
                .unwrap();
            assert_eq!(
                oneclient_folder_from(root.path()).await.unwrap(),
                root.path()
            );
        }
        polyio::write(
            root.join(paths::SETTINGS_FILE),
            b"{\"data_dir\":\"relative\"}",
        )
        .await
        .unwrap();
        assert!(oneclient_folder_from(root.path()).await.is_err());
        polyio::write(root.join(paths::SETTINGS_FILE), b"invalid json")
            .await
            .unwrap();
        assert!(oneclient_folder_from(root.path()).await.is_err());
    }
}
