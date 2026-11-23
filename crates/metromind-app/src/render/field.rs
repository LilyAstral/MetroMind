//! Optional soft heatmap of energy. Computed on the CPU into a grid at 1/8 of
//! screen resolution, uploaded once, and only recomputed when the plan changes.

use super::colors::{energy_color, with_alpha};
use crate::frame_plan::{FramePlan, STATION_RADIUS};
use eframe::egui::{
    Color32, ColorImage, Context, Painter, Rect, TextureHandle, TextureId, TextureOptions, pos2,
};

/// Screen pixels per field cell.
const CELL_PX: f32 = 8.0;
/// How much wider than a halo each station's field reaches.
const SPREAD: f32 = 4.0;
const MAX_ALPHA: f32 = 0.35;

/// Splat each station's falloff `(1 − (d/r)²)² × E` over only the cells within
/// its radius, then color cells along the energy ramp.
pub fn field_image(plan: &FramePlan) -> ColorImage {
    let size = plan.viewport.size() / CELL_PX;
    let (w, h) = (
        size.x.ceil().max(1.0) as usize,
        size.y.ceil().max(1.0) as usize,
    );
    let mut heat = vec![0.0_f32; w * h];
    let origin = plan.viewport.min;
    for s in &plan.stations {
        let radius = STATION_RADIUS * (1.5 + 3.0 * s.energy) * s.magnification * SPREAD / CELL_PX;
        let center = (
            (s.pos.x - origin.x) / CELL_PX,
            (s.pos.y - origin.y) / CELL_PX,
        );
        let range = |c: f32, max: usize| {
            let lo = (c - radius).floor().max(0.0) as usize;
            let hi = ((c + radius).ceil().max(0.0) as usize).min(max);
            lo..hi
        };
        for y in range(center.1, h) {
            for x in range(center.0, w) {
                let d2 = ((x as f32 + 0.5 - center.0).powi(2)
                    + (y as f32 + 0.5 - center.1).powi(2))
                    / (radius * radius);
                if d2 < 1.0 {
                    heat[y * w + x] += (1.0 - d2).powi(2) * s.energy;
                }
            }
        }
    }
    let pixels = heat.into_iter().map(heat_color).collect();
    ColorImage::new([w, h], pixels)
}

fn heat_color(v: f32) -> Color32 {
    let v = v.min(1.0);
    with_alpha(energy_color(v), v * MAX_ALPHA)
}

pub fn draw_field(painter: &Painter, plan: &FramePlan, texture: TextureId) {
    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    painter.image(texture, plan.viewport, uv, Color32::WHITE);
}

/// Holds the uploaded heatmap and the key it was computed for. The key is a
/// cheap stand-in for a dirty flag: anything that changes the plan changes it.
#[derive(Default)]
pub struct FieldCache {
    texture: Option<TextureHandle>,
    key: Option<FieldKey>,
}

/// What the field depends on: the map revision, the view, the lens, and a coarse
/// clock tick (energy decays slowly, so once a minute is plenty).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FieldKey {
    pub revision: u64,
    pub minute: i64,
    pub camera: metromind_core::geometry::Camera,
    pub lens: metromind_core::geometry::FisheyeLens,
    pub viewport: Rect,
    /// Position of a station being dragged, which moves without an edit.
    pub drag: Option<metromind_core::Vec2>,
}

impl FieldCache {
    pub fn texture_for(&mut self, ctx: &Context, plan: &FramePlan, key: FieldKey) -> TextureId {
        if self.key != Some(key) || self.texture.is_none() {
            let image = field_image(plan);
            match &mut self.texture {
                Some(texture) => texture.set(image, TextureOptions::LINEAR),
                None => {
                    self.texture =
                        Some(ctx.load_texture("energy-field", image, TextureOptions::LINEAR));
                }
            }
            self.key = Some(key);
        }
        self.texture
            .as_ref()
            .map_or(TextureId::default(), TextureHandle::id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame_plan::ScreenStation;
    use metromind_core::Vec2;
    use metromind_core::model::StationId;

    #[test]
    fn heat_peaks_at_station_and_is_empty_elsewhere() {
        let station = ScreenStation {
            id: StationId(1),
            pos: Vec2::new(100.0, 100.0),
            radius: 7.0,
            magnification: 1.0,
            energy: 1.0,
            is_interchange: false,
            line_count: 0,
            color: None,
            dimmed: false,
            selected: false,
            primary: false,
            hovered: false,
        };
        let plan = FramePlan {
            viewport: Rect::from_min_max(pos2(0.0, 0.0), pos2(400.0, 400.0)),
            stations: vec![station],
            ..FramePlan::default()
        };
        let image = field_image(&plan);
        assert_eq!(image.size, [50, 50]);
        let at = |x: usize, y: usize| image.pixels[y * 50 + x].a();
        assert!(at(12, 12) > 0);
        assert_eq!(at(45, 45), 0);
    }
}
