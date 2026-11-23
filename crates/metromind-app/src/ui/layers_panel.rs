//! Left side panel: layer checkboxes, the line legend (number keys, rename,
//! delete), and lens settings.

use crate::render::Layer;
use crate::render::colors::line_color;
use crate::settings::AppSettings;
use eframe::egui::{self, RichText, Sense, TextEdit, Ui, vec2};
use metromind_core::editing::Edit;
use metromind_core::geometry::{RADIUS_RANGE, STRENGTH_RANGE};
use metromind_core::model::{LineId, MetroMap};
use std::collections::BTreeMap;

const PANEL_WIDTH: f32 = 270.0;
const SWATCH: f32 = 16.0;

/// Text buffers for line names being edited, keyed by line.
pub type LineNameDrafts = BTreeMap<LineId, String>;

pub fn show(
    ui: &mut Ui,
    settings: &mut AppSettings,
    map: &MetroMap,
    drafts: &mut LineNameDrafts,
) -> Vec<Edit> {
    let mut edits = Vec::new();
    let mut open = settings.show_side_panel;
    egui::Panel::left("side_panel")
        .default_size(PANEL_WIDTH)
        .show_collapsible(ui, &mut open, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                lines_section(ui, map, drafts, &mut edits);
                ui.separator();
                layers_section(ui, settings);
                ui.separator();
                lens_section(ui, settings);
            });
        });
    settings.show_side_panel = open;
    edits
}

fn lines_section(ui: &mut Ui, map: &MetroMap, drafts: &mut LineNameDrafts, edits: &mut Vec<Edit>) {
    ui.heading("Lines");
    if map.lines.is_empty() {
        ui.label(RichText::new("Select an idea and press N to start a line.").weak());
    }
    for (index, line) in map.lines.values().enumerate() {
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(SWATCH, SWATCH), Sense::hover());
            ui.painter()
                .rect_filled(rect, SWATCH / 2.0, line_color(line.color_index));
            ui.label(format!("{}", index + 1));
            let draft = drafts.entry(line.id).or_insert_with(|| line.name.clone());
            let response = ui.add(
                TextEdit::singleline(draft)
                    .desired_width(120.0)
                    .align(egui::Align2::LEFT_CENTER),
            );
            if response.lost_focus() {
                if !draft.trim().is_empty() && *draft != line.name {
                    edits.push(Edit::RenameLine {
                        id: line.id,
                        name: draft.trim().to_owned(),
                    });
                }
                drafts.remove(&line.id);
            } else if !response.has_focus() {
                draft.clone_from(&line.name);
            }
            let delete = ui
                .button("✕")
                .on_hover_text("Delete this line (Ctrl+Z brings it back)");
            if delete.clicked() {
                edits.push(Edit::DeleteLine { id: line.id });
            }
        });
    }
    ui.label(
        RichText::new("1–9 adds the selected idea · Shift+number removes it")
            .small()
            .weak(),
    );
}

fn layers_section(ui: &mut Ui, settings: &mut AppSettings) {
    ui.heading("Layers");
    for layer in Layer::ALL {
        ui.checkbox(settings.layers.flag_mut(layer), layer.label());
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
