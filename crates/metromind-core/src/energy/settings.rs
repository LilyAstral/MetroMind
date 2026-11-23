use super::Touch;
use serde::{Deserialize, Serialize};

const SECS_PER_DAY: f64 = 86_400.0;

/// All energy tunables in one place. Serializable so the app can persist them.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EnergySettings {
    /// Time for energy to halve, in seconds (default: 3 days).
    pub half_life_secs: f64,
    /// Energy of a freshly captured idea.
    pub new_station: f64,
    pub select: f64,
    pub edit: f64,
    pub connect: f64,
    pub drag: f64,
    pub rekindle: f64,
    /// Rendering only: cold stations are drawn as if they had at least this much.
    /// Cold is quiet, not gone.
    pub visual_floor: f64,
}

impl Default for EnergySettings {
    fn default() -> Self {
        Self {
            half_life_secs: 3.0 * SECS_PER_DAY,
            new_station: 0.6,
            select: 0.05,
            edit: 0.20,
            connect: 0.15,
            drag: 0.05,
            rekindle: 0.30,
            visual_floor: 0.08,
        }
    }
}

impl EnergySettings {
    /// How much a given kind of interaction adds.
    pub fn amount(&self, touch: Touch) -> f64 {
        match touch {
            Touch::Select => self.select,
            Touch::Edit => self.edit,
            Touch::Connect => self.connect,
            Touch::Drag => self.drag,
            Touch::Rekindle => self.rekindle,
        }
    }

    /// Energy as drawn: never below the visual floor.
    pub fn visible(&self, energy: f64) -> f64 {
        energy.max(self.visual_floor)
    }
}
