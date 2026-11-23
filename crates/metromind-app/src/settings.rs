//! Per-user view settings, stored with eframe's persistence (not in the map file).

use crate::render::ViewVisibility;
use metromind_core::energy::EnergySettings;
use metromind_core::geometry::{Camera, FisheyeLens};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const APP_ID: &str = "MetroMind";
const DEFAULT_MAP_NAME: &str = "My ideas.metromind";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub camera: Camera,
    /// Radius and strength persist; the focus simply follows the cursor.
    pub lens: FisheyeLens,
    pub lens_enabled: bool,
    #[serde(alias = "layers")]
    pub views: ViewVisibility,
    pub last_file: Option<PathBuf>,
    /// Turns off lens smoothing for people who find motion distracting.
    pub reduce_motion: bool,
    pub show_side_panel: bool,
    pub energy: EnergySettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            camera: Camera::default(),
            lens: FisheyeLens::default(),
            lens_enabled: true,
            views: ViewVisibility::default(),
            last_file: None,
            reduce_motion: false,
            show_side_panel: true,
            energy: EnergySettings::default(),
        }
    }
}

impl AppSettings {
    pub fn load(storage: Option<&dyn eframe::Storage>) -> Self {
        storage
            .and_then(|s| eframe::get_value(s, eframe::APP_KEY))
            .unwrap_or_default()
    }

    pub fn save(&self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// The file to open on startup: the last one used, or a default map in the
    /// app's data folder so ideas are saved from the very first one, no dialog needed.
    pub fn startup_file(&self) -> Option<PathBuf> {
        self.last_file
            .clone()
            .or_else(|| eframe::storage_dir(APP_ID).map(|dir| dir.join(DEFAULT_MAP_NAME)))
    }
}
