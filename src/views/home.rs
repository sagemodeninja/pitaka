use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::empty::{
    Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle,
};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::{
    App, Context, Div, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
};

use crate::components::LaunchAction;
use crate::components::{column, row};
use crate::core::document::collection::Transaction;
use crate::core::state::PitakaState;
use crate::utils::actions::{NewFile, OpenFile};

pub struct Pitaka {
    pub state: Entity<PitakaState>,
    pub trans_desc_ipt: Entity<InputState>,
    pub trans_amnt_ipt: Entity<InputState>,
}

impl Render for Pitaka {
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
fn render_main(view: &mut Pitaka, cx: &mut Context<Pitaka>) -> Div {
    column()
        .size_full()
        .gap_2()
        .p_4()
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
                            let amnt = this.trans_amnt_ipt.read(cx).value();

                            if let Some(amnt) = amnt_str_to_centavos(amnt.into()) {
                                println!("Desc: {}, Amnt: {}", desc, amnt);
                                doc.transactions.add(Transaction {
                                    description: desc.into(),
                                    amount_centavos: amnt,
                                });
                                cx.notify();
                            }
                        }
                    })
                })),
        )
        .child(render_trans(view, cx))
}

fn render_trans(view: &Pitaka, cx: &mut App) -> Div {
    view.state.update(cx, |state, _| match &mut state.document {
        Some(doc) => match doc.get_all_trans() {
            Ok(records) => column()
                .child(row().gap_1().child("Description").child("Amount"))
                .children(records.iter().map(|trans| {
                    row()
                        .gap_1()
                        .child(trans.description.clone())
                        .child(format_centavos(&trans.amount_centavos))
                })),
            Err(err) => div().child(format!("Something went wrong: {err}")),
        },
        None => div().child("Please create/open a file first..."),
    })
}

fn format_centavos(amount: &i64) -> String {
    let sign = if *amount < 0 { "-" } else { "" };
    let magnitude = amount.unsigned_abs();

    let pesos = magnitude / 100;
    let centavos = magnitude % 100;

    format!("{sign}₱{pesos}.{centavos:02}")
}

fn amnt_str_to_centavos(amount: String) -> Option<i64> {
    let amount = amount.trim().replace(',', "");

    if !amount.is_empty() {
        let parts: Vec<&str> = amount.split('.').collect();

        if let Ok(whole) = parts[0].parse::<i64>() {
            let centavos = if parts.len() > 1 {
                format!("{:0<2}", parts[1]).parse::<i64>()
            } else {
                Ok(0)
            };

            if let Ok(centavos) = centavos {
                return Some(whole * 100 + centavos);
            }
        }
    }

    None
}
