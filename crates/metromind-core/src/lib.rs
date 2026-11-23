#![forbid(unsafe_code)]
//! MetroMind core: the metro-map model, geometry, energy, undo and persistence.
//!
//! This crate deliberately has no GUI dependency. Everything here is plain data
//! and pure functions so it can be unit-tested without a window.

pub mod clock;
pub mod editing;
pub mod energy;
mod error;
pub mod geometry;
pub mod model;
pub mod persistence;
pub mod sample;

pub use chrono;
pub use error::MapError;
pub use glam::Vec2;
