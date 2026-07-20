use super::edge::EdgeCache;
use super::subdivision_args::SubdivisionArgs;
use crate::geometry::mesh::{Mesh, MeshError};
use crate::geometry::vertex::Vertex;

pub(crate) trait SubdivisionStrategy {
    fn split_triangle(
        &mut self,
        vertices: &mut Vec<Vertex>,
        edges: &mut EdgeCache,
        triangle: (usize, usize, usize),
    ) -> Vec<(usize, usize, usize)>;
}

// A triangle produces at most 4 children (red split) and at most one new vertex
// per edge (3), so these are safe upper bounds for any SubdivisionStrategy.
fn max_new_vertices(triangle_count: usize) -> usize {
    3 * triangle_count
}

fn max_round_triangles(triangle_count: usize) -> usize {
    4 * triangle_count
}

fn face_triangle(mesh: &Mesh, face_index: usize) -> (usize, usize, usize) {
    let face = &mesh.faces()[face_index];
    let corners: Vec<usize> = face
        .edges
        .iter()
        .map(|&edge_index| mesh.edges()[edge_index].start)
        .collect();
    (corners[0], corners[1], corners[2])
}

fn split_round(mesh: &Mesh, strategy: &mut dyn SubdivisionStrategy) -> Result<Mesh, MeshError> {
    let triangle_count = mesh.faces().len();
    let mut vertices: Vec<Vertex> = mesh.vertices().to_vec();
    vertices.reserve(max_new_vertices(triangle_count));
    let mut edges = EdgeCache::new();
    let mut triangles = Vec::with_capacity(max_round_triangles(triangle_count));
    for face_index in 0..mesh.faces().len() {
        let triangle = face_triangle(mesh, face_index);
        triangles.extend(strategy.split_triangle(&mut vertices, &mut edges, triangle));
    }
    let positions = vertices.into_iter().map(|vertex| vertex.position).collect();
    Mesh::new(positions, triangles)
}

pub fn subdivide(mesh: &Mesh, mut args: SubdivisionArgs) -> Result<Mesh, MeshError> {
    let mut strategy = args.mode.strategy(args.seed);
    let mut current = mesh.clone();
    for step in 1..=args.steps.value() {
        current = split_round(&current, strategy.as_mut())?;
        if let Some(update_cb) = args.update_cb.as_mut() {
            current = update_cb(current, step)?;
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
