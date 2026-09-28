use freya::prelude::*;
use oneclient_auth::MinecraftAccount;

use crate::components::{Avatar, Button, Icon, IconType, OverlayPopup, TextInput, use_microsoft_login};
use crate::hooks::{try_default_account, use_add_ely_by_account, AddElyByAccountKeys, use_current_account};
use crate::theme::colors;
use crate::ui::border_all_color;
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
        let ely = use_add_ely_by_account();
        let mut show_ely = use_state(|| false);
        let mut ely_username = use_state(String::new);
        let mut ely_password = use_state(String::new);
        let ely_error = match &*ely.read().state() {
            freya::query::MutationStateData::Settled { res: Err(err), .. } => Some(err.to_string()),
            freya::query::MutationStateData::Loading { res: Some(Err(err)) } => Some(err.to_string()),
            _ => None,
        };

        let account = try_default_account(&account_query);
        let has_account = account.is_some();

        let content = rect()
            .vertical()
            .width(Size::fill())
            .spacing(24.)
            .child(step_heading(
                "Account",
                "Before you continue, we require you to own a copy of Minecraft: Java Edition.",
            ))
            .child(match &account {
                Some(account) => account_preview(account).into_element(),
                None => {
                    let start = msa.clone();
                    let mut show_ely_open = show_ely;
                    let ely_pending = ely.read().state().is_loading();
                    sign_in_card(
                        msa.pending,
                        msa.error.clone(),
                        ely_pending,
                        move |_| start.start(),
                        move |_| show_ely_open.set(true),
                    )
                    .into_element()
                }
            })
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

        let on_confirm_ely = move |_| {
            let username = ely_username.peek().trim().to_string();
            let password = ely_password.peek().clone();
            if username.is_empty() || password.is_empty() {
                return;
            }
            ely.mutate(AddElyByAccountKeys { username, password });
        };

        rect()
            .width(Size::fill())
            .height(Size::fill())
            .child(page)
            .maybe_child(msa.popup())
            .maybe_child(show_ely.read().then(|| {
                ely_dialog(
                    ely_username,
                    ely_password,
                    ely_error,
                    on_confirm_ely,
                    show_ely,
                )
            })
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
                    Avatar::new(account.id.to_string())
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
    ely_pending: bool,
    on_add: impl FnMut(Event<PressEventData>) + 'static,
    on_ely: impl FnMut(Event<PressEventData>) + 'static,
) -> impl IntoElement {
    rect()
        .vertical()
        .spacing(12.)
        .cross_align(Alignment::Start)
        .child(
            rect()
                .horizontal()
                .spacing(8.)
                .child(
                    Button::new()
                        .primary()
                        .large()
                        .enabled(!pending && !ely_pending)
                        .on_press(on_add)
                        .child(Icon::new(IconType::Globe01).size(16.))
                        .text(if pending { "Signing in..." } else { "Add Account" }),
                )
                .child(
                    Button::new()
                        .secondary()
                        .large()
                        .enabled(!pending && !ely_pending)
                        .on_press(on_ely)
                        .child(Icon::new(IconType::Globe01).size(16.))
                        .text(if ely_pending { "Signing in..." } else { "Add Ely.by" }),
                ),
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


fn ely_dialog(
    username: State<String>,
    password: State<String>,
    error: Option<String>,
    on_confirm: impl FnMut(Event<PressEventData>) + 'static,
    mut show_ely: State<bool>,
) -> impl IntoElement {
    OverlayPopup::new()
        .on_close(move |()| show_ely.set(false))
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
                                .text("Add Ely.by account")
                                .font_size(18.)
                                .font_weight(FontWeight::SEMI_BOLD)
                                .color(colors::fg_primary()),
                        )
                        .child(
                            rect()
                                .vertical()
                                .width(Size::fill())
                                .spacing(6.)
                                .child(label().text("E-mail or username").font_size(12.).color(colors::fg_secondary()))
                                .child(TextInput::new(username).placeholder("Ely.by username or e-mail")),
                        )
                        .child(
                            rect()
                                .vertical()
                                .width(Size::fill())
                                .spacing(6.)
                                .child(label().text("Password").font_size(12.).color(colors::fg_secondary()))
                                .child(TextInput::new(password).placeholder("Ely.by password")),
                        )
                        .maybe_child(error.map(|msg| {
                            label().text(msg).font_size(12.).color(colors::danger()).into_element()
                        }))
                        .child(
                            rect()
                                .horizontal()
                                .width(Size::fill())
                                .main_align(Alignment::End)
                                .spacing(8.)
                                .child(
                                    Button::new()
                                        .ghost()
                                        .on_press(move |_| show_ely.set(false))
                                        .text("Cancel"),
                                )
                                .child(
                                    Button::new()
                                        .primary()
                                        .on_press(on_confirm)
                                        .child(Icon::new(IconType::Globe01).size(16.))
                                        .text("Sign in"),
                                ),
                        ),
                ),
        )
        .into_element()
}
