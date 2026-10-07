use freya::prelude::*;
use oneclient_auth::MinecraftAccount;

use crate::components::{
    Avatar, Button, Icon, IconType, use_custom_login, use_elyby_login, use_microsoft_login,
    use_offline_login,
};
use crate::hooks::{try_default_account, use_current_account};
use crate::routes::Route;
use crate::theme::colors;
use crate::view::onboarding::{
    onboarding_illustration, onboarding_nav, onboarding_page, step_heading,
};

#[derive(PartialEq)]
pub struct OnboardingAccount;

impl Component for OnboardingAccount {
    fn render(&self) -> impl IntoElement {
        let account_query = use_current_account();
        let msa = use_microsoft_login();
        let offline = use_offline_login();
        let elyby = use_elyby_login();
        let custom = use_custom_login();

        let account = try_default_account(&account_query);
        let has_account = account.is_some();

        let content = rect()
            .vertical()
            .width(Size::fill())
            .spacing(24.)
            .child(step_heading(
                "Account",
                "Add a Microsoft, offline, Ely.by or custom account to start playing Minecraft: Java Edition.",
            ))
            .maybe_child(account.as_ref().map(|account| account_preview(account).into_element()))
            .child(sign_in_card(msa.pending, msa.error.clone(), { let msa = msa.clone(); move |_| msa.start() }))
            .child(Button::new().secondary().large().width(Size::fill())
                .on_press({ let offline = offline.clone(); move |_| offline.open() })
                .child(Icon::new(IconType::Offline).size(22.)).text("Add an offline account"))
            .child(Button::new().secondary().large().width(Size::fill()).enabled(!elyby.pending)
                .on_press({ let elyby = elyby.clone(); move |_| elyby.start() })
                .child(Icon::new(IconType::Elyby).size(22.)).text("Add an Ely.by account"))
            .child(Button::new().secondary().large().width(Size::fill())
                .on_press({ let custom = custom.clone(); move |_| custom.open() })
                .child(Icon::new(IconType::Custom).size(22.)).text("Add a custom account"))
            .maybe_child(elyby.error.clone().map(|error| label().text(error).font_size(12.).color(colors::danger()).into_element()))
            .into_element();

        let page = onboarding_page(
            onboarding_illustration(IconType::OnboardingAccount),
            content,
            onboarding_nav(
                Some(Route::OnboardingLanguage {}),
                Route::OnboardingBundles {},
                has_account,
            ),
        );

        rect()
            .width(Size::fill())
            .height(Size::fill())
            .child(page)
            .maybe_child(msa.popup())
            .maybe_child(offline.popup())
            .maybe_child(elyby.popup())
            .maybe_child(custom.popup())
    }
}

fn account_preview(account: &MinecraftAccount) -> impl IntoElement {
    rect()
        .horizontal()
        .width(Size::fill())
        .spacing(24.)
        .child(
            rect()
                .horizontal()
                .spacing(12.)
                .cross_align(Alignment::Center)
                .child(
                    Avatar::new(account.skin_profile_key())
                        .width(Size::px(48.))
                        .height(Size::px(48.)),
                )
                .child(
                    rect()
                        .vertical()
                        .spacing(4.)
                        .child(
                            label()
                                .text(account.username.clone())
                                .font_size(16.)
                                .font_weight(FontWeight::SEMI_BOLD)
                                .color(colors::fg_primary()),
                        )
                        .child(
                            label()
                                .text(account.id.to_string())
                                .font_size(12.)
                                .color(colors::fg_secondary()),
                        ),
                ),
        )
        .into_element()
}

fn sign_in_card(
    pending: bool,
    error: Option<String>,
    on_add: impl FnMut(Event<PressEventData>) + 'static,
) -> impl IntoElement {
    rect()
        .vertical()
        .spacing(12.)
        .width(Size::fill())
        .cross_align(Alignment::Start)
        .child(
            Button::new()
                .primary()
                .large()
                .width(Size::fill())
                .enabled(!pending)
                .on_press(on_add)
                .child(Icon::new(IconType::Globe01).size(22.))
                .text(if pending {
                    "Signing in..."
                } else {
                    "Add a Microsoft account"
                }),
        )
        .maybe_child(error.map(|message| {
            rect()
                .horizontal()
                .cross_align(Alignment::Center)
                .spacing(6.)
                .child(
                    Icon::new(IconType::AlertTriangle)
                        .size(13.)
                        .color(colors::danger()),
                )
                .child(label().text(message).font_size(12.).color(colors::danger()))
                .into_element()
        }))
        .into_element()
}
