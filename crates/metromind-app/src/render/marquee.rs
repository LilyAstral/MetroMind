//! The Shift+drag selection box: a rounded, translucent rectangle.

use super::Theme;
use crate::frame_plan::FramePlan;
use eframe::egui::{Painter, Stroke};

const CORNER_RADIUS: f32 = 6.0;
const EDGE_WIDTH: f32 = 1.5;

pub fn draw_marquee(painter: &Painter, plan: &FramePlan, theme: &Theme) {
    let Some(rect) = plan.marquee else {
        return;
    };
    painter.rect_filled(rect, CORNER_RADIUS, theme.selection.gamma_multiply(0.15));
    let stroke = Stroke::new(EDGE_WIDTH, theme.selection.gamma_multiply(0.7));
    painter.rect_stroke(
        rect,
        CORNER_RADIUS,
        stroke,
        eframe::egui::StrokeKind::Middle,
    );
}
