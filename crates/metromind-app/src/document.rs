//! The open map file: model, undo history, path, and autosave bookkeeping.

use metromind_core::MapError;
use metromind_core::clock::Clock;
use metromind_core::editing::{Edit, UndoStack, touch};
use metromind_core::energy::{EnergySettings, Touch};
use metromind_core::model::{MetroMap, StationId};
use metromind_core::persistence::{self, FILE_EXTENSION};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Autosave this long after the last change.
pub const AUTOSAVE_DELAY: Duration = Duration::from_secs(2);

pub struct Document {
    pub map: MetroMap,
    undo: UndoStack,
    /// Where the map lives. `None` means it has never been saved (or the file
    /// it came from could not be read, so it must not be overwritten).
    pub path: Option<PathBuf>,
    /// Bumped on every change; cheap cache key for derived data.
    pub revision: u64,
    dirty_since: Option<Instant>,
}

impl Document {
    pub fn new(map: MetroMap, path: Option<PathBuf>) -> Self {
        Self {
            map,
            undo: UndoStack::default(),
            path,
            revision: 0,
            dirty_since: None,
        }
    }

    pub fn open(path: PathBuf) -> Result<Self, MapError> {
        Ok(Self::new(persistence::load(&path)?, Some(path)))
    }

    pub fn apply(
        &mut self,
        edit: Edit,
        clock: &dyn Clock,
        energy: &EnergySettings,
    ) -> Result<(), MapError> {
        self.undo.apply(&mut self.map, edit, clock, energy)?;
        self.changed();
        Ok(())
    }

    pub fn touch(
        &mut self,
        id: StationId,
        kind: Touch,
        clock: &dyn Clock,
        energy: &EnergySettings,
    ) -> Result<(), MapError> {
        touch(&mut self.map, id, kind, clock, energy)?;
        self.changed();
        Ok(())
    }

    pub fn undo(&mut self) -> Result<bool, MapError> {
        let undone = self.undo.undo(&mut self.map)?;
        if undone {
            self.changed();
        }
        Ok(undone)
    }

    pub fn redo(&mut self) -> Result<bool, MapError> {
        let redone = self.undo.redo(&mut self.map)?;
        if redone {
            self.changed();
        }
        Ok(redone)
    }

    pub fn can_undo(&self) -> bool {
        self.undo.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.undo.can_redo()
    }

    fn changed(&mut self) {
        self.revision += 1;
        self.dirty_since = Some(Instant::now());
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty_since.is_some()
    }

    /// Time left until autosave should run, or `None` if nothing is pending
    /// (or there is no file to save to yet).
    pub fn autosave_in(&self) -> Option<Duration> {
        let since = self.dirty_since?;
        self.path.as_ref()?;
        Some(AUTOSAVE_DELAY.saturating_sub(since.elapsed()))
    }

    /// Save to the current path. Returns `false` if there is no path yet.
    pub fn save(&mut self) -> Result<bool, MapError> {
        let Some(path) = self.path.clone() else {
            return Ok(false);
        };
        self.save_to(path)?;
        Ok(true)
    }

    pub fn save_to(&mut self, path: PathBuf) -> Result<(), MapError> {
        persistence::save(&self.map, &path)?;
        self.path = Some(path);
        self.dirty_since = None;
        Ok(())
    }

    /// A short name for the title bar and status line.
    pub fn display_name(&self) -> String {
        self.path.as_deref().and_then(Path::file_stem).map_or_else(
            || "Unsaved map".into(),
            |s| s.to_string_lossy().into_owned(),
        )
    }
}

/// Native "open map" dialog.
pub fn pick_open_path(near: Option<&Path>) -> Option<PathBuf> {
    dialog(near).set_title("Open a map").pick_file()
}

/// Native "save map" dialog; always returns a path ending in `.metromind`.
pub fn pick_save_path(title: &str, near: Option<&Path>, file_name: &str) -> Option<PathBuf> {
    dialog(near)
        .set_title(title)
        .set_file_name(file_name)
        .save_file()
        .map(persistence::with_extension)
}

fn dialog(near: Option<&Path>) -> rfd::FileDialog {
    let dialog = rfd::FileDialog::new().add_filter("MetroMind map", &[FILE_EXTENSION]);
    match near.and_then(Path::parent) {
        Some(dir) => dialog.set_directory(dir),
        None => dialog,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use metromind_core::chrono::DateTime;
    use metromind_core::clock::FakeClock;
    use metromind_core::sample::sample_map;

    #[test]
    fn edits_mark_dirty_and_undo_tracks_revision() {
        let clock = FakeClock::new(DateTime::UNIX_EPOCH);
        let mut doc = Document::new(sample_map(clock.now()), None);
        assert!(!doc.is_dirty());
        let id = *doc.map.stations.keys().next().expect("stations");
        let edit = Edit::Rename {
            id,
            title: "x".into(),
        };
        doc.apply(edit, &clock, &EnergySettings::default())
            .expect("applies");
        assert!(doc.is_dirty());
        assert_eq!(doc.revision, 1);
        assert!(doc.undo().expect("undo"));
        assert_eq!(doc.revision, 2);
        assert!(!doc.undo().expect("nothing left"));
    }

    #[test]
    fn no_autosave_without_a_path() {
        let mut doc = Document::new(MetroMap::default(), None);
        doc.changed();
        assert!(doc.autosave_in().is_none());
        assert!(!doc.save().expect("no-op"));
    }

    #[test]
    fn display_name_uses_file_stem() {
        let doc = Document::new(
            MetroMap::default(),
            Some(PathBuf::from("C:/x/Big ideas.metromind")),
        );
        assert_eq!(doc.display_name(), "Big ideas");
    }
}
