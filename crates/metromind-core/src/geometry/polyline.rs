use glam::Vec2;

/// Smallest miter scale; stops sharp corners from shooting the offset to infinity.
const MIN_MITER_DOT: f32 = 0.35;

/// Shortest distance from `p` to the polyline `points`.
pub fn distance_to_polyline(p: Vec2, points: &[Vec2]) -> f32 {
    match points {
        [] => f32::INFINITY,
        [only] => p.distance(*only),
        _ => points
            .windows(2)
            .map(|w| distance_to_segment(p, w[0], w[1]))
            .fold(f32::INFINITY, f32::min),
    }
}

fn distance_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_squared();
    if len_sq == 0.0 {
        return p.distance(a);
    }
    let t = ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

/// Left-hand normal of the run `a → b`.
fn normal(a: Vec2, b: Vec2) -> Vec2 {
    (b - a).normalize_or_zero().perp()
}

/// Shift a polyline sideways by `distance` (positive = left of travel), with
/// mitered joins so parallel lines stay parallel through bends. Used to draw
/// several lines sharing one segment as side-by-side strokes.
pub fn offset_polyline(points: &[Vec2], distance: f32) -> Vec<Vec2> {
    if points.len() < 2 || distance == 0.0 {
        return points.to_vec();
    }
    let last = points.len() - 1;
    (0..=last)
        .map(|i| {
            let before = normal(points[i.saturating_sub(1)], points[i.max(1)]);
            let after = normal(points[i.min(last - 1)], points[(i + 1).min(last)]);
            let miter = (before + after).normalize_or(before);
            let scale = distance / miter.dot(before).max(MIN_MITER_DOT);
            points[i] + miter * scale
        })
        .collect()
}

/// Replace each interior corner with a short quadratic curve of up to
/// `radius`, for the soft, rounded bends of the design language.
pub fn round_corners(points: &[Vec2], radius: f32, samples: usize) -> Vec<Vec2> {
    if points.len() < 3 || radius <= 0.0 || samples == 0 {
        return points.to_vec();
    }
    let mut out = vec![points[0]];
    for w in points.windows(3) {
        let (prev, corner, next) = (w[0], w[1], w[2]);
        let r = radius
            .min(corner.distance(prev) * 0.5)
            .min(corner.distance(next) * 0.5);
        let start = corner + (prev - corner).normalize_or_zero() * r;
        let end = corner + (next - corner).normalize_or_zero() * r;
        let t = |k: usize| k as f32 / samples as f32;
        out.extend((0..=samples).map(|k| quadratic(start, corner, end, t(k))));
    }
    out.push(points[points.len() - 1]);
    out
}

fn quadratic(a: Vec2, control: Vec2, b: Vec2, t: f32) -> Vec2 {
    a.lerp(control, t).lerp(control.lerp(b, t), t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn distance_to_horizontal_line() {
        let pts = [Vec2::ZERO, Vec2::new(10.0, 0.0)];
        assert_relative_eq!(distance_to_polyline(Vec2::new(5.0, 3.0), &pts), 3.0);
        assert_relative_eq!(distance_to_polyline(Vec2::new(13.0, 4.0), &pts), 5.0);
    }

    #[test]
    fn offset_straight_line_is_parallel() {
        let pts = [Vec2::ZERO, Vec2::new(10.0, 0.0)];
        let out = offset_polyline(&pts, 2.0);
        assert!(out.iter().all(|p| (p.y.abs() - 2.0).abs() < 1e-5));
    }

    #[test]
    fn offset_keeps_distance_through_bend() {
        let pts = [Vec2::ZERO, Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)];
        let out = offset_polyline(&pts, 1.0);
        assert_relative_eq!(distance_to_polyline(out[1], &pts), 1.0, epsilon = 1e-4);
    }

    #[test]
    fn rounding_keeps_ends_and_softens_corner() {
        let pts = [Vec2::ZERO, Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)];
        let out = round_corners(&pts, 3.0, 4);
        assert_eq!(out.first(), pts.first());
        assert_eq!(out.last(), pts.last());
        assert!(!out.contains(&pts[1]));
        assert!(distance_to_polyline(Vec2::new(10.0, 0.0), &out) > 0.5);
    }
}
