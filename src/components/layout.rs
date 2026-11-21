use gpui_kit::{Div, Styled, div};

pub fn column() -> Div {
    div().flex().flex_col()
}

pub fn row() -> Div {
    div().flex().flex_row()
}

pub fn center() -> Div {
    row().items_center().justify_center()
}
