use std::time::Instant;

pub struct LiquidButtonState {
    pub hovered: bool,
    pub pressed: bool,

    pub from_scale: f32,
    pub started_at: Instant,
    pub animating_press: bool,
}

impl LiquidButtonState {
    pub fn new() -> Self {
        Self {
            hovered: false,
            pressed: false,
            from_scale: 1.0,
            started_at: Instant::now(),
            animating_press: false,
        }
    }

    pub fn scale_now(&self) -> (f32, Option<f32>) {
        if self.animating_press || self.pressed {
            let tick = self.started_at.elapsed().as_millis();
            let progress = (tick as f32 / 100.0).clamp(0.0, 1.0);
            let eased = gpui::ease_in_out(progress);

            let from_scale = self.from_scale;
            let to_scale = if self.pressed { 1.1 } else { 1.0 };

            (from_scale + (to_scale - from_scale) * eased, Some(eased))
        } else {
            (1.0, None)
        }
    }
}
