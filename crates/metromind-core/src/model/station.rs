use super::StationId;
use crate::energy::{EnergySettings, decayed};
use chrono::{DateTime, Utc};
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Shown for an idea captured without a name yet.
pub const UNTITLED: &str = "untitled idea";

/// One line in a station's list. `done` is kept even when the list is shown
/// as bullets, so switching styles doesn't lose ticks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoteItem {
    pub text: String,
    pub done: bool,
}

/// How a station's list is drawn.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListStyle {
    #[default]
    Bullets,
    Checkboxes,
}

/// One idea on the map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Station {
    pub id: StationId,
    pub title: String,
    pub notes: String,
    /// A checklist-style list shown below the free-form notes.
    #[serde(default)]
    pub items: Vec<NoteItem>,
    #[serde(default)]
    pub list_style: ListStyle,
    /// World space, snapped to the grid.
    pub position: Vec2,
    /// Energy (0..=1) at the moment of the last touch; see [`decayed`].
    pub energy_at_touch: f64,
    pub last_touched: DateTime<Utc>,
    pub created: DateTime<Utc>,
}

impl Station {
    /// A freshly captured idea, starting at the configured new-station energy.
    pub fn new(
        id: StationId,
        title: impl Into<String>,
        position: Vec2,
        now: DateTime<Utc>,
        settings: &EnergySettings,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            notes: String::new(),
            items: Vec::new(),
            list_style: ListStyle::default(),
            position,
            energy_at_touch: settings.new_station,
            last_touched: now,
            created: now,
        }
    }

    /// Whether the idea still has no name.
    pub fn is_untitled(&self) -> bool {
        self.title.trim().is_empty()
    }

    /// The title, or a gentle placeholder for an idea captured without one.
    pub fn display_title(&self) -> &str {
        if self.is_untitled() {
            UNTITLED
        } else {
            &self.title
        }
    }

    /// Current energy, computed lazily from the last touch.
    pub fn energy(&self, now: DateTime<Utc>, settings: &EnergySettings) -> f64 {
        decayed(
            self.energy_at_touch,
            self.last_touched,
            now,
            settings.half_life_secs,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn untitled_ideas_get_a_placeholder() {
        let settings = EnergySettings::default();
        let mut s = Station::new(
            StationId(1),
            "  ",
            Vec2::ZERO,
            DateTime::UNIX_EPOCH,
            &settings,
        );
        assert!(s.is_untitled());
        assert_eq!(s.display_title(), UNTITLED);
        s.title = "Idea".into();
        assert_eq!(s.display_title(), "Idea");
    }
}
