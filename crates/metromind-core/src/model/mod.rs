//! Plain-data metro map model.

mod ids;
mod line;
mod map;
mod station;

pub use ids::{LineId, StationId};
pub use line::{LINE_COLOR_COUNT, Line};
pub use map::{MetroMap, SCHEMA_VERSION, Segment, canonical_pair};
pub use station::{ListStyle, NoteItem, Station, UNTITLED};
