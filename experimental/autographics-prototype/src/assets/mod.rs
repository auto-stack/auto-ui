//! Asset pipeline: GLB model loading (T-02 decision: gltf 1.4) into
//! GPU-agnostic mesh data, with classified failures. No wgpu types here —
//! render3d uploads the resulting `MeshData`.

pub mod glb;

pub use glb::{load_glb_bytes, load_glb_file, AssetError, MaterialDef, MeshData, ModelData};
