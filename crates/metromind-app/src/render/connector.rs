//! The dashed connector preview, drawn above the lines and below the stations.

use super::Theme;
use crate::frame_plan::{FramePlan, to_pos2};
use eframe::egui::{Painter, Pos2, Shape, Stroke};

const WIDTH: f32 = 2.5;
const DASH: f32 = 8.0;
const GAP: f32 = 6.0;

pub fn draw_connector(painter: &Painter, plan: &FramePlan, theme: &Theme) {
    let Some(points) = &plan.connector else {
        return;
    };
    let path: Vec<Pos2> = points.iter().map(|&p| to_pos2(p)).collect();
    if path.len() < 2 {
        return;
    }
    let stroke = Stroke::new(WIDTH, theme.selection);
    painter.extend(Shape::dashed_line(&path, stroke, DASH, GAP));
}
