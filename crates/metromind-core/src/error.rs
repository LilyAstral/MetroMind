use crate::model::{LineId, StationId};

/// Everything that can go wrong in the core. The app shows these in its status
/// line; it never panics on them.
#[derive(Debug, thiserror::Error)]
pub enum MapError {
    #[error("could not read or write the file: {0}")]
    Io(#[from] std::io::Error),
    #[error("the file is not a valid MetroMind map: {0}")]
    Json(#[from] serde_json::Error),
    #[error(
        "this map was made with a newer MetroMind (format v{found}); this version reads up to v{supported}"
    )]
    NewerSchema { found: u32, supported: u32 },
    #[error("station {0} does not exist")]
    UnknownStation(StationId),
    #[error("line {0} does not exist")]
    UnknownLine(LineId),
    #[error("id {0} is already in use")]
    DuplicateId(u64),
    #[error("that station is already on this line")]
    AlreadyOnLine,
    #[error("position {index} is outside the line")]
    BadIndex { index: usize },
}
