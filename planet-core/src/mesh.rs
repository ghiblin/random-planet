use crate::vec3::Vec3;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    pub position: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Triangle {
    pub a: usize,
    pub b: usize,
    pub c: usize,
}

impl Triangle {
    pub fn new(a: usize, b: usize, c: usize) -> Triangle {
        Triangle { a, b, c }
    }
}

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
    triangles: Vec<Triangle>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, triangles: Vec<Triangle>) -> Result<Mesh, MeshError> {
        let vertex_count = vertices.len();
        for triangle in &triangles {
            for index in [triangle.a, triangle.b, triangle.c] {
                if index >= vertex_count {
                    return Err(MeshError::VertexIndexOutOfBounds {
                        index,
                        vertex_count,
                    });
                }
            }
        }
        Ok(Mesh {
            vertices,
            triangles,
        })
    }

    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    pub fn triangles(&self) -> &[Triangle] {
        &self.triangles
    }

    pub fn cube(side: f32) -> Result<Mesh, MeshError> {
        if side < 0.0 {
            return Err(MeshError::NegativeCubeSide { side });
        }

        let half = side / 2.0;
        let vertices = [
            (-half, -half, -half),
            (half, -half, -half),
            (half, half, -half),
            (-half, half, -half),
            (-half, -half, half),
            (half, -half, half),
            (half, half, half),
            (-half, half, half),
        ]
        .into_iter()
        .map(|(x, y, z)| Vertex {
            position: Vec3::new(x, y, z),
        })
        .collect();

        let triangles = [
            // -Z
            (0, 2, 1),
            (0, 3, 2),
            // +Z
            (4, 5, 6),
            (4, 6, 7),
            // -Y
            (0, 1, 5),
            (0, 5, 4),
            // +Y
            (3, 6, 2),
            (3, 7, 6),
            // -X
            (0, 4, 7),
            (0, 7, 3),
            // +X
            (1, 2, 6),
            (1, 6, 5),
        ]
        .into_iter()
        .map(|(a, b, c)| Triangle::new(a, b, c))
        .collect();

        Mesh::new(vertices, triangles)
    }
}
