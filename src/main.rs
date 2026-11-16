mod components;
mod core;
mod utils;
mod views;

use std::borrow::Cow;

use gpui_kit::base::input::MaskPattern;
use gpui_kit::component::Theme;
use gpui_kit::component::input::InputState;
use gpui_kit::{App, AppContext, Entity, WindowOptions, assets};

use crate::utils::actions::AppActions;
use crate::utils::keybinding::AppKeyBindings;
use crate::views::home::PitakaView;
use crate::{core::state::PitakaState, utils::menu::PitakaMenuUtils};

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
    gpui_kit::open_window(WindowOptions::default(), app, |window, cx| {
        cx.new(|cx| PitakaView {
            state: state.clone(),
            selected_acct_id: 0,
            acct_dname_ipt: cx.new(|cx| InputState::new(window, cx).placeholder("Enter name")),
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
}
