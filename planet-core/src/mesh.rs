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
}
