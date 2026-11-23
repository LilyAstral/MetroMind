use eframe::egui::{Color32, Mesh, Pos2, Vec2, epaint::Vertex};
use std::f32::consts::TAU;

/// Rim vertices per halo: enough to look round, few enough for 500 stations.
const RIM_VERTICES: u32 = 24;

/// A soft radial gradient. egui has no radial gradients, so this is a
/// triangle fan whose center vertex has the full color and whose rim is fully
/// transparent; vertex interpolation draws the falloff.
pub fn radial_glow(center: Pos2, radius: f32, color: Color32) -> Mesh {
    let mut mesh = Mesh::default();
    mesh.colored_vertex(center, color);
    for i in 0..RIM_VERTICES {
        let angle = i as f32 / RIM_VERTICES as f32 * TAU;
        let rim = center + Vec2::angled(angle) * radius;
        mesh.vertices.push(Vertex {
            pos: rim,
            uv: Pos2::ZERO,
            color: Color32::TRANSPARENT,
        });
    }
    for i in 0..RIM_VERTICES {
        mesh.add_triangle(0, 1 + i, 1 + (i + 1) % RIM_VERTICES);
    }
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fan_has_center_plus_rim() {
        let mesh = radial_glow(Pos2::new(10.0, 10.0), 5.0, Color32::WHITE);
        assert_eq!(mesh.vertices.len(), 1 + RIM_VERTICES as usize);
        assert_eq!(mesh.indices.len(), 3 * RIM_VERTICES as usize);
        assert_eq!(mesh.vertices[0].color, Color32::WHITE);
        assert!(
            mesh.vertices[1..]
                .iter()
                .all(|v| v.color == Color32::TRANSPARENT)
        );
        assert!(mesh.is_valid());
    }
}
