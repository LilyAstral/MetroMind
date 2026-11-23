//! `MetroMindApp`: owns all state and runs the per-frame flow
//!
//! ```text
//! panels → input::collect → apply edits → apply commands → frame_plan::build → render
//! → autosave check → repaint scheduling
//! ```
//!
//! Data flows one way: Input → Edit → Model → this frame's plan → Render.

mod commands;
mod files;
mod map_view;

use crate::document::Document;
use crate::frame_plan::{FramePlan, to_pos2};
use crate::input::{Command, Interaction};
use crate::render::{self, Theme, field::FieldCache};
use crate::settings::AppSettings;
use crate::ui::{
    self, layers_panel::LineNameDrafts, notes_panel::NotesState, rename::RenameState,
    status::Status,
};
use eframe::egui::{self, Context, TextureHandle, TextureOptions, Ui};
use metromind_core::clock::SystemClock;
use metromind_core::editing::Edit;
use std::path::PathBuf;
use std::time::Duration;

/// Decay is slow; refresh colors this often while idle.
const IDLE_REFRESH: Duration = Duration::from_secs(60);

pub struct MetroMindApp {
    doc: Document,
    settings: AppSettings,
    clock: SystemClock,
    theme: Theme,
    interaction: Interaction,
    lens_pinned: bool,
    plan: FramePlan,
    rename: Option<RenameState>,
    notes: Option<NotesState>,
    line_drafts: LineNameDrafts,
    status: Status,
    show_help: bool,
    grain: TextureHandle,
    field: FieldCache,
    /// Edits and commands from panels, applied with the map's input.
    queued_edits: Vec<Edit>,
    queued_commands: Vec<Command>,
    window_title: String,
    /// Fit the map on the next frame, once the viewport size is known.
    fit_pending: bool,
}

impl MetroMindApp {
    /// `file` (from the command line) overrides the last-opened map.
    pub fn new(cc: &eframe::CreationContext<'_>, file: Option<PathBuf>) -> Self {
        ui::style::apply(&cc.egui_ctx);
        let settings = AppSettings::load(cc.storage);
        let grain = cc.egui_ctx.load_texture(
            "grain",
            render::grain_image(),
            TextureOptions::LINEAR_REPEAT,
        );
        let mut app = Self {
            doc: Document::new(Default::default(), None),
            settings,
            clock: SystemClock,
            theme: Theme::default(),
            interaction: Interaction::default(),
            lens_pinned: false,
            plan: FramePlan::default(),
            rename: None,
            notes: None,
            line_drafts: LineNameDrafts::new(),
            status: Status::default(),
            show_help: false,
            grain,
            field: FieldCache::default(),
            queued_edits: Vec::new(),
            queued_commands: Vec::new(),
            window_title: String::new(),
            fit_pending: false,
        };
        app.open_startup_file(file);
        app
    }

    fn panels(&mut self, ui: &mut Ui) {
        let menu_state = ui::menu::MenuState {
            can_undo: self.doc.can_undo(),
            can_redo: self.doc.can_redo(),
        };
        egui::Panel::top("menu").show(ui, |ui| {
            let commands = ui::menu::show(ui, &mut self.settings, &menu_state);
            self.queued_commands.extend(commands);
        });
        let info = ui::status::StatusInfo {
            file: self.doc.display_name(),
            save_state: match (self.doc.path.is_some(), self.doc.is_dirty()) {
                (false, _) => "not saved yet",
                (true, true) => "saving…",
                (true, false) => "saved",
            },
            zoom: self.settings.camera.zoom,
            lens: self
                .settings
                .lens_enabled
                .then_some((self.settings.lens.strength, self.settings.lens.radius)),
        };
        ui::status::show(ui, &self.status, &info, &self.theme);
        let edits =
            ui::layers_panel::show(ui, &mut self.settings, &self.doc.map, &mut self.line_drafts);
        self.queued_edits.extend(edits);
        self.notes_panel(ui);
    }

    fn notes_panel(&mut self, ui: &mut Ui) {
        let Some(notes) = &mut self.notes else {
            return;
        };
        let outcome = ui::notes_panel::show(ui, notes, &self.doc.map);
        self.queued_edits.extend(outcome.edits);
        self.queued_commands.extend(outcome.commands);
        if outcome.close {
            self.close_notes();
        }
    }

    fn rename_popup(&mut self, ctx: &Context) {
        let Some(state) = &mut self.rename else {
            return;
        };
        let Some(station) = self.plan.station(state.id) else {
            self.rename = None;
            return;
        };
        let at = to_pos2(station.pos) + egui::vec2(station.radius + 6.0, -20.0);
        match ui::rename::show(ctx, state, at) {
            ui::rename::RenameResult::Editing => {}
            ui::rename::RenameResult::Commit => self.finish_rename(true),
            ui::rename::RenameResult::Cancel => self.finish_rename(false),
        }
    }

    fn schedule_repaint(&mut self, ctx: &Context) {
        if let Some(left) = self.doc.autosave_in() {
            if left.is_zero() {
                self.save_quietly();
            } else {
                ctx.request_repaint_after(left);
            }
        }
        ctx.request_repaint_after(IDLE_REFRESH);
        let title = format!(
            "{}{} — MetroMind",
            self.doc.display_name(),
            if self.doc.is_dirty() { " •" } else { "" }
        );
        if title != self.window_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.window_title = title;
        }
    }
}

impl eframe::App for MetroMindApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.panels(ui);
        egui::CentralPanel::no_frame().show(ui, |ui| self.map_view(ui));
        let ctx = ui.ctx().clone();
        self.rename_popup(&ctx);
        ui::help::show(&ctx, &mut self.show_help);
        self.schedule_repaint(&ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.flush_drafts();
        if self.doc.is_dirty() {
            self.save_quietly();
        }
        self.settings.last_file.clone_from(&self.doc.path);
        self.settings.save(storage);
    }
}
