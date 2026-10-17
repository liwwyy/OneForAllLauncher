use freya::prelude::*;

use crate::components::IconType;
use crate::hooks::{use_dispatch, use_onboarding_selection};
use crate::routes::Route;
use crate::view::onboarding::{
    choice_row, onboarding_illustration, onboarding_nav, onboarding_page, step_heading,
};

#[derive(PartialEq)]
pub struct OnboardingUpdates;

impl Component for OnboardingUpdates {
    fn render(&self) -> impl IntoElement {
        let mut choice = use_onboarding_selection().automatic_updates;
        let dispatch = use_dispatch();
        let enable = dispatch.clone();
        let content = rect().vertical().width(Size::fill()).spacing(20.)
            .child(step_heading("Launcher updates", "Choose how OneForAllLauncher stays up to date. You can change this later in Settings → Launcher → Updates."))
            .child(choice_row("Enable automatic updates", "Check GitHub on startup and install verified launcher updates automatically. Restart to apply them.", *choice.read() == Some(true), move |()| {
                choice.set(Some(true));
                enable.edit_settings(|settings| settings.auto_update = true);
            }))
            .child(choice_row("Update manually", "No background update checks. Use Check for updates in Launcher settings whenever you want.", *choice.read() == Some(false), move |()| {
                choice.set(Some(false));
                dispatch.edit_settings(|settings| settings.auto_update = false);
            }))
            .child(label().text("DEB and RPM installations receive an update notice instead of replacing package-managed files.").font_size(13.).width(Size::fill()))
            .into_element();
        onboarding_page(
            onboarding_illustration(IconType::DownloadCloud02),
            content,
            onboarding_nav(
                Some(Route::OnboardingPreferences {}),
                Route::OnboardingSummary {},
                choice.read().is_some(),
            ),
        )
    }
}
