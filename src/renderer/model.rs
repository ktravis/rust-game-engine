use crate::geom::ModelVertexData;
use crate::renderer::mesh::Mesh;

use super::mesh::LoadMesh;

#[derive(Debug)]
pub struct ModelMesh {
    mesh: Mesh<ModelVertexData>,
    material: Option<tobj::Material>,
}

#[derive(Debug)]
pub struct Model {
    meshes: Vec<ModelMesh>,
}

pub trait LoadModel {
    type Error: std::fmt::Debug;

    fn load_model(&self, path: impl AsRef<str>) -> Result<Model, Self::Error>;
}

impl LoadModel for wgpu::Device {
    type Error = tobj::LoadError;

    fn load_model(&self, path: impl AsRef<str>) -> Result<Model, Self::Error> {
        let (models, mtls) = tobj::load_obj(path.as_ref(), &tobj::GPU_LOAD_OPTIONS)?;
        let mtls = mtls?;
        let mut meshes = vec![];
        for m in models {
            let raw_mesh = m.mesh;
            let material = raw_mesh.material_id.map(|id| mtls[id].clone());
            let vertices = raw_mesh
                .positions
                .iter()
                .enumerate()
                .step_by(3)
                .map(|(i, _)| {
                    glam::vec4(
                        raw_mesh.positions[i],
                        raw_mesh.positions[i + 1],
                        raw_mesh.positions[i + 2],
                        1.,
                    )
                })
                .zip(
                    raw_mesh
                        .texcoords
                        .iter()
                        .enumerate()
                        .step_by(2)
                        .map(|(i, _)| glam::vec2(raw_mesh.texcoords[i], raw_mesh.texcoords[i + 1])),
                )
                .map(|(pos, uv)| ModelVertexData {
                    pos: pos.into(),
                    uv: uv.into(),
                })
                .collect::<Vec<_>>();
            let indices = raw_mesh
                .indices
                .iter()
                .map(|i| *i as u16)
                .collect::<Vec<_>>();
            let mesh = self.load_mesh(&vertices, &indices).unwrap();
            meshes.push(ModelMesh { mesh, material })
        }

        Ok(Model { meshes })
    }
}
