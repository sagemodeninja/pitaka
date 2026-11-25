use chrono::Utc;
use gpui::{
    App, AppContext, Context, Div, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Subscription, Window, div, px, rgba,
};
use gpui_kit::{
    base::input::{InputEvent, InputState, MaskPattern},
    component::{
        IconName,
        button::{Button, ButtonVariants},
        input::Input,
    },
};

use crate::{
    components::{
        AccountCard, CustomIcon, ScalableIcon, column,
        liquid::{LiquidButton, LiquidButtonAppearance},
        row,
    },
    core::{
        state::PitakaState,
        types::Money,
        vault::{
            self,
            collection::{Account, Transaction},
        },
    },
};

pub struct AccountsPage {
    state: Entity<PitakaState>,
    selected_acct_id: Option<u64>,

    acct_form_visible: bool,
    acct_dname_ipt: Entity<InputState>,
    acct_r_dname_ipt: Entity<InputState>,
    trans_desc_ipt: Entity<InputState>,
    trans_amnt_ipt: Entity<InputState>,

    _rename_dname_sub: Subscription,
}

impl AccountsPage {
    pub fn new(state: &Entity<PitakaState>, window: &mut Window, app: &mut App) -> Entity<Self> {
        app.new(|cx| {
            let acct_r_dname_ipt =
                cx.new(|cx| InputState::new(window, cx).placeholder("Enter name"));

            let rename_dname_sub = cx.subscribe_in(
                &acct_r_dname_ipt,
                window,
                |this: &mut AccountsPage, input, event: &InputEvent, _window, cx| {
                    if matches!(event, InputEvent::PressEnter { .. } | InputEvent::Blur) {
                        let id = this.selected_acct_id.unwrap();
                        let name = input.read(cx).value();
                        this.state.update(cx, |state, cx| {
                            if let Some(vault) = &mut state.vault {
                                if let Some(account) = vault.get_account(id) {
                                    println!(">>account: {id}/{}", account.id);
                                }
                            }
                        });
                    }
                },
            );

            Self {
                state: state.clone(),
                selected_acct_id: None,
                acct_form_visible: false,
                acct_dname_ipt: cx.new(|cx| InputState::new(window, cx).placeholder("Enter name")),
                acct_r_dname_ipt,
                trans_desc_ipt: cx
                    .new(|cx| InputState::new(window, cx).placeholder("Enter description")),
                trans_amnt_ipt: cx.new(|cx| {
                    InputState::new(window, cx)
                        .placeholder("Enter amount")
                        .mask_pattern(MaskPattern::Number {
                            separator: Some(','),
                            fraction: Some(2),
                        })
                }),
                _rename_dname_sub: rename_dname_sub,
            }
        })
    }
}

impl Render for AccountsPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let form = if self.acct_form_visible {
            column()
                .gap_2()
                .p_4()
                .border_b_1()
                .border_color(rgba(0x42424240))
                .child(Input::new(&self.acct_dname_ipt).cleanable(true))
                .child(
                    Button::new("add-acct-btn")
                        .primary()
                        .label("Add Account")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.state.update(cx, |state, cx| {
                                if let Some(doc) = &mut state.vault {
                                    let dname = this.acct_dname_ipt.read(cx).value();
                                    doc.accounts().add(Account {
                                        display_name: dname.into(),
                                    });
                                };

                                this.acct_form_visible = false;
                                cx.notify();
                            });
                        })),
                )
        } else {
            div()
        };

        row()
            .size_full()
            .child(
                column()
                    .gap_2()
                    .w(px(320.))
                    .child(
                        row()
                            .items_center()
                            .justify_between()
                            .p_2()
                            .border_b_1()
                            .border_color(rgba(0x42424240))
                            .child("Accounts")
                            .child(
                                row()
                                    .gap_1()
                                    .child(
                                        LiquidButton::new("liquid-add-button")
                                            .icon(ScalableIcon::new(IconName::Plus).size(20.0))
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.acct_form_visible = true;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        LiquidButton::new("liquid-edit-button")
                                            .appearance(LiquidButtonAppearance::Danger)
                                            .icon(ScalableIcon::new(CustomIcon::Trash).size(18.0))
                                            .label("Delete")
                                            .disabled(self.selected_acct_id.is_none())
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                if let Some(id) = this.selected_acct_id {
                                                    this.state.update(cx, |state, cx| {
                                                        if let Some(doc) = &mut state.vault {
                                                            doc.accounts().delete(id);
                                                            cx.notify();
                                                        }
                                                    });
                                                };
                                            })),
                                    ),
                            ),
                    )
                    .child(form)
                    .child(render_accts(self, cx))
                    .border_r_1()
                    .border_color(rgba(0x42424240)),
            )
            .child(
                column()
                    .gap_2()
                    .p_4()
                    .w(px(320.))
                    .child(Input::new(&self.acct_r_dname_ipt).cleanable(true))
                    .child(Input::new(&self.trans_desc_ipt).cleanable(true))
                    .child(Input::new(&self.trans_amnt_ipt))
                    .child(
                        Button::new("add-trans-btn")
                            .primary()
                            .label("Add Transaction")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.state.update(cx, |state, cx| {
                                    if let Some(doc) = &mut state.vault {
                                        if let Some(id) = this.selected_acct_id {
                                            let desc = this.trans_desc_ipt.read(cx).value();
                                            let s_amnt = this.trans_amnt_ipt.read(cx).value();

                                            if let Ok(amount) = s_amnt.parse::<Money>() {
                                                doc.transactions().add(Transaction {
                                                    account_id: id,
                                                    occured_at: Utc::now(),
                                                    description: desc.into(),
                                                    amount,
                                                });
                                                cx.notify();
                                            }
                                        }
                                    }
                                })
                            })),
                    )
                    .child(render_trans(self, cx)),
            )
    }
}

fn render_accts(page: &AccountsPage, cx: &mut Context<AccountsPage>) -> Div {
    let records = page.state.update(cx, |state, _| {
        state.vault.as_mut().map(|doc| doc.get_all_accts())
    });

    match records {
        Some(Ok(records)) => div().size_full().child(
            column()
                .id("acct-list-container")
                .gap_2()
                .p_3()
                .size_full()
                .children(records.into_iter().map(|record| {
                    let selected = record.id == page.selected_acct_id.unwrap_or(0);
                    let display_name = record.item.display_name;

                    div()
                        .id(format!("acct_row_{}", record.id))
                        .child(AccountCard::new(display_name.clone()).selected(selected))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            cx.stop_propagation();
                            this.selected_acct_id = Some(record.id);
                            this.acct_r_dname_ipt.update(cx, |input, cx| {
                                input.set_value(display_name.as_str(), window, cx);
                            });
                            cx.notify();
                        }))
                }))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.selected_acct_id = None;
                    cx.notify();
                })),
        ),
        Some(Err(err)) => div().child(format!("Something went wrong: {err}")),
        None => div().child("Please create/open a file first..."),
    }
}

fn render_trans(view: &AccountsPage, cx: &mut App) -> Div {
    if let Some(acct_id) = view.selected_acct_id {
        let records = view.state.update(cx, |state, _| {
            state.vault.as_mut().map(|doc| doc.get_acct_trans(acct_id))
        });

        match records {
            Some(Ok(records)) => column()
                .child(row().gap_1().child("Description").child("Amount"))
                .children(records.iter().map(|record| {
                    let trans = &record.item;
                    row()
                        .gap_1()
                        .child(trans.description.clone())
                        .child(trans.amount.to_string())
                })),
            Some(Err(err)) => div().child(format!("Something went wrong: {err}")),
            None => div().child("Please create/open a file first..."),
        }
    } else {
        div().child("Please select an account...")
    }
}
