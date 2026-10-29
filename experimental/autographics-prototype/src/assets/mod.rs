//! Asset pipeline: GLB model loading (T-02 decision: gltf 1.4) into
//! GPU-agnostic mesh data, with classified failures. No wgpu types here —
//! render3d uploads the resulting `MeshData`. PLAN-004 adds a bounded static
//! PNG payload beside the model payload; neither decoder is reachable from
//! untrusted input before structural admission.

pub mod glb;
pub mod node_animation;
pub mod png;
pub mod textured;

pub use glb::{load_glb_bytes, load_glb_file, AssetError, MaterialDef, MeshData, ModelData};
pub use png::{
    decode_png_bytes, encode_png_rgba, preflight as preflight_png, ImageData, PngHeader,
};
