use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use wgpu::{util::DeviceExt, BufferUsages};

use crate::geom::{
    cube_with_normals, quad, ModelVertexData, ModelVertexDataWithNormal, VertexData,
};

slotmap::new_key_type! {
    pub struct RawMeshRef;
}

#[derive(Debug, Default, Copy, Clone)]
pub struct MeshRef<V> {
    raw: RawMeshRef,
    _marker: PhantomData<V>,
}

impl<V> MeshRef<V> {
    pub fn raw(&self) -> RawMeshRef {
        self.raw
    }
}

impl<V> From<RawMeshRef> for MeshRef<V> {
    fn from(raw: RawMeshRef) -> Self {
        MeshRef {
            raw,
            _marker: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct Mesh<V = ModelVertexData> {
    pub inner: UntypedMesh,
    _marker: PhantomData<V>,
}

impl<V> Deref for Mesh<V> {
    type Target = UntypedMesh;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<V> DerefMut for Mesh<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Debug)]
pub struct UntypedMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

pub trait LoadMesh {
    type Error: std::fmt::Debug;

    fn load_mesh<V: VertexData>(
        &self,
        verts: &[V],
        indices: &[u16],
    ) -> Result<Mesh<V>, Self::Error>;

    fn load_quad_mesh(&self) -> Mesh {
        self.load_mesh(
            &quad::verts(0., 0., 1., 1., (0., 0.), (1., 1.)),
            &quad::INDICES,
        )
        .unwrap()
    }

    fn load_cube_mesh(&self) -> Mesh<ModelVertexDataWithNormal> {
        self.load_mesh(&cube_with_normals::VERTICES, &cube_with_normals::INDICES)
            .unwrap()
    }
}

impl LoadMesh for wgpu::Device {
    type Error = ();
    fn load_mesh<V: VertexData>(
        &self,
        verts: &[V],
        indices: &[u16],
    ) -> Result<Mesh<V>, Self::Error> {
        Ok(Mesh {
            inner: UntypedMesh {
                vertex_buffer: self.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(verts),
                    usage: BufferUsages::VERTEX,
                }),
                index_buffer: self.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(indices),
                    usage: BufferUsages::INDEX,
                }),
                num_indices: indices.len() as _,
            },
            _marker: PhantomData,
        })
    }
}
