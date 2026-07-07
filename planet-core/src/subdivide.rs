use crate::edge::EdgeCache;
use crate::mesh::{Mesh, MeshError, Triangle, Vertex};

pub trait SubdivisionStrategy {
    fn split_triangle(
        &mut self,
        vertices: &mut Vec<Vertex>,
        edges: &mut EdgeCache,
        triangle: Triangle,
    ) -> Vec<Triangle>;
}

fn split_round(mesh: &Mesh, strategy: &mut dyn SubdivisionStrategy) -> Result<Mesh, MeshError> {
    let mut vertices = mesh.vertices().to_vec();
    let mut edges = EdgeCache::new();
    let mut triangles = Vec::new();
    for triangle in mesh.triangles() {
        triangles.extend(strategy.split_triangle(&mut vertices, &mut edges, *triangle));
    }
    Mesh::new(vertices, triangles)
}

pub fn subdivide(
    mesh: &Mesh,
    depth: u32,
    strategy: &mut dyn SubdivisionStrategy,
) -> Result<Mesh, MeshError> {
    let mut current = mesh.clone();
    for _ in 0..depth {
        current = split_round(&current, strategy)?;
    }
    Ok(current)
}
