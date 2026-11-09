use crate::{i18n::t, theme::text_size};
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
            .child(step_heading(t("fork.updates.title"), t("fork.updates.description")))
            .child(choice_row(t("fork.updates.auto"), t("fork.updates.auto_description"), *choice.read() == Some(true), move |()| {
                choice.set(Some(true));
                enable.edit_settings(|settings| settings.auto_update = true);
            }))
            .child(choice_row(t("fork.updates.manual"), t("fork.updates.manual_description"), *choice.read() == Some(false), move |()| {
                choice.set(Some(false));
                dispatch.edit_settings(|settings| settings.auto_update = false);
            }))
            .child(label().text(t("fork.updates.package_notice")).font_size(text_size(13.)).width(Size::fill()))
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
