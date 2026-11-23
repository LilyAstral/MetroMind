//! The palette row above the line list: one striped icon per palette, the
//! current one outlined in the accent colour.

use super::style::ACCENT;
use super::swatch::striped_swatch;
use crate::render::palettes::PALETTES;
use eframe::egui::{Rect, Sense, Stroke, StrokeKind, Ui, vec2};
use metromind_core::editing::Edit;

/// Each icon's hit area; the icon itself is smaller.
const TARGET: f32 = 44.0;
const ICON: f32 = 24.0;
const OUTLINE_GAP: f32 = 3.0;
const OUTLINE_WIDTH: f32 = 2.0;

pub fn show(ui: &mut Ui, current: u8, edits: &mut Vec<Edit>) {
    ui.horizontal(|ui| {
        for (index, palette) in PALETTES.iter().enumerate() {
            let (hit, response) = ui.allocate_exact_size(vec2(TARGET, TARGET), Sense::click());
            let response = response.on_hover_text(palette.name);
            let icon = Rect::from_center_size(hit.center(), vec2(ICON, ICON));
            let painter = ui.painter();
            striped_swatch(painter, icon, &palette.colors);
            let selected = usize::from(current) == index;
            if selected {
                let outline = Stroke::new(OUTLINE_WIDTH, ACCENT);
                let ring = icon.expand(OUTLINE_GAP);
                painter.rect_stroke(ring, 6.0, outline, StrokeKind::Middle);
            }
            if response.clicked() && !selected {
                edits.push(Edit::SetPalette {
                    palette: index as u8,
                });
            }
        }
    });
}
