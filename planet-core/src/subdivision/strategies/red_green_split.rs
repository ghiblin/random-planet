use rand::{RngExt, SeedableRng};
use rand_distr::StandardNormal;
use rand_pcg::Pcg32;

use crate::geometry::mesh::{Triangle, Vertex};
use crate::subdivision::edge::EdgeCache;
use crate::subdivision::elevation_noise_range::ElevationNoiseRange;
use crate::subdivision::min_edge_length::MinEdgeLength;
use crate::subdivision::normal_noise_range::NormalNoiseRange;
use crate::subdivision::seed::Seed;
use crate::subdivision::split_point_variance::SplitPointVariance;
use crate::subdivision::subdivide::SubdivisionStrategy;

pub(crate) const MIN_SPLIT_T: f32 = 0.05;
pub(crate) const MAX_SPLIT_T: f32 = 0.95;
const MIN_VERTEX_RADIUS: f32 = 0.05;

#[allow(clippy::too_many_arguments)]
fn displaced_split_point(
    a: &Vertex,
    b: &Vertex,
    rng: &mut Pcg32,
    split_point_variance: SplitPointVariance,
    elevation_noise_range: ElevationNoiseRange,
    normal_noise_range: NormalNoiseRange,
) -> Vertex {
    // Equivalent to Normal::new(0.5, split_point_variance.value()).sample(rng) — see
    // rand_distr's own Distribution<F> impl for Normal, which computes exactly
    // `mean + std_dev * StandardNormal.sample(rng)` — but without Normal::new's
    // fallible (non-finite std_dev) construction step, which production code must
    // never unwrap/expect on.
    let z: f32 = rng.sample(StandardNormal);
    let t = (0.5 + split_point_variance.value() * z).clamp(MIN_SPLIT_T, MAX_SPLIT_T);
    let point = a.position.add(b.position.sub(a.position).scale(t));
    let radius = point.length();
    if radius == 0.0 {
        return Vertex { position: point };
    }
    let delta = rng.random_range(elevation_noise_range.low()..=elevation_noise_range.high());
    let new_radius = (radius + delta).max(MIN_VERTEX_RADIUS);
    let radial = point.scale(new_radius / radius);
    let normal_delta = rng.random_range(normal_noise_range.low()..=normal_noise_range.high());
    match a.position.cross(b.position).normalized() {
        Some(normal) => Vertex {
            position: radial.add(normal.scale(normal_delta)),
        },
        None => Vertex { position: radial },
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
    normal_noise_range: NormalNoiseRange,
) -> Option<usize> {
    let length = vertices[b].position.sub(vertices[a].position).length();
    if length < min_edge_length.value() {
        return None;
    }
    Some(edges.get_or_insert_with(a, b, vertices, |va, vb| {
        displaced_split_point(
            va,
            vb,
            rng,
            split_point_variance,
            elevation_noise_range,
            normal_noise_range,
        )
    }))
}

pub(crate) struct RedGreenSplit {
    rng: Pcg32,
    elevation_noise_range: ElevationNoiseRange,
    normal_noise_range: NormalNoiseRange,
    min_edge_length: MinEdgeLength,
    split_point_variance: SplitPointVariance,
}

impl RedGreenSplit {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        seed: Seed,
        elevation_noise_range: ElevationNoiseRange,
        normal_noise_range: NormalNoiseRange,
        min_edge_length: MinEdgeLength,
        split_point_variance: SplitPointVariance,
    ) -> RedGreenSplit {
        RedGreenSplit {
            rng: Pcg32::seed_from_u64(seed.value()),
            elevation_noise_range,
            normal_noise_range,
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
        let normal_noise_range = self.normal_noise_range;

        let ab = maybe_split(
            triangle.a,
            triangle.b,
            vertices,
            edges,
            &mut self.rng,
            min_edge_length,
            split_point_variance,
            elevation_noise_range,
            normal_noise_range,
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
            normal_noise_range,
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
            normal_noise_range,
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
