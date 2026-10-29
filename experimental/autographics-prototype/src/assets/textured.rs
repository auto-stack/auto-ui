//! Explicit texture metadata supplied only after no-IO GLB admission.
use super::ImageData;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Sampling {
    /// glTF enums, validated by the producer. Mip filters normalize to base filtering.
    pub mag: u32,
    pub min: u32,
    pub wrap_s: u32,
    pub wrap_t: u32,
}
impl Default for Sampling {
    fn default() -> Self {
        Self {
            mag: 9729,
            min: 9729,
            wrap_s: 10497,
            wrap_t: 10497,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MeshVisual {
    /// Effective UV after KHR_texture_transform. Geometry and collision stay unchanged.
    pub uv: Vec<[f32; 2]>,
    pub image: Option<Arc<ImageData>>,
    pub sampling: Sampling,
    pub double_sided: bool,
    /// Original source associations and invertible world matrix keep 006 recoverable.
    pub node_index: usize,
    pub mesh_index: usize,
    pub primitive_index: usize,
    pub world: [[f32; 4]; 4],
}
