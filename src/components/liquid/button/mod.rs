mod state;

use std::time::Instant;

use gpui::{
    App, Div, ElementId, Entity, InteractiveElement, IntoElement, MouseButton, ParentElement,
    RenderOnce, Stateful, StatefulInteractiveElement, Styled, Window, div, px, rgb,
};

use crate::components::{liquid::LiquidSurface, row, scalable_icon::ScalableIcon};
use state::LiquidButtonState;

#[derive(IntoElement, Clone)]
pub struct LiquidButton {
    id: ElementId,
    icon: Option<ScalableIcon>,
    label: Option<String>,
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
    pub fn icon(mut self, icon: ScalableIcon) -> Self {
        self.icon = Some(icon);
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

        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| LiquidButtonState::new());
        let state_ref = state.read(cx);

        let tint = if state_ref.hovered && !state_ref.pressed {
            0xE0E0E0
        } else {
            0xF8F8F8
        };
        let (scale, progress) = state_ref.scale_now();

        if let Some(progress) = progress {
            if progress < 1.0 {
                window.request_animation_frame();
            } else {
                state.update(cx, |state, cx| {
                    state.animating_press = false;
                    cx.notify();
                });
            }
        }

        let id = self.id.clone();
        let dummy = base_button(self.clone(), 1.0).invisible();
        let mut button = base_button(self, scale).id(id);

        button = on_hover(button, &state);
        button = on_press(button, &state);
        button = on_release(button, &state);

        div().relative().child(dummy).child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    LiquidSurface::new()
                        .tint(rgb(tint))
                        .radius(99.0 * scale)
                        .child(button),
                ),
        )
    }
}

fn base_button(this: LiquidButton, scale: f32) -> Div {
    let p_x = px(if this.label.is_some() { 16.0 } else { 8.0 });

    let mut button = row()
        .items_center()
        .gap(px(12.0 * scale))
        .text_size(px(14.0 * scale))
        .px(p_x * scale)
        .py(px(8.0 * scale));

    if let Some(icon) = this.icon {
        button = button.child(icon.scale(scale));
    }

    if let Some(label) = this.label {
        button = button.child(label);
    }

    button
}

fn on_hover(button: Stateful<Div>, state: &Entity<LiquidButtonState>) -> Stateful<Div> {
    let state = state.clone();
    button.on_hover(move |hovered, _, cx| {
        state.update(cx, |state, cx| {
            state.hovered = *hovered;
            cx.notify();
        });
    })
}

fn on_press(button: Stateful<Div>, state: &Entity<LiquidButtonState>) -> Stateful<Div> {
    let state = state.clone();

    button.on_mouse_down(MouseButton::Left, move |_, _, cx| {
        state.update(cx, |state, cx| {
            let (current_scale, _) = state.scale_now();

            state.pressed = true;
            state.from_scale = current_scale;
            state.started_at = Instant::now();
            state.animating_press = true;

            cx.notify();
        });
    })
}

fn on_release(button: Stateful<Div>, state: &Entity<LiquidButtonState>) -> Stateful<Div> {
    let state = state.clone();

    let release = move |_: &gpui::MouseUpEvent, _: &mut Window, cx: &mut App| {
        state.update(cx, |state, cx| {
            if !state.pressed {
                return;
            };

            let (current_scale, _) = state.scale_now();

            state.pressed = false;
            state.from_scale = current_scale;
            state.started_at = Instant::now();
            state.animating_press = true;

            cx.notify();
        });
    };

    button
        .on_mouse_up(MouseButton::Left, release.clone())
        .on_mouse_up_out(MouseButton::Left, release)
}
