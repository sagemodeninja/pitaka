use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::empty::{
    Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle,
};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::{
    App, Context, Div, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};

use crate::components::LaunchAction;
use crate::components::{column, row};
use crate::core::document::collection::{Account, Transaction};
use crate::core::state::PitakaState;
use crate::core::types::Money;
use crate::utils::actions::{NewFile, OpenFile};

pub struct PitakaView {
    pub state: Entity<PitakaState>,
    pub selected_acct_id: u16,
    pub acct_dname_ipt: Entity<InputState>,
    pub trans_desc_ipt: Entity<InputState>,
    pub trans_amnt_ipt: Entity<InputState>,
}

impl Render for PitakaView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let content = if self.state.read(_cx).document.is_some() {
            render_main(self, _cx)
        } else {
            render_welcome(_cx)
        };

        div().size_full().child(content)
    }
}

// View when no docs is open.
fn render_welcome(_: &App) -> Div {
    div().size_full().child(
        Empty::new()
            .size_full()
            .header(
                EmptyHeader::new()
                    .media(
                        EmptyMedia::new()
                            .with_variant(EmptyMediaVariant::Icon)
                            .child(Icon::new(IconName::Folder)),
                    )
                    .title(EmptyTitle::new().child("Welcome to Pitaka"))
                    .description(EmptyDescription::new().child(
                        "Open a document or create a new one to start tracking your money.",
                    )),
            )
            .child(
                column().py_5().items_center().justify_center().child(
                    column()
                        .child(
                            LaunchAction::new("launch-action-open")
                                .icon(IconName::FolderOpen)
                                .label("Open Document…")
                                .action(OpenFile),
                        )
                        .child(
                            LaunchAction::new("launch-action-new")
                                .icon(IconName::FilePlusCorner)
                                .label("New Document…")
                                .action(NewFile),
                        ),
                ),
            ),
    )
}

// View when a doc is open.
fn render_main(view: &mut PitakaView, cx: &mut Context<PitakaView>) -> Div {
    row()
        .size_full()
        .child(
            column()
                .gap_2()
                .p_4()
                .w(px(320.))
                .child(Input::new(&view.acct_dname_ipt).cleanable(true))
                .child(
                    Button::new("add-acct-btn")
                        .primary()
                        .label("Add Account")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.state.update(cx, |state, cx| {
                                if let Some(doc) = &mut state.document {
                                    let dname = this.acct_dname_ipt.read(cx).value();
                                    doc.accounts.add(|id| Account {
                                        id: id as u16,
                                        display_name: dname.into(),
                                    });
                                    cx.notify();
                                }
                            })
                        })),
                )
                .child(render_accts(view, cx)),
        )
        .child(
            column()
                .gap_2()
                .p_4()
                .w(px(320.))
                .child(Input::new(&view.trans_desc_ipt).cleanable(true))
                .child(Input::new(&view.trans_amnt_ipt))
                .child(
                    Button::new("add-trans-btn")
                        .primary()
                        .label("Add Transaction")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.state.update(cx, |state, cx| {
                                if let Some(doc) = &mut state.document {
                                    let desc = this.trans_desc_ipt.read(cx).value();
                                    let s_amnt = this.trans_amnt_ipt.read(cx).value();

                                    if let Ok(amount) = s_amnt.parse::<Money>() {
                                        doc.transactions.add(|id| Transaction {
                                            id,
                                            account_id: this.selected_acct_id,
                                            description: desc.into(),
                                            amount,
                                        });
                                        cx.notify();
                                    }
                                }
                            })
                        })),
                )
                .child(render_trans(view, cx)),
        )
}

fn render_accts(view: &PitakaView, cx: &mut Context<PitakaView>) -> Div {
    let records = view.state.update(cx, |state, _| {
        state.document.as_mut().map(|doc| doc.get_all_accts())
    });

    match records {
        Some(Ok(records)) => {
            column()
                .child(row().gap_1().child("Name"))
                .children(records.into_iter().map(|acct| {
                    let highlight = if acct.id == view.selected_acct_id {
                        rgb(0xff0000)
                    } else {
                        rgb(0xffffff)
                    };

                    row()
                        .id(format!("acct_row_{}", acct.id))
                        .bg(highlight)
                        .gap_1()
                        .child(acct.display_name)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.selected_acct_id = acct.id;
                            cx.notify();
                        }))
                }))
        }
        Some(Err(err)) => div().child(format!("Something went wrong: {err}")),
        None => div().child("Please create/open a file first..."),
    }
}

fn render_trans(view: &PitakaView, cx: &mut App) -> Div {
    let records = view.state.update(cx, |state, _| {
        state
            .document
            .as_mut()
            .map(|doc| doc.get_acct_trans(view.selected_acct_id))
    });

    match records {
        Some(Ok(records)) => column()
            .child(row().gap_1().child("Description").child("Amount"))
            .children(records.iter().map(|trans| {
                row()
                    .gap_1()
                    .child(trans.description.clone())
                    .child(trans.amount.to_string())
            })),
        Some(Err(err)) => div().child(format!("Something went wrong: {err}")),
        None => div().child("Please create/open a file first..."),
    }
}
