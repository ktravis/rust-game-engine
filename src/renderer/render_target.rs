use super::TextureRef;

#[derive(Debug, Clone, Copy)]
pub enum RenderTarget<'a> {
    TextureView(&'a wgpu::TextureView),
    TextureRef(TextureRef),
}
