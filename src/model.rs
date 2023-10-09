use crate::geom::VertexData;
use crate::{geom::ModelVertexData, mesh::Mesh};

#[derive(Clone, Default, Debug)]
pub struct ModelMesh<V = ModelVertexData, I = u16>
where
    V: VertexData,
{
    mesh: Mesh<V, I>,
    material: Option<tobj::Material>,
}

#[derive(Clone, Default, Debug)]
pub struct Model<V = ModelVertexData, I = u16>
where
    V: VertexData,
{
    meshes: Vec<ModelMesh<V, I>>,
}

// TODO
// impl<T> Model<T> {
impl Model {
    pub fn load(path: impl AsRef<str>) -> anyhow::Result<Model> {
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
                .map(|(pos, uv)| ModelVertexData { pos, uv })
                .collect();
            let indices = raw_mesh.indices.iter().map(|i| *i as u16).collect();
            let mesh = Mesh { vertices, indices };
            meshes.push(ModelMesh { mesh, material })
        }

        Ok(Model { meshes })
    }

    pub fn to_single_mesh(self) -> Mesh {
        self.meshes
            .into_iter()
            .map(|ModelMesh { mesh, .. }| mesh)
            .reduce(|mut m, n| m.merge(n))
            .unwrap_or_default()
    }
}
