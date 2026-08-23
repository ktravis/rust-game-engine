use crate::{
    geom::*,
    renderer::{bindings::DepthTextureView, state::BoundTexture},
};

#[rustfmt::skip]
pub const DEFAULT_TEXTURE_DATA: [u8; 16] = [
    255, 255, 255, 255,
    255, 255, 255, 255,
    255, 255, 255, 255,
    255, 255, 255, 255,
];

// TODO: needed?
pub struct OffscreenFramebuffer {
    pub color: BoundTexture,
    pub depth: Option<DepthTextureView>,
    pub(super) size: Point<u32>,
    pub(super) format: wgpu::TextureFormat,
}

impl OffscreenFramebuffer {
    pub fn size_pixels(&self) -> Point<u32> {
        self.size
    }

    pub fn color_format(&self) -> wgpu::TextureFormat {
        self.format
    }
}
