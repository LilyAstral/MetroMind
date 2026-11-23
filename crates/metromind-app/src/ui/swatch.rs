//! The palette icon: a square of 45° stripes, one per colour, in order.

use eframe::egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, vec2};

/// Width of one stripe, measured along the horizontal.
const STRIPE: f32 = 3.0;
/// How far each stripe reaches past the square; the clip rect trims it.
const REACH: f32 = 100.0;

/// Fill `rect` with 45° diagonal stripes that repeat `colors` in order.
pub fn striped_swatch(painter: &Painter, rect: Rect, colors: &[Color32]) {
    if colors.is_empty() {
        return;
    }
    let clipped = painter.with_clip_rect(rect);
    let bands = ((rect.width() + rect.height()) / STRIPE).ceil() as usize;
    for band in 0..bands {
        let start = band as f32 * STRIPE;
        let color = colors[band % colors.len()];
        clipped.add(Shape::convex_polygon(
            stripe(rect.left_top(), start, start + STRIPE),
            color,
            Stroke::NONE,
        ));
    }
}

/// The band between the diagonals `x + y = from` and `x + y = to`, reaching past
/// the bottom of the square.
fn stripe(origin: Pos2, from: f32, to: f32) -> Vec<Pos2> {
    vec![
        origin + vec2(from, 0.0),
        origin + vec2(to, 0.0),
        origin + vec2(to - REACH, REACH),
        origin + vec2(from - REACH, REACH),
    ]
}
