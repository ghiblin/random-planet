use rand::{RngExt, SeedableRng};
use rand_pcg::Pcg32;

use super::vertex_scramble_range::VertexScrambleRange;
use crate::geometry::mesh::{Mesh, MeshError};
use crate::geometry::vec3::Vec3;
use crate::geometry::vertex::Vertex;
use crate::subdivision::seed::Seed;

const MIN_VERTEX_RADIUS: f32 = 0.05;

fn scrambled_component(component: f32, factor_offset: f32) -> f32 {
    if component == 0.0 {
        factor_offset
    } else {
        component * (1.0 + factor_offset)
    }
}

fn scrambled(vertex: &Vertex, rng: &mut Pcg32, range: VertexScrambleRange) -> Vec3 {
    let a = rng.random_range(range.low()..=range.high());
    let b = rng.random_range(range.low()..=range.high());
    let c = rng.random_range(range.low()..=range.high());
    let position = vertex.position;
    let jittered = Vec3::new(
        scrambled_component(position.x, a),
        scrambled_component(position.y, b),
        scrambled_component(position.z, c),
    );
    let radius = jittered.length();
    if radius == 0.0 {
        return jittered;
    }
    if radius < MIN_VERTEX_RADIUS {
        return jittered.scale(MIN_VERTEX_RADIUS / radius);
    }
    jittered
}

pub fn scramble_vertices(
    mesh: &Mesh,
    seed: Seed,
    range: VertexScrambleRange,
) -> Result<Mesh, MeshError> {
    let mut rng = Pcg32::seed_from_u64(seed.value());
    let positions = mesh
        .vertices()
        .iter()
        .map(|vertex| scrambled(vertex, &mut rng, range))
        .collect();
    Ok(mesh.with_repositioned(positions))
}
