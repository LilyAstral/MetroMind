//! The dark backdrop with a subtle film-grain texture. Always drawn; not a layer.

use super::Theme;
use crate::frame_plan::FramePlan;
use eframe::egui::{Color32, ColorImage, Painter, Rect, TextureId, pos2};

/// Side of the square grain tile in pixels; it repeats across the map.
const GRAIN_SIZE: usize = 128;
/// Strongest grain alpha (out of 255). Kept tiny so it reads as texture, not noise.
const GRAIN_MAX_ALPHA: u32 = 10;

/// A deterministic tileable noise tile (no RNG crate needed).
pub fn grain_image() -> ColorImage {
    let pixels = (0..GRAIN_SIZE * GRAIN_SIZE)
        .map(|i| {
            let alpha = (hash(i as u32) % (GRAIN_MAX_ALPHA + 1)) as u8;
            Color32::from_white_alpha(alpha)
        })
        .collect();
    ColorImage::new([GRAIN_SIZE, GRAIN_SIZE], pixels)
}

/// Integer hash (lowbias32) for repeatable pseudo-random grain.
fn hash(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}

pub fn draw_background(painter: &Painter, plan: &FramePlan, theme: &Theme, grain: TextureId) {
    let rect = plan.viewport;
    painter.rect_filled(rect, 0.0, theme.background);
    let tiles = rect.size() / GRAIN_SIZE as f32;
    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(tiles.x, tiles.y));
    painter.image(grain, rect, uv, Color32::WHITE);
}
