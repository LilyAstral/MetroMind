//! Panels and popups around the map. Each returns edits/commands for the app
//! to apply; none of them mutates the map directly.

pub mod help;
pub mod layers_panel;
pub mod menu;
pub mod notes_panel;
pub mod picker;
pub mod picker_view;
pub mod rename;
pub mod status;
pub mod style;

use crate::render::Theme;
use eframe::egui::{Align2, FontId, Painter, Rect, vec2};

/// Shown on an empty map so the very first gesture is obvious.
pub fn draw_empty_hint(painter: &Painter, rect: Rect, theme: &Theme) {
    let center = rect.center();
    painter.text(
        center,
        Align2::CENTER_CENTER,
        "Double-click anywhere to drop your first idea",
        FontId::proportional(22.0),
        theme.text,
    );
    painter.text(
        center + vec2(0.0, 34.0),
        Align2::CENTER_CENTER,
        "Then just type its name. F1 shows every shortcut · File → Load sample map to explore.",
        FontId::proportional(15.0),
        theme.text_dim,
    );
}
