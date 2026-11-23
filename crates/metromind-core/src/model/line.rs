use super::{LineId, StationId};
use serde::{Deserialize, Serialize};

/// How many distinct line colors (and number-key shortcuts) exist.
pub const LINE_COLOR_COUNT: u8 = 9;

/// A named, colored theme: an ordered run of stations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub id: LineId,
    pub name: String,
    pub color_index: u8,
    pub stations: Vec<StationId>,
}

impl Line {
    pub fn contains(&self, station: StationId) -> bool {
        self.stations.contains(&station)
    }

    /// Where `station` sits on this line, if it does.
    pub fn index_of(&self, station: StationId) -> Option<usize> {
        self.stations.iter().position(|&s| s == station)
    }
}
