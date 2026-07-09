use rand::{RngExt, SeedableRng};
use rand_pcg::Pcg32;

use crate::geometry::mesh::{Triangle, Vertex};
use crate::subdivision::edge::EdgeCache;
use crate::subdivision::elevation_noise_range::ElevationNoiseRange;
use crate::subdivision::normal_noise_range::NormalNoiseRange;
use crate::subdivision::seed::Seed;
use crate::subdivision::subdivide::SubdivisionStrategy;

pub(crate) const MIN_VERTEX_RADIUS: f32 = 0.05;

fn displaced_midpoint(
    a: &Vertex,
    b: &Vertex,
    rng: &mut Pcg32,
    elevation_noise_range: ElevationNoiseRange,
    normal_noise_range: NormalNoiseRange,
) -> Vertex {
    let midpoint = a.position.add(b.position).scale(0.5);
    let radius = midpoint.length();
    if radius == 0.0 {
        return Vertex { position: midpoint };
    }
    let delta = rng.random_range(elevation_noise_range.low()..=elevation_noise_range.high());
    let new_radius = (radius + delta).max(MIN_VERTEX_RADIUS);
    let radial = midpoint.scale(new_radius / radius);
    let normal_delta = rng.random_range(normal_noise_range.low()..=normal_noise_range.high());
    match a.position.cross(b.position).normalized() {
        Some(normal) => Vertex {
            position: radial.add(normal.scale(normal_delta)),
        },
        None => Vertex { position: radial },
    }
}

pub(crate) struct RadialRandomSplit {
    rng: Pcg32,
    elevation_noise_range: ElevationNoiseRange,
    normal_noise_range: NormalNoiseRange,
}

impl RadialRandomSplit {
    pub(crate) fn new(
        seed: Seed,
        elevation_noise_range: ElevationNoiseRange,
        normal_noise_range: NormalNoiseRange,
    ) -> RadialRandomSplit {
        RadialRandomSplit {
            rng: Pcg32::seed_from_u64(seed.value()),
            elevation_noise_range,
            normal_noise_range,
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
        let elevation_noise_range = self.elevation_noise_range;
        let normal_noise_range = self.normal_noise_range;
        let ab = edges.get_or_insert_with(triangle.a, triangle.b, vertices, |a, b| {
            displaced_midpoint(
                a,
                b,
                &mut self.rng,
                elevation_noise_range,
                normal_noise_range,
            )
        });
        let bc = edges.get_or_insert_with(triangle.b, triangle.c, vertices, |a, b| {
            displaced_midpoint(
                a,
                b,
                &mut self.rng,
                elevation_noise_range,
                normal_noise_range,
            )
        });
        let ca = edges.get_or_insert_with(triangle.c, triangle.a, vertices, |a, b| {
            displaced_midpoint(
                a,
                b,
                &mut self.rng,
                elevation_noise_range,
                normal_noise_range,
            )
        });

        vec![
            Triangle::new(triangle.a, ab, ca),
            Triangle::new(triangle.b, bc, ab),
            Triangle::new(triangle.c, ca, bc),
            Triangle::new(ab, bc, ca),
        ]
    }
}
