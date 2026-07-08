use rand::{RngExt, SeedableRng};
use rand_pcg::Pcg32;

use crate::geometry::mesh::{Triangle, Vertex};
use crate::subdivision::edge::EdgeCache;
use crate::subdivision::elevation_noise_range::ElevationNoiseRange;
use crate::subdivision::seed::Seed;
use crate::subdivision::subdivide::SubdivisionStrategy;

pub(crate) const MIN_VERTEX_RADIUS: f32 = 0.05;

fn displaced_midpoint(
    a: &Vertex,
    b: &Vertex,
    rng: &mut Pcg32,
    range: ElevationNoiseRange,
) -> Vertex {
    let midpoint = a.position.add(b.position).scale(0.5);
    let radius = midpoint.length();
    if radius == 0.0 {
        return Vertex { position: midpoint };
    }
    let delta = rng.random_range(range.low()..=range.high());
    let new_radius = (radius + delta).max(MIN_VERTEX_RADIUS);
    Vertex {
        position: midpoint.scale(new_radius / radius),
    }
}

pub(crate) struct RadialRandomSplit {
    rng: Pcg32,
    elevation_noise_range: ElevationNoiseRange,
}

impl RadialRandomSplit {
    pub(crate) fn new(seed: Seed, elevation_noise_range: ElevationNoiseRange) -> RadialRandomSplit {
        RadialRandomSplit {
            rng: Pcg32::seed_from_u64(seed.value()),
            elevation_noise_range,
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
        let range = self.elevation_noise_range;
        let ab = edges.get_or_insert_with(triangle.a, triangle.b, vertices, |a, b| {
            displaced_midpoint(a, b, &mut self.rng, range)
        });
        let bc = edges.get_or_insert_with(triangle.b, triangle.c, vertices, |a, b| {
            displaced_midpoint(a, b, &mut self.rng, range)
        });
        let ca = edges.get_or_insert_with(triangle.c, triangle.a, vertices, |a, b| {
            displaced_midpoint(a, b, &mut self.rng, range)
        });

        vec![
            Triangle::new(triangle.a, ab, ca),
            Triangle::new(triangle.b, bc, ab),
            Triangle::new(triangle.c, ca, bc),
            Triangle::new(ab, bc, ca),
        ]
    }
}
