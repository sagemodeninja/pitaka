use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, px, rgb,
};
use gpui_kit::component::Icon;

use crate::components::{liquid::LiquidSurface, row};

#[derive(IntoElement)]
pub struct LiquidButton {
    id: ElementId,
    icon: Option<Icon>,
    label: Option<String>,
}

struct LiquidButtonState {
    hovered: bool,
}

impl LiquidButton {
    #[allow(dead_code)]
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            icon: None,
            label: None,
        }
    }

    #[allow(dead_code)]
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    #[allow(dead_code)]
    pub fn label(mut self, text: &str) -> Self {
        self.label = Some(text.into());
        self
    }
}

impl RenderOnce for LiquidButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        assert!(
            self.label.is_some() || self.icon.is_some(),
            "LiquidButton requires a label or an icon"
        );

        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| LiquidButtonState {
            hovered: false,
        });

        let p_x = px(if self.label.is_some() { 16.0 } else { 8.0 });
        let tint = rgb(if state.read(cx).hovered {
            0xE0E0E0
        } else {
            0xF8F8F8
        });

        let mut button = row()
            .id(self.id)
            .text_size(px(14.0))
            .gap(px(12.0))
            .items_center()
            .px(p_x)
            .py(px(8.0))
            .on_hover(move |hovered, _, cx| {
                state.update(cx, |state, cx| {
                    state.hovered = *hovered;
                    cx.notify();
                });
            });

        if let Some(icon) = self.icon {
            button = button.child(icon);
        }

        if let Some(label) = self.label {
            button = button.child(label);
        }

        LiquidSurface::new().child(button).tint(tint)
    }
}
