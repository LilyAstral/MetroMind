//! Grid intersections, thinned out when zoomed far out and bent by the lens.

use super::{GridDot, MIN_GRID_SPACING_PX, to_vec2};
use eframe::egui::Rect;
use metromind_core::Vec2;
use metromind_core::geometry::{Camera, FisheyeLens, GRID_CELL};

pub(super) fn build_grid(camera: Camera, lens: FisheyeLens, viewport: Rect) -> Vec<GridDot> {
    if !viewport.is_positive() {
        return Vec::new();
    }
    let mut cell = GRID_CELL;
    while cell * camera.zoom < MIN_GRID_SPACING_PX {
        cell *= 2.0;
    }
    let lo = (camera.view_to_world(to_vec2(viewport.min)) / cell).floor() * cell;
    let hi = camera.view_to_world(to_vec2(viewport.max));
    let mut dots = Vec::new();
    let mut y = lo.y;
    while y <= hi.y {
        let mut x = lo.x;
        while x <= hi.x {
            let view = camera.world_to_view(Vec2::new(x, y));
            dots.push(GridDot {
                pos: lens.apply(view),
                magnification: lens.magnification(view),
            });
            x += cell;
        }
        y += cell;
    }
    dots
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::pos2;

    #[test]
    fn grid_thins_out_when_zoomed_out() {
        let viewport = Rect::from_min_max(pos2(0.0, 0.0), pos2(800.0, 600.0));
        let lens = FisheyeLens::default().disabled();
        let cells = |spacing: f32| ((800.0 / spacing + 2.0) * (600.0 / spacing + 2.0)) as usize;
        for zoom in [0.1, 0.33, 1.0, 4.0] {
            let camera = Camera {
                zoom,
                ..Camera::default()
            };
            let dots = build_grid(camera, lens, viewport).len();
            assert!(
                dots > 0 && dots <= cells(MIN_GRID_SPACING_PX),
                "zoom {zoom}: {dots}"
            );
        }
    }
}
