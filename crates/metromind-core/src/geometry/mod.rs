//! Pure geometry: world → view (camera) → screen (fisheye lens), plus
//! octilinear routing and polyline helpers.
//!
//! ```text
//! world ──Camera──▶ view (px) ──FisheyeLens──▶ screen (px)
//! ```

mod camera;
mod lens;
mod polyline;
mod router;
mod subdivide;

pub use camera::{Camera, MAX_ZOOM, MIN_ZOOM};
pub use lens::{FisheyeLens, RADIUS_RANGE, STRENGTH_RANGE};
pub use polyline::{distance_to_polyline, offset_polyline, round_corners};
pub use router::{GRID_CELL, route, snap_to_grid};
pub use subdivide::subdivide;
