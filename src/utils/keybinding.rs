use gpui_kit::{App, KeyBinding};

use crate::utils::actions::*;

pub trait AppKeyBindings {
    fn add_kbd_binding(&mut self);
}

impl AppKeyBindings for App {
    fn add_kbd_binding(&mut self) {
        let [quit, new_file, open_file, save_file] = if cfg!(target_os = "macos") {
            ["cmd-q", "cmd-n", "cmd-o", "cmd-s"]
        } else {
            ["ctrl-q", "ctrl-n", "ctrl-o", "ctrl-s"]
        };

        self.bind_keys([
            KeyBinding::new(quit, Quit, None),
            KeyBinding::new(new_file, NewFile, None),
            KeyBinding::new(open_file, OpenFile, None),
            KeyBinding::new(save_file, SaveFile, None),
        ]);
    }
}
