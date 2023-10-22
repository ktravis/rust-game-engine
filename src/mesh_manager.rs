use crate::{geom::ModelVertexData, mesh::Mesh};
use std::cell::Cell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::geom::VertexData;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MeshRef(usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MeshOffsets {
    pub offset: u16,
    pub count: u16,
}

#[derive(Default)]
pub struct MeshManager<V = ModelVertexData>
where
    V: VertexData,
{
    mesh_refs: Vec<(String, MeshOffsets)>,
    meshes_by_name: HashMap<String, Rc<Cell<MeshOffsets>>>,
    vertices: Vec<V>,
    indices: Vec<u16>,
    pub needs_rebuild: bool,
}

// TODO: *Model*Manager, manage meshes and materials? generate bindings

impl<V: VertexData> MeshManager<V> {
    pub fn add(&mut self, name: impl Into<String>, mesh: Mesh<V>) -> Rc<Cell<MeshOffsets>> {
        let mesh_offsets = MeshOffsets {
            offset: self.indices.len() as _,
            count: mesh.indices.len() as _,
        };
        let vertex_offset: u16 = self.vertices.len() as _;
        self.vertices.extend(mesh.vertices.iter().cloned());
        self.indices
            .extend(mesh.indices.iter().map(|i| i + vertex_offset));
        let mesh_offsets = Rc::new(Cell::new(mesh_offsets));
        self.meshes_by_name
            .insert(name.into(), mesh_offsets.clone());
        self.needs_rebuild = true;
        mesh_offsets
    }

    pub fn get(&self, name: impl AsRef<str>) -> Option<Rc<Cell<MeshOffsets>>> {
        let name = name.as_ref();
        self.meshes_by_name.get(name).map(Clone::clone)
    }

    pub fn buffers(&mut self, ctx: &mut miniquad::GraphicsContext) -> GeometryBuffers<V> {
        self.needs_rebuild = false;
        GeometryBuffers::from_slices(ctx, &self.vertices, &self.indices)
    }
}

#[derive(Clone)]
pub struct GeometryBuffers<V: VertexData> {
    pub vertices: miniquad::Buffer,
    pub indices: miniquad::Buffer,
    _marker: PhantomData<V>,
}

impl<V: VertexData> GeometryBuffers<V> {
    pub fn from_meshes(ctx: &mut miniquad::GraphicsContext, meshes: &[Mesh<V>]) -> Self {
        let (vertices, indices): (Vec<V>, Vec<u16>) =
            meshes
                .iter()
                .fold((vec![], vec![]), |(mut verts, mut inds), m| {
                    verts.extend(m.vertices.iter());
                    inds.extend(m.indices.iter());
                    (verts, inds)
                });
        Self::from_slices(ctx, &vertices, &indices)
    }

    pub fn from_slices(
        ctx: &mut miniquad::GraphicsContext,
        vertices: &[V],
        indices: &[u16],
    ) -> Self {
        use miniquad::{Buffer, BufferType};
        GeometryBuffers {
            vertices: Buffer::immutable(ctx, BufferType::VertexBuffer, vertices),
            indices: Buffer::immutable(ctx, BufferType::IndexBuffer, indices),
            _marker: PhantomData,
        }
    }
}

impl<V: VertexData> Drop for GeometryBuffers<V> {
    fn drop(&mut self) {
        self.vertices.delete();
        self.indices.delete();
    }
}
