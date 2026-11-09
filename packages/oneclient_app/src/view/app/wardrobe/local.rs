use freya::prelude::*;
use freya::query::QueriesStorage;
use oneclient_auth::MinecraftAccount;

use crate::components::{Button, Icon, IconType, PlayerModel, ScrollArea, toggle_controlled};
use crate::hooks::{LocalSkinQuery, query_error, settled_or_loading, use_local_skin};
use crate::i18n::t;
use crate::theme::{colors, rounded, text_size};

pub struct LocalWardrobe {
    pub account: MinecraftAccount,
}

impl PartialEq for LocalWardrobe {
    fn eq(&self, other: &Self) -> bool {
        self.account.id == other.account.id
    }
}

impl Component for LocalWardrobe {
    fn render(&self) -> impl IntoElement {
        let account = self.account.clone();
        let id = account.id;
        let saved = use_local_skin(Some(id));
        let skin = settled_or_loading(&saved).flatten();
        let mut slim = use_state(|| false);
        let mut pending = use_state(|| false);
        let mut message = use_state(|| None::<String>);
        let selecting = account.clone();
        let restoring = account.clone();
        let changing_model = account.clone();
        let model_skin = skin.clone();
        let loading_error = query_error(&saved);
        let existing_slim = skin.as_ref().map(|(_, slim)| *slim);
        let mut last_saved_model = use_state(|| None::<bool>);
        let mut model = slim;
        use_side_effect(move || {
            if existing_slim != *last_saved_model.peek() {
                last_saved_model.set(existing_slim);
                model.set(existing_slim.unwrap_or(false));
            }
        });
        let content = rect()
            .width(Size::fill())
            .vertical()
            .spacing(20.)
            .padding(Gaps::new_all(24.))
            .child(
                label()
                    .text(t("fork.wardrobe.title"))
                    .width(Size::fill())
                    .font_size(text_size(28.))
                    .font_weight(FontWeight::BOLD)
                    .color(colors::fg_primary()),
            )
            .child(
                label()
                    .text(t("fork.wardrobe.description"))
                    .width(Size::fill())
                    .font_size(text_size(14.))
                    .color(colors::fg_secondary()),
            )
            .child(
                label()
                    .text(t("fork.wardrobe.supported"))
                    .width(Size::fill())
                    .font_size(text_size(12.))
                    .color(colors::fg_secondary()),
            )
            .child(
                rect()
                    .height(Size::px(300.))
                    .corner_radius(rounded(16.))
                    .background(colors::component_bg())
                    .child(
                        PlayerModel::new(account.skin_profile_key())
                            .local_only()
                            .skin(skin.map(|(bytes, _)| (bytes, *slim.read()))),
                    ),
            )
            .child(
                rect()
                    .horizontal()
                    .spacing(12.)
                    .cross_align(Alignment::Center)
                    .child(
                        label()
                            .text(t("fork.wardrobe.slim"))
                            .color(colors::fg_primary()),
                    )
                    .child(toggle_controlled(
                        *slim.read(),
                        (move |()| {
                            if *pending.peek() {
                                return;
                            }
                            let value = !*slim.peek();
                            slim.set(value);
                            if let Some((bytes, _)) = model_skin.clone() {
                                pending.set(true);
                                let account = changing_model.clone();
                                spawn(async move {
                                    let result =
                                        oneclient_core::local_skin::save(&account, &bytes, value)
                                            .await;
                                    message.set(Some(match result {
                                        Ok(()) => t("fork.wardrobe.saved").into(),
                                        Err(error) => error.to_string(),
                                    }));
                                    QueriesStorage::<LocalSkinQuery>::invalidate_matching(id).await;
                                    pending.set(false);
                                });
                            }
                        })
                        .into(),
                    )),
            )
            .child(
                rect()
                    .horizontal()
                    .spacing(12.)
                    .child(
                        Button::new()
                            .primary()
                            .enabled(!*pending.read())
                            .child(Icon::new(IconType::Brush01).size(16.))
                            .text(t("fork.wardrobe.choose"))
                            .on_press(move |_| {
                                if *pending.peek() {
                                    return;
                                }
                                pending.set(true);
                                let account = selecting.clone();
                                let chosen_slim = *slim.peek();
                                spawn(async move {
                                    if let Some(file) = rfd::AsyncFileDialog::new()
                                        .add_filter("PNG skin", &["png"])
                                        .pick_file()
                                        .await
                                    {
                                        let result = async {
                                            let bytes = tokio::fs::read(file.path()).await?;
                                            oneclient_core::local_skin::save(
                                                &account,
                                                &bytes,
                                                chosen_slim,
                                            )
                                            .await
                                        }
                                        .await;
                                        message.set(Some(match result {
                                            Ok(()) => t("fork.wardrobe.saved").into(),
                                            Err(error) => error.to_string(),
                                        }));
                                        QueriesStorage::<LocalSkinQuery>::invalidate_matching(id)
                                            .await;
                                    }
                                    pending.set(false);
                                });
                            }),
                    )
                    .child(
                        Button::new()
                            .secondary()
                            .enabled(!*pending.read())
                            .text(t("fork.wardrobe.restore"))
                            .on_press(move |_| {
                                if *pending.peek() {
                                    return;
                                }
                                pending.set(true);
                                let account = restoring.clone();
                                spawn(async move {
                                    let result =
                                        oneclient_core::local_skin::restore(&account).await;
                                    message.set(Some(match result {
                                        Ok(()) => t("fork.wardrobe.restored").into(),
                                        Err(error) => error.to_string(),
                                    }));
                                    QueriesStorage::<LocalSkinQuery>::invalidate_matching(id).await;
                                    pending.set(false);
                                });
                            }),
                    ),
            )
            .maybe_child(message.read().clone().or(loading_error).map(|text| {
                label()
                    .text(text)
                    .color(colors::fg_secondary())
                    .into_element()
            }));
        ScrollArea::new()
            .width(Size::fill())
            .height(Size::fill())
            .child(content)
    }
}
