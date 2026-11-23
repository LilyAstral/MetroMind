use super::Theme;
use crate::frame_plan::{FramePlan, to_pos2};
use eframe::egui::{Align2, FontId, Painter, vec2};

const PLATE_PADDING: (f32, f32) = (6.0, 3.0);
const PLATE_ROUNDING: f32 = 6.0;
const EMPHASIS_SCALE: f32 = 1.05;

/// Station titles on a dark plate, so contrast stays well above 3:1 over
/// lines, halos and the field.
pub fn draw_labels(painter: &Painter, plan: &FramePlan, theme: &Theme) {
    for label in &plan.labels {
        let color = if label.placeholder {
            theme.text_dim
        } else {
            theme.text
        };
        let size = if label.emphasized {
            label.size * EMPHASIS_SCALE
        } else {
            label.size
        };
        let galley = painter.layout_no_wrap(label.text.clone(), FontId::proportional(size), color);
        let rect = Align2::LEFT_CENTER.anchor_size(to_pos2(label.anchor), galley.size());
        let plate = rect.expand2(vec2(PLATE_PADDING.0, PLATE_PADDING.1));
        painter.rect_filled(plate, PLATE_ROUNDING, theme.label_plate);
        painter.galley(rect.min, galley, color);
    }
}
