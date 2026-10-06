use freya::prelude::*;

use crate::components::window_controls;
use crate::theme;

#[derive(PartialEq)]
pub struct OnboardingNavbar;

impl Component for OnboardingNavbar {
    fn render(&self) -> impl IntoElement {
        rect()
            .width(Size::fill())
            .height(Size::px(theme::NAVBAR_HEIGHT_PX))
            .child(
                rect()
                    .horizontal()
                    .width(Size::fill())
                    .height(Size::fill())
                    .content(Content::Flex)
                    .cross_align(Alignment::Center)
                    .padding(Gaps::new_symmetric(0., 24.))
                    .child(logo())
                    .child(rect().width(Size::flex(1.0)).height(Size::fill()))
                    .child(window_controls()),
            )
            .child(
                rect()
                    .window_drag()
                    .width(Size::window_percent(100.))
                    .height(Size::px(theme::NAVBAR_HEIGHT_PX))
                    .position(Position::new_absolute().top(0.).left(0.).right(0.)),
            )
    }
}

fn logo() -> impl IntoElement {
    super::brand_wordmark()
}
