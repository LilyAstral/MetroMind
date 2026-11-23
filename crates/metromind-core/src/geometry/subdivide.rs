use glam::Vec2;

/// Split every run of `points` into steps no longer than `max_step`, so a
/// straight segment can bend smoothly when the lens is applied per point.
/// Keeps all original points; a non-positive `max_step` returns a copy.
pub fn subdivide(points: &[Vec2], max_step: f32) -> Vec<Vec2> {
    let Some(&first) = points.first() else {
        return Vec::new();
    };
    if max_step <= 0.0 {
        return points.to_vec();
    }
    let mut out = vec![first];
    for pair in points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let steps = (a.distance(b) / max_step).ceil().max(1.0) as usize;
        out.extend((1..=steps).map(|k| a.lerp(b, k as f32 / steps as f32)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_never_exceed_max() {
        let pts = [Vec2::ZERO, Vec2::new(100.0, 0.0), Vec2::new(100.0, 37.0)];
        let out = subdivide(&pts, 6.0);
        assert!(out.windows(2).all(|w| w[0].distance(w[1]) <= 6.0 + 1e-4));
        assert_eq!(out.first(), pts.first());
        assert_eq!(out.last(), pts.last());
        assert!(out.contains(&pts[1]));
    }

    #[test]
    fn short_runs_are_kept() {
        let pts = [Vec2::ZERO, Vec2::new(2.0, 0.0)];
        assert_eq!(subdivide(&pts, 6.0), pts.to_vec());
    }

    #[test]
    fn empty_input_is_empty() {
        assert!(subdivide(&[], 6.0).is_empty());
    }
}
