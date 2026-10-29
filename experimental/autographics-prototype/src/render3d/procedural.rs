//! Procedural environment meshes for the frozen squash scene (the sample
//! builds them from Godot primitive meshes in `Main.tscn`; T-06 needs the
//! same geometry, so it lives here, not in the probe).

use crate::assets::{MaterialDef, MeshData};

fn white_material(name: &str) -> MaterialDef {
    MaterialDef {
        name: name.to_string(),
        base_color: [1.0, 1.0, 1.0, 1.0],
        metallic: 0.0,
        roughness: 0.5,
    }
}

/// Ground box top: 60×60 quad at y=0 (Main.tscn lines 48–57: BoxMesh
/// 60×2×60 with top at y=0; the visible surface is the top face).
pub fn ground_mesh() -> MeshData {
    let h = 30.0f32;
    let positions = vec![[-h, 0.0, -h], [h, 0.0, -h], [h, 0.0, h], [-h, 0.0, h]];
    let normals = vec![[0.0, 1.0, 0.0]; 4];
    MeshData {
        visual: None,
        name: "ground#0".into(),
        positions,
        normals,
        indices: vec![0, 2, 1, 0, 3, 2],
        material: white_material("ground"),
    }
}

/// Decorative cylinder: `Main.tscn` lines 32–35, 108–126 — CylinderMesh with
/// `radial_segments = 24`, `rings = 1`, default radius 1 / height 2, albedo
/// (0.635294, 0.211765, 0.0235294). World positions (parent Cylinders at
/// (8,0,0) + child offsets): (14,1,−15), (−13,1,−15), (−13,1,16), (14,1,16).
pub const CYLINDER_ALBEDO: [f32; 3] = [0.635294, 0.211765, 0.0235294];
pub const CYLINDER_POSITIONS: [[f32; 3]; 4] = [
    [14.0, 1.0, -15.0],
    [-13.0, 1.0, -15.0],
    [-13.0, 1.0, 16.0],
    [14.0, 1.0, 16.0],
];

pub fn cylinder_mesh() -> MeshData {
    const SEGMENTS: usize = 24;
    let radius = 1.0f32;
    let half_h = 1.0f32; // height 2, centered
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    // Side surface: two vertices per segment (bottom, top).
    for i in 0..=SEGMENTS {
        let a = (i as f32 / SEGMENTS as f32) * std::f32::consts::TAU;
        let (s, c) = a.sin_cos();
        let n = [c, 0.0, s];
        positions.push([c * radius, -half_h, s * radius]);
        normals.push(n);
        positions.push([c * radius, half_h, s * radius]);
        normals.push(n);
    }
    for i in 0..SEGMENTS {
        let b = (i * 2) as u32;
        indices.extend_from_slice(&[b, b + 1, b + 2, b + 1, b + 3, b + 2]);
    }
    // Top and bottom caps (fans), flat normals.
    let top_center = positions.len() as u32;
    positions.push([0.0, half_h, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    for i in 0..=SEGMENTS {
        let a = (i as f32 / SEGMENTS as f32) * std::f32::consts::TAU;
        let (s, c) = a.sin_cos();
        positions.push([c * radius, half_h, s * radius]);
        normals.push([0.0, 1.0, 0.0]);
    }
    for i in 0..SEGMENTS {
        indices.extend_from_slice(&[
            top_center,
            top_center + 1 + i as u32 + 1,
            top_center + 1 + i as u32,
        ]);
    }
    let bottom_center = positions.len() as u32;
    positions.push([0.0, -half_h, 0.0]);
    normals.push([0.0, -1.0, 0.0]);
    for i in 0..=SEGMENTS {
        let a = (i as f32 / SEGMENTS as f32) * std::f32::consts::TAU;
        let (s, c) = a.sin_cos();
        positions.push([c * radius, -half_h, s * radius]);
        normals.push([0.0, -1.0, 0.0]);
    }
    for i in 0..SEGMENTS {
        indices.extend_from_slice(&[
            bottom_center,
            bottom_center + 1 + i as u32,
            bottom_center + 1 + i as u32 + 1,
        ]);
    }

    MeshData {
        visual: None,
        name: "cylinder#0".into(),
        positions,
        normals,
        indices,
        material: MaterialDef {
            name: "cylinder".into(),
            base_color: [
                CYLINDER_ALBEDO[0],
                CYLINDER_ALBEDO[1],
                CYLINDER_ALBEDO[2],
                1.0,
            ],
            metallic: 0.0,
            roughness: 0.5,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cylinder_has_24_segments_and_caps() {
        let m = cylinder_mesh();
        // sides: 25 rings × 2 verts + caps: 2 centers + 2×25 rim = 100 verts
        assert_eq!(m.positions.len(), 25 * 2 + 2 + 2 * 25);
        // side quads: 24×2 triangles; caps: 24×2 triangles
        assert_eq!(m.indices.len(), (24 * 2 + 24 * 2) * 3);
        assert!(m.indices.iter().all(|i| (*i as usize) < m.positions.len()));
        // height 2 centered
        let min_y = m.positions.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
        let max_y = m.positions.iter().map(|p| p[1]).fold(f32::MIN, f32::max);
        assert_eq!((min_y, max_y), (-1.0, 1.0));
    }
}
