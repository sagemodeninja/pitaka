mod components;
mod core;
mod utils;

use gpui_kit::base::input::MaskPattern;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::{
    App, AppContext, Context, Div, Entity, IntoElement, ParentElement, Render, Styled, Window,
    WindowOptions, assets, div,
};

use crate::core::document::collection::Transaction;
use crate::{
    components::{column, row},
    core::PitakaCore,
    utils::menu::PitakaMenuUtils,
};

struct Pitaka {
    core: Entity<PitakaCore>,
    trans_desc_ipt: Entity<InputState>,
    trans_amnt_ipt: Entity<InputState>,
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

fn render_trans(this: &Pitaka, cx: &mut App) -> Div {
    this.core.update(cx, |core, _| match &mut core.document {
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

impl Render for Pitaka {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        column()
            .size_full()
            .gap_2()
            .p_4()
            .child(Input::new(&self.trans_desc_ipt).cleanable(true))
            .child(Input::new(&self.trans_amnt_ipt))
            .child(
                Button::new("add-trans-btn")
                    .primary()
                    .label("Add Transaction")
                    .on_click(_cx.listener(|this, _, _, cx| {
                        this.core.update(cx, |core, cx| {
                            if let Some(doc) = &mut core.document {
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
            .child(render_trans(self, _cx))
    }
}

fn main() {
    gpui_kit::application()
        .with_assets(assets::Assets)
        .run(|cx: &mut App| {
            gpui_kit::init(cx);

            let core = cx.new(|_| PitakaCore::init().unwrap());

            cx.activate(true);
            cx.add_menus(&core);

            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| Pitaka {
                    core: core,
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
                })
            })
            .unwrap();
        });
}
