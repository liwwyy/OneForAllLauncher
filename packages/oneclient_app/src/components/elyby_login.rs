use crate::{
    components::{Button, Icon, IconType, OverlayPopup},
    hooks::*,
    platform,
    theme::colors,
    ui::border_all_color,
};
use freya::prelude::*;
use freya::query::{MutationCapability, MutationStateData, UseMutation};
use freya::text_edit::Clipboard;
use oneclient_auth::ElybyLoginSession;

#[derive(Clone)]
pub struct ElybyLogin {
    begin: UseMutation<BeginElybyLoginMutation>,
    cancel: UseMutation<CancelElybyLoginMutation>,
    session: State<Option<ElybyLoginSession>>,
    pub pending: bool,
    pub error: Option<String>,
}
impl ElybyLogin {
    pub fn start(&self) {
        self.begin.mutate(());
    }
    pub fn popup(&self) -> Option<Element> {
        let login = self.session.read().clone()?;
        let mut session = self.session;
        let cancel = self.cancel;
        let close = move || {
            if let Some(login) = session.peek().clone() {
                cancel.mutate(login.device.device_code);
            }
            session.set(None);
        };
        let mut scrim_close = close;
        let mut button_close = close;
        let url = login.device.verification_uri;
        let code = login.device.user_code;
        let copy = code.clone();
        Some(OverlayPopup::new().on_close(move |()| scrim_close()).child(
            rect().width(Size::window_percent(100.)).height(Size::window_percent(100.)).center().child(
                rect().vertical().width(Size::px(420.)).max_width(Size::window_percent(90.)).spacing(18.).padding(Gaps::new_all(28.))
                    .corner_radius(CornerRadius::new_all(16.)).background(colors::page_elevated()).border(border_all_color(1., colors::component_border()))
                    .child(Icon::new(IconType::Elyby).size(48.))
                    .child(label().text("Sign in to Ely.by").font_size(20.).font_weight(FontWeight::SEMI_BOLD))
                    .child(label().text("Enter this code in your browser to authorize OneForAllLauncher.").font_size(13.).color(colors::fg_secondary()))
                    .child(label().text(code).font_size(36.).font_weight(FontWeight::BOLD))
                    .child(Button::new().secondary().on_press(move |_| { let _ = Clipboard::set(copy.clone()); }).child(Icon::new(IconType::Copy01).size(16.)).text("Copy code"))
                    .child(Button::new().primary().on_press(move |_| platform::open_url(&url)).text("Open Ely.by in browser"))
                    .child(label().text(self.error.clone().unwrap_or_else(|| "Waiting for browser authorization…".into())).font_size(12.).color(if self.error.is_some() { colors::danger() } else { colors::fg_secondary() }))
                    .child(Button::new().ghost().on_press(move |_| button_close()).text("Cancel"))
            )
        ).into_element())
    }
}
pub fn use_elyby_login() -> ElybyLogin {
    let begin = use_begin_elyby_login();
    let finish = use_finish_elyby_login();
    let cancel = use_cancel_elyby_login();
    let mut session = use_state(|| None::<ElybyLoginSession>);
    let mut handled = use_state(|| None::<String>);
    use_side_effect(move || {
        if let Some(login) = mutation_ok(&begin) {
            if handled.peek().as_deref() == Some(login.device.device_code.as_str()) {
                return;
            }
            handled.set(Some(login.device.device_code.clone()));
            platform::open_url(&login.device.verification_uri);
            finish.mutate(login.clone());
            session.set(Some(login));
        }
    });
    use_side_effect(move || {
        if mutation_ok(&finish).is_some() && session.peek().is_some() {
            session.set(None);
        }
    });
    let pending = mutation_is_running(&begin) || mutation_is_running(&finish);
    let error = login_error(&finish).or_else(|| login_error(&begin));
    ElybyLogin {
        begin,
        cancel,
        session,
        pending,
        error,
    }
}

fn login_error<M: MutationCapability<Err = oneclient_core::LauncherError>>(
    mutation: &UseMutation<M>,
) -> Option<String> {
    match &*mutation.read().state() {
        MutationStateData::Settled {
            res: Err(error), ..
        } if !error.is_login_cancelled() => Some(error.to_string()),
        _ => None,
    }
}
