use std::path::PathBuf;

use gpui_kit::{Action, App, Entity, actions};

use crate::{core::state::PitakaState, utils::dir::documents_dir};

actions!(pitaka, [Quit, NewFile, OpenFile, ClearRecents, SaveFile]);

#[derive(Clone, PartialEq, Action)]
#[action(namespace = pitaka, no_json)]
pub struct OpenRecents {
    pub path: PathBuf,
}

pub trait AppActions {
    fn handle_actions(&mut self, state: &Entity<PitakaState>);
}

impl AppActions for App {
    fn handle_actions(&mut self, state: &Entity<PitakaState>) {
        self.on_action(|_: &Quit, cx: &mut App| {
            cx.quit();
        });

        let action_state = state.clone();
        self.on_action(move |_: &NewFile, cx: &mut App| {
            if let Some(init_dir) = documents_dir() {
                let selection = cx.prompt_for_new_path(
                    &init_dir, // &Path
                    Some("Untitled.ptk"),
                );

                let state = action_state.clone();
                cx.spawn(async move |cx| {
                    match selection.await {
                        Ok(Ok(Some(path))) => {
                            let result = state.update(cx, |state, cx| {
                                state.new_vault("Gary's Vault", path)?;
                                cx.notify();
                                Ok::<(), std::io::Error>(())
                            });

                            if let Err(error) = result {
                                eprintln!("Could not update state: {error}");
                            }
                        }
                        Ok(Ok(None)) => {} // Cancelled
                        Ok(Err(error)) => {
                            eprintln!("Dialog failed: {error}")
                        }
                        Err(error) => {
                            eprintln!("Response channel closed: {error}")
                        }
                    }
                })
                .detach();
            }
        });

        let action_state = state.clone();
        self.on_action(move |_: &OpenFile, cx: &mut App| {
            let selection = cx.prompt_for_paths(gpui_kit::PathPromptOptions {
                files: true,
                directories: false,
                multiple: false,
                prompt: Some("Open Pitaka".into()),
            });

            let state = action_state.clone();
            cx.spawn(async move |cx| {
                match selection.await {
                    Ok(Ok(Some(paths))) => {
                        if let Some(path) = paths.into_iter().next() {
                            let result = state.update(cx, |state, cx| {
                                state.open_doc(path)?;
                                cx.notify();
                                Ok::<(), std::io::Error>(())
                            });

                            if let Err(error) = result {
                                eprintln!("Could not update state: {error}");
                            }
                        }
                    }
                    Ok(Ok(None)) => {} // Cancelled.
                    Ok(Err(error)) => eprintln!("Dialog failed: {error}"),
                    Err(error) => {
                        eprintln!("Response channel closed: {error}")
                    }
                }
            })
            .detach();
        });

        let action_state = state.clone();
        self.on_action(move |action: &OpenRecents, cx: &mut App| {
            let result = action_state.update(cx, |state, cx| {
                state.open_doc(action.path.clone())?;
                cx.notify();
                Ok::<(), std::io::Error>(())
            });

            // match result {
            //     Ok(()) => set_menus(cx, &action_state),
            //     Err(error) => eprintln!("Could not open project: {error}"),
            // }
        });

        let action_state = state.clone();
        self.on_action(move |_: &ClearRecents, cx: &mut App| {
            let result = action_state.update(cx, |state, cx| {
                state.settings.clear_recents()?;
                cx.notify();
                Ok::<(), std::io::Error>(())
            });

            // match result {
            //     Ok(()) => set_menus(cx, &action_state),
            //     Err(error) => {
            //         eprintln!("Could not clear recents: {error}")
            //     }
            // }
        });

        let action_state = state.clone();
        self.on_action(move |_: &SaveFile, cx: &mut App| {
            let result = action_state.update(cx, |state, cx| {
                state.save_doc()?;
                cx.notify();
                Ok::<(), std::io::Error>(())
            });

            if let Err(error) = result {
                eprintln!("Could not update state: {error}");
            }
        });
    }
}
