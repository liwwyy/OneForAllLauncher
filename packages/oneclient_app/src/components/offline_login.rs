use crate::components::{Button, Icon, IconType, OverlayPopup, TextInput, checkbox_labeled};
use crate::hooks::{AddOfflineAccountKeys, AddOfflineAccountMutation, use_add_offline_account};
use crate::theme::colors;
use crate::ui::border_all_color;
use freya::prelude::*;
use freya::query::{MutationStateData, UseMutation};

#[derive(Clone)]
pub struct OfflineLogin {
    username: State<String>,
    allow_invalid: State<bool>,
    show_offline: State<bool>,
    closing_offline: State<bool>,
    add_offline: UseMutation<AddOfflineAccountMutation>,
}

pub fn use_offline_login() -> OfflineLogin {
    let add_offline = use_add_offline_account();
    let mut username = use_state(String::new);
    let mut allow_invalid = use_state(|| false);
    let mut show_offline = use_state(|| false);
    let mut closing_offline = use_state(|| false);

    use_side_effect(move || {
        if !*closing_offline.read() {
            return;
        }
        match &*add_offline.read().state() {
            MutationStateData::Settled { res: Ok(_), .. } => {
                closing_offline.set(false);
                show_offline.set(false);
                username.set(String::new());
                allow_invalid.set(false);
            }
            MutationStateData::Settled { res: Err(_), .. } => {
                closing_offline.set(false);
            }
            _ => {}
        }
    });

    OfflineLogin {
        username,
        allow_invalid,
        show_offline,
        closing_offline,
        add_offline,
    }
}

impl OfflineLogin {
    pub fn open(&self) {
        let mut show = self.show_offline;
        show.set(true);
    }
    pub fn popup(&self) -> Option<Element> {
        let Self {
            username,
            allow_invalid,
            show_offline,
            mut closing_offline,
            add_offline,
        } = self.clone();
        let offline_name = username.read().clone();
        let offline_uuid = (!offline_name.is_empty())
            .then(|| oneclient_auth::offline_uuid(&offline_name).to_string());
        let offline_error = match &*add_offline.read().state() {
            MutationStateData::Settled { res: Err(err), .. } => Some(err.to_string()),
            MutationStateData::Loading {
                res: Some(Err(err)),
            } => Some(err.to_string()),
            _ => None,
        };
        let confirm = move |_| {
            let name = username.peek().clone();
            if oneclient_auth::validate_offline_username_with_override(&name, *allow_invalid.peek())
                .is_err()
            {
                return;
            }
            add_offline.mutate(AddOfflineAccountKeys {
                username: name,
                allow_invalid: *allow_invalid.peek(),
            });
            closing_offline.set(true);
        };
        let visible = *show_offline.read();
        visible.then(|| {
            offline_dialog(
                username,
                allow_invalid,
                *closing_offline.read(),
                offline_uuid,
                offline_error,
                confirm,
                show_offline,
            )
            .into_element()
        })
    }
}

fn offline_dialog(
    mut username: State<String>,
    allow_invalid: State<bool>,
    pending: bool,
    uuid_preview: Option<String>,
    error: Option<String>,
    on_confirm: impl FnMut(Event<PressEventData>) + 'static,
    mut show_offline: State<bool>,
) -> impl IntoElement {
    let name = username.read().clone();
    let validation =
        oneclient_auth::validate_offline_username_with_override(&name, *allow_invalid.read());
    let can_submit = validation.is_ok() && !pending;
    let validity_hint = if *allow_invalid.read() {
        "Invalid names may prevent joining servers or opening worlds.".to_string()
    } else if name.is_empty() {
        "Use 3–16 letters, digits or underscores.".to_string()
    } else {
        validation
            .err()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "Valid Minecraft username".into())
    };
    OverlayPopup::new()
        .on_close(move |()| show_offline.set(false))
        .child(
            rect()
                .width(Size::window_percent(100.))
                .height(Size::window_percent(100.))
                .center()
                .child(
                    rect()
                        .vertical()
                        .width(Size::px(380.))
                        .max_width(Size::window_percent(90.))
                        .spacing(16.)
                        .padding(Gaps::new_all(20.))
                        .corner_radius(CornerRadius::new_all(16.))
                        .background(colors::page_elevated())
                        .border(border_all_color(1., colors::component_border()))
                        .child(
                            label()
                                .text("Add offline account")
                                .font_size(18.)
                                .font_weight(FontWeight::SEMI_BOLD)
                                .color(colors::fg_primary()),
                        )
                        .child(
                            rect()
                                .vertical()
                                .width(Size::fill())
                                .spacing(6.)
                                .child(field_label("Username"))
                                .child(
                                    TextInput::new(username)
                                        .enabled(!pending)
                                        .placeholder("Offline username")
                                        .on_validate(move |validator: InputValidator| {
                                            validator.set_valid(
                                                oneclient_auth::offline_username_input_allowed(
                                                    &validator.text(),
                                                    *allow_invalid.peek(),
                                                ),
                                            );
                                        }),
                                ),
                        )
                        .child(checkbox_labeled(
                            allow_invalid,
                            "Allow invalid offline usernames",
                        ))
                        .child(
                            label()
                                .text(validity_hint)
                                .font_size(12.)
                                .color(if can_submit {
                                    colors::fg_secondary()
                                } else {
                                    colors::danger()
                                }),
                        )
                        .child(
                            rect()
                                .horizontal()
                                .spacing(8.)
                                .child(
                                    Button::new()
                                        .secondary()
                                        .enabled(!pending)
                                        .text("Random characters")
                                        .on_press(move |_| {
                                            username
                                                .set(oneclient_auth::random_offline_characters())
                                        }),
                                )
                                .child(
                                    Button::new()
                                        .secondary()
                                        .enabled(!pending)
                                        .text("Random username")
                                        .on_press(move |_| {
                                            username.set(oneclient_auth::random_offline_username())
                                        }),
                                ),
                        )
                        .child(
                            rect()
                                .vertical()
                                .width(Size::fill())
                                .spacing(6.)
                                .child(field_label("UUID"))
                                .child(
                                    label()
                                        .text(uuid_preview.unwrap_or_else(|| "-".to_string()))
                                        .font_size(12.)
                                        .color(colors::fg_secondary()),
                                ),
                        )
                        .map(error, |el, msg| {
                            el.child(hint_line(IconType::AlertTriangle, msg, colors::danger()))
                        })
                        .child(
                            rect()
                                .horizontal()
                                .width(Size::fill())
                                .main_align(Alignment::End)
                                .spacing(8.)
                                .child(
                                    Button::new()
                                        .ghost()
                                        .on_press(move |_| show_offline.set(false))
                                        .text("Cancel"),
                                )
                                .child(
                                    Button::new()
                                        .primary()
                                        .enabled(can_submit)
                                        .on_press(on_confirm)
                                        .child(Icon::new(IconType::Plus).size(16.))
                                        .text("Add account"),
                                ),
                        ),
                ),
        )
        .into_element()
}

fn field_label(text: &str) -> impl IntoElement {
    label()
        .text(text.to_string())
        .font_size(11.)
        .font_weight(FontWeight::MEDIUM)
        .color(colors::fg_secondary())
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
