use gpui::{IntoElement, Pixels, RenderOnce, Styled, px};
use gpui_kit::component::Icon;

#[derive(Clone, IntoElement)]
pub struct ScalableIcon {
    icon: Icon,
    size: Pixels,
    scale: f32,
}

impl ScalableIcon {
    pub fn new(icon: impl Into<Icon>) -> Self {
        Self {
            icon: icon.into(),
            size: px(16.0),
            scale: 1.0,
        }
    }

    #[allow(dead_code)]
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = size.into();
        self
    }

    #[allow(dead_code)]
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
}

impl RenderOnce for ScalableIcon {
    fn render(self, _: &mut gpui::Window, _: &mut gpui::App) -> impl IntoElement {
        self.icon.size(self.size * self.scale)
    }
}
