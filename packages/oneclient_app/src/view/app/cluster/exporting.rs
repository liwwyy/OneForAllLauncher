use freya::prelude::*;
use oneclient_core::export::{ExportFormat, export_cluster, supports_prism};

use crate::components::{Button, IconType};
use crate::hooks::{Actions, use_cluster, use_dispatch, use_installs_snapshot};
use crate::launcher::{self, off_ui};
use crate::view::app::settings::{section_header, settings_row};

use super::folder_list::use_game_folder_in_use;

#[derive(PartialEq)]
pub(super) struct ExportingSection {
    pub cluster_id: i64,
}

impl Component for ExportingSection {
    fn render(&self) -> impl IntoElement {
        let cluster_id = self.cluster_id;
        let cluster = use_cluster(cluster_id);
        let in_use = use_game_folder_in_use(cluster.as_ref());
        let installs = use_installs_snapshot();
        let exporting = use_state(|| None::<ExportFormat>);
        let busy = exporting.read().is_some();
        let blocked = busy || in_use || installs.cluster_busy(cluster_id) || cluster.is_none();
        let prism_supported = cluster
            .as_ref()
            .is_some_and(|cluster| supports_prism(cluster.mc_loader, &cluster.mc_version));
        let dispatch = use_dispatch();

        let mods_dispatch = dispatch.clone();
        let mods_button = Button::new()
            .small()
            .secondary()
            .enabled(!blocked)
            .tooltip(if in_use {
                "Close Minecraft before exporting"
            } else {
                "Save this instance's mods as a ZIP"
            })
            .on_press(move |_| {
                start_export(
                    cluster_id,
                    ExportFormat::Mods,
                    exporting,
                    mods_dispatch.clone(),
                )
            })
            .text(if *exporting.read() == Some(ExportFormat::Mods) {
                "Exporting..."
            } else {
                "Export"
            });
        let prism_button = Button::new()
            .small()
            .secondary()
            .enabled(!blocked && prism_supported)
            .tooltip(if !prism_supported {
                "The Ornithe Prism template supports Minecraft 1.8.9"
            } else if in_use {
                "Close Minecraft before exporting"
            } else {
                "Save an instance ZIP to import in Prism Launcher"
            })
            .on_press(move |_| {
                start_export(cluster_id, ExportFormat::Prism, exporting, dispatch.clone())
            })
            .text(if *exporting.read() == Some(ExportFormat::Prism) {
                "Exporting..."
            } else {
                "Export"
            });

        rect().vertical().width(Size::fill()).spacing(4.)
            .child(section_header("EXPORTING"))
            .child(settings_row(IconType::Export, "Export mods.zip",
                "Save this instance's installed mods in a portable ZIP.", mods_button))
            .child(settings_row(IconType::Export, "Export prism instance",
                "Export mods, Minecraft options and configs for Prism Launcher. Includes Ornithe Gen2 patches for 1.8.9.", prism_button))
    }
}

fn start_export(
    cluster_id: i64,
    format: ExportFormat,
    mut exporting: State<Option<ExportFormat>>,
    actions: Actions,
) {
    if exporting.peek().is_some() {
        return;
    }
    exporting.set(Some(format));
    spawn(async move {
        let result = async {
            let state = launcher::state()?;
            let cluster = state.clusters.get(cluster_id).await?;
            let file_name = if format == ExportFormat::Mods {
                "mods.zip".to_string()
            } else {
                format!("{}-prism.zip", polyio::sanitize_path(&cluster.name).display())
            };
            let Some(handle) = rfd::AsyncFileDialog::new()
                .set_title(if format == ExportFormat::Mods {
                    "Export mods.zip"
                } else {
                    "Export Prism instance"
                })
                .add_filter("ZIP archive", &["zip"])
                .set_file_name(&file_name)
                .save_file()
                .await
            else {
                return Ok(None);
            };
            let mut destination = handle.path().to_path_buf();
            if destination.extension().is_none() {
                destination.set_extension("zip");
            }
            off_ui(async move {
                export_cluster(&state, &cluster, format, &destination)
                    .await
                    .map(Some)
            })
            .await
        }
        .await;
        exporting.set(None);
        match result {
            Ok(Some(report)) => {
                actions
                    .notify("Export created")
                    .body(format!(
                        "Saved {} mods to {}",
                        report.mods,
                        report.path.display()
                    ))
                    .icon(IconType::Export)
                    .send();
            }
            Ok(None) => {}
            Err(err) => {
                tracing::error!(error = %err, "instance export failed");
                actions
                    .notify("Couldn't export this instance")
                    .body(format!("{err:#}"))
                    .error()
                    .icon(IconType::Export)
                    .send();
            }
        }
    });
}
