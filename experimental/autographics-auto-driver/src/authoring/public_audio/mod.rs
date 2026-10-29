//! Finite one-shot service. Only logical accept can insert a voice; rendering
//! and capture have no audio side effects. Device and offline paths share PCM.
pub mod codec;
pub mod mixer;
pub mod native;
