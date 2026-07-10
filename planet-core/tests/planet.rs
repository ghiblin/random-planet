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

fn generate(seed: u64, preset_name: &str, max_depth: usize) -> Planet {
    Planet::generate(
        parse_preset(preset_name),
        Seed::from(seed),
        Steps::new(max_depth).expect("Steps::new failed"),
        None,
    )
    .expect("Planet::generate failed")
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

#[then(
    regex = r"^the resulting Planet's mesh has no more triangles than (\d+) rounds? of subdivision can produce from an icosahedron$"
)]
fn then_triangle_count_within_hard_cap(world: &mut PlanetWorld, rounds: u32) {
    let planet = world
        .first_planet
        .as_ref()
        .expect("first Planet not generated");
    const ICOSAHEDRON_TRIANGLES: u64 = 20;
    let max_triangles = ICOSAHEDRON_TRIANGLES * 4u64.pow(rounds);
    assert!(
        (planet.mesh().triangles().len() as u64) <= max_triangles,
        "triangle count {} exceeds the hard cap of {max_triangles}",
        planet.mesh().triangles().len()
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
        Planet::generate(
            parse_preset(&preset_name),
            Seed::from(seed),
            Steps::new(max_depth).expect("Steps::new failed"),
            Some(on_progress),
        )
        .expect("Planet::generate failed"),
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
    regex = r"^the progress callback's (\d+)(?:st|nd|rd|th) invocation received round (\d+) with the resulting Planet's mesh$"
)]
fn then_callback_invocation_final_mesh(world: &mut PlanetWorld, index: usize, round: usize) {
    let planet = world
        .first_planet
        .as_ref()
        .expect("first Planet not generated");
    let invocations = world.invocations();
    let (mesh, actual_round) = &invocations[index - 1];
    assert_eq!(mesh, planet.mesh());
    assert_eq!(*actual_round, round);
}

#[tokio::main]
async fn main() {
    PlanetWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/planet.feature")
        .await;
}
