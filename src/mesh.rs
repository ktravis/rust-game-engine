use crate::geom::{ModelVertexData, VertexData};

#[derive(Clone, Default, Debug)]
pub struct Mesh<V = ModelVertexData, I = u16>
where
    V: VertexData,
{
    pub vertices: Vec<V>,
    pub indices: Vec<I>,
}

impl<V: VertexData, I, E> Mesh<V, I>
where
    I: std::ops::Add<I, Output = I> + std::convert::TryFrom<usize, Error = E>,
    E: std::fmt::Debug,
{
    // TODO try_from is ew here, use num_traits or something
    pub fn merge(mut self, Mesh { vertices, indices }: Mesh<V, I>) -> Self {
        self.indices.extend(
            indices
                .into_iter()
                .map(|i| I::try_from(self.vertices.len()).unwrap() + i),
        );
        self.vertices.extend(vertices.into_iter());
        self
    }
}
