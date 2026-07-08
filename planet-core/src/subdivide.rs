use crate::edge::EdgeCache;
use crate::mesh::{Mesh, MeshError, Triangle, Vertex};
use crate::subdivision_args::SubdivisionArgs;

pub(crate) trait SubdivisionStrategy {
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

pub fn subdivide(mesh: &Mesh, mut args: SubdivisionArgs) -> Result<Mesh, MeshError> {
    let mut strategy = args.mode.strategy();
    let mut current = mesh.clone();
    for step in 1..=args.steps.value() {
        current = split_round(&current, strategy.as_mut())?;
        if let Some(update_cb) = args.update_cb.as_mut() {
            update_cb(&current, step);
        }
    }
    Ok(current)
}
