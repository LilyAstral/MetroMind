//! The shortcut cheat sheet (F1). Nothing is hidden permanently.

use eframe::egui::{self, Context, Grid, RichText};

const SHORTCUTS: [(&str, &str); 20] = [
    ("New idea", "Double-click empty space"),
    ("New idea on a line", "Double-click a line"),
    ("Rename", "F2, double-click, or just start typing"),
    ("Notes", "Enter"),
    ("Move", "Drag (snaps to the grid)"),
    ("Add to line", "1–9"),
    ("Start a new line here", "N"),
    ("Remove from line", "Shift + line number"),
    ("Delete", "Del"),
    ("Rekindle", "R"),
    ("Deselect", "Esc"),
    ("Pan", "Drag empty space, middle- or right-drag"),
    ("Zoom", "Mouse wheel"),
    ("Lens on/off", "L"),
    ("Pin lens", "Hold Space"),
    ("Lens strength / radius", "Ctrl + wheel / Shift + wheel"),
    ("Undo / redo", "Ctrl+Z / Ctrl+Y"),
    ("Open / save / save as", "Ctrl+O / Ctrl+S / Ctrl+Shift+S"),
    ("New map", "Ctrl+N"),
    ("Fit whole map", "Home"),
];

const COMPACT_ROW: f32 = 22.0;

pub fn show(ctx: &Context, open: &mut bool) {
    egui::Window::new("Shortcuts")
        .open(open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            // A reference table, not a set of buttons: rows can be compact.
            ui.spacing_mut().interact_size.y = COMPACT_ROW;
            Grid::new("shortcuts")
                .striped(true)
                .spacing([24.0, 6.0])
                .show(ui, |ui| {
                    for (action, keys) in SHORTCUTS {
                        ui.label(action);
                        ui.label(RichText::new(keys).strong());
                        ui.end_row();
                    }
                });
        });
}
