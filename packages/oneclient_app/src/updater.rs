use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use cargo_packager_updater::{Config, Update};
use oneclient_events::{Choice, EventBus, Prompt};
use uuid::Uuid;

use crate::constants::{RELEASES_URL, UPDATER_ENDPOINT, UPDATER_PUBKEY};

pub const UPDATE_CHOICE_INSTALL: &str = "update.install";

enum UpdateAnswer {
    Install,
}

fn update_prompt(version: &str) -> Prompt<UpdateAnswer> {
    Prompt::new(
        "Update available",
        format!("OneForAllLauncher {version} is ready to install. Download and install it now?"),
    )
    .option(
        Choice::primary(UPDATE_CHOICE_INSTALL, "Install"),
        UpdateAnswer::Install,
    )
    .dismiss("Not now")
}

const PROGRESS_STEP: u64 = 256 * 1024;

static UPDATE_RUNNING: AtomicBool = AtomicBool::new(false);

struct UpdateCheckGuard;
impl Drop for UpdateCheckGuard {
    fn drop(&mut self) {
        UPDATE_RUNNING.store(false, Ordering::Release);
    }
}

fn automatic_checks_enabled(settings: &oneclient_core::settings::LauncherSettings) -> bool {
    settings.seen_onboarding && settings.auto_update
}

fn automatic_updates_enabled() -> bool {
    crate::launcher::state().is_ok_and(|state| automatic_checks_enabled(&state.settings.read()))
}

pub fn spawn_automatic_update_check(events: EventBus) {
    if automatic_updates_enabled() {
        spawn_update_check(true, events);
    }
}

/// Automatic calls require opt-in. Manual calls always ask before installing.
pub fn spawn_update_check(auto_install: bool, events: EventBus) {
    if auto_install && !automatic_updates_enabled() {
        return;
    }
    if UPDATE_RUNNING.swap(true, Ordering::AcqRel) {
        if !auto_install {
            events.notify("Update check already in progress").send();
        }
        return;
    }
    tokio::spawn(async move {
        let _guard = UpdateCheckGuard;
        if !auto_install {
            events
                .notify("Checking for updates")
                .body("Checking OneForAllLauncher releases on GitHub.")
                .send();
        }
        if let Err(err) = run_check(auto_install, events.clone()).await {
            tracing::warn!("update check failed: {err:#}");
            events
                .notify("Update check failed")
                .body(format!(
                    "{err}. You can try again or download a package from {RELEASES_URL}."
                ))
                .send();
        }
    });
}

/// Debug-only drives the full auto-update UX
pub fn spawn_simulated_update() {
    tokio::spawn(async move {
        if let Err(err) = run_simulated_update().await {
            tracing::warn!("simulated update failed: {err:#}");
        }
    });
}

async fn run_simulated_update() -> anyhow::Result<()> {
    const FAKE_VERSION: &str = "9999.9999.9999";
    const FAKE_TOTAL: u64 = 48 * 1024 * 1024;

    let events = crate::launcher::state()?.services.events.clone();

    if events.ask(update_prompt(FAKE_VERSION)).await?.is_none() {
        tracing::info!("user declined simulated update");
        return Ok(());
    }

    let progress_id = Uuid::new_v4();
    let label = format!("Downloading OneForAllLauncher {FAKE_VERSION}");

    let mut downloaded = 0u64;
    events.progress(progress_id, &label, downloaded, FAKE_TOTAL);
    while downloaded < FAKE_TOTAL {
        downloaded = (downloaded + PROGRESS_STEP * 8).min(FAKE_TOTAL);
        events.progress(progress_id, &label, downloaded, FAKE_TOTAL);
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    events.finish_progress(
        progress_id,
        "Finished Downloading",
        format!("OneForAllLauncher {FAKE_VERSION} is ready. Restart to apply."),
    );

    Ok(())
}

async fn run_check(auto_install: bool, events: EventBus) -> anyhow::Result<()> {
    // `check_update` performs a blocking HTTP request so offload it to a thread pool
    let Some(update) = tokio::task::spawn_blocking(check_for_update).await?? else {
        tracing::info!("no update available");
        if !auto_install {
            events
                .notify("You’re up to date")
                .body(format!(
                    "OneForAllLauncher {} is the latest available version.",
                    env!("CARGO_PKG_VERSION")
                ))
                .send();
        }
        return Ok(());
    };

    tracing::info!("update available: {}", update.version);
    if auto_install && !automatic_updates_enabled() {
        return Ok(());
    }

    // cargo-packager-updater can only replace an AppImage in place deb/rpm installs live
    // under a package-managed path that an install would fail on or clobber
    if !can_self_update() {
        tracing::info!("install is not self-updatable (non-AppImage Linux); notifying only");
        notify_manual_update(&events, &update.version);
        return Ok(());
    }

    if !auto_install && events.ask(update_prompt(&update.version)).await?.is_none() {
        tracing::info!("user declined update {}", update.version);
        return Ok(());
    }

    download_and_install(update, events, auto_install).await
}

fn notify_manual_update(events: &EventBus, version: &str) {
    events
        .notify("Update available")
        .body(format!(
            "OneForAllLauncher {version} is available. Download the latest package from {RELEASES_URL} to update."
        ))
        .link(RELEASES_URL)
        .send();
}

/// Debug-only shows the notify-only toast that non-self-updatable installs get
pub fn simulate_manual_update_toast() {
    match crate::launcher::state() {
        Ok(state) => notify_manual_update(&state.services.events, "9999.9999.9999"),
        Err(err) => tracing::warn!("simulated update toast failed: {err:#}"),
    }
}

fn can_self_update() -> bool {
    if cfg!(debug_assertions) {
        return false;
    }

    if std::env::var_os("ONEFORALL_DISABLE_AUTOUPDATE")
        .is_some_and(|val| val.eq_ignore_ascii_case("1"))
    {
        return false;
    }

    #[cfg(target_os = "linux")]
    {
        std::env::var_os("APPIMAGE").is_some()
    }
    #[cfg(not(target_os = "linux"))]
    {
        true
    }
}

fn check_for_update() -> anyhow::Result<Option<Update>> {
    let current = env!("CARGO_PKG_VERSION").parse()?;
    let config = Config {
        endpoints: vec![UPDATER_ENDPOINT.parse()?],
        pubkey: UPDATER_PUBKEY.trim().into(),
        ..Default::default()
    };
    let mut update = cargo_packager_updater::UpdaterBuilder::new(current, config)
        .timeout(Duration::from_secs(30))
        .build()?
        .check()?;
    if let Some(update) = &mut update {
        // Large packages need more time than the small metadata request.
        update.timeout = Some(Duration::from_secs(30 * 60));
    }
    Ok(update)
}

async fn download_and_install(
    update: Update,
    events: EventBus,
    automatic: bool,
) -> anyhow::Result<()> {
    let progress_id = Uuid::new_v4();
    let version = update.version.clone();
    let label = format!("Downloading OneForAllLauncher {version}");

    events.progress(progress_id, &label, 0, 0);

    let progress_events = events.clone();
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let downloaded = Cell::new(0u64);
        let last_sent = Cell::new(0u64);

        let bytes = update.download_extended(
            |chunk, total| {
                let now = downloaded.get() + chunk as u64;
                downloaded.set(now);
                let total = total.unwrap_or(0);

                if now == chunk as u64
                    || (total > 0 && now >= total)
                    || now - last_sent.get() >= PROGRESS_STEP
                {
                    last_sent.set(now);
                    events.progress(progress_id, &label, now, total);
                }
            },
            || {},
        )?;

        let total = downloaded.get().max(1);
        events.progress(progress_id, &label, total, total);

        if automatic && !automatic_updates_enabled() {
            events.finish_progress(
                progress_id,
                "Update cancelled",
                "Automatic updates were turned off. No update was installed.",
            );
            return Ok(());
        }
        update.install(bytes)?;

        // Converts the same download card into its finished state rather than adding a second
        events.finish_progress(
            progress_id,
            "Finished Downloading",
            format!("OneForAllLauncher {version} is ready. Restart to apply."),
        );
        Ok(())
    })
    .await;
    let result = result
        .map_err(anyhow::Error::from)
        .and_then(|result| result);
    if let Err(error) = &result {
        progress_events.finish_progress(progress_id, "Update failed", error.to_string());
    }
    result?;

    tracing::info!("update download completed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automatic_updates_require_completed_setup_and_opt_in() {
        let mut settings = oneclient_core::settings::LauncherSettings::default();
        assert!(!automatic_checks_enabled(&settings));
        settings.auto_update = true;
        assert!(!automatic_checks_enabled(&settings));
        settings.seen_onboarding = true;
        assert!(automatic_checks_enabled(&settings));
        settings.auto_update = false;
        assert!(!automatic_checks_enabled(&settings));
    }

    #[test]
    fn release_endpoint_belongs_to_this_fork() {
        assert_eq!(
            UPDATER_ENDPOINT,
            "https://github.com/liwwyy/OneForAllLauncher/releases/latest/download/latest.json"
        );
        assert!(!UPDATER_PUBKEY.trim().is_empty());
    }

    // Exercise the real updater's HTTP, metadata and signature checks without
    // installing a package or contacting GitHub. Only a public signed fixture is stored.
    fn download_fixture(tamper: bool) -> cargo_packager_updater::Result<Vec<u8>> {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let format = if cfg!(target_os = "windows") {
            "nsis"
        } else if cfg!(target_os = "macos") {
            "app"
        } else {
            "appimage"
        };
        let target = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
        let metadata = serde_json::json!({
            "version": "9999.0.0",
            "platforms": {(target): {
                "format": format,
                "url": format!("{base}/payload"),
                "signature": include_str!("../tests/fixtures/updater/payload.sig").trim(),
            }},
        })
        .to_string();
        let server = std::thread::spawn(move || {
            let mut payload = include_bytes!("../tests/fixtures/updater/payload").to_vec();
            if tamper {
                payload[0] ^= 1;
            }
            for (path, body) in [
                ("/latest.json", metadata.into_bytes()),
                ("/payload", payload),
            ] {
                let deadline = std::time::Instant::now() + Duration::from_secs(10);
                let mut socket = loop {
                    match listener.accept() {
                        Ok((socket, _)) => break socket,
                        Err(error)
                            if error.kind() == std::io::ErrorKind::WouldBlock
                                && std::time::Instant::now() < deadline =>
                        {
                            std::thread::sleep(Duration::from_millis(10))
                        }
                        Err(error) => panic!("fixture request missing: {error}"),
                    }
                };
                socket
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut request = [0; 4096];
                let n = socket.read(&mut request).unwrap();
                assert!(
                    String::from_utf8_lossy(&request[..n]).starts_with(&format!("GET {path} "))
                );
                write!(socket, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).unwrap();
                socket.write_all(&body).unwrap();
            }
        });
        let config = Config {
            endpoints: vec![format!("{base}/latest.json").parse().unwrap()],
            pubkey: UPDATER_PUBKEY.trim().into(),
            ..Default::default()
        };
        let update = cargo_packager_updater::UpdaterBuilder::new("0.0.0".parse().unwrap(), config)
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap()
            .check()
            .unwrap()
            .unwrap();
        let result = update.download();
        server.join().unwrap();
        result
    }

    #[test]
    fn signed_fork_updates_are_accepted_and_modified_packages_are_rejected() {
        assert_eq!(
            download_fixture(false).unwrap(),
            include_bytes!("../tests/fixtures/updater/payload")
        );
        assert!(download_fixture(true).is_err());
    }
}
