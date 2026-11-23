//! Energy: a 0–1 value per station that rises when you work on it and decays
//! exponentially. Nothing ticks; energy is computed lazily from the last touch.

mod model;
mod settings;

pub use model::{Touch, bump, decayed};
pub use settings::EnergySettings;
