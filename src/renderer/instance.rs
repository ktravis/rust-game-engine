use std::ops::{Deref, DerefMut};

use crate::transform::Transform;

use super::state::RenderPass;
use super::{MeshRef, PipelineRef, TextureRef};

use super::ModelInstanceData;

#[derive(Debug)]
pub struct InstanceRenderData<T> {
    pub texture: Option<TextureRef>,
    pub pipeline: PipelineRef,
    pub mesh: MeshRef,
    pub model: ModelInstanceData<T>,
}

impl<T> Deref for InstanceRenderData<T> {
    type Target = ModelInstanceData<T>;

    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

pub trait DrawInstance {
    fn draw_instance<T: Transform>(&mut self, instance: &InstanceRenderData<T>);
}

// 'enc: lifetime of the encoder itself (and the mutable reference it holds to a RenderPass)
// 'pass: the lifetime of the RenderPass this encoder operates on, as well as vertex buffers and
//   other render state. This must be at least as long as 'enc.
pub struct InstanceEncoder<'enc, 'pass: 'enc> {
    buffer_view: wgpu::QueueWriteBufferView<'pass>,
    render_pass: &'enc mut RenderPass<'pass>,

    start_index: u32,
    count: u32,
    active_texture: Option<TextureRef>,
    active_mesh: Option<MeshRef>,
    active_pipeline: Option<PipelineRef>,
}

impl<'pass> Deref for InstanceEncoder<'_, 'pass> {
    type Target = RenderPass<'pass>;

    fn deref(&self) -> &Self::Target {
        &self.render_pass
    }
}

impl<'pass> DerefMut for InstanceEncoder<'_, 'pass> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.render_pass
    }
}

impl<'enc, 'pass: 'enc> InstanceEncoder<'enc, 'pass> {
    pub fn new(
        queue: &'pass wgpu::Queue,
        render_pass: &'enc mut RenderPass<'pass>,
        instance_buffer: &'pass wgpu::Buffer,
    ) -> Self {
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));

        let buffer_view = queue
            .write_buffer_with(
                instance_buffer,
                0,
                wgpu::BufferSize::new(instance_buffer.size()).unwrap(),
            )
            .unwrap(); // this should never fail, since we use the instance_buffer size and
                       // offset zero
        Self {
            buffer_view,
            render_pass,
            start_index: 0,
            count: 0,
            active_texture: None,
            active_mesh: None,
            active_pipeline: None,
        }
    }
}

impl<'enc, 'pass: 'enc> DrawInstance for InstanceEncoder<'enc, 'pass> {
    fn draw_instance<T: Transform>(&mut self, instance: &InstanceRenderData<T>) {
        // TODO: handling out of buffer space
        let dst = &mut bytemuck::cast_slice_mut(&mut self.buffer_view)
            [(self.start_index + self.count) as usize];
        if Some(instance.pipeline) != self.active_pipeline {
            if self.count > 0 {
                self.render_pass
                    .draw_active_mesh(self.start_index..self.start_index + self.count);
                self.start_index += self.count;
                self.count = 0;
            }
            self.render_pass.set_active_pipeline(instance.pipeline);
            self.active_pipeline = Some(instance.pipeline);
        }
        if Some(instance.mesh) != self.active_mesh {
            if self.count > 0 {
                self.render_pass
                    .draw_active_mesh(self.start_index..self.start_index + self.count);
                self.start_index += self.count;
                self.count = 0;
            }
            self.render_pass.set_active_mesh(instance.mesh);
            self.active_mesh = Some(instance.mesh);
        }
        if instance.texture != self.active_texture {
            if self.count > 0 {
                self.render_pass
                    .draw_active_mesh(self.start_index..self.start_index + self.count);
                self.start_index += self.count;
                self.count = 0;
            }
            self.render_pass.bind_texture(instance.texture);
            self.active_texture = instance.texture;
        }
        self.count += 1;
        *dst = instance.as_raw();
    }
}

impl Drop for InstanceEncoder<'_, '_> {
    fn drop(&mut self) {
        if self.count > 0 {
            self.render_pass
                .draw_active_mesh(self.start_index..self.start_index + self.count);
        }
    }
}
