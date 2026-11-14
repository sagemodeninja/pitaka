use gpui::{Action, Entity};
use gpui::{App, KeyBinding, Menu, MenuItem, actions};
use std::path::PathBuf;

use crate::core::PitakaCore;
use crate::utils::dir::documents_dir;

#[derive(Clone, PartialEq, Action)]
#[action(namespace = pitaka, no_json)]
struct OpenRecents {
    path: PathBuf,
}

actions!(pitaka, [Quit, NewFile, OpenFile, ClearRecents, SaveFile]);

pub trait PitakaMenuUtils {
    fn add_menus(&mut self, core: &Entity<PitakaCore>);
}

impl PitakaMenuUtils for App {
    fn add_menus(&mut self, core: &Entity<PitakaCore>) {
        self.bind_keys([
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("cmd-n", NewFile, None),
            KeyBinding::new("cmd-o", OpenFile, None),
            KeyBinding::new("cmd-s", SaveFile, None),
        ]);

        add_handlers(self, core);
        set_menus(self, core);
    }
}

fn add_handlers(cx: &mut App, core: &Entity<PitakaCore>) {
    cx.on_action(|_: &Quit, cx: &mut App| {
        cx.quit();
    });

    let action_core = core.clone();
    cx.on_action(move |_: &NewFile, cx: &mut App| {
        if let Some(init_dir) = documents_dir() {
            let selection = cx.prompt_for_new_path(
                &init_dir, // &Path
                Some("Untitled.ptk"),
            );

            let core = action_core.clone();
            cx.spawn(async move |cx| {
                match selection.await {
                    Ok(Ok(Some(path))) => {
                        let result = core.update(cx, |core, cx| {
                            core.new_file(path)?;
                            cx.notify();
                            Ok::<(), std::io::Error>(())
                        });

                        if let Err(error) = result {
                            eprintln!("Could not update core: {error}");
                        }
                    }
                    Ok(Ok(None)) => {} // Cancelled
                    Ok(Err(error)) => eprintln!("Dialog failed: {error}"),
                    Err(error) => eprintln!("Response channel closed: {error}"),
                }
            })
            .detach();
        }
    });

    let action_core = core.clone();
    cx.on_action(move |_: &OpenFile, cx: &mut App| {
        let selection = cx.prompt_for_paths(gpui::PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Open Pitaka".into()),
        });

        let core = action_core.clone();
        cx.spawn(async move |cx| {
            match selection.await {
                Ok(Ok(Some(paths))) => {
                    if let Some(path) = paths.into_iter().next() {
                        let result = core.update(cx, |core, cx| {
                            core.open_file(path);
                            cx.notify();
                        });

                        if let Err(error) = result {
                            eprintln!("Could not update core: {error}");
                        }
                    }
                }
                Ok(Ok(None)) => {} // Cancelled.
                Ok(Err(error)) => eprintln!("Dialog failed: {error}"),
                Err(error) => eprintln!("Response channel closed: {error}"),
            }
        })
        .detach();
    });

    let action_core = core.clone();
    cx.on_action(move |action: &OpenRecents, cx: &mut App| {
        let result = action_core.update(cx, |core, cx| {
            core.open_file(action.path.clone())?;
            cx.notify();
            Ok::<(), std::io::Error>(())
        });

        match result {
            Ok(()) => set_menus(cx, &action_core),
            Err(error) => eprintln!("Could not open project: {error}"),
        }
    });

    let action_core = core.clone();
    cx.on_action(move |_: &ClearRecents, cx: &mut App| {
        action_core.update(cx, |core, cx| {
            core.settings.recent_files.clear();
            core.settings.last_opened_file = None;
            cx.notify();
        });
        set_menus(cx, &action_core)
    });

    let action_core = core.clone();
    cx.on_action(move |_: &SaveFile, cx: &mut App| {
        let result = action_core.update(cx, |core, cx| {
            core.save_file()?;
            cx.notify();
            Ok::<(), std::io::Error>(())
        });

        if let Err(error) = result {
            eprintln!("Could not update core: {error}");
        }
    });
}

fn set_menus(cx: &mut App, core: &Entity<PitakaCore>) {
    cx.set_menus(vec![
        Menu {
            name: "Pitaka".into(),
            items: vec![MenuItem::action("Quit Pitaka", Quit)],
        },
        Menu {
            name: "File".into(),
            items: vec![
                MenuItem::action("New", NewFile),
                MenuItem::action("Open...", OpenFile),
                MenuItem::submenu(Menu {
                    name: "Open Recent".into(),
                    items: get_recents(cx, core),
                }),
                MenuItem::separator(),
                MenuItem::action("Save", SaveFile),
            ],
        },
    ]);
}

fn get_recents(cx: &App, core: &Entity<PitakaCore>) -> Vec<MenuItem> {
    let recents = &core.read(cx).settings.recent_files;
    let mut items: Vec<MenuItem> = recents.iter().cloned().map(|path| recent(path)).collect();

    items.push(MenuItem::action("Clear Recents...", ClearRecents));
    items
}

fn recent(path: PathBuf) -> MenuItem {
    MenuItem::action(format!("{}", path.display()), OpenRecents { path: path })
}
