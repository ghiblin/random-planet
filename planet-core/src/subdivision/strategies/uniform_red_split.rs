use rand::SeedableRng;
use rand_pcg::Pcg32;

use crate::geometry::vertex::Vertex;
use crate::processor::jitter::jitter;
use crate::processor::vertex_operator::VertexOperator;
use crate::subdivision::edge::EdgeCache;
use crate::subdivision::seed::Seed;
use crate::subdivision::subdivide::SubdivisionStrategy;

fn exact_midpoint(a: &Vertex, b: &Vertex) -> Vertex {
    Vertex::at(a.position.add(b.position).scale(0.5))
}

pub(crate) struct UniformRedSplit {
    rng: Pcg32,
    pipeline: VertexOperator,
}

impl UniformRedSplit {
    pub(crate) fn new(seed: Seed) -> UniformRedSplit {
        UniformRedSplit {
            rng: Pcg32::seed_from_u64(seed.value()),
            pipeline: jitter(),
        }
    }
}

impl SubdivisionStrategy for UniformRedSplit {
    fn split_triangle(
        &mut self,
        vertices: &mut Vec<Vertex>,
        edges: &mut EdgeCache,
        triangle: (usize, usize, usize),
    ) -> Vec<(usize, usize, usize)> {
        let (a, b, c) = triangle;
        let rng = &mut self.rng;
        let pipeline = &self.pipeline;
        let ab = edges.get_or_insert_with(a, b, vertices, |a, b| {
            pipeline(&mut *rng, a, b, exact_midpoint(a, b))
        });
        let bc = edges.get_or_insert_with(b, c, vertices, |a, b| {
            pipeline(&mut *rng, a, b, exact_midpoint(a, b))
        });
        let ca = edges.get_or_insert_with(c, a, vertices, |a, b| {
            pipeline(&mut *rng, a, b, exact_midpoint(a, b))
        });

        vec![(a, ab, ca), (b, bc, ab), (c, ca, bc), (ab, bc, ca)]
    }
}
