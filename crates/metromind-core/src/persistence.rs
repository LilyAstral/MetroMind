//! The `.metromind` file: UTF-8, pretty-printed JSON with a schema version.
//!
//! Saving writes `<name>.metromind.tmp` next to the target and then renames it
//! over the original. The rename is atomic, so a crash mid-save can never
//! leave a half-written map behind.

use crate::MapError;
use crate::model::{MetroMap, SCHEMA_VERSION};
use serde::Deserialize;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

pub const FILE_EXTENSION: &str = "metromind";

/// Just enough of the file to check its version before parsing the rest, so
/// a newer file with a different shape gets a friendly message instead of a
/// confusing parse error.
#[derive(Deserialize)]
struct Header {
    schema_version: u32,
}

pub fn to_json(map: &MetroMap) -> Result<String, MapError> {
    Ok(serde_json::to_string_pretty(map)?)
}

pub fn from_json(json: &str) -> Result<MetroMap, MapError> {
    let header: Header = serde_json::from_str(json)?;
    if header.schema_version > SCHEMA_VERSION {
        return Err(MapError::NewerSchema {
            found: header.schema_version,
            supported: SCHEMA_VERSION,
        });
    }
    let map: MetroMap = serde_json::from_str(json)?;
    map.validate()?;
    Ok(map)
}

pub fn load(path: &Path) -> Result<MetroMap, MapError> {
    from_json(&fs::read_to_string(path)?)
}

pub fn save(map: &MetroMap, path: &Path) -> Result<(), MapError> {
    let json = to_json(map)?;
    let tmp = temp_path(path);
    fs::write(&tmp, json)?;
    fs::rename(&tmp, path).inspect_err(|_| {
        // Best effort: don't leave the temp file lying around on failure.
        let _ = fs::remove_file(&tmp);
    })?;
    Ok(())
}

/// `<path>.tmp` in the same folder, so the final rename never crosses volumes.
pub fn temp_path(path: &Path) -> PathBuf {
    let mut name = OsString::from(path.as_os_str());
    name.push(".tmp");
    PathBuf::from(name)
}

/// Make sure a user-chosen path ends in `.metromind`.
pub fn with_extension(path: PathBuf) -> PathBuf {
    if path.extension().is_some_and(|e| e == FILE_EXTENSION) {
        path
    } else {
        path.with_extension(FILE_EXTENSION)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temp_path_appends_suffix() {
        let p = temp_path(Path::new("C:/maps/ideas.metromind"));
        assert_eq!(p, PathBuf::from("C:/maps/ideas.metromind.tmp"));
    }

    #[test]
    fn extension_is_added_once() {
        assert_eq!(
            with_extension(PathBuf::from("ideas")),
            PathBuf::from("ideas.metromind")
        );
        assert_eq!(
            with_extension(PathBuf::from("ideas.metromind")),
            PathBuf::from("ideas.metromind")
        );
    }

    #[test]
    fn newer_schema_is_rejected() {
        let json = r#"{ "schema_version": 99, "anything": "else" }"#;
        assert!(matches!(
            from_json(json),
            Err(MapError::NewerSchema { found: 99, .. })
        ));
    }

    #[test]
    fn garbage_is_a_json_error() {
        assert!(matches!(from_json("not json"), Err(MapError::Json(_))));
    }
}
