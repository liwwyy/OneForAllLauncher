use crate::{
    components::{Button, Icon, IconType, OverlayPopup},
    hooks::*,
    platform,
    theme::colors,
    ui::border_all_color,
};
use freya::prelude::*;
use freya::query::{MutationStateData, UseMutation};
use freya::text_edit::Clipboard;
use oneclient_auth::ElybyLoginSession;

#[derive(Clone)]
pub struct ElybyLogin {
    begin: UseMutation<BeginElybyLoginMutation>,
    finish: UseMutation<FinishElybyLoginMutation>,
    cancel: UseMutation<CancelElybyLoginMutation>,
    session: State<Option<ElybyLoginSession>>,
    busy: State<bool>,
    attempt: State<u64>,
    failure: State<Option<String>>,
    pub pending: bool,
    pub error: Option<String>,
}
impl ElybyLogin {
    pub fn start(&self) {
        if *self.busy.peek() {
            return;
        }
        let mut handle = self.clone();
        let attempt = *handle.attempt.peek() + 1;
        handle.attempt.set(attempt);
        handle.busy.set(true);
        handle.failure.set(None);
        // Only an explicit click starts a flow. Never replay shared mutation
        // results when this hook mounts on another account screen.
        spawn(async move {
            let result = handle.begin.mutate_async(()).await;
            let login = match &*result.state() {
                MutationStateData::Settled { res: Ok(login), .. } => Some(login.clone()),
                MutationStateData::Settled {
                    res: Err(error), ..
                } => {
                    if *handle.attempt.peek() == attempt {
                        handle.failure.set(Some(error.to_string()));
                    }
                    None
                }
                _ => None,
            };
            let Some(login) = login else {
                if *handle.attempt.peek() == attempt {
                    handle.busy.set(false);
                }
                return;
            };
            if *handle.attempt.peek() != attempt {
                handle.cancel.mutate(login.device.device_code);
                return;
            }
            handle.session.set(Some(login.clone()));
            platform::open_url(&login.device.verification_uri);
            let result = handle.finish.mutate_async(login).await;
            if *handle.attempt.peek() != attempt {
                return;
            }
            if let MutationStateData::Settled {
                res: Err(error), ..
            } = &*result.state()
            {
                if !error.is_login_cancelled() {
                    handle.failure.set(Some(error.to_string()));
                } else {
                    handle.session.set(None);
                }
            } else {
                handle.session.set(None);
            }
            handle.busy.set(false);
        });
    }
    pub fn popup(&self) -> Option<Element> {
        let login = self.session.read().clone()?;
        let mut session = self.session;
        let cancel = self.cancel;
        let mut attempt = self.attempt;
        let mut busy = self.busy;
        let close = move || {
            let next = *attempt.peek() + 1;
            attempt.set(next);
            busy.set(false);
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
    let session = use_state(|| None::<ElybyLoginSession>);
    let busy = use_state(|| false);
    let attempt = use_state(|| 0u64);
    let failure = use_state(|| None::<String>);
    use_drop(move || {
        if let Some(login) = session.peek().clone() {
            cancel.mutate(login.device.device_code);
        }
    });
    let pending = *busy.read();
    let error = failure.read().clone();
    ElybyLogin {
        begin,
        finish,
        cancel,
        session,
        busy,
        attempt,
        failure,
        pending,
        error,
    }
}
