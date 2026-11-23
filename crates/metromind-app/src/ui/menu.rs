//! Top menu bar. Every action is also reachable here, so nothing depends on
//! remembering a shortcut.

use crate::input::Command;
use crate::settings::AppSettings;
use eframe::egui::{Button, MenuBar, Ui};

pub struct MenuState {
    pub can_undo: bool,
    pub can_redo: bool,
}

pub fn show(ui: &mut Ui, settings: &mut AppSettings, state: &MenuState) -> Vec<Command> {
    let mut commands = Vec::new();
    MenuBar::new().ui(ui, |ui| {
        ui.menu_button("File", |ui| file_menu(ui, &mut commands));
        ui.menu_button("Edit", |ui| edit_menu(ui, state, &mut commands));
        ui.menu_button("View", |ui| view_menu(ui, settings, &mut commands));
        if ui.button("Shortcuts (F1)").clicked() {
            commands.push(Command::ToggleHelp);
        }
    });
    commands
}

fn item(ui: &mut Ui, label: &str, shortcut: &str, enabled: bool) -> bool {
    let button = Button::new(label).shortcut_text(shortcut);
    ui.add_enabled(enabled, button).clicked()
}

fn file_menu(ui: &mut Ui, commands: &mut Vec<Command>) {
    let entries = [
        ("New map…", "Ctrl+N", Command::NewMap),
        ("Open…", "Ctrl+O", Command::Open),
        ("Save", "Ctrl+S", Command::Save),
        ("Save as…", "Ctrl+Shift+S", Command::SaveAs),
        ("Load sample map", "", Command::LoadSample),
    ];
    for (label, shortcut, command) in entries {
        if item(ui, label, shortcut, true) {
            commands.push(command);
        }
    }
}

fn edit_menu(ui: &mut Ui, state: &MenuState, commands: &mut Vec<Command>) {
    if item(ui, "Undo", "Ctrl+Z", state.can_undo) {
        commands.push(Command::Undo);
    }
    if item(ui, "Redo", "Ctrl+Y", state.can_redo) {
        commands.push(Command::Redo);
    }
}

fn view_menu(ui: &mut Ui, settings: &mut AppSettings, commands: &mut Vec<Command>) {
    if item(ui, "Fit whole map", "Home", true) {
        commands.push(Command::FitMap);
    }
    let lens_label = if settings.lens_enabled {
        "Turn lens off"
    } else {
        "Turn lens on"
    };
    if item(ui, lens_label, "L", true) {
        commands.push(Command::ToggleLens);
    }
    ui.checkbox(&mut settings.show_side_panel, "Side panel");
    ui.checkbox(&mut settings.reduce_motion, "Reduce motion");
}
