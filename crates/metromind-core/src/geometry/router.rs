use glam::Vec2;

/// Default world grid cell; station positions snap to multiples of this.
pub const GRID_CELL: f32 = 40.0;

pub fn snap_to_grid(p: Vec2, cell: f32) -> Vec2 {
    if cell <= 0.0 {
        return p;
    }
    (p / cell).round() * cell
}

/// Octilinear route from `a` to `b`: one diagonal run for `min(|dx|, |dy|)`,
/// then a straight run for the rest. Only 0°, 45° and 90° angles appear.
///
/// Returns 2 points when a single run suffices (axis-aligned or exactly
/// diagonal), otherwise 3 (start, bend, end). Pure; no state, no cache.
pub fn route(a: Vec2, b: Vec2) -> Vec<Vec2> {
    let delta = b - a;
    let diagonal = delta.x.abs().min(delta.y.abs());
    let straight = delta.x.abs().max(delta.y.abs()) - diagonal;
    if diagonal == 0.0 || straight == 0.0 {
        return vec![a, b];
    }
    let bend = a + Vec2::new(delta.x.signum(), delta.y.signum()) * diagonal;
    vec![a, bend, b]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_octilinear(a: Vec2, b: Vec2) -> bool {
        let d = (b - a).abs();
        d.x == 0.0 || d.y == 0.0 || (d.x - d.y).abs() < 1e-4
    }

    #[test]
    fn straight_runs_are_two_points() {
        let a = Vec2::new(0.0, 0.0);
        assert_eq!(route(a, Vec2::new(120.0, 0.0)).len(), 2);
        assert_eq!(route(a, Vec2::new(0.0, -80.0)).len(), 2);
    }

    #[test]
    fn exact_diagonal_is_two_points() {
        assert_eq!(route(Vec2::ZERO, Vec2::new(-40.0, 40.0)).len(), 2);
    }

    #[test]
    fn general_route_is_diagonal_then_straight() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(160.0, 40.0);
        assert_eq!(route(a, b), vec![a, Vec2::new(40.0, 40.0), b]);
    }

    #[test]
    fn every_run_is_octilinear() {
        let a = Vec2::new(40.0, 80.0);
        for (x, y) in [
            (-200.0, 40.0),
            (120.0, -360.0),
            (-80.0, -120.0),
            (40.0, 280.0),
        ] {
            let pts = route(a, Vec2::new(x, y));
            assert!(pts.windows(2).all(|w| is_octilinear(w[0], w[1])));
            assert_eq!(pts.last().copied(), Some(Vec2::new(x, y)));
        }
    }

    #[test]
    fn same_point_is_degenerate_but_safe() {
        assert_eq!(route(Vec2::ONE, Vec2::ONE).len(), 2);
    }

    #[test]
    fn snapping_rounds_to_nearest_cell() {
        assert_eq!(
            snap_to_grid(Vec2::new(59.0, -21.0), 40.0),
            Vec2::new(40.0, -40.0)
        );
    }
}
