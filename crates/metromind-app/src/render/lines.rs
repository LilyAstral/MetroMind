use super::Theme;
use super::colors::line_color;
use crate::frame_plan::{FramePlan, to_pos2};
use eframe::egui::{Painter, Shape, Stroke};

/// Subdivided, distorted, octilinear strokes, one per segment.
pub fn draw_lines(painter: &Painter, plan: &FramePlan, _theme: &Theme) {
    for line in &plan.lines {
        let points = line.points.iter().copied().map(to_pos2).collect();
        let stroke = Stroke::new(plan.line_width, line_color(line.color_index));
        painter.add(Shape::line(points, stroke));
    }
}
