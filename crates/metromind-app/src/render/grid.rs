use super::Theme;
use crate::frame_plan::{FramePlan, to_pos2};
use eframe::egui::Painter;

const DOT_RADIUS: f32 = 1.3;
const MAX_DOT_SCALE: f32 = 2.0;

/// Faint dots at grid intersections, distorted by the lens.
pub fn draw_grid(painter: &Painter, plan: &FramePlan, theme: &Theme) {
    for dot in &plan.grid {
        let radius = DOT_RADIUS * dot.magnification.clamp(1.0, MAX_DOT_SCALE);
        painter.circle_filled(to_pos2(dot.pos), radius, theme.grid_dot);
    }
}
