//! 2D rendering helpers: glyph-atlas text (fontdue) on top of the wgpu
//! foundation. Shapes/colored quads live in `renderer::QuadPipeline`.

pub mod text;

pub use text::{GlyphQuad, TextRenderer};
