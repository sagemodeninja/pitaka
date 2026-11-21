mod launch_action;
pub mod liquid;
pub mod scalable_icon;

pub use launch_action::LaunchAction;

use gpui_kit::{Div, Styled, div};

pub fn column() -> Div {
    div().flex().flex_col()
}

pub fn row() -> Div {
    div().flex().flex_row()
}
