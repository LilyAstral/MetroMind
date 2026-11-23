//! Opening, saving and switching map files.

use super::MetroMindApp;
use crate::document::{Document, pick_open_path, pick_save_path};
use crate::input::Interaction;
use eframe::egui::Rect;
use metromind_core::clock::Clock;
use metromind_core::model::MetroMap;
use metromind_core::sample::sample_map;
use std::path::PathBuf;

impl MetroMindApp {
    /// Open the last file (or the default map). A missing file just means a
    /// fresh start; an unreadable one is left untouched on disk.
    pub(super) fn open_startup_file(&mut self, file: Option<PathBuf>) {
        let Some(path) = file.or_else(|| self.settings.startup_file()) else {
            self.status
                .problem("No data folder found; use File → Save as to keep your map.");
            return;
        };
        // The saved camera belongs to the last file; any other file starts fitted.
        self.fit_pending = self.settings.last_file.as_ref() != Some(&path);
        if !path.exists() {
            self.doc = Document::new(MetroMap::default(), Some(path));
            return;
        }
        match Document::open(path.clone()) {
            Ok(doc) => self.doc = doc,
            Err(err) => {
                self.doc = Document::new(MetroMap::default(), None);
                self.status
                    .problem(format!("Couldn't open {}: {err}", path.display()));
            }
        }
    }

    /// Replace the open document, resetting view state and fitting the map.
    fn switch_to(&mut self, doc: Document, viewport: Rect) {
        self.doc = doc;
        self.interaction = Interaction::default();
        self.rename = None;
        self.notes = None;
        self.line_drafts.clear();
        self.settings.last_file.clone_from(&self.doc.path);
        self.fit(viewport);
    }

    /// Save the current map first, so switching files never loses ideas.
    fn save_before_switch(&mut self) -> bool {
        self.flush_drafts();
        if !self.doc.is_dirty() {
            return true;
        }
        if self.doc.path.is_none() {
            self.save_as();
        } else {
            self.save_quietly();
        }
        !self.doc.is_dirty()
    }

    pub(super) fn open_dialog(&mut self, viewport: Rect) {
        if !self.save_before_switch() {
            return;
        }
        let Some(path) = pick_open_path(self.doc.path.as_deref()) else {
            return;
        };
        match Document::open(path.clone()) {
            Ok(doc) => {
                self.switch_to(doc, viewport);
                self.status
                    .info(format!("Opened {}", self.doc.display_name()));
            }
            Err(err) => self
                .status
                .problem(format!("Couldn't open {}: {err}", path.display())),
        }
    }

    pub(super) fn new_map(&mut self, viewport: Rect) {
        if !self.save_before_switch() {
            return;
        }
        let Some(path) = pick_save_path("Create a new map", self.doc.path.as_deref(), "New ideas")
        else {
            return;
        };
        let mut doc = Document::new(MetroMap::default(), Some(path));
        match doc.save() {
            Ok(_) => {
                self.switch_to(doc, viewport);
                self.status
                    .info("Fresh map. Double-click anywhere to start.");
            }
            Err(err) => self
                .status
                .problem(format!("Couldn't create the map: {err}")),
        }
    }

    /// The sample only loads into an empty map, so it can never clobber ideas.
    pub(super) fn load_sample(&mut self, viewport: Rect) {
        if !self.doc.map.is_empty() {
            self.status
                .info("The sample loads into an empty map. Start one with File → New map.");
            return;
        }
        let path = self.doc.path.clone();
        let mut doc = Document::new(sample_map(self.clock.now()), path);
        doc.revision = self.doc.revision + 1;
        self.switch_to(doc, viewport);
        self.save_quietly();
        self.status
            .info("Sample map loaded. Hover to look around; press F1 for shortcuts.");
    }

    pub(super) fn save_now(&mut self) {
        self.flush_drafts();
        if self.doc.path.is_none() {
            self.save_as();
            return;
        }
        self.save_quietly();
        if !self.doc.is_dirty() {
            self.status
                .info(format!("Saved {}", self.doc.display_name()));
        }
    }

    pub(super) fn save_as(&mut self) {
        self.flush_drafts();
        let name = self.doc.display_name();
        let Some(path) = pick_save_path("Save map as", self.doc.path.as_deref(), &name) else {
            return;
        };
        self.save_to(path);
    }

    fn save_to(&mut self, path: PathBuf) {
        match self.doc.save_to(path) {
            Ok(()) => {
                self.settings.last_file.clone_from(&self.doc.path);
                self.status
                    .info(format!("Saved {}", self.doc.display_name()));
            }
            Err(err) => self.status.problem(format!("Couldn't save: {err}")),
        }
    }

    /// Autosave path: no message on success, a calm one on failure.
    pub(super) fn save_quietly(&mut self) {
        if let Some(dir) = self.doc.path.as_deref().and_then(std::path::Path::parent) {
            // The default map lives in the app data folder, which may not exist yet.
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(err) = self.doc.save() {
            self.status.problem(format!("Couldn't save: {err}"));
        }
    }
}
