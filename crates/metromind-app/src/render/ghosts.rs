//! Hollow markers at the spots where dragged stations will land.

use super::Theme;
use crate::frame_plan::{FramePlan, STATION_RADIUS, to_pos2};
use eframe::egui::{Painter, Stroke};

const GHOST_WIDTH: f32 = 1.5;

pub fn draw_ghosts(painter: &Painter, plan: &FramePlan, theme: &Theme) {
    let stroke = Stroke::new(GHOST_WIDTH, theme.selection.gamma_multiply(0.6));
    for &pos in &plan.ghosts {
        painter.circle_stroke(to_pos2(pos), STATION_RADIUS, stroke);
    }
}
