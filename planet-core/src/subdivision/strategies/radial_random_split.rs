use rand::SeedableRng;
use rand_pcg::Pcg32;

use crate::geometry::mesh::{Triangle, Vertex};
use crate::processor::compose::compose;
use crate::processor::normal_displacement::normal_displacement;
use crate::processor::radial_displacement::radial_displacement;
use crate::processor::vertex_operator::VertexOperator;
use crate::subdivision::edge::EdgeCache;
use crate::subdivision::elevation_noise_range::ElevationNoiseRange;
use crate::subdivision::normal_noise_range::NormalNoiseRange;
use crate::subdivision::seed::Seed;
use crate::subdivision::subdivide::SubdivisionStrategy;

fn exact_midpoint(a: &Vertex, b: &Vertex) -> Vertex {
    Vertex {
        position: a.position.add(b.position).scale(0.5),
    }
}

pub(crate) struct RadialRandomSplit {
    rng: Pcg32,
    pipeline: VertexOperator,
}

impl RadialRandomSplit {
    pub(crate) fn new(
        seed: Seed,
        elevation_noise_range: ElevationNoiseRange,
        normal_noise_range: NormalNoiseRange,
    ) -> RadialRandomSplit {
        RadialRandomSplit {
            rng: Pcg32::seed_from_u64(seed.value()),
            pipeline: compose(
                radial_displacement(elevation_noise_range),
                normal_displacement(normal_noise_range),
            ),
        }
    }
}

impl SubdivisionStrategy for RadialRandomSplit {
    fn split_triangle(
        &mut self,
        vertices: &mut Vec<Vertex>,
        edges: &mut EdgeCache,
        triangle: Triangle,
    ) -> Vec<Triangle> {
        let ab = edges.get_or_insert_with(triangle.a, triangle.b, vertices, |a, b| {
            (self.pipeline)(&mut self.rng, a, b, exact_midpoint(a, b))
        });
        let bc = edges.get_or_insert_with(triangle.b, triangle.c, vertices, |a, b| {
            (self.pipeline)(&mut self.rng, a, b, exact_midpoint(a, b))
        });
        let ca = edges.get_or_insert_with(triangle.c, triangle.a, vertices, |a, b| {
            (self.pipeline)(&mut self.rng, a, b, exact_midpoint(a, b))
        });

        vec![
            Triangle::new(triangle.a, ab, ca),
            Triangle::new(triangle.b, bc, ab),
            Triangle::new(triangle.c, ca, bc),
            Triangle::new(ab, bc, ca),
        ]
    }
}
