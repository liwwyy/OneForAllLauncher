use crate::{i18n::t, theme::text_size};
use freya::prelude::*;
use freya::router::RouterContext;
use oneclient_core::settings::LauncherSettings;

use crate::components::IconType;
use crate::hooks::{use_dispatch, use_onboarding_selection, use_settings_snapshot};
use crate::routes::Route;
use crate::view::onboarding::{
    choice_row, onboarding_default_nav, onboarding_illustration, onboarding_page, step_heading,
};

#[derive(PartialEq)]
pub struct OnboardingUpdates;

fn update_choice(choice: Option<bool>, settings: &LauncherSettings) -> bool {
    choice.unwrap_or(if settings.seen_onboarding {
        settings.auto_update
    } else {
        true
    })
}

impl Component for OnboardingUpdates {
    fn render(&self) -> impl IntoElement {
        let mut choice = use_onboarding_selection().automatic_updates;
        let settings = use_settings_snapshot().settings;
        let automatic = update_choice(*choice.read(), &settings);
        let dispatch = use_dispatch();
        let content = rect()
            .vertical()
            .width(Size::fill())
            .spacing(20.)
            .child(step_heading(
                t("fork.updates.title"),
                t("fork.updates.description"),
            ))
            .child(choice_row(
                t("fork.updates.auto"),
                t("fork.updates.auto_description"),
                automatic,
                move |()| choice.set(Some(true)),
            ))
            .child(choice_row(
                t("fork.updates.manual"),
                t("fork.updates.manual_description"),
                !automatic,
                move |()| choice.set(Some(false)),
            ))
            .child(
                label()
                    .text(t("fork.updates.package_notice"))
                    .font_size(text_size(13.))
                    .width(Size::fill()),
            )
            .into_element();
        onboarding_page(
            onboarding_illustration(IconType::DownloadCloud02),
            content,
            onboarding_default_nav(Some(Route::OnboardingPreferences {}), move |()| {
                // Read the latest selection, including a click made just before Enter.
                let automatic = update_choice(*choice.peek(), &settings);
                // Merely showing the page does not opt in: Next confirms its visible choice.
                choice.set(Some(automatic));
                dispatch.edit_settings(|settings| settings.auto_update = automatic);
                let _ = RouterContext::get().replace(Route::OnboardingSummary {});
            }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_setup_preselects_automatic_without_changing_saved_consent() {
        let settings = LauncherSettings::default();
        assert!(!settings.auto_update);
        assert!(update_choice(None, &settings));
        assert!(!update_choice(Some(false), &settings));
        assert!(update_choice(Some(true), &settings));
        assert!(!settings.auto_update);
    }

    #[test]
    fn revisiting_setup_preserves_an_existing_opt_out() {
        let settings = LauncherSettings {
            seen_onboarding: true,
            auto_update: false,
            ..Default::default()
        };
        assert!(!update_choice(None, &settings));
        assert!(update_choice(Some(true), &settings));
        let enabled = LauncherSettings {
            auto_update: true,
            ..settings
        };
        assert!(update_choice(None, &enabled));
        assert!(!update_choice(Some(false), &enabled));
    }
}
