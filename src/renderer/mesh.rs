use std::marker::PhantomData;

use wgpu::{util::DeviceExt, BufferUsages};

use crate::geom::{cube, quad, ModelVertexData, VertexData};

#[derive(Debug)]
pub struct Mesh<V = ModelVertexData> {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    _marker: PhantomData<V>,
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

    fn load_cube_mesh(&self) -> Mesh {
        self.load_mesh(&cube::VERTICES, &cube::INDICES).unwrap()
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
            _marker: PhantomData,
        })
    }
}
