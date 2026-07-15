use cucumber::{World as _, given, then, when};
use planet_core::geometry::mesh::Mesh;
use planet_core::geometry::vec3::Vec3;
use planet_core::processor::terrain_noise::{TerrainNoise, apply_terrain_noise};
use planet_core::subdivision::seed::Seed;
use planet_core::subdivision::subdivide::subdivide;
use planet_core::subdivision::subdivision_args::SubdivisionArgs;
use planet_core::subdivision::subdivision_mode::SubdivisionMode;

#[derive(Debug, Default, cucumber::World)]
pub struct ApplyTerrainNoiseWorld {
    icosahedron_mesh: Option<Mesh>,
    positions: Vec<Vec3>,
    subdivided_mesh: Option<Mesh>,
    terrain_noise: Option<TerrainNoise>,
    source: Option<Mesh>,
    result: Option<Mesh>,
    first_mesh: Option<Mesh>,
    second_mesh: Option<Mesh>,
}

impl ApplyTerrainNoiseWorld {
    fn source_mesh(&self) -> Mesh {
        if let Some(mesh) = &self.subdivided_mesh {
            mesh.clone()
        } else if let Some(mesh) = &self.icosahedron_mesh {
            mesh.clone()
        } else {
            Mesh::new(self.positions.clone(), vec![]).expect("source Mesh construction failed")
        }
    }

    fn face_corner_indices(mesh: &Mesh, face_index: usize) -> Vec<usize> {
        mesh.faces()[face_index]
            .edges
            .iter()
            .map(|&edge_index| mesh.edges()[edge_index].start)
            .collect()
    }

    fn terrain_noise(&self) -> TerrainNoise {
        self.terrain_noise.expect("TerrainNoise not given")
    }

    fn result(&self) -> &Mesh {
        self.result
            .as_ref()
            .expect("apply_terrain_noise result not computed")
    }
}

#[given("an icosahedron mesh")]
fn given_icosahedron(world: &mut ApplyTerrainNoiseWorld) {
    world.icosahedron_mesh = Some(Mesh::icosahedron().expect("Mesh::icosahedron() failed"));
}

#[given(
    regex = r"^an icosahedron mesh subdivided (\d+) steps with SubdivisionMode::UniformRedSplit and seed (\d+)$"
)]
fn given_subdivided_icosahedron(world: &mut ApplyTerrainNoiseWorld, steps: usize, seed: u64) {
    let base = Mesh::icosahedron().expect("Mesh::icosahedron() failed");
    world.icosahedron_mesh = Some(base.clone());
    let args = SubdivisionArgs::new(
        Some(planet_core::subdivision::steps::Steps::new(steps).expect("valid steps fixture")),
        Some(SubdivisionMode::UniformRedSplit),
        Some(Seed::from(seed)),
        None,
    );
    world.subdivided_mesh = Some(subdivide(&base, args).expect("subdivide failed"));
}

#[given("a Mesh with a vertex exactly at the origin")]
fn given_vertex_at_origin(world: &mut ApplyTerrainNoiseWorld) {
    world.positions = vec![Vec3::new(0.0, 0.0, 0.0)];
}

#[given("a Mesh with no vertices and no triangles")]
fn given_empty_mesh(world: &mut ApplyTerrainNoiseWorld) {
    world.positions = vec![];
}

#[given(regex = r"^a TerrainNoise with amplitude (-?\d+(?:\.\d+)?)$")]
fn given_terrain_noise(world: &mut ApplyTerrainNoiseWorld, amplitude: f32) {
    world.terrain_noise = Some(
        TerrainNoise::new(1.5, 4, 0.5, 2.0, amplitude, 1.0, None)
            .expect("valid TerrainNoise fixture"),
    );
}

#[given(regex = r"^a TerrainNoise with amplitude (-?\d+(?:\.\d+)?) and (\d+) terrace levels$")]
fn given_terrain_noise_with_terraces(
    world: &mut ApplyTerrainNoiseWorld,
    amplitude: f32,
    terrace_levels: u32,
) {
    world.terrain_noise = Some(
        TerrainNoise::new(1.5, 4, 0.5, 2.0, amplitude, 1.0, Some(terrace_levels))
            .expect("valid TerrainNoise fixture"),
    );
}

#[when(regex = r"^terrain noise is applied to that mesh with seed (\d+) and that TerrainNoise$")]
fn when_applied(world: &mut ApplyTerrainNoiseWorld, seed: u64) {
    let source = world.source_mesh();
    world.source = Some(source.clone());
    let terrain_noise = world.terrain_noise();
    world.result = Some(
        apply_terrain_noise(&source, Seed::from(seed), terrain_noise)
            .expect("apply_terrain_noise failed"),
    );
}

#[when(
    regex = r"^terrain noise is applied to that mesh with seed (\d+) and that TerrainNoise, producing the first Mesh$"
)]
fn when_applied_first(world: &mut ApplyTerrainNoiseWorld, seed: u64) {
    let source = world.source_mesh();
    let terrain_noise = world.terrain_noise();
    world.first_mesh = Some(
        apply_terrain_noise(&source, Seed::from(seed), terrain_noise)
            .expect("apply_terrain_noise failed"),
    );
}

#[when(
    regex = r"^terrain noise is applied to the same icosahedron mesh with seed (\d+) and that TerrainNoise, producing the second Mesh$"
)]
fn when_applied_second(world: &mut ApplyTerrainNoiseWorld, seed: u64) {
    let source = world.source_mesh();
    let terrain_noise = world.terrain_noise();
    world.second_mesh = Some(
        apply_terrain_noise(&source, Seed::from(seed), terrain_noise)
            .expect("apply_terrain_noise failed"),
    );
}

#[then(
    regex = r"^every vertex of the resulting Mesh has a radius less than or equal to (-?\d+(?:\.\d+)?)$"
)]
fn then_radius_at_most(world: &mut ApplyTerrainNoiseWorld, bound: f32) {
    for vertex in world.result().vertices() {
        let radius = vertex.position.length();
        assert!(radius <= bound, "radius {radius} exceeds {bound}");
    }
}

#[then(
    regex = r"^every vertex of the resulting Mesh has a radius greater than or equal to (-?\d+(?:\.\d+)?)$"
)]
fn then_radius_at_least(world: &mut ApplyTerrainNoiseWorld, bound: f32) {
    for vertex in world.result().vertices() {
        let radius = vertex.position.length();
        assert!(radius >= bound, "radius {radius} is below {bound}");
    }
}

#[then("the first Mesh and the second Mesh are identical")]
fn then_first_and_second_identical(world: &mut ApplyTerrainNoiseWorld) {
    let first = world.first_mesh.as_ref().expect("first Mesh not computed");
    let second = world
        .second_mesh
        .as_ref()
        .expect("second Mesh not computed");
    assert_eq!(first, second);
}

#[then("the first Mesh and the second Mesh are not identical")]
fn then_first_and_second_not_identical(world: &mut ApplyTerrainNoiseWorld) {
    let first = world.first_mesh.as_ref().expect("first Mesh not computed");
    let second = world
        .second_mesh
        .as_ref()
        .expect("second Mesh not computed");
    assert_ne!(first, second);
}

#[then(
    "every vertex of the resulting Mesh has a radius equal to the corresponding vertex's radius in the icosahedron mesh"
)]
fn then_radius_unchanged(world: &mut ApplyTerrainNoiseWorld) {
    let source = world
        .icosahedron_mesh
        .as_ref()
        .expect("icosahedron mesh not given");
    let result = world.result();
    assert_eq!(result.vertices().len(), source.vertices().len());
    for (actual, expected) in result.vertices().iter().zip(source.vertices().iter()) {
        assert!(
            (actual.position.length() - expected.position.length()).abs() < 1e-5,
            "expected radius {}, got {}",
            expected.position.length(),
            actual.position.length()
        );
    }
}

#[then(
    regex = r"^the resulting Mesh has at most (\d+) distinct vertex radii, within floating-point tolerance$"
)]
fn then_at_most_distinct_radii(world: &mut ApplyTerrainNoiseWorld, max_distinct: usize) {
    let mut radii: Vec<f32> = world
        .result()
        .vertices()
        .iter()
        .map(|v| v.position.length())
        .collect();
    radii.sort_by(f32::total_cmp);
    let mut distinct_count = 0;
    let mut last: Option<f32> = None;
    for radius in radii {
        if last.is_none_or(|l| (radius - l).abs() > 1e-4) {
            distinct_count += 1;
            last = Some(radius);
        }
    }
    assert!(
        distinct_count <= max_distinct,
        "expected at most {max_distinct} distinct radii, got {distinct_count}"
    );
}

#[then("no panic occurs")]
fn then_no_panic(world: &mut ApplyTerrainNoiseWorld) {
    for vertex in world.result().vertices() {
        assert!(
            vertex.position.x.is_finite()
                && vertex.position.y.is_finite()
                && vertex.position.z.is_finite(),
            "vertex position {:?} is not finite",
            vertex.position
        );
    }
}

#[then("the resulting Mesh is identical to the original mesh")]
fn then_identical_to_original(world: &mut ApplyTerrainNoiseWorld) {
    let source = world.source.as_ref().expect("source mesh not recorded");
    assert_eq!(world.result(), source);
}

#[then(regex = r"^the resulting Mesh has (\d+) vertices$")]
fn then_vertex_count(world: &mut ApplyTerrainNoiseWorld, count: usize) {
    assert_eq!(world.result().vertices().len(), count);
}

#[then("the resulting Mesh has the same faces as the icosahedron mesh")]
fn then_same_faces(world: &mut ApplyTerrainNoiseWorld) {
    let source = world
        .icosahedron_mesh
        .as_ref()
        .expect("icosahedron mesh not given");
    assert_eq!(world.result().faces(), source.faces());
}

fn angle_at(p: Vec3, q: Vec3, r: Vec3) -> f32 {
    let v1 = p.sub(q);
    let v2 = r.sub(q);
    let cos_angle = (v1.dot(v2) / (v1.length() * v2.length())).clamp(-1.0, 1.0);
    cos_angle.acos().to_degrees()
}

#[then(
    regex = r"^every face in the resulting Mesh has all 3 angles between (\d+) and (\d+) degrees$"
)]
fn then_angles_within_bound(world: &mut ApplyTerrainNoiseWorld, min_angle: f32, max_angle: f32) {
    let mesh = world.result();
    for face_index in 0..mesh.faces().len() {
        let corners = ApplyTerrainNoiseWorld::face_corner_indices(mesh, face_index);
        let a = mesh.vertices()[corners[0]].position;
        let b = mesh.vertices()[corners[1]].position;
        let c = mesh.vertices()[corners[2]].position;
        for angle in [angle_at(b, a, c), angle_at(a, b, c), angle_at(a, c, b)] {
            assert!(
                angle >= min_angle && angle <= max_angle,
                "face angle {angle} outside [{min_angle}, {max_angle}]"
            );
        }
    }
}

#[tokio::main]
async fn main() {
    ApplyTerrainNoiseWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/apply_terrain_noise.feature")
        .await;
}
