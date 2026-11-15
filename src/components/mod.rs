use gpui_kit::{Div, Fill, InteractiveElement, IntoElement, ParentElement, Styled, div, rgb};

pub trait ButtonElement: InteractiveElement {
    fn hover_bg(self, color: impl Into<Fill>) -> Self;
}

impl<E: InteractiveElement> ButtonElement for E {
    fn hover_bg(self, color: impl Into<Fill>) -> Self {
        self.hover(|style| style.bg(color))
    }
}

pub fn button(text: impl IntoElement) -> Div {
    let mut button = div()
        .bg(rgb(0xdddddd))
        .cursor_pointer()
        .px(gpui_kit::px(12.0))
        .py(gpui_kit::px(8.0))
        .rounded(gpui_kit::px(8.0))
        .child(text);

    button.style().align_self = Some(gpui_kit::AlignSelf::FlexStart);
    button
}

pub fn column() -> Div {
    div().flex().flex_col()
}

pub fn row() -> Div {
    div().flex().flex_row()
}
