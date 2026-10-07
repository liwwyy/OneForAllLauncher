use chrono::{DateTime, Utc};
use freya::animation::{AnimNum, Function, OnFinish, use_animation};
use freya::prelude::*;
use oneclient_auth::{AccountKind, MinecraftAccount};
use uuid::Uuid;

use super::{section_header, settings_page};
use crate::components::{
    Avatar, Button, Icon, IconType, PlayerModel, use_custom_login, use_elyby_login,
    use_microsoft_login, use_offline_login,
};
use crate::hooks::{
    RefreshAccountKeys, RemoveAccountKeys, SetDefaultAccountKeys, try_accounts,
    try_default_account, use_accounts, use_current_account, use_refresh_account,
    use_remove_account, use_set_default_account,
};
use crate::theme::colors;
use crate::ui::border_all_color;

/// The shader scales the model off `res.y` alone so height makes the player bigger width only needs to clear the arms
const MODEL_WIDTH_PX: f32 = 160.;
const MODEL_HEIGHT_PX: f32 = 264.;

const AVATAR_SIZE_PX: f32 = 36.;

#[derive(PartialEq)]
pub struct SettingsAccounts;

impl Component for SettingsAccounts {
    fn render(&self) -> impl IntoElement {
        let accounts_query = use_accounts();
        let default_query = use_current_account();

        let msa = use_microsoft_login();
        let elyby = use_elyby_login();
        let custom = use_custom_login();
        let set_default = use_set_default_account();
        let remove = use_remove_account();
        let refresh = use_refresh_account();

        let offline = use_offline_login();
        let accounts = try_accounts(&accounts_query).unwrap_or_default();
        let default_account = try_default_account(&default_query);
        let default_id = default_account.as_ref().map(|a| a.id);

        let mut rows: Vec<Element> = accounts
            .iter()
            .map(|account| {
                AccountRow {
                    id: account.id,
                    username: account.username.clone(),
                    kind: account.kind,
                    expires: account.expires,
                    skin_key: account.skin_profile_key(),
                    is_default: Some(account.id) == default_id,
                    set_default,
                    remove,
                    refresh,
                }
                .into_element()
            })
            .collect();
        if rows.is_empty() {
            rows.push(empty_state());
        }

        settings_page()
            .child(hero(
                default_account,
                msa.pending,
                msa.error.clone(),
                elyby.pending,
                elyby.error.clone(),
                {
                    let offline = offline.clone();
                    move |_| offline.open()
                },
                {
                    let msa = msa.clone();
                    move |_| msa.start()
                },
                {
                    let elyby = elyby.clone();
                    move |_| elyby.start()
                },
                {
                    let custom = custom.clone();
                    move |_| custom.open()
                },
            ))
            .child(section_header("YOUR ACCOUNTS"))
            .children(rows)
            .maybe_child(offline.popup())
            .maybe_child(msa.popup())
            .maybe_child(elyby.popup())
            .maybe_child(custom.popup())
            .into_element()
    }
}

fn hero(
    account: Option<MinecraftAccount>,
    microsoft_pending: bool,
    error: Option<String>,
    elyby_pending: bool,
    elyby_error: Option<String>,
    on_open_offline: impl FnMut(Event<PressEventData>) + 'static,
    on_add_microsoft: impl FnMut(Event<PressEventData>) + 'static,
    on_add_elyby: impl FnMut(Event<PressEventData>) + 'static,
    on_add_custom: impl FnMut(Event<PressEventData>) + 'static,
) -> impl IntoElement {
    let (name, subtitle) = match &account {
        Some(account) => (account.username.clone(), kind_label(account.kind)),
        None => (
            "No active account".to_string(),
            "Add an account to start playing",
        ),
    };

    rect()
        .horizontal()
        .width(Size::fill())
        .content(Content::Flex)
        .cross_align(Alignment::Center)
        .spacing(16.)
        .padding(Gaps::new_all(16.))
        .corner_radius(CornerRadius::new_all(12.))
        .background(colors::page_elevated())
        .child(model_frame(
            account.as_ref().map(|account| account.skin_profile_key()),
        ))
        .child(
            rect()
                .vertical()
                .width(Size::flex(1.0))
                .spacing(16.)
                .child(
                    rect()
                        .vertical()
                        .width(Size::fill())
                        .spacing(2.)
                        .child(
                            label()
                                .text("ACTIVE ACCOUNT")
                                .font_size(11.)
                                .font_weight(FontWeight::MEDIUM)
                                .color(colors::fg_secondary()),
                        )
                        .child(
                            label()
                                .text(name)
                                .font_size(24.)
                                .font_weight(FontWeight::SEMI_BOLD)
                                .max_lines(1)
                                .color(colors::fg_primary()),
                        )
                        .child(
                            label()
                                .text(subtitle)
                                .font_size(12.)
                                .color(colors::fg_secondary()),
                        ),
                )
                .child(
                    rect()
                        .vertical()
                        .width(Size::fill())
                        .spacing(8.)
                        .child(
                            // Keep all account providers together, wrapping at narrow widths.
                            rect()
                                .horizontal()
                                .width(Size::fill())
                                .content(Content::wrap_spacing(8.))
                                .spacing(8.)
                                .child(
                                    Button::new()
                                        .primary()
                                        .enabled(!microsoft_pending)
                                        .on_press(on_add_microsoft)
                                        .child(Icon::new(IconType::Globe01).size(16.))
                                        .text(if microsoft_pending {
                                            "Signing in..."
                                        } else {
                                            "Add Microsoft"
                                        }),
                                )
                                .child(
                                    Button::new()
                                        .secondary()
                                        .on_press(on_open_offline)
                                        .child(Icon::new(IconType::Offline).size(16.))
                                        .text("Add offline"),
                                )
                                .child(
                                    Button::new()
                                        .secondary()
                                        .enabled(!elyby_pending)
                                        .on_press(on_add_elyby)
                                        .child(Icon::new(IconType::Elyby).size(16.))
                                        .text(if elyby_pending {
                                            "Signing in…"
                                        } else {
                                            "Add Ely.by"
                                        }),
                                )
                                .child(
                                    Button::new()
                                        .secondary()
                                        .on_press(on_add_custom)
                                        .child(Icon::new(IconType::Custom).size(16.))
                                        .text("Add custom"),
                                ),
                        )
                        .map(error, |el, msg| {
                            el.child(hint_line(IconType::AlertTriangle, msg, colors::danger()))
                        })
                        .map(elyby_error, |el, msg| {
                            el.child(hint_line(IconType::AlertTriangle, msg, colors::danger()))
                        }),
                ),
        )
        .into_element()
}

fn model_frame(id: Option<String>) -> impl IntoElement {
    rect()
        .vertical()
        .cross_align(Alignment::Center)
        .spacing(6.)
        .child(
            rect()
                .width(Size::px(MODEL_WIDTH_PX))
                .height(Size::px(MODEL_HEIGHT_PX))
                .center()
                .overflow(Overflow::Clip)
                .corner_radius(CornerRadius::new_all(12.))
                .background(colors::component_bg())
                .border(border_all_color(1., colors::component_border()))
                .child(match id.clone() {
                    Some(id) => PlayerModel::new(id)
                        .yaw(-0.5)
                        .width(Size::fill())
                        .height(Size::fill())
                        .into_element(),
                    None => Icon::new(IconType::Users01)
                        .size(28.)
                        .color(colors::fg_secondary())
                        .into_element(),
                }),
        )
        .maybe_child(id.map(|_| {
            label()
                .text("Drag to rotate")
                .font_size(10.)
                .color(colors::fg_secondary())
                .into_element()
        }))
        .into_element()
}

/// Sits in whatever is left of the hero beside the model so the text must wrap
fn hint_line(icon: IconType, text: String, color: Color) -> impl IntoElement {
    rect()
        .horizontal()
        .width(Size::fill())
        .content(Content::Flex)
        .cross_align(Alignment::Center)
        .spacing(6.)
        .child(Icon::new(icon).size(13.).color(color))
        .child(
            label()
                .text(text)
                .font_size(12.)
                .width(Size::flex(1.0))
                .color(color),
        )
        .into_element()
}

const REFRESH_SPIN_TIME: u64 = 800;

const REFRESHING_OPACITY: f32 = 0.7;

struct AccountRow {
    id: Uuid,
    username: String,
    kind: AccountKind,
    expires: DateTime<Utc>,
    skin_key: String,
    is_default: bool,
    set_default: crate::hooks::UseSetDefaultAccount,
    remove: crate::hooks::UseRemoveAccount,
    refresh: crate::hooks::UseRefreshAccount,
}

impl PartialEq for AccountRow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.username == other.username
            && self.kind == other.kind
            && self.skin_key == other.skin_key
            && self.expires == other.expires
            && self.is_default == other.is_default
    }
}

impl Component for AccountRow {
    fn render_key(&self) -> DiffKey {
        DiffKey::from(&self.id)
    }

    fn render(&self) -> impl IntoElement {
        let id = self.id;
        let is_default = self.is_default;
        let set_default = self.set_default;
        let remove = self.remove;
        let refresh = self.refresh;

        let is_online = self.kind != AccountKind::Offline;
        let expired = is_online && self.expires <= Utc::now();

        let mut refreshing = use_state(|| false);
        let is_refreshing = *refreshing.read();

        let spin = use_animation(|conf| {
            conf.on_finish(OnFinish::restart());
            AnimNum::new(0., 360.)
                .time(REFRESH_SPIN_TIME)
                .function(Function::Linear)
        });

        use_side_effect_with_deps(&is_refreshing, move |&is_refreshing| {
            let mut spin = spin;
            if is_refreshing {
                spin.start();
            } else {
                spin.reset();
            }
        });

        let rotation = if is_refreshing {
            spin.get().value()
        } else {
            0.
        };

        let border_color = if expired {
            colors::danger()
        } else if is_default {
            colors::brand()
        } else {
            colors::component_border()
        };

        rect()
            .horizontal()
            .width(Size::fill())
            .cross_align(Alignment::Center)
            .content(Content::Flex)
            .spacing(12.)
            .padding(Gaps::new_all(12.))
            .corner_radius(CornerRadius::new_all(12.))
            .background(colors::page_elevated())
            .border(border_all_color(1., border_color))
            .opacity(if is_refreshing {
                REFRESHING_OPACITY
            } else {
                1.
            })
            .a11y_role(AccessibilityRole::Button)
            .maybe(!is_default, |el| {
                el.on_press(move |_| set_default.mutate(SetDefaultAccountKeys { id: Some(id) }))
            })
            .child(
                Avatar::new(self.skin_key.clone())
                    .width(Size::px(AVATAR_SIZE_PX))
                    .height(Size::px(AVATAR_SIZE_PX)),
            )
            .child(
                rect()
                    .vertical()
                    .width(Size::flex(1.0))
                    .spacing(4.)
                    .child(
                        rect()
                            .horizontal()
                            .cross_align(Alignment::Center)
                            .spacing(6.)
                            .child(
                                label()
                                    .text(self.username.clone())
                                    .font_size(15.)
                                    .font_weight(FontWeight::MEDIUM)
                                    .max_lines(1)
                                    .color(colors::fg_primary()),
                            )
                            .maybe_child(is_default.then(default_badge))
                            .maybe_child(expired.then(expired_badge)),
                    )
                    // The kind rides with the id a third badge and a full uuid do not both fit
                    .child(
                        label()
                            .text(format!("{} · {id}", kind_label(self.kind)))
                            .font_size(11.)
                            .max_lines(1)
                            .color(colors::fg_secondary()),
                    ),
            )
            .maybe_child(is_online.then(|| {
                Button::new()
                    .ghost()
                    .icon()
                    .tooltip("Refresh this account")
                    .on_press(move |e: Event<PressEventData>| {
                        e.stop_propagation();
                        if *refreshing.peek() {
                            return;
                        }
                        refreshing.set(true);
                        spawn(async move {
                            refresh.mutate_async(RefreshAccountKeys { id }).await;
                            refreshing.set(false);
                        });
                    })
                    .child(
                        rect().rotate(rotation).child(
                            Icon::new(IconType::RefreshCw01)
                                .size(18.)
                                .color(if expired {
                                    colors::danger()
                                } else {
                                    colors::fg_secondary()
                                }),
                        ),
                    )
                    .into_element()
            }))
            .child(
                Button::new()
                    .ghost()
                    .icon()
                    .enabled(false)
                    .tooltip("Edit")
                    .child(
                        Icon::new(IconType::Pencil01)
                            .size(18.)
                            .color(colors::fg_secondary()),
                    ),
            )
            .child(
                Button::new()
                    .ghost()
                    .icon()
                    .tooltip("Remove account")
                    .on_press(move |e: Event<PressEventData>| {
                        e.stop_propagation();
                        remove.mutate(RemoveAccountKeys { id });
                    })
                    .child(
                        Icon::new(IconType::Trash01)
                            .size(18.)
                            .color(colors::fg_secondary()),
                    ),
            )
            .into_element()
    }
}

fn kind_label(kind: AccountKind) -> &'static str {
    match kind {
        AccountKind::Microsoft => "Microsoft",
        AccountKind::Elyby => "Ely.by",
        AccountKind::Offline => "Offline",
        AccountKind::Custom => "Custom",
    }
}

fn default_badge() -> impl IntoElement {
    badge(
        Icon::new(IconType::CheckCircle)
            .size(12.)
            .color(colors::brand())
            .into_element(),
        "Default".to_string(),
        colors::brand(),
        colors::brand(),
    )
}

fn expired_badge() -> impl IntoElement {
    badge(
        Icon::new(IconType::AlertTriangle)
            .size(12.)
            .color(colors::danger())
            .into_element(),
        "Expired".to_string(),
        colors::danger(),
        colors::danger(),
    )
}

fn badge(icon: impl IntoElement, text: String, border: Color, fg: Color) -> impl IntoElement {
    rect()
        .horizontal()
        .cross_align(Alignment::Center)
        .spacing(4.)
        .padding(Gaps::new_symmetric(2., 8.))
        .corner_radius(CornerRadius::new_all(999.))
        .border(border_all_color(1., border))
        .background(colors::component_bg())
        .child(icon)
        .child(
            label()
                .text(text)
                .font_size(10.)
                .font_weight(FontWeight::MEDIUM)
                .color(fg),
        )
        .into_element()
}

fn empty_state() -> Element {
    rect()
        .vertical()
        .width(Size::fill())
        .center()
        .padding(Gaps::new_all(32.))
        .spacing(8.)
        .corner_radius(CornerRadius::new_all(12.))
        .background(colors::page_elevated())
        .child(
            Icon::new(IconType::Users01)
                .size(32.)
                .color(colors::fg_secondary()),
        )
        .child(
            label()
                .text("No accounts yet. Add one above.")
                .font_size(14.)
                .color(colors::fg_secondary()),
        )
        .into_element()
}
