use cucumber::{World as _, given, then, when};
use planet_core::geometry::mesh::Mesh;
use planet_core::planets::planet::{GenerationProgress, Planet};
use planet_core::presets::preset::Preset;
use planet_core::subdivision::seed::Seed;
use planet_core::subdivision::steps::Steps;
use std::cell::RefCell;
use std::rc::Rc;

type Invocations = Rc<RefCell<Vec<(Mesh, usize)>>>;

#[derive(Debug, Default, cucumber::World)]
pub struct PlanetWorld {
    first_planet: Option<Planet>,
    second_planet: Option<Planet>,
    callback_invocations: Option<Invocations>,
}

impl PlanetWorld {
    fn invocations(&self) -> std::cell::Ref<'_, Vec<(Mesh, usize)>> {
        self.callback_invocations
            .as_ref()
            .expect("no recording progress callback given")
            .borrow()
    }
}

fn parse_preset(name: &str) -> Preset {
    match name {
        "Earthy" => Preset::Earthy,
        "Volcano" => Preset::Volcano,
        "Rocky" => Preset::Rocky,
        other => panic!("unknown preset: {other}"),
    }
}

fn create(seed: u64, preset_name: &str) -> Planet {
    Planet::builder()
        .with_preset(parse_preset(preset_name))
        .with_seed(Seed::from(seed))
        .build()
        .expect("PlanetBuilder::build failed")
}

fn generate(seed: u64, preset_name: &str, max_depth: usize) -> Planet {
    create(seed, preset_name)
        .subdivide(Steps::new(max_depth).expect("Steps::new failed"), None)
        .expect("Planet::subdivide failed")
}

#[given(
    regex = r"^a Planet generated with seed (\d+) and the (Earthy|Volcano|Rocky) preset at max depth (\d+)$"
)]
fn given_planet(world: &mut PlanetWorld, seed: u64, preset_name: String, max_depth: usize) {
    world.first_planet = Some(generate(seed, &preset_name, max_depth));
}

#[when(
    regex = r"^another Planet is generated with seed (\d+) and the (Earthy|Volcano|Rocky) preset at max depth (\d+)$"
)]
fn when_another_planet(world: &mut PlanetWorld, seed: u64, preset_name: String, max_depth: usize) {
    world.second_planet = Some(generate(seed, &preset_name, max_depth));
}

#[then("the two Planets have identical meshes")]
fn then_identical_meshes(world: &mut PlanetWorld) {
    let first = world
        .first_planet
        .as_ref()
        .expect("first Planet not generated");
    let second = world
        .second_planet
        .as_ref()
        .expect("second Planet not generated");
    assert_eq!(first.mesh(), second.mesh());
}

#[then("the two Planets have identical colors")]
fn then_identical_colors(world: &mut PlanetWorld) {
    let first = world
        .first_planet
        .as_ref()
        .expect("first Planet not generated");
    let second = world
        .second_planet
        .as_ref()
        .expect("second Planet not generated");
    assert_eq!(first.colors(), second.colors());
}

#[then("the two Planets do not have identical meshes")]
fn then_not_identical_meshes(world: &mut PlanetWorld) {
    let first = world
        .first_planet
        .as_ref()
        .expect("first Planet not generated");
    let second = world
        .second_planet
        .as_ref()
        .expect("second Planet not generated");
    assert_ne!(first.mesh(), second.mesh());
}

#[then(
    regex = r"^every vertex's color in the resulting Planet equals the (Earthy|Volcano|Rocky) preset's color gradient sampled at that vertex's radius$"
)]
fn then_colors_match_gradient(world: &mut PlanetWorld, preset_name: String) {
    let planet = world
        .first_planet
        .as_ref()
        .expect("first Planet not generated");
    let gradient = parse_preset(&preset_name).params().color_gradient().clone();
    for (vertex, color) in planet.mesh().vertices().iter().zip(planet.colors()) {
        let expected = gradient.sample(vertex.position.length());
        assert_eq!(*color, expected);
    }
}

#[then(
    regex = r"^every vertex of the resulting Planet's mesh has a radius less than or equal to (\d+(?:\.\d+)?)$"
)]
fn then_radius_upper_bound(world: &mut PlanetWorld, bound: f32) {
    let planet = world
        .first_planet
        .as_ref()
        .expect("first Planet not generated");
    for vertex in planet.mesh().vertices() {
        assert!(
            vertex.position.length() <= bound + 1e-5,
            "vertex radius {} exceeds {bound}",
            vertex.position.length()
        );
    }
}

#[then(
    regex = r"^every vertex of the resulting Planet's mesh has a radius greater than or equal to (\d+(?:\.\d+)?)$"
)]
fn then_radius_lower_bound(world: &mut PlanetWorld, bound: f32) {
    let planet = world
        .first_planet
        .as_ref()
        .expect("first Planet not generated");
    for vertex in planet.mesh().vertices() {
        assert!(
            vertex.position.length() >= bound - 1e-5,
            "vertex radius {} is below {bound}",
            vertex.position.length()
        );
    }
}

#[then("the resulting Planet's mesh is identical to the icosahedron mesh")]
fn then_mesh_identical_to_icosahedron(world: &mut PlanetWorld) {
    let planet = world
        .first_planet
        .as_ref()
        .expect("first Planet not generated");
    let icosahedron = Mesh::icosahedron().expect("Mesh::icosahedron() failed");
    assert_eq!(*planet.mesh(), icosahedron);
}

#[then(regex = r"^the resulting Planet has exactly (\d+) colors?$")]
fn then_color_count(world: &mut PlanetWorld, count: usize) {
    let planet = world
        .first_planet
        .as_ref()
        .expect("first Planet not generated");
    assert_eq!(planet.colors().len(), count);
}

#[then(regex = r"^the resulting Planet's mesh has exactly (\d+) triangles$")]
fn then_exact_triangle_count(world: &mut PlanetWorld, count: usize) {
    let planet = world
        .first_planet
        .as_ref()
        .expect("first Planet not generated");
    assert_eq!(planet.mesh().triangles().len(), count);
}

#[then(regex = r"^the resulting Planet's mesh has (\d+) vertices$")]
fn then_vertex_count(world: &mut PlanetWorld, count: usize) {
    let planet = world
        .first_planet
        .as_ref()
        .expect("first Planet not generated");
    assert_eq!(planet.mesh().vertices().len(), count);
}

#[then("the resulting Planet's mesh has the same triangles as the icosahedron mesh")]
fn then_same_triangles_as_icosahedron(world: &mut PlanetWorld) {
    let planet = world
        .first_planet
        .as_ref()
        .expect("first Planet not generated");
    let icosahedron = Mesh::icosahedron().expect("Mesh::icosahedron() failed");
    assert_eq!(planet.mesh().triangles(), icosahedron.triangles());
}

#[then("the second Planet's mesh has more vertices than the first Planet's mesh")]
fn then_second_has_more_vertices(world: &mut PlanetWorld) {
    let first = world
        .first_planet
        .as_ref()
        .expect("first Planet not generated");
    let second = world
        .second_planet
        .as_ref()
        .expect("second Planet not generated");
    assert!(
        second.mesh().vertices().len() > first.mesh().vertices().len(),
        "expected second Planet ({} vertices) to have more vertices than first ({})",
        second.mesh().vertices().len(),
        first.mesh().vertices().len()
    );
}

#[then(
    regex = r"^the resulting Planet's mesh has at most (\d+) distinct vertex radii, within floating-point tolerance$"
)]
fn then_at_most_distinct_radii(world: &mut PlanetWorld, max_distinct: usize) {
    let planet = world
        .first_planet
        .as_ref()
        .expect("first Planet not generated");
    let mut radii: Vec<f32> = planet
        .mesh()
        .vertices()
        .iter()
        .map(|vertex| vertex.position.length())
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

#[given("a recording progress callback")]
fn given_recording_callback(world: &mut PlanetWorld) {
    world.callback_invocations = Some(Rc::new(RefCell::new(Vec::new())));
}

#[when(
    regex = r"^a Planet is generated with seed (\d+) and the (Earthy|Volcano|Rocky) preset at max depth (\d+) using that callback$"
)]
fn when_generated_with_callback(
    world: &mut PlanetWorld,
    seed: u64,
    preset_name: String,
    max_depth: usize,
) {
    let invocations = world
        .callback_invocations
        .as_ref()
        .expect("no recording progress callback given")
        .clone();
    let recorder = invocations.clone();
    let on_progress: GenerationProgress = Box::new(move |mesh, round| {
        recorder.borrow_mut().push((mesh.clone(), round));
    });
    world.first_planet = Some(
        create(seed, &preset_name)
            .subdivide(
                Steps::new(max_depth).expect("Steps::new failed"),
                Some(on_progress),
            )
            .expect("Planet::subdivide failed"),
    );
}

#[then(regex = r"^the progress callback was invoked (\d+) times?$")]
fn then_callback_invocation_count(world: &mut PlanetWorld, count: usize) {
    assert_eq!(world.invocations().len(), count);
}

#[then(
    regex = r"^the progress callback's (\d+)(?:st|nd|rd|th) invocation received round (\d+) with the base icosahedron mesh$"
)]
fn then_callback_invocation_base_mesh(world: &mut PlanetWorld, index: usize, round: usize) {
    let icosahedron = Mesh::icosahedron().expect("Mesh::icosahedron() failed");
    let invocations = world.invocations();
    let (mesh, actual_round) = &invocations[index - 1];
    assert_eq!(*mesh, icosahedron);
    assert_eq!(*actual_round, round);
}

#[then(
    regex = r"^the progress callback's (\d+)(?:st|nd|rd|th) invocation received a Mesh with (\d+) triangles$"
)]
fn then_callback_invocation_triangles(world: &mut PlanetWorld, index: usize, count: usize) {
    let invocations = world.invocations();
    let (mesh, _) = &invocations[index - 1];
    assert_eq!(mesh.triangles().len(), count);
}

#[given("a Planet built with no fields set")]
fn given_planet_built_with_no_fields_set(world: &mut PlanetWorld) {
    world.first_planet = Some(
        Planet::builder()
            .build()
            .expect("PlanetBuilder::build failed"),
    );
}

#[then(regex = r"^the resulting Planet's preset is (Earthy|Volcano|Rocky)$")]
fn then_preset_is(world: &mut PlanetWorld, preset_name: String) {
    let planet = world
        .first_planet
        .as_ref()
        .expect("first Planet not generated");
    assert_eq!(planet.preset(), parse_preset(&preset_name));
}

#[then(regex = r"^the resulting Planet's seed is (\d+)$")]
fn then_seed_is(world: &mut PlanetWorld, seed: u64) {
    let planet = world
        .first_planet
        .as_ref()
        .expect("first Planet not generated");
    assert_eq!(planet.seed(), Seed::from(seed));
}

#[then(
    regex = r"^the resulting Planet's mesh is identical to a Planet generated with seed (\d+) and the (Earthy|Volcano|Rocky) preset at max depth (\d+)$"
)]
fn then_mesh_identical_to_generated(
    world: &mut PlanetWorld,
    seed: u64,
    preset_name: String,
    max_depth: usize,
) {
    let planet = world
        .first_planet
        .as_ref()
        .expect("first Planet not generated");
    let expected = generate(seed, &preset_name, max_depth);
    assert_eq!(planet.mesh(), expected.mesh());
}

#[then("the resulting Planet has no max depth set")]
fn then_no_max_depth(world: &mut PlanetWorld) {
    let planet = world
        .first_planet
        .as_ref()
        .expect("first Planet not generated");
    assert_eq!(planet.max_depth(), None);
}

#[given(regex = r"^a Planet created with the (Earthy|Volcano|Rocky) preset and seed (\d+)$")]
fn given_planet_created(world: &mut PlanetWorld, preset_name: String, seed: u64) {
    world.first_planet = Some(create(seed, &preset_name));
}

#[when(regex = r"^that Planet is subdivided to max depth (\d+)$")]
fn when_subdivided_to_max_depth(world: &mut PlanetWorld, max_depth: usize) {
    let planet = world
        .first_planet
        .take()
        .expect("first Planet not generated");
    world.first_planet = Some(
        planet
            .subdivide(Steps::new(max_depth).expect("Steps::new failed"), None)
            .expect("Planet::subdivide failed"),
    );
}

#[then(regex = r"^the resulting Planet's max depth is (\d+)$")]
fn then_max_depth_is(world: &mut PlanetWorld, max_depth: usize) {
    let planet = world
        .first_planet
        .as_ref()
        .expect("first Planet not generated");
    assert_eq!(
        planet.max_depth(),
        Some(Steps::new(max_depth).expect("Steps::new failed"))
    );
}

fn fraction_at_minimum_radius(planet: &Planet) -> f32 {
    let radii: Vec<f32> = planet
        .mesh()
        .vertices()
        .iter()
        .map(|vertex| vertex.position.length())
        .collect();
    let min_radius = radii.iter().cloned().fold(f32::INFINITY, f32::min);
    let at_min = radii
        .iter()
        .filter(|radius| (**radius - min_radius).abs() < 1e-4)
        .count();
    at_min as f32 / radii.len() as f32
}

#[then(
    regex = r"^the fraction of the resulting Planet's mesh vertices at its minimum vertex radius is within (\d+(?:\.\d+)?) of the (Earthy|Volcano|Rocky) preset's configured OceanQuota$"
)]
fn then_ocean_quota_fraction_within_tolerance(
    world: &mut PlanetWorld,
    tolerance: f32,
    preset_name: String,
) {
    let planet = world
        .first_planet
        .as_ref()
        .expect("first Planet not generated");
    let fraction = fraction_at_minimum_radius(planet);
    let quota = parse_preset(&preset_name)
        .params()
        .ocean_quota()
        .expect("preset has no OceanQuota")
        .value();
    assert!(
        (fraction - quota).abs() <= tolerance,
        "fraction at sea level {fraction} is not within {tolerance} of configured quota {quota}"
    );
}

#[tokio::main]
async fn main() {
    PlanetWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/planet.feature")
        .await;
}
