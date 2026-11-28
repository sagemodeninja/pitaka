mod components;
mod core;
mod utils;
mod views;

use std::borrow::Cow;

use gpui_kit::component::{Theme, TitleBar};
use gpui_kit::{App, AppContext, Entity, assets};

use crate::utils::actions::AppActions;
use crate::utils::keybinding::AppKeyBindings;
use crate::views::MainView;
use crate::{core::PitakaState, utils::menu::PitakaMenuUtils};

fn main() {
    gpui_kit::application()
        .with_assets(assets::AllAssets)
        .run(|app: &mut App| {
            gpui_kit::init(app);

            app.text_system()
                .add_fonts(vec![
                    Cow::Borrowed(
                        include_bytes!("../assets/fonts/inter/Inter-Variable.ttf").as_slice(),
                    ),
                    Cow::Borrowed(
                        include_bytes!("../assets/fonts/inter/Inter-Italic-Variable.ttf")
                            .as_slice(),
                    ),
                ])
                .expect("Failed to load bundled Inter fonts");

            Theme::update(app, |theme| {
                theme.font_family = "Inter".into();
            });

            let state = app.new(|_| PitakaState::init().unwrap());

            app.activate(true);
            app.handle_actions(&state);
            app.add_kbd_binding();

            if cfg!(target_os = "macos") {
                app.add_mac_menus(&state);
            }

            go_home(app, &state);
        });
}

fn go_home(app: &mut App, state: &Entity<PitakaState>) {
    gpui_kit::open_window(TitleBar::window_options(), app, |window, cx| {
        Theme::sync_system_appearance(Some(window), cx);

        window
            .observe_window_appearance(|window, cx| {
                Theme::sync_system_appearance(Some(window), cx);
            })
            .detach();

        MainView::new(state, window, cx)
    })
    .unwrap();
}
