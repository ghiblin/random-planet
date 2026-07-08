use crate::geometry::mesh::{Triangle, Vertex};
use crate::subdivision::edge::EdgeCache;
use crate::subdivision::subdivide::SubdivisionStrategy;

fn exact_midpoint(a: &Vertex, b: &Vertex) -> Vertex {
    Vertex {
        position: a.position.add(b.position).scale(0.5),
    }
}

pub(crate) struct UniformRedSplit;

impl SubdivisionStrategy for UniformRedSplit {
    fn split_triangle(
        &mut self,
        vertices: &mut Vec<Vertex>,
        edges: &mut EdgeCache,
        triangle: Triangle,
    ) -> Vec<Triangle> {
        let ab = edges.get_or_insert_with(triangle.a, triangle.b, vertices, exact_midpoint);
        let bc = edges.get_or_insert_with(triangle.b, triangle.c, vertices, exact_midpoint);
        let ca = edges.get_or_insert_with(triangle.c, triangle.a, vertices, exact_midpoint);

        vec![
            Triangle::new(triangle.a, ab, ca),
            Triangle::new(triangle.b, bc, ab),
            Triangle::new(triangle.c, ca, bc),
            Triangle::new(ab, bc, ca),
        ]
    }
}
