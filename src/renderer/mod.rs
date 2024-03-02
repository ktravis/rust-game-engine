pub mod display;
pub mod instance;
pub mod mesh;
pub mod model;
pub mod render_target;
pub mod sprite_renderer;
pub mod state;
pub mod text;
pub mod texture;

mod renderer;

pub use display::*;
pub use render_target::*;
pub use renderer::*;
pub use state::{RenderPass, RenderState};
pub use texture::*;

slotmap::new_key_type! {
    pub struct MeshRef;
    pub struct PipelineRef;
}
