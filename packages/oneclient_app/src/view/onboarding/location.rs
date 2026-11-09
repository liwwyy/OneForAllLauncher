use std::path::PathBuf;

use freya::prelude::*;
use freya::router::RouterContext;

use crate::components::IconType;
use crate::hooks::{use_dispatch, use_launcher, use_onboarding_selection};
use crate::i18n::t;
use crate::routes::Route;
use crate::state::AppChannel;
use crate::theme::{colors, text_size};
use crate::ui::note;
use crate::view::onboarding::{
    choice_row, onboarding_illustration, onboarding_nav_action, onboarding_page, step_heading,
};

#[derive(Clone, PartialEq)]
enum FolderChoice {
    ThisPc,
    OneClient,
    AnotherDrive(PathBuf),
}

#[derive(PartialEq)]
pub struct OnboardingLocation;

impl Component for OnboardingLocation {
    fn render(&self) -> impl IntoElement {
        let actions = use_dispatch();
        let launcher = use_launcher();
        let selection = use_onboarding_selection();

        // Only fresh onboarding reaches an unanswered location step. Saved installs
        // keep their existing fork/custom path in main's startup resolution.
        let mut picked = use_state(|| FolderChoice::OneClient);
        let mut upstream = use_state(|| None::<Result<PathBuf, String>>);
        use_hook(move || {
            spawn(async move {
                upstream.set(Some(
                    oneclient_core::settings::data_dir::oneclient_folder().await,
                ));
            });
        });
        let mut warning = use_state(|| None::<String>);
        let mut error = use_state(|| None::<String>);
        let mut waiting = use_state(|| false);

        let answered = !launcher.needs_location;
        let settled = answered && launcher.ready;

        if !*selection.picks_location.read() && settled {
            let _ = RouterContext::get().replace(Route::OnboardingTerms {});
            return rect().into_element();
        }

        if *waiting.read() && launcher.ready {
            let _ = RouterContext::get().replace(Route::OnboardingTerms {});
            return rect().into_element();
        }

        let default_dir = oneclient_common::paths::config_dir()
            .map(|dir| dir.display().to_string())
            .unwrap_or_default();

        let upstream_dir = upstream.read().clone();

        let confirm = move |()| {
            if *waiting.peek() {
                return;
            }

            if settled {
                let _ = RouterContext::get().replace(Route::OnboardingTerms {});
                return;
            }

            let chosen = picked.peek().clone();
            let actions = actions.clone();
            let station = actions.station();
            let events = actions.events();

            waiting.set(true);
            error.set(None);

            spawn_in_window(async move {
                if !answered {
                    let result = match chosen {
                        FolderChoice::ThisPc => {
                            oneclient_core::settings::data_dir::apply(None).await
                        }
                        FolderChoice::OneClient => {
                            match oneclient_core::settings::data_dir::oneclient_folder().await {
                                Ok(path) => {
                                    oneclient_core::settings::data_dir::apply_exact(path).await
                                }
                                Err(message) => Err(message),
                            }
                        }
                        FolderChoice::AnotherDrive(path) => {
                            oneclient_core::settings::data_dir::apply(Some(path)).await
                        }
                    };
                    if let Err(message) = result {
                        error.set(Some(message));
                        waiting.set(false);
                        return;
                    }

                    let mut station = station;
                    let mut guard = station.write_channel(AppChannel::Launcher);
                    guard.launcher.needs_location = false;
                }

                match crate::events::start_launcher(station, events).await {
                    Ok(()) => actions.sync_bundles(),
                    Err(err) => {
                        crate::events::report_startup_failure(&station, &err);
                        waiting.set(false);
                    }
                }
            });
        };

        let chosen = picked.read().clone();
        let running = *waiting.read();

        let mut content = rect()
            .vertical()
            .width(Size::fill())
            .spacing(16.)
            .child(step_heading(
                t("oneclient.onboarding.location.title"),
                t("oneclient.onboarding.location.subtitle"),
            ));

        content = if answered {
            let folder = if launcher.data_dir.is_empty() {
                match &chosen {
                    FolderChoice::ThisPc => default_dir.clone(),
                    FolderChoice::OneClient => upstream_dir
                        .as_ref()
                        .and_then(|result| result.as_ref().ok())
                        .map(|path| path.display().to_string())
                        .unwrap_or_default(),
                    FolderChoice::AnotherDrive(path) => path.display().to_string(),
                }
            } else {
                launcher.data_dir.clone()
            };

            content
                .child(choice_row(
                    t("oneclient.onboarding.location.your_folder"),
                    &folder,
                    true,
                    |()| {},
                ))
                .child(
                    label()
                        .text(t("oneclient.onboarding.location.answered"))
                        .font_size(text_size(13.))
                        .max_lines(2)
                        .color(colors::fg_secondary()),
                )
        } else {
            content.child(
                rect()
                    .vertical()
                    .width(Size::fill())
                    .spacing(10.)
                    .child(choice_row(
                        t("fork.onboarding.location.oneforall"),
                        &default_dir,
                        matches!(chosen, FolderChoice::ThisPc),
                        move |()| {
                            picked.set(FolderChoice::ThisPc);
                            warning.set(None);
                            error.set(None);
                        },
                    ))
                    .child(choice_row(
                        t("fork.onboarding.location.oneclient"),
                        &upstream_dir
                            .as_ref()
                            .and_then(|result| result.as_ref().ok())
                            .map(|path| path.display().to_string())
                            .unwrap_or_default(),
                        matches!(chosen, FolderChoice::OneClient),
                        move |()| {
                            picked.set(FolderChoice::OneClient);
                            warning.set(None);
                            error.set(None);
                        },
                    ))
                    .child(choice_row(
                        t("oneclient.onboarding.location.another_drive"),
                        &match &chosen {
                            FolderChoice::AnotherDrive(path) => path.display().to_string(),
                            _ => t("oneclient.onboarding.location.pick_folder").to_string(),
                        },
                        matches!(chosen, FolderChoice::AnotherDrive(_)),
                        move |()| browse(picked, warning, error),
                    )),
            )
        };

        if !answered && matches!(chosen, FolderChoice::OneClient) {
            content = content.child(note(
                t("fork.onboarding.location.shared").to_string(),
                colors::fg_secondary(),
            ));
            if let Some(Err(message)) = &upstream_dir {
                content = content.child(note(message.clone(), colors::danger()));
            }
        }

        if let Some(message) = warning.read().clone() {
            content = content.child(note(message, colors::code_warn()));
        }

        let failure = error.read().clone().or_else(|| launcher.error.clone());

        if let Some(message) = failure {
            content = content.child(note(message, colors::danger()));
        }

        let next_label = if running {
            t("oneclient.onboarding.location.setting_up")
        } else if answered && !settled {
            t("oneclient.common.try_again")
        } else {
            t("oneclient.common.next")
        };

        onboarding_page(
            onboarding_illustration(IconType::OnboardingFolder),
            content.into_element(),
            onboarding_nav_action(
                Some(Route::OnboardingWelcome {}),
                next_label,
                !running
                    && (answered
                        || !matches!(chosen, FolderChoice::OneClient)
                        || matches!(upstream_dir, Some(Ok(_)))),
                confirm,
            ),
        )
        .into_element()
    }
}

fn browse(
    mut picked: State<FolderChoice>,
    mut warning: State<Option<String>>,
    mut error: State<Option<String>>,
) {
    spawn(async move {
        let Some(handle) = rfd::AsyncFileDialog::new()
            .set_title(t("oneclient.onboarding.location.picker_title"))
            .pick_folder()
            .await
        else {
            return;
        };

        match oneclient_core::settings::data_dir::check(handle.path()).await {
            Ok(checked) => {
                error.set(None);
                warning.set(checked.warning);
                picked.set(FolderChoice::AnotherDrive(checked.path));
            }
            Err(message) => {
                picked.set(FolderChoice::ThisPc);
                warning.set(None);
                error.set(Some(message));
            }
        }
    });
}
