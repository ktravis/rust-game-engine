pub mod display;
pub mod instance;
pub mod mesh;
pub mod model;
pub mod state;
pub mod text;
pub mod texture;

mod renderer;

pub use display::*;
pub use renderer::*;
pub use state::{RenderPass, RenderState};
pub use texture::*;

slotmap::new_key_type! {
    pub struct MeshRef;
    pub struct PipelineRef;
}
