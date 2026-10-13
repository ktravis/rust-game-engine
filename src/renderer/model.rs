use crate::geom::ModelVertexData;

#[derive(Clone, Debug, Default)]
pub struct MeshData {
    pub vertices: Vec<ModelVertexData>,
    pub indices: Vec<u16>,
}

#[derive(Clone, Debug, Default)]
pub struct ModelMesh {
    pub mesh_data: MeshData,
    pub material: Option<tobj::Material>,
}

#[derive(Debug)]
pub struct Model {
    pub meshes: Vec<ModelMesh>,
}

impl Model {
    pub fn load(path: impl AsRef<str>) -> Result<Self, tobj::LoadError> {
        let (models, mtls) = tobj::load_obj(path.as_ref(), &tobj::GPU_LOAD_OPTIONS)?;
        let mtls = mtls?;
        let mut meshes = vec![];
        for m in models {
            let raw_mesh = m.mesh;
            let mut model_mesh = ModelMesh::default();
            model_mesh.material = raw_mesh.material_id.map(|id| mtls[id].clone());
            model_mesh.mesh_data.vertices = raw_mesh
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
                .zip(
                    raw_mesh
                        .normals
                        .iter()
                        .enumerate()
                        .step_by(3)
                        .map(|(i, _)| {
                            glam::vec3(
                                raw_mesh.normals[i],
                                raw_mesh.normals[i + 1],
                                raw_mesh.normals[i + 2],
                            )
                        }),
                )
                .map(|((pos, uv), normal)| ModelVertexData {
                    position: pos.into(),
                    tex_coords: uv.into(),
                    normal: normal.into(),
                    ..Default::default()
                })
                .collect::<Vec<_>>();
            model_mesh.mesh_data.indices = raw_mesh
                .indices
                // rewind faces to Ccw
                .chunks_exact(3)
                .map(|x| [x[0] as u16, x[2] as u16, x[1] as u16])
                .flatten()
                .collect::<Vec<_>>();
            meshes.push(model_mesh);
        }

        Ok(Model { meshes })
    }
}
