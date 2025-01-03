use std::ops::Deref;

use crate::geom::BasicVertexData;

use super::{Display, InstanceData, MeshRef, PipelineRef, TextureRef};

use super::BasicInstanceData;

#[derive(Debug)]
pub struct InstanceRenderData<V = BasicVertexData, I = BasicInstanceData> {
    pub mesh: MeshRef<V>,
    pub instance: I,
    pub texture: Option<TextureRef>,
    pub pipeline: Option<PipelineRef<V, I>>,
}

impl<V, I> Deref for InstanceRenderData<V, I> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

pub struct InstanceStorage {
    raw_instance_bytes: Vec<u8>,
    instance_buffer: Option<wgpu::Buffer>,
    destroy_list: Vec<wgpu::Buffer>,
}

impl InstanceStorage {
    pub fn new(display: &Display, initial_size: usize) -> Self {
        let instance_buffer = display.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance storage"),
            size: (initial_size * std::mem::size_of::<BasicInstanceData>()) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });
        Self {
            raw_instance_bytes: vec![],
            instance_buffer: Some(instance_buffer),
            destroy_list: vec![],
        }
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        self.instance_buffer.as_ref().unwrap()
    }

    pub fn add<I: InstanceData>(&mut self, instance: &I) {
        self.raw_instance_bytes
            .extend_from_slice(bytemuck::bytes_of(instance));
    }

    pub fn clear(&mut self) {
        self.raw_instance_bytes.clear();
        for b in self.destroy_list.drain(..) {
            b.destroy();
        }
    }

    pub fn update_buffer(&mut self, display: &Display) {
        let req_size = self.raw_instance_bytes.len() as wgpu::BufferAddress;
        if self.buffer().size() < req_size {
            self.destroy_list.push(self.instance_buffer.take().unwrap());
            self.instance_buffer = Some(display.device().create_buffer(&wgpu::BufferDescriptor {
                label: Some("instance storage (resized)"),
                size: req_size * 2,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            }));
        }
        let mut w = display
            .queue()
            .write_buffer_with(
                self.buffer(),
                0,
                std::num::NonZeroU64::new(req_size).unwrap(),
            )
            .unwrap();
        w.copy_from_slice(&self.raw_instance_bytes);
    }
}

// TODO: next parametrize this type with the vertex/instance data types
//
// pub struct Batch<'r> {
//     instance_storage: &'r mut InstanceStorage,
//     active_mesh: Option<RawMeshRef>,
//     active_texture: Option<TextureRef>,
//     active_pipeline: Option<RawPipelineRef>,
// }
