use freya::prelude::*;

use crate::components::IconType;
use crate::hooks::use_onboarding_selection;
use crate::routes::Route;
use crate::theme::colors;
use crate::view::onboarding::{onboarding_illustration, onboarding_nav, onboarding_page};

#[derive(PartialEq)]
pub struct OnboardingWelcome;

impl Component for OnboardingWelcome {
    fn render(&self) -> impl IntoElement {
        let next = if *use_onboarding_selection().picks_location.read() {
            Route::OnboardingLocation {}
        } else {
            Route::OnboardingTerms {}
        };

        let content = rect()
            .vertical()
            .width(Size::fill())
            .spacing(16.)
            .child(rect().vertical().spacing(8.)
                .child(crate::components::brand_wordmark_sized(36.))
                .child(label().text("Let's get you all set-up with the most advanced client.")
                    .font_size(16.).color(colors::fg_secondary())))
            .child(
                label()
                    .text("This quick setup will pick your language, sign you in, and prepare your first versions. It only takes a minute.")
                    .font_size(15.)
                    .color(colors::fg_secondary()),
            )
            .into_element();

        onboarding_page(
            onboarding_illustration(IconType::OnboardingWelcome),
            content,
            onboarding_nav(None, next, true),
        )
    }
}
