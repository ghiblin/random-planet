use rand::{RngExt, SeedableRng};
use rand_distr::Normal;
use rand_pcg::Pcg32;

use crate::geometry::mesh::{Triangle, Vertex};
use crate::subdivision::edge::EdgeCache;
use crate::subdivision::elevation_noise_range::ElevationNoiseRange;
use crate::subdivision::min_edge_length::MinEdgeLength;
use crate::subdivision::seed::Seed;
use crate::subdivision::split_point_variance::SplitPointVariance;
use crate::subdivision::subdivide::SubdivisionStrategy;

pub(crate) const MIN_SPLIT_T: f32 = 0.05;
pub(crate) const MAX_SPLIT_T: f32 = 0.95;
const MIN_VERTEX_RADIUS: f32 = 0.05;

fn displaced_split_point(
    a: &Vertex,
    b: &Vertex,
    rng: &mut Pcg32,
    split_point_variance: SplitPointVariance,
    elevation_noise_range: ElevationNoiseRange,
) -> Vertex {
    let normal = Normal::new(0.5, split_point_variance.value())
        .expect("SplitPointVariance guarantees a non-negative standard deviation");
    let t = rng.sample(normal).clamp(MIN_SPLIT_T, MAX_SPLIT_T);
    let point = a.position.add(b.position.sub(a.position).scale(t));
    let radius = point.length();
    if radius == 0.0 {
        return Vertex { position: point };
    }
    let delta = rng.random_range(elevation_noise_range.low()..=elevation_noise_range.high());
    let new_radius = (radius + delta).max(MIN_VERTEX_RADIUS);
    Vertex {
        position: point.scale(new_radius / radius),
    }
}

#[allow(clippy::too_many_arguments)]
fn maybe_split(
    a: usize,
    b: usize,
    vertices: &mut Vec<Vertex>,
    edges: &mut EdgeCache,
    rng: &mut Pcg32,
    min_edge_length: MinEdgeLength,
    split_point_variance: SplitPointVariance,
    elevation_noise_range: ElevationNoiseRange,
) -> Option<usize> {
    let length = vertices[b].position.sub(vertices[a].position).length();
    if length < min_edge_length.value() {
        return None;
    }
    Some(edges.get_or_insert_with(a, b, vertices, |va, vb| {
        displaced_split_point(va, vb, rng, split_point_variance, elevation_noise_range)
    }))
}

pub(crate) struct RedGreenSplit {
    rng: Pcg32,
    elevation_noise_range: ElevationNoiseRange,
    min_edge_length: MinEdgeLength,
    split_point_variance: SplitPointVariance,
}

impl RedGreenSplit {
    pub(crate) fn new(
        seed: Seed,
        elevation_noise_range: ElevationNoiseRange,
        min_edge_length: MinEdgeLength,
        split_point_variance: SplitPointVariance,
    ) -> RedGreenSplit {
        RedGreenSplit {
            rng: Pcg32::seed_from_u64(seed.value()),
            elevation_noise_range,
            min_edge_length,
            split_point_variance,
        }
    }
}

impl SubdivisionStrategy for RedGreenSplit {
    fn split_triangle(
        &mut self,
        vertices: &mut Vec<Vertex>,
        edges: &mut EdgeCache,
        triangle: Triangle,
    ) -> Vec<Triangle> {
        let min_edge_length = self.min_edge_length;
        let split_point_variance = self.split_point_variance;
        let elevation_noise_range = self.elevation_noise_range;

        let ab = maybe_split(
            triangle.a,
            triangle.b,
            vertices,
            edges,
            &mut self.rng,
            min_edge_length,
            split_point_variance,
            elevation_noise_range,
        );
        let bc = maybe_split(
            triangle.b,
            triangle.c,
            vertices,
            edges,
            &mut self.rng,
            min_edge_length,
            split_point_variance,
            elevation_noise_range,
        );
        let ca = maybe_split(
            triangle.c,
            triangle.a,
            vertices,
            edges,
            &mut self.rng,
            min_edge_length,
            split_point_variance,
            elevation_noise_range,
        );

        match (ab, bc, ca) {
            (Some(ab), Some(bc), Some(ca)) => vec![
                Triangle::new(triangle.a, ab, ca),
                Triangle::new(triangle.b, bc, ab),
                Triangle::new(triangle.c, ca, bc),
                Triangle::new(ab, bc, ca),
            ],
            (Some(ab), Some(bc), None) => vec![
                Triangle::new(ab, triangle.b, bc),
                Triangle::new(ab, bc, triangle.c),
                Triangle::new(ab, triangle.c, triangle.a),
            ],
            (None, Some(bc), Some(ca)) => vec![
                Triangle::new(bc, triangle.c, ca),
                Triangle::new(bc, ca, triangle.a),
                Triangle::new(bc, triangle.a, triangle.b),
            ],
            (Some(ab), None, Some(ca)) => vec![
                Triangle::new(ab, triangle.b, triangle.c),
                Triangle::new(ab, triangle.c, ca),
                Triangle::new(ab, ca, triangle.a),
            ],
            (Some(ab), None, None) => vec![
                Triangle::new(triangle.a, ab, triangle.c),
                Triangle::new(ab, triangle.b, triangle.c),
            ],
            (None, Some(bc), None) => vec![
                Triangle::new(triangle.b, bc, triangle.a),
                Triangle::new(bc, triangle.c, triangle.a),
            ],
            (None, None, Some(ca)) => vec![
                Triangle::new(triangle.c, ca, triangle.b),
                Triangle::new(ca, triangle.a, triangle.b),
            ],
            (None, None, None) => vec![triangle],
        }
    }
}
