use super::Theme;
use super::colors::line_color;
use crate::frame_plan::{FramePlan, ScreenStation, to_pos2};
use eframe::egui::{Painter, Rect, Stroke, StrokeKind, vec2};

const RING_WIDTH: f32 = 3.0;
const OUTLINE_WIDTH: f32 = 2.0;
const SELECTION_GAP: f32 = 5.0;
const SELECTION_WIDTH: f32 = 2.5;

/// Regular stations are rings in their line color; interchanges are white
/// pills with a dark outline; free-floating ideas are hollow circles.
pub fn draw_stations(painter: &Painter, plan: &FramePlan, theme: &Theme) {
    for station in &plan.stations {
        let outer = if station.is_interchange {
            draw_interchange(painter, station, plan.line_width, theme)
        } else {
            draw_regular(painter, station, theme)
        };
        draw_focus_ring(painter, station, outer, theme);
    }
}

fn draw_regular(painter: &Painter, s: &ScreenStation, theme: &Theme) -> Rect {
    let center = to_pos2(s.pos);
    let ring = match s.color_index {
        Some(index) => line_color(index),
        None => theme.free_station,
    };
    painter.circle_filled(center, s.radius, theme.station_fill);
    painter.circle_stroke(center, s.radius, Stroke::new(RING_WIDTH, ring));
    Rect::from_center_size(center, vec2(2.0, 2.0) * s.radius)
}

/// The pill grows by one line width per extra line, like a real metro map.
fn draw_interchange(painter: &Painter, s: &ScreenStation, line_width: f32, theme: &Theme) -> Rect {
    let extra = line_width * (s.line_count.saturating_sub(1)) as f32;
    let size = vec2(2.0 * s.radius + extra, 2.0 * s.radius);
    let rect = Rect::from_center_size(to_pos2(s.pos), size);
    painter.rect_filled(rect, s.radius, theme.interchange_fill);
    let outline = Stroke::new(OUTLINE_WIDTH, theme.interchange_outline);
    painter.rect_stroke(rect, s.radius, outline, StrokeKind::Outside);
    rect
}

/// "Where am I" at a glance: a clear ring around the selection, a soft one on hover.
fn draw_focus_ring(painter: &Painter, s: &ScreenStation, outer: Rect, theme: &Theme) {
    let color = if s.primary {
        theme.selection
    } else if s.selected {
        theme.selection.gamma_multiply(0.55)
    } else if s.hovered {
        theme.hover
    } else {
        return;
    };
    let ring = outer.expand(SELECTION_GAP);
    let radius = ring.height() / 2.0;
    let stroke = Stroke::new(SELECTION_WIDTH, color);
    painter.rect_stroke(ring, radius, stroke, StrokeKind::Middle);
}
