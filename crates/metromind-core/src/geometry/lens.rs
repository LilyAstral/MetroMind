use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

pub const DEFAULT_RADIUS: f32 = 260.0;
pub const DEFAULT_STRENGTH: f32 = 3.0;
pub const RADIUS_RANGE: RangeInclusive<f32> = 120.0..=500.0;
pub const STRENGTH_RANGE: RangeInclusive<f32> = 0.0..=6.0;

/// Sarkar & Brown graphical fisheye, radial form, in view pixels.
///
/// Because it works in pixels (after the camera), it feels the same at every
/// zoom level. It is a render-time transform only and never moves the model.
/// A strength of 0 is the identity, which is how "lens off" is represented.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FisheyeLens {
    pub focus: Vec2,
    /// Lens radius `R` in pixels.
    pub radius: f32,
    /// Distortion `d ≥ 0`; magnification at the center is `d + 1`.
    pub strength: f32,
}

impl Default for FisheyeLens {
    fn default() -> Self {
        Self {
            focus: Vec2::ZERO,
            radius: DEFAULT_RADIUS,
            strength: DEFAULT_STRENGTH,
        }
    }
}

impl FisheyeLens {
    /// The same lens with no distortion.
    pub fn disabled(self) -> Self {
        Self {
            strength: 0.0,
            ..self
        }
    }

    pub fn with_strength(self, strength: f32) -> Self {
        Self {
            strength: strength.clamp(*STRENGTH_RANGE.start(), *STRENGTH_RANGE.end()),
            ..self
        }
    }

    pub fn with_radius(self, radius: f32) -> Self {
        Self {
            radius: radius.clamp(*RADIUS_RANGE.start(), *RADIUS_RANGE.end()),
            ..self
        }
    }

    pub fn is_identity(&self) -> bool {
        self.strength <= 0.0 || self.radius <= 0.0
    }

    /// `(x, r)` for `p`, where `r = |p − f|` and `x = r / R`; `None` outside
    /// the lens or at the exact focus (both are fixed points).
    fn normalized(&self, p: Vec2) -> Option<(f32, f32)> {
        let r = p.distance(self.focus);
        (!self.is_identity() && r > 0.0 && r < self.radius).then(|| (r / self.radius, r))
    }

    /// ```text
    /// x  = r / R
    /// g  = (d + 1) * x / (d * x + 1)
    /// p' = f + (p − f) * (g * R / r)
    /// ```
    /// Points with `r >= R` (or `r == 0`) are unchanged.
    pub fn apply(&self, p: Vec2) -> Vec2 {
        let Some((x, r)) = self.normalized(p) else {
            return p;
        };
        let d = self.strength;
        let g = (d + 1.0) * x / (d * x + 1.0);
        self.focus + (p - self.focus) * (g * self.radius / r)
    }

    /// Local magnification `m(x) = (d + 1) / (d * x + 1)^2` for `x < 1`, else 1.
    /// Used to scale labels, station sizes and hit radii.
    pub fn magnification(&self, p: Vec2) -> f32 {
        if self.is_identity() {
            return 1.0;
        }
        let r = p.distance(self.focus);
        if r >= self.radius {
            return 1.0;
        }
        let d = self.strength;
        let x = r / self.radius;
        (d + 1.0) / (d * x + 1.0).powi(2)
    }

    /// Whether any point of the box `lo..hi` lies inside the lens. Lets callers
    /// skip subdividing segments the lens can't bend.
    pub fn touches_box(&self, lo: Vec2, hi: Vec2) -> bool {
        !self.is_identity() && self.focus.clamp(lo, hi).distance(self.focus) < self.radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn lens() -> FisheyeLens {
        FisheyeLens {
            focus: Vec2::new(100.0, 50.0),
            ..FisheyeLens::default()
        }
    }

    #[test]
    fn identity_outside_radius() {
        let l = lens();
        let p = l.focus + Vec2::new(300.0, 0.0);
        assert_eq!(l.apply(p), p);
        assert_relative_eq!(l.magnification(p), 1.0);
    }

    #[test]
    fn focus_is_a_fixed_point() {
        let l = lens();
        assert_eq!(l.apply(l.focus), l.focus);
    }

    #[test]
    fn monotonic_inside() {
        let l = lens();
        let mut last = 0.0;
        for i in 1..260 {
            let p = l.focus + Vec2::new(i as f32, 0.0);
            let r = l.apply(p).distance(l.focus);
            assert!(r > last, "not monotonic at {i}");
            last = r;
        }
    }

    #[test]
    fn continuous_at_boundary() {
        let l = lens();
        let inside = l.focus + Vec2::new(l.radius - 1e-3, 0.0);
        let outside = l.focus + Vec2::new(l.radius + 1e-3, 0.0);
        assert!(l.apply(inside).distance(l.apply(outside)) < 0.01);
    }

    #[test]
    fn center_magnification_is_d_plus_one() {
        let l = lens();
        let near = l.focus + Vec2::new(1e-4, 0.0);
        assert_relative_eq!(l.magnification(near), l.strength + 1.0, epsilon = 1e-3);
    }

    #[test]
    fn magnification_matches_numeric_derivative() {
        let l = lens();
        let p = l.focus + Vec2::new(80.0, 0.0);
        let h = Vec2::new(0.01, 0.0);
        let numeric = (l.apply(p + h).x - l.apply(p - h).x) / 0.02;
        assert_relative_eq!(numeric, l.magnification(p), epsilon = 1e-2);
    }

    #[test]
    fn disabled_lens_is_identity() {
        let l = lens().disabled();
        let p = l.focus + Vec2::new(10.0, 10.0);
        assert_eq!(l.apply(p), p);
        assert_relative_eq!(l.magnification(p), 1.0);
    }

    #[test]
    fn settings_are_clamped() {
        let l = lens().with_strength(99.0).with_radius(1.0);
        assert_relative_eq!(l.strength, 6.0);
        assert_relative_eq!(l.radius, 120.0);
    }

    #[test]
    fn touches_box_detects_overlap() {
        let l = lens();
        assert!(l.touches_box(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)));
        assert!(!l.touches_box(Vec2::new(1000.0, 0.0), Vec2::new(1010.0, 10.0)));
    }
}
