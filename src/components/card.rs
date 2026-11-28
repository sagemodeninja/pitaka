use gpui::{
    App, BoxShadow, FontWeight, IntoElement, ParentElement, RenderOnce, Styled, Window, div, hsla,
    point, px, rgb,
};

const ASPECT_RATIO: f32 = 85.60 / 53.98;

#[derive(IntoElement)]
pub struct AccountCard {
    name: String,
    selected: bool,
}

impl AccountCard {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            selected: false,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

impl RenderOnce for AccountCard {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let mut card = div()
            .p_4()
            .w_full()
            .aspect_ratio(ASPECT_RATIO)
            .rounded(px(12.0))
            .bg(rgb(0xEEEEEE))
            .text_color(rgb(0x212121))
            .text_size(px(22.))
            .font_family("Helvetica Neue")
            .font_weight(FontWeight::BOLD)
            .child(self.name);

        if self.selected {
            card = card.shadow(vec![BoxShadow {
                color: hsla(211.0 / 360.0, 1.0, 0.5, 1.0),
                offset: point(px(0.0), px(0.0)),
                blur_radius: px(0.),
                spread_radius: px(2.0),
                inset: false,
            }])
        }

        card
    }
}
