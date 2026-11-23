//! Left side panel: the palette and line list (number keys, rename, colour,
//! order, delete), the Views toggles, and lens settings.

use super::line_rows::{self, LineNameDrafts};
use super::palette_row;
use crate::input::Command;
use crate::render::View;
use crate::settings::AppSettings;
use eframe::egui::{self, RichText, Ui};
use metromind_core::editing::Edit;
use metromind_core::geometry::{RADIUS_RANGE, STRENGTH_RANGE};
use metromind_core::model::{LineId, MetroMap};

const PANEL_WIDTH: f32 = 360.0;

/// Draw the panel. Edits go back as the return value; view-only changes
/// (hover and highlight) go into `commands`.
pub fn show(
    ui: &mut Ui,
    settings: &mut AppSettings,
    map: &MetroMap,
    drafts: &mut LineNameDrafts,
    locked: Option<LineId>,
    commands: &mut Vec<Command>,
) -> Vec<Edit> {
    let mut edits = Vec::new();
    let mut open = settings.show_side_panel;
    egui::Panel::left("side_panel")
        .default_size(PANEL_WIDTH)
        .show_collapsible(ui, &mut open, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                lines_section(ui, map, locked, drafts, &mut edits, commands);
                ui.separator();
                views_section(ui, settings);
                ui.separator();
                lens_section(ui, settings);
            });
        });
    settings.show_side_panel = open;
    edits
}

fn lines_section(
    ui: &mut Ui,
    map: &MetroMap,
    locked: Option<LineId>,
    drafts: &mut LineNameDrafts,
    edits: &mut Vec<Edit>,
    commands: &mut Vec<Command>,
) {
    ui.heading("Lines");
    palette_row::show(ui, map.palette, edits);
    if map.lines.is_empty() {
        ui.label(RichText::new("Select an idea and press N to start a line.").weak());
    }
    line_rows::show(ui, map, locked, drafts, edits, commands);
    ui.label(
        RichText::new("1–9 adds the selected idea · Shift+number removes it")
            .small()
            .weak(),
    );
}

fn views_section(ui: &mut Ui, settings: &mut AppSettings) {
    ui.heading("Views");
    for view in View::ALL {
        ui.checkbox(settings.views.flag_mut(view), view.label());
    }
}

fn lens_section(ui: &mut Ui, settings: &mut AppSettings) {
    ui.heading("Lens");
    ui.checkbox(&mut settings.lens_enabled, "Fisheye lens (L)");
    let lens = &mut settings.lens;
    ui.add(egui::Slider::new(&mut lens.strength, STRENGTH_RANGE).text("Strength"))
        .on_hover_text("Ctrl + mouse wheel");
    ui.add(egui::Slider::new(&mut lens.radius, RADIUS_RANGE).text("Radius"))
        .on_hover_text("Shift + mouse wheel");
    ui.label(
        RichText::new("Hold Space to pin the lens in place.")
            .small()
            .weak(),
    );
    ui.checkbox(&mut settings.reduce_motion, "Reduce motion");
}
