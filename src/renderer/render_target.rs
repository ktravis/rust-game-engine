use crate::geom::Point;

use super::{DisplayView, OffscreenFramebuffer, RenderState};

pub trait RenderTarget {
    fn size_pixels(&self) -> Point<u32>;
    fn color_attachment<'a, 'st: 'a>(
        &'a self,
        state: &'st RenderState,
        load_op: wgpu::LoadOp<wgpu::Color>,
    ) -> wgpu::RenderPassColorAttachment<'a>;

    fn depth_stencil_attachment<'a, 'st: 'a>(
        &'a self,
        _state: &'st RenderState,
        _depth_load_op: wgpu::LoadOp<f32>,
        _stencil_ops: impl Into<Option<wgpu::Operations<u32>>>,
    ) -> Option<wgpu::RenderPassDepthStencilAttachment<'a>> {
        None
    }

    fn clear(&self, clear_color: wgpu::Color) -> Clear<'_, Self>
    where
        Self: Sized,
    {
        Clear {
            inner: self,
            clear_color,
        }
    }

    fn load(&self) -> Load<'_, Self>
    where
        Self: Sized,
    {
        Load { inner: self }
    }

    fn load_depth(&self) -> LoadDepth<'_, Self>
    where
        Self: Sized,
    {
        LoadDepth { inner: self }
    }

    fn clear_depth(&self, value: f32) -> ClearDepth<'_, Self>
    where
        Self: Sized,
    {
        ClearDepth { inner: self, value }
    }
}

impl<'d> RenderTarget for DisplayView<'d> {
    fn size_pixels(&self) -> Point<u32> {
        self.display().size_pixels()
    }

    fn color_attachment<'a, 'st: 'a>(
        &'a self,
        _state: &'st RenderState,
        load_op: wgpu::LoadOp<wgpu::Color>,
    ) -> wgpu::RenderPassColorAttachment<'a> {
        wgpu::RenderPassColorAttachment {
            view: &self.view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: load_op,
                store: wgpu::StoreOp::Store,
            },
        }
    }

    fn depth_stencil_attachment<'a, 'st: 'a>(
        &'a self,
        _state: &'st RenderState,
        depth_load_op: wgpu::LoadOp<f32>,
        stencil_ops: impl Into<Option<wgpu::Operations<u32>>>,
    ) -> Option<wgpu::RenderPassDepthStencilAttachment<'a>> {
        self.display()
            .depth_stencil_attachment(depth_load_op, stencil_ops)
    }
}

impl RenderTarget for OffscreenFramebuffer {
    fn size_pixels(&self) -> Point<u32> {
        self.size
    }

    fn color_attachment<'a, 'st: 'a>(
        &'a self,
        state: &'st RenderState,
        load_op: wgpu::LoadOp<wgpu::Color>,
    ) -> wgpu::RenderPassColorAttachment<'a> {
        let view = &state.get_texture(self.color).view;
        wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: load_op,
                store: wgpu::StoreOp::Store,
            },
        }
    }

    fn depth_stencil_attachment<'a, 'st: 'a>(
        &'a self,
        state: &'st RenderState,
        depth_load_op: wgpu::LoadOp<f32>,
        stencil_ops: impl Into<Option<wgpu::Operations<u32>>>,
    ) -> Option<wgpu::RenderPassDepthStencilAttachment<'a>> {
        self.depth
            .map(|tex| wgpu::RenderPassDepthStencilAttachment {
                view: &state.get_texture(tex).view,
                depth_ops: Some(wgpu::Operations {
                    load: depth_load_op,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: stencil_ops.into(),
            })
    }
}

pub struct Clear<'r, R: RenderTarget> {
    inner: &'r R,
    clear_color: wgpu::Color,
}

impl<'r, R: RenderTarget> RenderTarget for Clear<'r, R> {
    fn size_pixels(&self) -> Point<u32> {
        self.inner.size_pixels()
    }

    fn color_attachment<'a, 'st: 'a>(
        &'a self,
        state: &'st RenderState,
        _load_op: wgpu::LoadOp<wgpu::Color>,
    ) -> wgpu::RenderPassColorAttachment<'a> {
        self.inner
            .color_attachment(state, wgpu::LoadOp::Clear(self.clear_color))
    }

    fn depth_stencil_attachment<'a, 'st: 'a>(
        &'a self,
        state: &'st RenderState,
        depth_load_op: wgpu::LoadOp<f32>,
        stencil_ops: impl Into<Option<wgpu::Operations<u32>>>,
    ) -> Option<wgpu::RenderPassDepthStencilAttachment<'a>> {
        self.inner
            .depth_stencil_attachment(state, depth_load_op, stencil_ops)
    }
}

pub struct Load<'r, R: RenderTarget> {
    inner: &'r R,
}

impl<'r, R: RenderTarget> RenderTarget for Load<'r, R> {
    fn size_pixels(&self) -> Point<u32> {
        self.inner.size_pixels()
    }

    fn color_attachment<'a, 'st: 'a>(
        &'a self,
        state: &'st RenderState,
        _load_op: wgpu::LoadOp<wgpu::Color>,
    ) -> wgpu::RenderPassColorAttachment<'a> {
        self.inner.color_attachment(state, wgpu::LoadOp::Load)
    }

    fn depth_stencil_attachment<'a, 'st: 'a>(
        &'a self,
        state: &'st RenderState,
        depth_load_op: wgpu::LoadOp<f32>,
        stencil_ops: impl Into<Option<wgpu::Operations<u32>>>,
    ) -> Option<wgpu::RenderPassDepthStencilAttachment<'a>> {
        self.inner
            .depth_stencil_attachment(state, depth_load_op, stencil_ops)
    }
}

pub struct LoadDepth<'r, R: RenderTarget> {
    inner: &'r R,
}

impl<'r, R: RenderTarget> RenderTarget for LoadDepth<'r, R> {
    fn size_pixels(&self) -> Point<u32> {
        self.inner.size_pixels()
    }

    fn color_attachment<'a, 'st: 'a>(
        &'a self,
        state: &'st RenderState,
        load_op: wgpu::LoadOp<wgpu::Color>,
    ) -> wgpu::RenderPassColorAttachment<'a> {
        self.inner.color_attachment(state, load_op)
    }

    fn depth_stencil_attachment<'a, 'st: 'a>(
        &'a self,
        state: &'st RenderState,
        _depth_load_op: wgpu::LoadOp<f32>,
        stencil_ops: impl Into<Option<wgpu::Operations<u32>>>,
    ) -> Option<wgpu::RenderPassDepthStencilAttachment<'a>> {
        self.inner
            .depth_stencil_attachment(state, wgpu::LoadOp::Load, stencil_ops)
    }
}

pub struct ClearDepth<'r, R: RenderTarget> {
    inner: &'r R,
    value: f32,
}

impl<'r, R: RenderTarget> RenderTarget for ClearDepth<'r, R> {
    fn size_pixels(&self) -> Point<u32> {
        self.inner.size_pixels()
    }

    fn color_attachment<'a, 'st: 'a>(
        &'a self,
        state: &'st RenderState,
        load_op: wgpu::LoadOp<wgpu::Color>,
    ) -> wgpu::RenderPassColorAttachment<'a> {
        self.inner.color_attachment(state, load_op)
    }

    fn depth_stencil_attachment<'a, 'st: 'a>(
        &'a self,
        state: &'st RenderState,
        _depth_load_op: wgpu::LoadOp<f32>,
        stencil_ops: impl Into<Option<wgpu::Operations<u32>>>,
    ) -> Option<wgpu::RenderPassDepthStencilAttachment<'a>> {
        self.inner
            .depth_stencil_attachment(state, wgpu::LoadOp::Clear(self.value), stencil_ops)
    }
}
