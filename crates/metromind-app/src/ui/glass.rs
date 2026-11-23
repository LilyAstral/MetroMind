//! The layered "glass" look for popups. egui can't blur what is behind a
//! panel, so this fakes it: a soft shadow, a translucent body, a sheen over
//! the top third, and a light edge. Every number that shapes the look is here.

use crate::render::colors::with_alpha;
use eframe::egui::Shadow;
use eframe::egui::{Color32, Mesh, Pos2, Rect, Shape, Stroke, epaint::Vertex, vec2};

pub const RADIUS: u8 = 14;
const SHADOW_OFFSET: [i8; 2] = [0, 6];
const SHADOW_BLUR: u8 = 24;
const SHADOW_ALPHA: f32 = 0.25;
/// Fraction of the background that shows through the body.
const BODY_ALPHA: f32 = 0.7;
const SHEEN_ALPHA: f32 = 0.08;
/// The sheen covers this much of the panel's height, from the top.
const SHEEN_HEIGHT: f32 = 1.0 / 3.0;
/// Keeps the sheen inside the rounded corners.
const SHEEN_INSET: f32 = 6.0;
const EDGE_ALPHA: f32 = 0.14;
const EDGE_TOP_ALPHA: f32 = 0.26;

/// The panel body's colour over `background`.
pub fn body(background: Color32) -> Color32 {
    with_alpha(background, BODY_ALPHA)
}

/// `top` composited over an opaque `bottom`. Only the contrast test needs it.
#[cfg(test)]
pub fn blend(top: Color32, bottom: Color32) -> Color32 {
    let a = f32::from(top.a()) / 255.0;
    let mix = |t: u8, b: u8| (f32::from(t) * a + f32::from(b) * (1.0 - a)).round() as u8;
    Color32::from_rgb(
        mix(top.r(), bottom.r()),
        mix(top.g(), bottom.g()),
        mix(top.b(), bottom.b()),
    )
}

/// The sheen's brightest colour, for measuring text contrast against it.
pub fn sheen_color() -> Color32 {
    with_alpha(Color32::WHITE, SHEEN_ALPHA)
}

/// Every shape of the glass panel covering `rect`, faded to `opacity`
/// (0..=1) so the whole panel can fade in together.
pub fn shapes(rect: Rect, background: Color32, opacity: f32) -> Vec<Shape> {
    let fade = |c: Color32| c.gamma_multiply(opacity);
    let shadow = Shadow {
        offset: SHADOW_OFFSET,
        blur: SHADOW_BLUR,
        spread: 0,
        color: fade(with_alpha(Color32::BLACK, SHADOW_ALPHA)),
    };
    let edge = Stroke::new(1.0, fade(with_alpha(Color32::WHITE, EDGE_ALPHA)));
    let top_edge = Stroke::new(1.0, fade(with_alpha(Color32::WHITE, EDGE_TOP_ALPHA)));
    let r = f32::from(RADIUS);
    vec![
        Shape::from(shadow.as_shape(rect, RADIUS)),
        Shape::rect_filled(rect, RADIUS, fade(body(background))),
        Shape::mesh(sheen(rect, opacity)),
        Shape::rect_stroke(rect, RADIUS, edge, eframe::egui::StrokeKind::Inside),
        Shape::line_segment(
            [
                rect.left_top() + vec2(r, 0.5),
                rect.right_top() + vec2(-r, 0.5),
            ],
            top_edge,
        ),
    ]
}

/// A vertical gradient over the top third: sheen at the top, nothing below.
/// Built like `radial_glow`, from vertex colours.
fn sheen(rect: Rect, opacity: f32) -> Mesh {
    let band = Rect::from_min_max(
        Pos2::new(rect.min.x + SHEEN_INSET, rect.min.y),
        Pos2::new(
            rect.max.x - SHEEN_INSET,
            rect.min.y + rect.height() * SHEEN_HEIGHT,
        ),
    );
    let bright = sheen_color().gamma_multiply(opacity);
    let clear = Color32::TRANSPARENT;
    let mut mesh = Mesh::default();
    for (pos, color) in [
        (band.left_top(), bright),
        (band.right_top(), bright),
        (band.right_bottom(), clear),
        (band.left_bottom(), clear),
    ] {
        mesh.vertices.push(Vertex {
            pos,
            uv: Pos2::ZERO,
            color,
        });
    }
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_top_hides_bottom() {
        let top = Color32::from_rgb(200, 10, 10);
        assert_eq!(blend(top, Color32::BLACK), top);
    }

    #[test]
    fn transparent_top_shows_bottom() {
        let bottom = Color32::from_rgb(30, 40, 50);
        assert_eq!(blend(Color32::TRANSPARENT, bottom), bottom);
    }

    #[test]
    fn panel_is_built_from_several_layers() {
        let rect = Rect::from_min_size(Pos2::ZERO, vec2(280.0, 120.0));
        assert_eq!(shapes(rect, Color32::BLACK, 1.0).len(), 5);
    }
}
