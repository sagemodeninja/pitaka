use gpui_kit::Entity;
use gpui_kit::{App, Menu, MenuItem};
use std::path::PathBuf;

use crate::core::state::PitakaState;
use crate::utils::actions::*;

pub trait PitakaMenuUtils {
    fn add_mac_menus(&mut self, state: &Entity<PitakaState>);
}

impl PitakaMenuUtils for App {
    fn add_mac_menus(&mut self, state: &Entity<PitakaState>) {
        self.set_menus(vec![
            Menu {
                name: "Pitaka".into(),
                disabled: false,
                items: vec![MenuItem::action("Quit Pitaka", Quit)],
            },
            Menu {
                name: "File".into(),
                disabled: false,
                items: vec![
                    MenuItem::action("New", NewFile),
                    MenuItem::separator(),
                    MenuItem::action("Open...", OpenFile),
                    MenuItem::submenu(Menu {
                        name: "Open Recent".into(),
                        disabled: false,
                        items: get_recents(self, state),
                    }),
                    MenuItem::separator(),
                    MenuItem::action("Save", SaveFile),
                ],
            },
        ]);
    }
}

fn get_recents(cx: &App, state: &Entity<PitakaState>) -> Vec<MenuItem> {
    let recents = &state.read(cx).settings.recent_files;
    let mut items: Vec<MenuItem> = recents.iter().cloned().map(|path| recent(path)).collect();

    items.push(MenuItem::separator());
    items.push(MenuItem::action("Clear Recents...", ClearRecents));

    items
}

fn recent(path: PathBuf) -> MenuItem {
    MenuItem::action(format!("{}", path.display()), OpenRecents { path: path })
}
