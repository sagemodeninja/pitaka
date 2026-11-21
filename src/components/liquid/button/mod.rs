mod appearance;
mod state;

pub use self::appearance::*;

use std::{rc::Rc, time::Instant};

use gpui::{
    App, ClickEvent, Div, ElementId, Entity, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, Rgba, Stateful, StatefulInteractiveElement, Styled, Window, div, px,
    rgb,
};

use crate::components::{ScalableIcon, center, liquid::LiquidSurface, row};
use appearance::LiquidButtonAppearance::{Accent, Danger, Primary};
use state::LiquidButtonState;

#[derive(IntoElement, Clone)]
pub struct LiquidButton {
    id: ElementId,
    icon: Option<ScalableIcon>,
    label: Option<String>,
    prominent: bool,
    appearance: LiquidButtonAppearance,
    on_click: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
}

impl LiquidButton {
    #[allow(dead_code)]
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            icon: None,
            label: None,
            prominent: false,
            appearance: Primary,
            on_click: None,
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

    #[allow(dead_code)]
    pub fn appearance(mut self, appearance: LiquidButtonAppearance) -> Self {
        self.appearance = appearance;
        self
    }

    #[allow(dead_code)]
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    // Helpers...
    fn foreground(&self) -> Rgba {
        if self.prominent {
            rgb(0x000000) // Fallback for now...
        } else {
            match &self.appearance {
                Primary => rgb(0x000000),
                Accent => rgb(0xFFC022),
                Danger => rgb(0xDC3136),
            }
        }
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

        let mut base = base_button(&self.icon, &self.label, scale)
            .id(id)
            .text_color(self.foreground())
            .on_click(move |e, w, a| {
                if let Some(handler) = &self.on_click {
                    handler(e, w, a);
                }
            });

        base = on_hover(base, &state);
        base = on_press(base, &state);
        base = on_release(base, &state);

        div()
            .relative()
            .child(base_button(&self.icon, &self.label, 1.0).invisible())
            .child(
                center().absolute().top_0().left_0().size_full().child(
                    LiquidSurface::new()
                        .tint(rgb(tint))
                        .radius(99.0 * scale)
                        .child(base),
                ),
            )
    }
}

fn base_button(icon: &Option<ScalableIcon>, label: &Option<String>, scale: f32) -> Div {
    let t_s = px(14.0 * scale);
    let p_l = px(if icon.is_some() { 11.0 } else { 16.0 });
    let p_r = px(if label.is_some() { 16.0 } else { 11.0 });

    let mut button = row()
        .gap(px(8.0 * scale))
        .items_center()
        .pl(p_l * scale)
        .pr(p_r * scale)
        .py(px(11.0 * scale))
        .text_size(t_s);

    if let Some(icon) = icon {
        button = button.child(
            center()
                // .flex_shrink_0()
                .size(t_s)
                .child(icon.clone().scale(scale)),
        );
    }

    if let Some(label) = label {
        button = button.child(center().h(t_s).child(label.clone()));
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
