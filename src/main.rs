mod components;
mod core;

use gpui::{
    App, AppContext, Application, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, WindowOptions, div, rgb,
};

use crate::{
    components::{ButtonElement, button, column, row},
    core::{Collection, Transaction, write_transaction},
};

struct Pitaka {
    transaction_count: i32,
    transactions: Option<Collection<Transaction>>,
}

fn format_centavos(amount: &i64) -> String {
    let sign = if *amount < 0 { "-" } else { "" };
    let magnitude = amount.unsigned_abs();

    let pesos = magnitude / 100;
    let centavos = magnitude % 100;

    format!("{sign}₱{pesos}.{centavos:02}")
}

fn debug_trans(trans: &Option<Transaction>) -> String {
    match trans {
        Some(trans) => format!(
            "Description: {}, Amount: {}",
            trans.description,
            format_centavos(&trans.amount_centavos)
        ),
        _ => String::from("Please read file..."),
    }
}

impl Render for Pitaka {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        column()
            .size_full()
            .bg(rgb(0xffffff))
            .gap_1()
            .p_4()
            .child(format!("Transactions: {}", self.transaction_count))
            .child(
                row()
                    .gap_1()
                    .child(
                        button("-1")
                            .id("remove-transaction")
                            .bg(rgb(0x0071E3))
                            .text_color(rgb(0xffffff))
                            .on_click(_cx.listener(|this, _event, _window, cx| {
                                this.transaction_count -= 1;
                                cx.notify();
                            })),
                    )
                    .child(
                        button("0")
                            .id("reset-transaction")
                            .hover_bg(rgb(0x00ff00))
                            .on_click(_cx.listener(|this, _event, _window, cx| {
                                this.transaction_count = 0;
                                cx.notify();
                            })),
                    )
                    .child(button("+1").id("add-transaction").on_click(_cx.listener(
                        |this, _event, _window, cx| {
                            this.transaction_count += 1;
                            cx.notify();
                        },
                    ))),
            )
            .child(
                row()
                    .gap_1()
                    .child(
                        button("Test write...")
                            .id("test-write")
                            .on_click(|_, _, _| {
                                let mut trans = Collection::<Transaction>::new(0);

                                trans.add(Transaction {
                                    description: String::from("Transaction #1"),
                                    amount_centavos: 100_54,
                                });
                                trans.add(Transaction {
                                    description: String::from("Transaction #2"),
                                    amount_centavos: -694_32,
                                });

                                let result = write_transaction(&mut trans);
                                match result {
                                    Ok(()) => println!("Successful write!"),
                                    Err(err) => println!("Something went wrong: {err}."),
                                }
                            }),
                    )
                    .child(
                        button("Test read...")
                            .id("test-read")
                            .on_click(_cx.listener(|this, _, _, cx| {
                                let result = core::read_transaction(0);
                                match result {
                                    Ok(trans) => {
                                        this.transactions = Some(trans);
                                        cx.notify();
                                    }
                                    Err(err) => println!("Something went wrong: {err}."),
                                }
                            })),
                    ),
            )
            .child(match &mut self.transactions {
                Some(trans) => match trans.get_all() {
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
                None => div().child("Please red a file first..."),
            })
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|_| Pitaka {
                transaction_count: 0,
                transactions: None,
            })
        })
        .unwrap();
    });
}
