//! 3D renderer: perspective/orthographic camera with depth, lit mesh
//! pipeline with directional shadow mapping, and the frozen squash
//! environment (flat sky-horizon clear color, ground, decorative cylinders).
//!
//! wgpu 27 conventions per `renderer/`; the sample itself uses an
//! ORTHOGRAPHIC camera (`Main.tscn` lines 101–106: projection=1, size=24) —
//! perspective is implemented and tested too, but the sample view is ortho.

pub mod anim;
pub mod camera;
pub mod collision;
pub mod editor_grid;
pub mod pipeline;
pub mod procedural;

pub use anim::BobAnimation;
pub use camera::{Camera3d, Projection};
pub use collision::*;
pub use pipeline::{sample_light, LightDef, MeshInstance, MeshPipeline, MeshStore, Render3dStats};
pub use procedural::{cylinder_mesh, ground_mesh, CYLINDER_POSITIONS};
