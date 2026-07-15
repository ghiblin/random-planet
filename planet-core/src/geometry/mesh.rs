use super::edge::Edge;
use super::face::Face;
use super::primitives::{cube::cube, icosahedron::icosahedron};
use super::vec3::Vec3;
use super::vertex::Vertex;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum MeshError {
    VertexIndexOutOfBounds { index: usize, vertex_count: usize },
    NegativeCubeSide { side: f32 },
}

impl fmt::Display for MeshError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MeshError::VertexIndexOutOfBounds {
                index,
                vertex_count,
            } => write!(
                f,
                "vertex index {index} is out of bounds for {vertex_count} vertices"
            ),
            MeshError::NegativeCubeSide { side } => {
                write!(f, "cube side must not be negative, got {side}")
            }
        }
    }
}

impl std::error::Error for MeshError {}

#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
    faces: Vec<Face>,
}

impl Mesh {
    pub fn new(
        positions: Vec<Vec3>,
        triangles: Vec<(usize, usize, usize)>,
    ) -> Result<Mesh, MeshError> {
        let vertex_count = positions.len();
        for &(a, b, c) in &triangles {
            for index in [a, b, c] {
                if index >= vertex_count {
                    return Err(MeshError::VertexIndexOutOfBounds {
                        index,
                        vertex_count,
                    });
                }
            }
        }

        let mut vertices: Vec<Vertex> = positions.into_iter().map(Vertex::at).collect();
        let mut edges = Vec::with_capacity(triangles.len() * 3);
        let mut faces = Vec::with_capacity(triangles.len());

        for &(a, b, c) in &triangles {
            let face_index = faces.len();
            let edge_base = edges.len();
            edges.push(Edge {
                start: a,
                end: b,
                face: face_index,
            });
            edges.push(Edge {
                start: b,
                end: c,
                face: face_index,
            });
            edges.push(Edge {
                start: c,
                end: a,
                face: face_index,
            });
            faces.push(Face {
                edges: vec![edge_base, edge_base + 1, edge_base + 2],
                order: 3,
                normal: Vec3::new(0.0, 0.0, 0.0),
            });
        }

        for (edge_index, edge) in edges.iter().enumerate() {
            vertices[edge.start].edges.push(edge_index);
        }

        Ok(Mesh {
            vertices,
            edges,
            faces,
        })
    }

    /// Same topology (`edges`, `faces`, and each vertex's own `edges`/`normal`) as
    /// `self`, with every vertex's `position` replaced positionally from
    /// `positions`. Used by position-only whole-mesh transforms (terrain noise,
    /// ocean quota, vertex scramble) that never change the mesh's connectivity.
    pub(crate) fn with_repositioned(&self, positions: Vec<Vec3>) -> Mesh {
        let vertices = self
            .vertices
            .iter()
            .zip(positions)
            .map(|(vertex, position)| Vertex {
                position,
                normal: vertex.normal,
                edges: vertex.edges.clone(),
            })
            .collect();
        Mesh {
            vertices,
            edges: self.edges.clone(),
            faces: self.faces.clone(),
        }
    }

    /// Rebuilds a `Mesh` from already-computed parts, with no validation — used by
    /// whole-mesh transforms (e.g. `finalize_normals`) that only ever recompute
    /// derived per-vertex/per-face data (`normal`) on an already-valid `Mesh`,
    /// never its topology (`edges`, `Face.edges`/`order`, `Vertex.edges`).
    pub(crate) fn from_parts(vertices: Vec<Vertex>, edges: Vec<Edge>, faces: Vec<Face>) -> Mesh {
        Mesh {
            vertices,
            edges,
            faces,
        }
    }

    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    pub fn faces(&self) -> &[Face] {
        &self.faces
    }

    pub fn icosahedron() -> Result<Mesh, MeshError> {
        icosahedron()
    }

    pub fn cube(side: f32) -> Result<Mesh, MeshError> {
        cube(side)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_repositioned_preserves_normal_and_edges() {
        let base = Mesh::new(vec![Vec3::new(0.0, 0.0, 0.0)], vec![]).expect("valid mesh fixture");
        let distinguishable_normal = Vec3::new(0.0, 1.0, 0.0);
        let vertex = Vertex {
            position: base.vertices()[0].position,
            normal: distinguishable_normal,
            edges: vec![7, 8, 9],
        };
        let mesh = Mesh::from_parts(vec![vertex], base.edges().to_vec(), base.faces().to_vec());

        let repositioned = mesh.with_repositioned(vec![Vec3::new(5.0, 5.0, 5.0)]);

        assert_eq!(
            repositioned.vertices()[0].position,
            Vec3::new(5.0, 5.0, 5.0)
        );
        assert_eq!(repositioned.vertices()[0].normal, distinguishable_normal);
        assert_eq!(repositioned.vertices()[0].edges, vec![7, 8, 9]);
    }
}
