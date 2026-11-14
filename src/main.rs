mod components;
mod core;
mod utils;

use gpui::{
    App, AppContext, Application, Context, Entity, InteractiveElement, IntoElement, ParentElement,
    Render, StatefulInteractiveElement, Styled, Window, WindowOptions, div, rgb,
};

use crate::core::PitakaCore;
use crate::utils::dir::documents_dir;
use crate::utils::menu::{NewFile, OpenFile};
use crate::{
    components::{ButtonElement, button, column, row},
    core::file::collection::{Collection, Transaction},
    utils::menu::{PitakaMenuUtils, Quit},
};

struct Pitaka {
    core: Entity<PitakaCore>,
}

fn format_centavos(amount: &i64) -> String {
    let sign = if *amount < 0 { "-" } else { "" };
    let magnitude = amount.unsigned_abs();

    let pesos = magnitude / 100;
    let centavos = magnitude % 100;

    format!("{sign}₱{pesos}.{centavos:02}")
}

impl Render for Pitaka {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        column()
            .size_full()
            .bg(rgb(0xffffff))
            .gap_1()
            .p_4()
            // .child(format!("Transactions: {}", self.transaction_count))
            // .child(
            //     row()
            //         .gap_1()
            //         .child(
            //             button("-1")
            //                 .id("remove-transaction")
            //                 .bg(rgb(0x0071E3))
            //                 .text_color(rgb(0xffffff))
            //                 .on_click(_cx.listener(|this, _event, _window, cx| {
            //                     this.transaction_count -= 1;
            //                     cx.notify();
            //                 })),
            //         )
            //         .child(
            //             button("0")
            //                 .id("reset-transaction")
            //                 .hover_bg(rgb(0x00ff00))
            //                 .on_click(_cx.listener(|this, _event, _window, cx| {
            //                     this.transaction_count = 0;
            //                     cx.notify();
            //                 })),
            //         )
            //         .child(button("+1").id("add-transaction").on_click(_cx.listener(
            //             |this, _event, _window, cx| {
            //                 this.transaction_count += 1;
            //                 cx.notify();
            //             },
            //         ))),
            // )
            .child(
                row()
                    .gap_1()
                    .child(
                        button("Test write...")
                            .id("test-write")
                            .on_click(|_, _, _| {
                                // let mut trans = Collection::<Transaction>::new(0);

                                // trans.add(Transaction {
                                //     description: String::from("Transaction #1"),
                                //     amount_centavos: 100_54,
                                // });
                                // trans.add(Transaction {
                                //     description: String::from("Transaction #2"),
                                //     amount_centavos: -694_32,
                                // });

                                // let result = write_transaction(&mut trans);
                                // match result {
                                //     Ok(()) => println!("Successful write!"),
                                //     Err(err) => println!("Something went wrong: {err}."),
                                // }
                            }),
                    ),
            )
            .child(match &self.core.read(_cx).active_file {
                Some(file) => match file.transactions.get_all() {
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
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let core = cx.new(|_| PitakaCore::init().unwrap());

        cx.activate(true);
        cx.add_menus(&core);

        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|_| Pitaka { core: core })
        })
        .unwrap();
    });
}
