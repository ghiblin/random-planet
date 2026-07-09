use rand::SeedableRng;
use rand_pcg::Pcg32;

use crate::geometry::mesh::{Triangle, Vertex};
use crate::processor::identity::identity;
use crate::processor::vertex_operator::VertexOperator;
use crate::subdivision::edge::EdgeCache;
use crate::subdivision::subdivide::SubdivisionStrategy;

fn exact_midpoint(a: &Vertex, b: &Vertex) -> Vertex {
    Vertex {
        position: a.position.add(b.position).scale(0.5),
    }
}

pub(crate) struct UniformRedSplit {
    pipeline: VertexOperator,
}

impl UniformRedSplit {
    pub(crate) fn new() -> UniformRedSplit {
        UniformRedSplit {
            pipeline: identity(),
        }
    }
}

impl SubdivisionStrategy for UniformRedSplit {
    fn split_triangle(
        &mut self,
        vertices: &mut Vec<Vertex>,
        edges: &mut EdgeCache,
        triangle: Triangle,
    ) -> Vec<Triangle> {
        // Unused by `identity`; only present to satisfy VertexOperator's shared call signature.
        let mut rng = Pcg32::seed_from_u64(0);
        let ab = edges.get_or_insert_with(triangle.a, triangle.b, vertices, |a, b| {
            (self.pipeline)(&mut rng, a, b, exact_midpoint(a, b))
        });
        let bc = edges.get_or_insert_with(triangle.b, triangle.c, vertices, |a, b| {
            (self.pipeline)(&mut rng, a, b, exact_midpoint(a, b))
        });
        let ca = edges.get_or_insert_with(triangle.c, triangle.a, vertices, |a, b| {
            (self.pipeline)(&mut rng, a, b, exact_midpoint(a, b))
        });

        vec![
            Triangle::new(triangle.a, ab, ca),
            Triangle::new(triangle.b, bc, ab),
            Triangle::new(triangle.c, ca, bc),
            Triangle::new(ab, bc, ca),
        ]
    }
}
