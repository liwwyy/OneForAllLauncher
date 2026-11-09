use freya::prelude::*;
use freya::query::{QueriesStorage, Query, QueryCapability, UseQuery, use_query};
use oneclient_core::LauncherError;

use super::{Button, ContextMenu, IconType, toggle_controlled};
use crate::hooks::{settled_or_loading, use_dispatch};
use crate::i18n::t;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct OfflinePreference;

impl QueryCapability for OfflinePreference {
    type Keys = i64;
    type Ok = bool;
    type Err = LauncherError;

    async fn run(&self, cluster_id: &i64) -> Result<bool, LauncherError> {
        {
            let state = crate::launcher::state()?;
            oneclient_core::launch_mode::preference(&state, *cluster_id).await
        }
    }
}

fn preference(cluster_id: i64) -> UseQuery<OfflinePreference> {
    use_query(Query::new(cluster_id, OfflinePreference))
}

fn use_online() -> bool {
    let mut status = use_state(oneclient_net::status::current);
    use_hook(move || {
        let mut rx = oneclient_net::status::subscribe();
        spawn(async move {
            while rx.changed().await.is_ok() {
                status.set(*rx.borrow());
            }
        });
    });
    status.read().online
}

#[derive(PartialEq)]
pub struct InstanceLaunchButton {
    pub cluster_id: i64,
    pub button: Button,
    pub label: &'static str,
    pub enabled: bool,
    pub grow: bool,
}

impl Component for InstanceLaunchButton {
    fn render(&self) -> impl IntoElement {
        let online = use_online();
        let always = settled_or_loading(&preference(self.cluster_id)).unwrap_or(false);
        let offline = oneclient_core::launch_mode::is_offline(online, always, false);
        let idle = self.label == t("oneclient.app.launch.launch");
        // Startup's remote catalog refresh must not block an installed offline game.
        // Checking/downloading/running and shared-directory locks still take precedence.
        let enabled = self.enabled || (offline && idle);
        let cluster_id = self.cluster_id;
        let launch = use_dispatch();
        let force = launch.clone();
        let mut menu_at = use_state(|| None::<(f32, f32)>);
        let mut button = self
            .button
            .clone()
            .enabled(enabled)
            .text(if offline && idle {
                t("fork.launch.offline")
            } else {
                self.label
            })
            .on_press(move |_| {
                if enabled {
                    launch.launch_cluster(cluster_id);
                }
            });
        if self.grow {
            button = button.width(Size::fill());
        }
        let menu = (*menu_at.read()).filter(|_| enabled).map(|(x, y)| {
            ContextMenu::new(x, y)
                .on_close(move |()| menu_at.set(None))
                .action(IconType::Offline, t("fork.launch.force"), move |()| {
                    menu_at.set(None);
                    force.launch_cluster_offline(cluster_id);
                })
                .into_element()
        });
        rect()
            .maybe(self.grow, |el| el.width(Size::flex(1.)))
            .on_secondary_down(move |event: Event<PressEventData>| {
                event.stop_propagation();
                if enabled && let PressEventData::Mouse(mouse) = event.data() {
                    menu_at.set(Some((
                        mouse.global_location.x as f32,
                        mouse.global_location.y as f32,
                    )));
                }
            })
            .child(button)
            .maybe_child(menu)
    }
}

#[derive(PartialEq)]
pub struct OfflineLaunchRow {
    pub cluster_id: i64,
}

impl Component for OfflineLaunchRow {
    fn render(&self) -> impl IntoElement {
        let cluster_id = self.cluster_id;
        let enabled = settled_or_loading(&preference(cluster_id)).unwrap_or(false);
        let mut pending = use_state(|| false);
        let actions = use_dispatch();
        crate::view::app::settings::settings_row(
            IconType::Offline,
            t("fork.launch.always.title"),
            t("fork.launch.always.description"),
            toggle_controlled(
                enabled,
                (move |()| {
                    if *pending.peek() {
                        return;
                    }
                    pending.set(true);
                    let actions = actions.clone();
                    spawn(async move {
                        let result = match crate::launcher::state() {
                            Ok(state) => {
                                oneclient_core::launch_mode::set_preference(
                                    &state, cluster_id, !enabled,
                                )
                                .await
                            }
                            Err(error) => Err(error),
                        };
                        match result {
                            Ok(()) => {
                                QueriesStorage::<OfflinePreference>::invalidate_matching(cluster_id)
                                    .await
                            }
                            Err(error) => {
                                actions
                                    .notify(t("fork.launch.save_failed"))
                                    .body(error.to_string())
                                    .send();
                            }
                        }
                        pending.set(false);
                    });
                })
                .into(),
            ),
        )
    }
}
