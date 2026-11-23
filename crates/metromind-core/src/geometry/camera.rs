use glam::Vec2;
use serde::{Deserialize, Serialize};

pub const MIN_ZOOM: f32 = 0.1;
pub const MAX_ZOOM: f32 = 8.0;
/// "Fit map" never zooms in further than this, so a tiny map isn't comically big.
pub const FIT_MAX_ZOOM: f32 = 1.5;

/// Pan and zoom: `view = world * zoom + pan`, where view is in screen points
/// before the lens is applied.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    pub pan: Vec2,
    pub zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

impl Camera {
    pub fn world_to_view(&self, world: Vec2) -> Vec2 {
        world * self.zoom + self.pan
    }

    pub fn view_to_world(&self, view: Vec2) -> Vec2 {
        (view - self.pan) / self.zoom
    }

    pub fn panned(self, delta: Vec2) -> Self {
        Self {
            pan: self.pan + delta,
            ..self
        }
    }

    /// Zoom by `factor`, keeping the world point under `anchor` (view space) fixed.
    pub fn zoomed_around(self, anchor: Vec2, factor: f32) -> Self {
        let zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        let world = self.view_to_world(anchor);
        Self {
            pan: anchor - world * zoom,
            zoom,
        }
    }

    /// A camera that shows the world box `lo..hi` centered in the viewport,
    /// leaving `margin` points free on every side.
    pub fn fit(lo: Vec2, hi: Vec2, viewport_min: Vec2, viewport_size: Vec2, margin: f32) -> Self {
        let content = (hi - lo).max(Vec2::splat(1.0));
        let room = (viewport_size - Vec2::splat(2.0 * margin)).max(Vec2::splat(1.0));
        let zoom = (room / content).min_element().clamp(MIN_ZOOM, FIT_MAX_ZOOM);
        let center = (lo + hi) * 0.5;
        Self {
            pan: viewport_min + viewport_size * 0.5 - center * zoom,
            zoom,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn view_and_world_round_trip() {
        let cam = Camera {
            pan: Vec2::new(30.0, -12.0),
            zoom: 2.5,
        };
        let w = Vec2::new(17.0, 4.0);
        let back = cam.view_to_world(cam.world_to_view(w));
        assert_relative_eq!(back.x, w.x, epsilon = 1e-4);
        assert_relative_eq!(back.y, w.y, epsilon = 1e-4);
    }

    #[test]
    fn zoom_keeps_anchor_fixed() {
        let cam = Camera::default();
        let anchor = Vec2::new(200.0, 100.0);
        let before = cam.view_to_world(anchor);
        let after = cam.zoomed_around(anchor, 1.7).view_to_world(anchor);
        assert_relative_eq!(before.x, after.x, epsilon = 1e-3);
        assert_relative_eq!(before.y, after.y, epsilon = 1e-3);
    }

    #[test]
    fn zoom_is_clamped() {
        let cam = Camera::default().zoomed_around(Vec2::ZERO, 1e6);
        assert_relative_eq!(cam.zoom, MAX_ZOOM);
    }

    #[test]
    fn fit_centers_content_inside_margin() {
        let cam = Camera::fit(
            Vec2::new(-100.0, -50.0),
            Vec2::new(100.0, 50.0),
            Vec2::new(10.0, 20.0),
            Vec2::new(800.0, 600.0),
            40.0,
        );
        let center = cam.world_to_view(Vec2::ZERO);
        assert_relative_eq!(center.x, 410.0, epsilon = 1e-3);
        assert_relative_eq!(center.y, 320.0, epsilon = 1e-3);
        let right = cam.world_to_view(Vec2::new(100.0, 0.0));
        assert!(right.x <= 10.0 + 800.0 - 40.0 + 1e-3);
    }
}
