//! Drawing the line picker: a small rounded panel at the release point.

use super::picker::{Choice, Picker, Row};
use crate::render::colors::line_color;
use eframe::egui::{
    Align2, Area, Color32, Context, FontId, Frame, Id, Order, Pos2, Rect, Response, Sense, Stroke,
    Ui, Vec2, pos2, vec2,
};

const WIDTH: f32 = 260.0;
/// At least 44 px so every row is an easy target.
const ROW_HEIGHT: f32 = 44.0;
const PADDING: f32 = 8.0;
const SWATCH_RADIUS: f32 = 7.0;
const TEXT_INSET: f32 = 30.0;
const PANEL: Color32 = Color32::from_rgb(33, 37, 44);
const HIGHLIGHT: Color32 = Color32::from_rgb(58, 65, 77);
const TEXT: Color32 = Color32::from_rgb(237, 230, 218);
const DIM: Color32 = Color32::from_rgb(150, 150, 160);

/// What the player did with the picker this frame.
pub enum Action {
    Pick(usize),
    Cancel,
}

/// Draw the picker and report a row click, or a cancel: a press outside the
/// panel, or a right-click anywhere. Hovering a valid row highlights it.
pub fn show(ctx: &Context, picker: &mut Picker, rows: &[Row], viewport: Rect) -> Option<Action> {
    let size = vec2(WIDTH, rows.len() as f32 * ROW_HEIGHT + 2.0 * PADDING);
    let at = clamp_inside(picker.anchor, size, viewport);
    let mut action = None;
    let area = Area::new(Id::unique("line_picker"))
        .order(Order::Foreground)
        .fixed_pos(at)
        .show(ctx, |ui| {
            Frame::new()
                .fill(PANEL)
                .corner_radius(10.0)
                .inner_margin(PADDING)
                .show(ui, |ui| {
                    ui.set_width(WIDTH - 2.0 * PADDING);
                    for (index, row) in rows.iter().enumerate() {
                        let response = row_ui(ui, index, row, index == picker.highlight);
                        if row.is_valid() && response.hovered() {
                            picker.highlight = index;
                        }
                        if row.is_valid() && response.clicked() {
                            action = Some(Action::Pick(index));
                        }
                    }
                });
        });
    action.or_else(|| cancelled(ctx, area.response.rect))
}

/// A press outside the panel, or a right-click, closes the picker.
fn cancelled(ctx: &Context, panel: Rect) -> Option<Action> {
    let (outside, right) = ctx.input(|i| {
        let outside = i.pointer.primary_pressed()
            && i.pointer.press_origin().is_some_and(|p| !panel.contains(p));
        (
            outside,
            i.pointer
                .button_pressed(eframe::egui::PointerButton::Secondary),
        )
    });
    (outside || right).then_some(Action::Cancel)
}

fn row_ui(ui: &mut Ui, index: usize, row: &Row, highlighted: bool) -> Response {
    let sense = if row.is_valid() {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), ROW_HEIGHT), sense);
    let painter = ui.painter();
    let valid = row.is_valid();
    if highlighted && valid {
        painter.rect_filled(rect, 6.0, HIGHLIGHT);
    }
    let dim = if valid { 1.0 } else { 0.35 };
    let center = rect.left_center() + vec2(TEXT_INSET / 2.0, 0.0);
    match row.color_index {
        Some(color) => {
            painter.circle_filled(center, SWATCH_RADIUS, line_color(color).gamma_multiply(dim));
        }
        None => {
            painter.circle_stroke(center, SWATCH_RADIUS, Stroke::new(1.5, DIM));
        }
    }
    let label = match row.choice {
        Choice::Line(_) => format!("{}  {}", index + 1, row.name),
        Choice::New => row.name.clone(),
    };
    let text_at = pos2(rect.left() + TEXT_INSET, rect.top() + 14.0);
    let text_color = if valid { TEXT } else { DIM };
    painter.text(
        text_at,
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(14.0),
        text_color,
    );
    if let Some(reason) = &row.refusal {
        let reason_at = pos2(rect.left() + TEXT_INSET, rect.top() + 32.0);
        painter.text(
            reason_at,
            Align2::LEFT_CENTER,
            reason,
            FontId::proportional(11.0),
            DIM,
        );
    }
    response
}

/// Keep the panel fully inside the map, even when released near an edge.
fn clamp_inside(anchor: Pos2, size: Vec2, viewport: Rect) -> Pos2 {
    let max_x = (viewport.max.x - size.x).max(viewport.min.x);
    let max_y = (viewport.max.y - size.y).max(viewport.min.y);
    pos2(
        anchor.x.clamp(viewport.min.x, max_x),
        anchor.y.clamp(viewport.min.y, max_y),
    )
}
