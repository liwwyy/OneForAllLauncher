use crate::{
    components::{Button, Icon, IconType, OverlayPopup, TextInput, checkbox_controlled},
    theme::colors,
    ui::border_all_color,
};
use freya::prelude::*;
use oneclient_auth::{CustomLoginResponse, CustomServer};
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct CustomLogin {
    cancel: State<CancellationToken>,
    visible: State<bool>,
    busy: State<bool>,
    attempt: State<u64>,
    username: State<String>,
    password: State<String>,
    api: State<String>,
    login_path: State<String>,
    refresh_path: State<String>,
    server: State<Option<CustomServer>>,
    consent: State<bool>,
    response: State<Option<CustomLoginResponse>>,
    error: State<Option<String>>,
}

pub fn use_custom_login() -> CustomLogin {
    let handle = CustomLogin {
        cancel: use_state(CancellationToken::new),
        visible: use_state(|| false),
        busy: use_state(|| false),
        attempt: use_state(|| 0),
        username: use_state(String::new),
        password: use_state(String::new),
        api: use_state(String::new),
        login_path: use_state(String::new),
        refresh_path: use_state(String::new),
        server: use_state(|| None),
        consent: use_state(|| false),
        response: use_state(|| None),
        error: use_state(|| None),
    };
    let mut last_api = use_state(String::new);
    let api = handle.api;
    let mut server = handle.server;
    let mut consent = handle.consent;
    let mut login_path = handle.login_path;
    let mut refresh_path = handle.refresh_path;
    use_side_effect(move || {
        let current = api.read().clone();
        if current == *last_api.peek() {
            return;
        }
        last_api.set(current);
        server.set(None);
        consent.set(false);
        let (login, refresh) = oneclient_auth::custom_auth_paths_on_server_change(
            &login_path.peek(),
            &refresh_path.peek(),
        );
        if let Some(login) = login {
            login_path.set(login);
        }
        if let Some(refresh) = refresh {
            refresh_path.set(refresh);
        }
    });
    let cancel = handle.cancel;
    use_drop(move || cancel.peek().cancel());
    handle
}

impl CustomLogin {
    pub fn open(&self) {
        let mut cancel = self.cancel;
        cancel.set(CancellationToken::new());
        let mut visible = self.visible;
        visible.set(true);
    }

    fn close(&mut self) {
        self.cancel.peek().cancel();
        let next = *self.attempt.peek() + 1;
        self.attempt.set(next);
        self.visible.set(false);
        self.busy.set(false);
        self.password.set(String::new());
        self.response.set(None);
        self.server.set(None);
        self.consent.set(false);
        self.error.set(None);
    }

    fn add(&mut self) {
        if !*self.consent.peek() || *self.busy.peek() {
            return;
        }
        let api = self.api.peek().trim().to_string();
        let login = self.login_path.peek().trim().to_string();
        let refresh = self.refresh_path.peek().trim().to_string();
        if CustomServer::new(&api, &login, &refresh).is_err() {
            return;
        }
        let user = self.username.peek().clone();
        let password = self.password.peek().clone();
        if user.is_empty() || password.is_empty() {
            return;
        }
        self.password.set(String::new());
        self.busy.set(true);
        self.server.set(None);
        self.error.set(None);
        let mut handle = self.clone();
        let attempt = *handle.attempt.peek();
        let cancel = handle.cancel.peek().clone();
        spawn(async move {
            let discovery = crate::launcher::off_ui({
                let cancel = cancel.clone();
                async move {
                    let state = crate::launcher::state().map_err(|e| e.to_string())?;
                    tokio::select! {
                        () = cancel.cancelled() => Err("Login cancelled".to_string()),
                        result = state.auth.discover_custom_server(&api, &login, &refresh) => result.map_err(|e| e.to_string()),
                    }
                }
            }).await;
            if *handle.attempt.peek() != attempt {
                return;
            }
            let server = match discovery {
                Ok(server) => server,
                Err(error) => {
                    handle.error.set(Some(error));
                    handle.busy.set(false);
                    return;
                }
            };
            handle.server.set(Some(server.clone()));
            let result = crate::launcher::off_ui(async move {
                let state = crate::launcher::state().map_err(|e| e.to_string())?;
                tokio::select! {
                    () = cancel.cancelled() => Err("Login cancelled".to_string()),
                    result = state.auth.begin_custom_login(server, &user, &password) => result.map_err(|e| e.to_string()),
                }
            }).await;
            if *handle.attempt.peek() != attempt {
                return;
            }
            match result {
                Ok(response) => {
                    let only = (response.profiles.len() == 1).then(|| response.profiles[0].id);
                    handle.response.set(Some(response));
                    handle.busy.set(false);
                    if let Some(profile) = only {
                        handle.select(profile);
                    }
                }
                Err(error) => {
                    handle.error.set(Some(error));
                    handle.busy.set(false);
                }
            }
        });
    }

    fn select(&mut self, profile: uuid::Uuid) {
        if *self.busy.peek() {
            return;
        }
        let Some(response) = self.response.peek().clone() else {
            return;
        };
        self.busy.set(true);
        self.error.set(None);
        let mut handle = self.clone();
        let attempt = *handle.attempt.peek();
        let cancel = handle.cancel.peek().clone();
        spawn(async move {
            let result = crate::launcher::off_ui(async move {
                crate::launcher::state()
                    .map_err(|e| e.to_string())?
                    .auth
                    .finish_custom_login(response, profile, cancel)
                    .await
                    .map_err(|e| e.to_string())
            })
            .await;
            if *handle.attempt.peek() != attempt {
                return;
            }
            match result {
                Ok(account) => {
                    crate::hooks::invalidate_auth_queries(Some(account.id)).await;
                    handle.close();
                }
                Err(e) => {
                    handle.error.set(Some(e));
                    handle.busy.set(false);
                }
            }
        });
    }

    pub fn popup(&self) -> Option<Element> {
        if !*self.visible.read() {
            return None;
        }
        let busy = *self.busy.read();
        let server = self.server.read().clone();
        let response = self.response.read().clone();
        let mut close = self.clone();
        let mut cancel = self.clone();
        let mut action = self.clone();
        let mut consent = self.consent;
        let valid = !self.username.read().is_empty() && !self.password.read().is_empty();
        let can_review = oneclient_auth::CustomServer::new(
            &self.api.read(),
            &self.login_path.read(),
            &self.refresh_path.read(),
        )
        .is_ok();
        let mut form = rect()
            .vertical()
            .width(Size::px(480.))
            .max_width(Size::window_percent(90.))
            .spacing(12.)
            .padding(Gaps::new_all(24.))
            .corner_radius(CornerRadius::new_all(16.))
            .background(colors::page_elevated())
            .border(border_all_color(1., colors::component_border()))
            .child(
                label()
                    .text("Add custom account")
                    .font_size(20.)
                    .font_weight(FontWeight::SEMI_BOLD),
            );
        if let Some(response) = response {
            form = form.child(label().text("Choose your Minecraft profile").font_size(14.));
            for profile in response.profiles {
                let mut handle = self.clone();
                form = form.child(
                    Button::new()
                        .secondary()
                        .enabled(!busy)
                        .text(profile.name)
                        .on_press(move |_| handle.select(profile.id)),
                );
            }
        } else {
            form = form
                .child(field("Email / username", TextInput::new(self.username).enabled(!busy)))
                .child(field("Password", TextInput::new(self.password).mode(InputMode::new_password()).enabled(!busy)))
                .child(field("Authentication server", TextInput::new(self.api).enabled(!busy).placeholder("https://auth.example.com")))
                .child(rect().vertical().width(Size::fill()).spacing(8.).padding(Gaps::new_all(12.))
                    .corner_radius(CornerRadius::new_all(8.)).background(colors::component_bg())
                    .child(label().text("Options").font_size(14.).font_weight(FontWeight::SEMI_BOLD))
                    .child(label().text("Leave default value if you're not sure").font_size(12.).color(colors::fg_secondary()))
                    .child(field("Login path", TextInput::new(self.login_path).enabled(!busy).placeholder("/authserver/authenticate")))
                    .child(field("Refresh path", TextInput::new(self.refresh_path).enabled(!busy).placeholder("/authserver/refresh"))))
                .child(checkbox_controlled(*self.consent.read(), "I trust this authentication server", move |()| { if !busy { consent.toggle(); } }))
                .child(label().text("Your login credentials will be sent to this server and its declared authentication endpoint.")
                    .width(Size::fill()).font_size(12.).color(colors::fg_secondary()))
                .maybe_child(self.api.read().starts_with("http:").then(|| label().text("HTTP sends credentials without encryption.").font_size(12.).color(colors::danger()).into_element()))
                .maybe_child(server.map(|server| label().text(format!("Authentication endpoint: {}", server.api_url))
                    .width(Size::fill()).font_size(12.).color(colors::fg_secondary()).into_element()))
                .child(Button::new().primary().enabled(valid && can_review && *self.consent.read() && !busy)
                    .text(if busy { "Signing in…" } else { "Add account" })
                    .child(Icon::new(IconType::Custom).size(16.))
                    .on_press(move |_| action.add()));
        }
        form = form
            .maybe_child(self.error.read().clone().map(|e| {
                label()
                    .text(e)
                    .font_size(12.)
                    .color(colors::danger())
                    .into_element()
            }))
            .child(
                Button::new()
                    .ghost()
                    .text("Cancel")
                    .on_press(move |_| cancel.close()),
            );
        Some(
            OverlayPopup::new()
                .on_close(move |()| close.close())
                .child(
                    rect()
                        .width(Size::window_percent(100.))
                        .height(Size::window_percent(100.))
                        .center()
                        .child(
                            ScrollView::new()
                                .width(Size::px(480.))
                                .max_width(Size::window_percent(90.))
                                .height(Size::Inner)
                                .max_height(Size::window_percent(90.))
                                .child(form),
                        ),
                )
                .into_element(),
        )
    }
}

fn field(name: &str, input: TextInput) -> impl IntoElement {
    rect()
        .vertical()
        .width(Size::fill())
        .spacing(6.)
        .child(
            label()
                .text(name.to_string())
                .font_size(12.)
                .color(colors::fg_secondary()),
        )
        .child(input)
}
