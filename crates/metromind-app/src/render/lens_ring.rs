use super::Theme;
use crate::frame_plan::{FramePlan, to_pos2};
use eframe::egui::{Painter, Stroke};

const RING_WIDTH: f32 = 1.5;
const FOCUS_DOT_RADIUS: f32 = 2.0;

/// A thin outline of the lens radius plus a tiny focus dot, so the lens
/// position is always obvious.
pub fn draw_lens_ring(painter: &Painter, plan: &FramePlan, theme: &Theme) {
    if !plan.lens_enabled || plan.lens.is_identity() {
        return;
    }
    let focus = to_pos2(plan.lens.focus);
    painter.circle_stroke(
        focus,
        plan.lens.radius,
        Stroke::new(RING_WIDTH, theme.lens_ring),
    );
    painter.circle_filled(focus, FOCUS_DOT_RADIUS, theme.lens_ring);
}
