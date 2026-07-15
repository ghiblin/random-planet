use super::edge::EdgeCache;
use super::subdivision_args::SubdivisionArgs;
use crate::geometry::mesh::{Mesh, MeshError, Triangle, Vertex};

pub(crate) trait SubdivisionStrategy {
    fn split_triangle(
        &mut self,
        vertices: &mut Vec<Vertex>,
        edges: &mut EdgeCache,
        triangle: Triangle,
    ) -> Vec<Triangle>;
}

// A triangle produces at most 4 children (red split) and at most one new vertex
// per edge (3), so these are safe upper bounds for any SubdivisionStrategy.
fn max_new_vertices(triangle_count: usize) -> usize {
    3 * triangle_count
}

fn max_round_triangles(triangle_count: usize) -> usize {
    4 * triangle_count
}

fn split_round(mesh: &Mesh, strategy: &mut dyn SubdivisionStrategy) -> Result<Mesh, MeshError> {
    let triangle_count = mesh.triangles().len();
    let mut vertices = mesh.vertices().to_vec();
    vertices.reserve(max_new_vertices(triangle_count));
    let mut edges = EdgeCache::new();
    let mut triangles = Vec::with_capacity(max_round_triangles(triangle_count));
    for triangle in mesh.triangles() {
        triangles.extend(strategy.split_triangle(&mut vertices, &mut edges, *triangle));
    }
    Mesh::new(vertices, triangles)
}

pub fn subdivide(mesh: &Mesh, mut args: SubdivisionArgs) -> Result<Mesh, MeshError> {
    let mut strategy = args.mode.strategy(args.seed);
    let mut current = mesh.clone();
    for step in 1..=args.steps.value() {
        current = split_round(&current, strategy.as_mut())?;
        if let Some(update_cb) = args.update_cb.as_mut() {
            update_cb(&current, step);
        }
    }
    Ok(current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_new_vertices_is_3_per_triangle() {
        assert_eq!(max_new_vertices(5), 15);
    }

    #[test]
    fn max_round_triangles_is_4_per_triangle() {
        assert_eq!(max_round_triangles(5), 20);
    }
}
