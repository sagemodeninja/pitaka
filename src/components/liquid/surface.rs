use std::f32::consts::FRAC_PI_2;

use gpui::{
    AnyElement, Bounds, ContentMask, IntoElement, Path, PathBuilder, Pixels, Point, RenderOnce,
    Rgba, Window, canvas, linear_color_stop, linear_gradient, point, px, rgb, rgba, size,
};
use gpui_kit::{
    ParentElement, Styled, div,
    private::anyhow::{Error, ensure},
};

#[derive(IntoElement)]
pub struct LiquidSurface {
    tint: Rgba,
    radius: f32,
    highlight_distance: f32,
    children: Vec<AnyElement>,
}

impl LiquidSurface {
    pub fn new() -> Self {
        Self {
            tint: rgb(0xF8F8F8),
            radius: 24.0,
            highlight_distance: 0.2,
            children: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn tint(mut self, tint: Rgba) -> Self {
        self.tint = tint;
        self
    }

    #[allow(dead_code)]
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    #[allow(dead_code)]
    pub fn highlight_distance(mut self, distance: f32) -> Self {
        self.highlight_distance = distance;
        self
    }
}

impl ParentElement for LiquidSurface {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for LiquidSurface {
    fn render(self, _: &mut Window, _: &mut gpui::App) -> impl IntoElement {
        div()
            .relative()
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, (), window, _| {
                        let radius = px(self.radius);
                        window.paint_backdrop(gpui::fill(bounds, self.tint));
                        // draw_tint(bounds, window, self.tint, radius);
                        draw_border(bounds, window, radius);
                        draw_highlight(bounds, window, radius);
                    },
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full(),
            )
            .children(self.children)
    }
}

fn draw_tint(bounds: Bounds<Pixels>, window: &mut Window, mut tint: Rgba, radius: Pixels) {
    tint.a = 0.4;

    if let Ok(fill) = fill_squircle(radius, bounds) {
        window.paint_path(fill, tint);
    }
}

fn draw_border(bounds: Bounds<Pixels>, window: &mut Window, radius: Pixels) {
    if let Ok(outline) = stroke_squircle(radius, px(1.0), bounds) {
        let peak = rgba(0x42424240);
        let fade = rgba(0x42424214);
        let half_height = bounds.size.height / 2.0;

        // Top...
        let border_top = linear_gradient(
            180.0,
            linear_color_stop(fade, 0.0),
            linear_color_stop(peak, 0.20),
        );
        let top_mask = Bounds {
            origin: bounds.origin,
            size: size(bounds.size.width, half_height),
        };

        #[rustfmt::skip]
        window.with_content_mask(
            Some(ContentMask { bounds: top_mask }),
            |window| window.paint_path(outline.clone(), border_top),
        );

        // Bottom...
        let border_bottom = linear_gradient(
            360.0,
            linear_color_stop(fade, 0.0),
            linear_color_stop(peak, 0.20),
        );
        let bottom_mask = Bounds {
            origin: point(bounds.origin.x, bounds.origin.y + half_height),
            size: top_mask.size,
        };

        #[rustfmt::skip]
        window.with_content_mask(
            Some(ContentMask { bounds: bottom_mask }),
            |window| window.paint_path(outline, border_bottom),
        );
    }
}

fn draw_highlight(bounds: Bounds<Pixels>, window: &mut Window, radius: Pixels) {
    if let Ok(outline) = stroke_squircle(radius, px(1.0), bounds.inset(0.5.into())) {
        let peak = rgba(0xFFFFFFBF);
        let fade = rgba(0xFFFFFF00);

        let border_top = linear_gradient(
            180.0,
            linear_color_stop(peak, 0.0),
            linear_color_stop(fade, 0.30),
        );
        let border_bottom = linear_gradient(
            360.0,
            linear_color_stop(peak, 0.0),
            linear_color_stop(fade, 0.30),
        );

        window.paint_path(outline.clone(), border_top);
        window.paint_path(outline, border_bottom);
    }
}

fn stroke_squircle(
    radius: Pixels,
    stroke: Pixels,
    bounds: Bounds<Pixels>,
) -> Result<Path<Pixels>, Error> {
    squircle_path(
        PathBuilder::stroke(stroke),
        bounds.inset(stroke / 2.0),
        radius,
    )
}

fn fill_squircle(radius: Pixels, bounds: Bounds<Pixels>) -> Result<Path<Pixels>, Error> {
    squircle_path(PathBuilder::fill(), bounds, radius)
}

fn squircle_path(
    mut path: PathBuilder,
    bounds: Bounds<Pixels>,
    radius: Pixels,
) -> Result<Path<Pixels>, Error> {
    ensure!(
        bounds.size.width > px(0.0) && bounds.size.height > px(0.0),
        "Superellipse bounds must have positive width and height"
    );

    let max_radius = bounds.size.width.min(bounds.size.height) / 2.0;
    let radius = radius.max(px(0.0)).min(max_radius);
    let exponent = 4.0 - 2.0 * (radius / max_radius);

    let left = bounds.left();
    let right = bounds.right();
    let top = bounds.top();
    let bottom = bounds.bottom();

    let corners = [
        (point(right - radius, top + radius), 0.0, -1.0),
        (point(right - radius, bottom - radius), 1.0, 0.0),
        (point(left + radius, bottom - radius), 0.0, 1.0),
        (point(left + radius, top + radius), -1.0, 0.0),
    ];

    for (index, (center, start_x, start_y)) in corners.into_iter().enumerate() {
        let start = point(center.x + radius * start_x, center.y + radius * start_y);

        if index == 0 {
            path.move_to(start);
        } else {
            path.line_to(start);
        }

        if radius == px(0.0) {
            continue;
        }

        for sample in squircle_corner(center, radius, start_x, start_y, exponent) {
            path.line_to(sample);
        }
    }

    path.close();
    path.build()
}

/// Samples one clockwise quarter-superellipse, excluding its start point.
fn squircle_corner(
    center: Point<Pixels>,
    radius: Pixels,
    start_x: f32,
    start_y: f32,
    exponent: f32,
) -> impl Iterator<Item = Point<Pixels>> {
    const SEGMENTS: usize = 90;

    let interior = (1..SEGMENTS).map(move |segment| {
        let angle = FRAC_PI_2 * segment as f32 / SEGMENTS as f32;
        let (sin, cos) = angle.sin_cos();

        // Normalize on the superellipse, then rotate to this corner.
        let length = (cos.powf(exponent) + sin.powf(exponent)).powf(1.0 / exponent);

        point(
            center.x + radius * ((start_x * cos - start_y * sin) / length),
            center.y + radius * ((start_y * cos + start_x * sin) / length),
        )
    });

    let end = point(center.x - radius * start_y, center.y + radius * start_x);
    interior.chain(std::iter::once(end))
}
