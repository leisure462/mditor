use anyhow::Context as _;
use copilot::{
    Copilot, GlobalCopilotAuth, Status,
    request::{self, PromptUserDeviceFlow},
};
use copilot_chat::{CopilotChat, CopilotChatConfiguration};
use gpui::{
    App, ClipboardItem, Context, DismissEvent, Element, Entity, EventEmitter, FocusHandle,
    Focusable, InteractiveElement, IntoElement, MouseDownEvent, ParentElement, Render, Styled,
    Subscription, Window, WindowBounds, WindowOptions, div, point,
};
use project::project_settings::ProjectSettings;
use settings::Settings as _;
use ui::{ButtonLike, CommonAnimationExt, ConfiguredApiCard, Vector, VectorName, prelude::*};
use util::ResultExt as _;
use workspace::{AppState, Toast, Workspace, notifications::NotificationId};

const COPILOT_SIGN_UP_URL: &str = "https://github.com/features/copilot";
const ERROR_LABEL: &str = "Copilot 启动时出现问题。你可以尝试重新安装并再次登录。";

struct CopilotStatusToast;

pub fn initiate_sign_in(copilot: Entity<Copilot>, window: &mut Window, cx: &mut App) {
    let is_reinstall = false;
    initiate_sign_in_impl(copilot, is_reinstall, window, cx)
}

pub fn initiate_sign_out(copilot: Entity<Copilot>, window: &mut Window, cx: &mut App) {
    copilot_toast(Some("正在退出 Copilot 登录…"), window, cx);

    let sign_out_task = copilot.update(cx, |copilot, cx| copilot.sign_out(cx));
    window
        .spawn(cx, async move |cx| match sign_out_task.await {
            Ok(()) => {
                cx.update(|window, cx| copilot_toast(Some("已退出 Copilot 登录"), window, cx))
            }
            Err(err) => cx.update(|window, cx| {
                if let Some(workspace) = Workspace::for_window(window, cx) {
                    workspace.update(cx, |workspace, cx| {
                        workspace.show_error(&err, cx);
                    })
                } else {
                    log::error!("{:?}", err);
                }
            }),
        })
        .detach();
}

pub fn reinstall_and_sign_in(copilot: Entity<Copilot>, window: &mut Window, cx: &mut App) {
    let _ = copilot.update(cx, |copilot, cx| copilot.reinstall(cx));
    let is_reinstall = true;
    initiate_sign_in_impl(copilot, is_reinstall, window, cx);
}

fn open_copilot_code_verification_window(copilot: &Entity<Copilot>, window: &Window, cx: &mut App) {
    let current_window_center = window.bounds().center();
    let height = px(450.);
    let width = px(350.);
    let window_bounds = WindowBounds::Windowed(gpui::bounds(
        current_window_center - point(height / 2.0, width / 2.0),
        gpui::size(height, width),
    ));
    cx.open_window(
        WindowOptions {
            kind: gpui::WindowKind::PopUp,
            window_bounds: Some(window_bounds),
            is_resizable: false,
            is_movable: true,
            titlebar: Some(gpui::TitlebarOptions {
                appears_transparent: true,
                ..Default::default()
            }),
            ..Default::default()
        },
        |window, cx| cx.new(|cx| CopilotCodeVerification::new(&copilot, window, cx)),
    )
    .context("打开 Copilot 验证窗口失败")
    .log_err();
}

fn copilot_toast(message: Option<&'static str>, window: &Window, cx: &mut App) {
    const NOTIFICATION_ID: NotificationId = NotificationId::unique::<CopilotStatusToast>();

    let Some(workspace) = Workspace::for_window(window, cx) else {
        return;
    };

    cx.defer(move |cx| {
        workspace.update(cx, |workspace, cx| match message {
            Some(message) => workspace.show_toast(Toast::new(NOTIFICATION_ID, message), cx),
            None => workspace.dismiss_toast(&NOTIFICATION_ID, cx),
        });
    })
}

pub fn initiate_sign_in_impl(
    copilot: Entity<Copilot>,
    is_reinstall: bool,
    window: &mut Window,
    cx: &mut App,
) {
    if matches!(copilot.read(cx).status(), Status::Disabled) {
        copilot.update(cx, |copilot, cx| copilot.start_copilot(false, true, cx));
    }
    match copilot.read(cx).status() {
        Status::Starting { task } => {
            copilot_toast(
                Some(if is_reinstall {
                    "正在重新安装 Copilot…"
                } else {
                    "正在启动 Copilot…"
                }),
                window,
                cx,
            );

            window
                .spawn(cx, async move |cx| {
                    task.await;
                    cx.update(|window, cx| match copilot.read(cx).status() {
                        Status::Authorized => copilot_toast(Some("Copilot 已启动。"), window, cx),
                        _ => {
                            copilot_toast(None, window, cx);
                            copilot
                                .update(cx, |copilot, cx| copilot.sign_in(cx))
                                .detach_and_log_err(cx);
                            open_copilot_code_verification_window(&copilot, window, cx);
                        }
                    })
                    .log_err();
                })
                .detach();
        }
        _ => {
            copilot
                .update(cx, |copilot, cx| copilot.sign_in(cx))
                .detach();
            open_copilot_code_verification_window(&copilot, window, cx);
        }
    }
}

pub struct CopilotCodeVerification {
    status: Status,
    connect_clicked: bool,
    focus_handle: FocusHandle,
    copilot: Entity<Copilot>,
    _subscription: Subscription,
    sign_up_url: Option<String>,
}

impl Focusable for CopilotCodeVerification {
    fn focus_handle(&self, _: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<DismissEvent> for CopilotCodeVerification {}

impl CopilotCodeVerification {
    pub fn new(copilot: &Entity<Copilot>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        window.on_window_should_close(cx, |window, cx| {
            if let Some(this) = window.root::<CopilotCodeVerification>().flatten() {
                this.update(cx, |this, cx| {
                    this.before_dismiss(cx);
                });
            }
            true
        });
        cx.subscribe_in(
            &cx.entity(),
            window,
            |this, _, _: &DismissEvent, window, cx| {
                window.remove_window();
                this.before_dismiss(cx);
            },
        )
        .detach();

        let status = copilot.read(cx).status();
        Self {
            status,
            connect_clicked: false,
            focus_handle: cx.focus_handle(),
            copilot: copilot.clone(),
            sign_up_url: None,
            _subscription: cx.observe(copilot, |this, copilot, cx| {
                let status = copilot.read(cx).status();
                match status {
                    Status::Authorized | Status::Unauthorized | Status::SigningIn { .. } => {
                        this.set_status(status, cx)
                    }
                    _ => cx.emit(DismissEvent),
                }
            }),
        }
    }

    pub fn set_status(&mut self, status: Status, cx: &mut Context<Self>) {
        self.status = status;
        cx.notify();
    }

    fn render_device_code(data: &PromptUserDeviceFlow, cx: &mut Context<Self>) -> impl IntoElement {
        let copied = cx
            .read_from_clipboard()
            .map(|item| item.text().as_ref() == Some(&data.user_code))
            .unwrap_or(false);

        ButtonLike::new("copy-button")
            .full_width()
            .style(ButtonStyle::Tinted(ui::TintColor::Accent))
            .size(ButtonSize::Medium)
            .child(
                h_flex()
                    .w_full()
                    .p_1()
                    .justify_between()
                    .child(Label::new(data.user_code.clone()))
                    .child(Label::new(if copied { "已复制" } else { "复制" })),
            )
            .on_click({
                let user_code = data.user_code.clone();
                move |_, window, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(user_code.clone()));
                    window.refresh();
                }
            })
    }

    fn render_prompting_modal(
        copilot: Entity<Copilot>,
        connect_clicked: bool,
        data: &PromptUserDeviceFlow,
        cx: &mut Context<Self>,
    ) -> impl Element {
        let connect_button_label = if connect_clicked {
            "等待连接中…"
        } else {
            "连接 GitHub"
        };

        v_flex()
            .flex_1()
            .gap_2p5()
            .items_center()
            .text_center()
            .child(Headline::new("在 Prism 中使用 GitHub Copilot").size(HeadlineSize::Large))
            .child(Label::new("使用 Copilot 需要你在 GitHub 上拥有有效订阅。").color(Color::Muted))
            .child(Self::render_device_code(data, cx))
            .child(Label::new("点击下方按钮后，将此代码粘贴到 GitHub 中。").color(Color::Muted))
            .child(
                v_flex()
                    .w_full()
                    .gap_1()
                    .child(
                        Button::new("connect-button", connect_button_label)
                            .full_width()
                            .style(ButtonStyle::Outlined)
                            .size(ButtonSize::Medium)
                            .on_click({
                                let command = data.command.clone();
                                cx.listener(move |this, _, _window, cx| {
                                    let command = command.clone();
                                    let copilot_clone = copilot.clone();
                                    let request_timeout = ProjectSettings::get_global(cx)
                                        .global_lsp_settings
                                        .get_request_timeout();
                                    copilot.update(cx, |copilot, cx| {
                                        if let Some(server) = copilot.language_server() {
                                            let server = server.clone();
                                            cx.spawn(async move |_, cx| {
                                                let result = server
                                                    .request::<lsp::request::ExecuteCommand>(
                                                        lsp::ExecuteCommandParams {
                                                            command: command.command.clone(),
                                                            arguments: command
                                                                .arguments
                                                                .clone()
                                                                .unwrap_or_default(),
                                                            ..Default::default()
                                                        },
                                                        request_timeout,
                                                    )
                                                    .await
                                                    .into_response()
                                                    .ok()
                                                    .flatten();
                                                if let Some(value) = result {
                                                    if let Ok(status) = serde_json::from_value::<
                                                        request::SignInStatus,
                                                    >(
                                                        value
                                                    ) {
                                                        copilot_clone.update(cx, |copilot, cx| {
                                                            copilot
                                                                .update_sign_in_status(status, cx);
                                                        });
                                                    }
                                                }
                                            })
                                            .detach();
                                        }
                                    });

                                    this.connect_clicked = true;
                                })
                            }),
                    )
                    .child(
                        Button::new("copilot-enable-cancel-button", "取消")
                            .full_width()
                            .size(ButtonSize::Medium)
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.emit(DismissEvent);
                            })),
                    ),
            )
    }

    fn render_enabled_modal(cx: &mut Context<Self>) -> impl Element {
        v_flex()
            .gap_2()
            .text_center()
            .justify_center()
            .child(Headline::new("Copilot 已启用！").size(HeadlineSize::Large))
            .child(Label::new("现在可以正常使用 GitHub Copilot 了。").color(Color::Muted))
            .child(
                Button::new("copilot-enabled-done-button", "完成")
                    .full_width()
                    .style(ButtonStyle::Outlined)
                    .size(ButtonSize::Medium)
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(DismissEvent))),
            )
    }

    fn render_unauthorized_modal(&self, cx: &mut Context<Self>) -> impl Element {
        let sign_up_url = self
            .sign_up_url
            .as_deref()
            .unwrap_or(COPILOT_SIGN_UP_URL)
            .to_owned();
        let description = "订阅或续订后，连接你现有的许可证即可启用 Copilot。";

        v_flex()
            .gap_2()
            .text_center()
            .justify_center()
            .child(
                Headline::new("你必须拥有有效的 GitHub Copilot 订阅。").size(HeadlineSize::Large),
            )
            .child(Label::new(description).color(Color::Warning))
            .child(
                Button::new("copilot-subscribe-button", "前往 GitHub 订阅")
                    .full_width()
                    .style(ButtonStyle::Outlined)
                    .size(ButtonSize::Medium)
                    .on_click(move |_, _, cx| cx.open_url(&sign_up_url)),
            )
            .child(
                Button::new("copilot-subscribe-cancel-button", "取消")
                    .full_width()
                    .size(ButtonSize::Medium)
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(DismissEvent))),
            )
    }

    fn render_error_modal(copilot: Entity<Copilot>, _cx: &mut Context<Self>) -> impl Element {
        v_flex()
            .gap_2()
            .text_center()
            .justify_center()
            .child(Headline::new("发生错误").size(HeadlineSize::Large))
            .child(Label::new(ERROR_LABEL).color(Color::Muted))
            .child(
                Button::new("copilot-subscribe-button", "重新安装 Copilot 并登录")
                    .full_width()
                    .style(ButtonStyle::Outlined)
                    .size(ButtonSize::Medium)
                    .start_icon(
                        Icon::new(IconName::Download)
                            .size(IconSize::Small)
                            .color(Color::Muted),
                    )
                    .on_click(move |_, window, cx| {
                        reinstall_and_sign_in(copilot.clone(), window, cx)
                    }),
            )
    }

    fn before_dismiss(
        &mut self,
        cx: &mut Context<'_, CopilotCodeVerification>,
    ) -> workspace::DismissDecision {
        self.copilot.update(cx, |copilot, cx| {
            if matches!(copilot.status(), Status::SigningIn { .. }) {
                copilot.sign_out(cx).detach_and_log_err(cx);
            }
        });
        workspace::DismissDecision::Dismiss(true)
    }
}

impl Render for CopilotCodeVerification {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let prompt = match &self.status {
            Status::SigningIn { prompt: None } => Icon::new(IconName::ArrowCircle)
                .color(Color::Muted)
                .with_rotate_animation(2)
                .into_any_element(),
            Status::SigningIn {
                prompt: Some(prompt),
            } => {
                Self::render_prompting_modal(self.copilot.clone(), self.connect_clicked, prompt, cx)
                    .into_any_element()
            }
            Status::Unauthorized => {
                self.connect_clicked = false;
                self.render_unauthorized_modal(cx).into_any_element()
            }
            Status::Authorized => {
                self.connect_clicked = false;
                Self::render_enabled_modal(cx).into_any_element()
            }
            Status::Error(..) => {
                Self::render_error_modal(self.copilot.clone(), cx).into_any_element()
            }
            _ => div().into_any_element(),
        };

        v_flex()
            .id("copilot_code_verification")
            .track_focus(&self.focus_handle(cx))
            .size_full()
            .px_4()
            .py_8()
            .gap_2()
            .items_center()
            .justify_center()
            .elevation_3(cx)
            .on_action(cx.listener(|_, _: &menu::Cancel, _, cx| {
                cx.emit(DismissEvent);
            }))
            .on_any_mouse_down(cx.listener(|this, _: &MouseDownEvent, window, cx| {
                window.focus(&this.focus_handle, cx);
            }))
            .child(
                Vector::new(VectorName::ZedXCopilot, rems(8.), rems(4.))
                    .color(Color::Custom(cx.theme().colors().icon)),
            )
            .child(prompt)
    }
}

pub struct ConfigurationView {
    copilot_status: Option<Status>,
    is_authenticated: Box<dyn Fn(&mut App) -> bool + 'static>,
    edit_prediction: bool,
    _copilot_chat_subscription: Option<Subscription>,
    _subscription: Option<Subscription>,
}

pub enum ConfigurationMode {
    Chat,
    EditPrediction,
}

impl ConfigurationView {
    pub fn new(
        is_authenticated: impl Fn(&mut App) -> bool + 'static,
        mode: ConfigurationMode,
        cx: &mut Context<Self>,
    ) -> Self {
        let app_state = AppState::try_global(cx).and_then(|state| state.upgrade());

        if CopilotChat::global(cx).is_none()
            && let Some(app_state) = app_state.as_ref()
        {
            copilot_chat::init(
                app_state.fs.clone(),
                app_state.client.http_client(),
                CopilotChatConfiguration::default(),
                cx,
            );
        }

        let copilot = app_state.and_then(|state| GlobalCopilotAuth::try_get_or_init(state, cx));
        let copilot_chat_subscription = CopilotChat::global(cx)
            .map(|copilot_chat| cx.observe(&copilot_chat, |_, _, cx| cx.notify()));

        Self {
            copilot_status: copilot.as_ref().map(|copilot| copilot.0.read(cx).status()),
            is_authenticated: Box::new(is_authenticated),
            edit_prediction: matches!(mode, ConfigurationMode::EditPrediction),
            _copilot_chat_subscription: copilot_chat_subscription,
            _subscription: copilot.as_ref().map(|copilot| {
                cx.observe(&copilot.0, |this, model, cx| {
                    this.copilot_status = Some(model.read(cx).status());
                    cx.notify();
                })
            }),
        }
    }
}

impl ConfigurationView {
    fn is_starting(&self) -> bool {
        matches!(&self.copilot_status, Some(Status::Starting { .. }))
    }

    fn is_signing_in(&self) -> bool {
        matches!(
            &self.copilot_status,
            Some(Status::SigningIn { .. })
                | Some(Status::SignedOut {
                    awaiting_signing_in: true
                })
        )
    }

    fn is_error(&self) -> bool {
        matches!(&self.copilot_status, Some(Status::Error(_)))
    }

    fn has_no_status(&self) -> bool {
        self.copilot_status.is_none()
    }

    fn loading_message(&self) -> Option<SharedString> {
        if self.is_starting() {
            Some("正在启动 Copilot…".into())
        } else if self.is_signing_in() {
            Some("正在登录 Copilot…".into())
        } else {
            None
        }
    }

    fn render_loading_button(
        &self,
        label: impl Into<SharedString>,
        edit_prediction: bool,
    ) -> impl IntoElement {
        ButtonLike::new("loading_button")
            .disabled(true)
            .style(ButtonStyle::Outlined)
            .when(edit_prediction, |this| this.size(ButtonSize::Medium))
            .child(
                h_flex()
                    .w_full()
                    .gap_1()
                    .justify_center()
                    .child(
                        Icon::new(IconName::ArrowCircle)
                            .size(IconSize::Small)
                            .color(Color::Muted)
                            .with_rotate_animation(4),
                    )
                    .child(Label::new(label)),
            )
    }

    fn render_sign_in_button(&self, edit_prediction: bool) -> impl IntoElement {
        let label = if edit_prediction {
            "登录 GitHub"
        } else {
            "登录以使用 GitHub Copilot"
        };

        Button::new("sign_in", label)
            .map(|this| {
                if edit_prediction {
                    this.size(ButtonSize::Medium)
                } else {
                    this.full_width()
                }
            })
            .style(ButtonStyle::Outlined)
            .start_icon(
                Icon::new(IconName::Github)
                    .size(IconSize::Small)
                    .color(Color::Muted),
            )
            .when(edit_prediction, |this| this.tab_index(0isize))
            .on_click(|_, window, cx| {
                if let Some(app_state) = AppState::global(cx).upgrade()
                    && let Some(copilot) = GlobalCopilotAuth::try_get_or_init(app_state, cx)
                {
                    initiate_sign_in(copilot.0, window, cx)
                }
            })
    }

    fn render_reinstall_button(&self, edit_prediction: bool) -> impl IntoElement {
        let label = if edit_prediction {
            "重新安装并登录"
        } else {
            "重新安装 Copilot 并登录"
        };

        Button::new("reinstall_and_sign_in", label)
            .map(|this| {
                if edit_prediction {
                    this.size(ButtonSize::Medium)
                } else {
                    this.full_width()
                }
            })
            .style(ButtonStyle::Outlined)
            .start_icon(
                Icon::new(IconName::Download)
                    .size(IconSize::Small)
                    .color(Color::Muted),
            )
            .on_click(|_, window, cx| {
                if let Some(app_state) = AppState::global(cx).upgrade()
                    && let Some(copilot) = GlobalCopilotAuth::try_get_or_init(app_state, cx)
                {
                    reinstall_and_sign_in(copilot.0, window, cx);
                }
            })
    }

    fn render_for_edit_prediction(&self) -> impl IntoElement {
        let container = |description: SharedString, action: AnyElement| {
            h_flex()
                .pt_2p5()
                .w_full()
                .justify_between()
                .child(
                    v_flex()
                        .w_full()
                        .max_w_1_2()
                        .child(Label::new("登录后使用"))
                        .child(
                            Label::new(description)
                                .color(Color::Muted)
                                .size(LabelSize::Small),
                        ),
                )
                .child(action)
        };

        let start_label = "要将 Copilot 用于编辑预测，你需要先登录 GitHub。请注意，你的 GitHub 账户必须拥有有效的 Copilot 订阅。".into();
        let no_status_label = "Copilot 需要有效的 GitHub Copilot 订阅。请确认 Copilot 已正确配置后重试，或改用其他编辑预测提供商。".into();

        if let Some(msg) = self.loading_message() {
            container(
                start_label,
                self.render_loading_button(msg, true).into_any_element(),
            )
            .into_any_element()
        } else if self.is_error() {
            container(
                ERROR_LABEL.into(),
                self.render_reinstall_button(true).into_any_element(),
            )
            .into_any_element()
        } else if self.has_no_status() {
            container(
                no_status_label,
                self.render_sign_in_button(true).into_any_element(),
            )
            .into_any_element()
        } else {
            container(
                start_label,
                self.render_sign_in_button(true).into_any_element(),
            )
            .into_any_element()
        }
    }

    fn render_for_chat(&self) -> impl IntoElement {
        let start_label = "要让 Prism Agent 使用 GitHub Copilot，你需要先登录 GitHub。请注意，你的 GitHub 账户必须拥有有效的 Copilot Chat 订阅。";
        let no_status_label = "Copilot Chat 需要有效的 GitHub Copilot 订阅。请确认 Copilot 已正确配置后重试，或改用其他 LLM 提供商。";

        if let Some(msg) = self.loading_message() {
            v_flex()
                .gap_2()
                .child(Label::new(start_label))
                .child(self.render_loading_button(msg, false))
                .into_any_element()
        } else if self.is_error() {
            v_flex()
                .gap_2()
                .child(Label::new(ERROR_LABEL))
                .child(self.render_reinstall_button(false))
                .into_any_element()
        } else if self.has_no_status() {
            v_flex()
                .gap_2()
                .child(Label::new(no_status_label))
                .child(self.render_sign_in_button(false))
                .into_any_element()
        } else {
            v_flex()
                .gap_2()
                .child(Label::new(start_label))
                .child(self.render_sign_in_button(false))
                .into_any_element()
        }
    }
}

impl Render for ConfigurationView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_authenticated = &self.is_authenticated;

        if is_authenticated(cx) {
            return ConfiguredApiCard::new("已授权")
                .button_label("退出登录")
                .on_click(|_, window, cx| {
                    if let Some(auth) = GlobalCopilotAuth::try_global(cx) {
                        initiate_sign_out(auth.0.clone(), window, cx);
                    }
                })
                .into_any_element();
        }

        if self.edit_prediction {
            self.render_for_edit_prediction().into_any_element()
        } else {
            self.render_for_chat().into_any_element()
        }
    }
}
