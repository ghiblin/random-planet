use cucumber::{World as _, given, then, when};
use planet_core::color::rgb::Rgb;
use planet_core::geometry::mesh::Mesh;
use planet_renderer::scene::growth_animation::GrowthAnimation;

fn placeholder_frame() -> (Mesh, Vec<Rgb>) {
    let mesh = Mesh::new(
        vec![
            planet_core::geometry::vec3::Vec3::new(0.0, 0.0, 0.0),
            planet_core::geometry::vec3::Vec3::new(1.0, 0.0, 0.0),
            planet_core::geometry::vec3::Vec3::new(0.0, 1.0, 0.0),
        ],
        vec![(0, 1, 2)],
    )
    .expect("placeholder triangle mesh must be valid");
    (mesh, vec![Rgb::new(0.0, 0.0, 0.0).expect("valid Rgb")])
}

#[derive(Debug, Default, cucumber::World)]
pub struct GrowthAnimationWorld {
    animation: Option<GrowthAnimation>,
    tick_result: Option<bool>,
}

#[given(regex = r"^a GrowthAnimation constructed with (\d+) frames? and started at ([\d.]+)ms$")]
fn given_constructed(world: &mut GrowthAnimationWorld, frame_count: usize, started_ms: f64) {
    let frames = (0..frame_count).map(|_| placeholder_frame()).collect();
    world.animation = Some(GrowthAnimation::new(frames, started_ms));
}

#[given(regex = r"^the GrowthAnimation has already been ticked at ([\d.]+)ms$")]
fn given_already_ticked(world: &mut GrowthAnimationWorld, now_ms: f64) {
    let animation = world
        .animation
        .as_mut()
        .expect("GrowthAnimation not constructed");
    animation.tick(now_ms);
}

#[when(regex = r"^the GrowthAnimation is ticked at ([\d.]+)ms$")]
fn when_ticked(world: &mut GrowthAnimationWorld, now_ms: f64) {
    let animation = world
        .animation
        .as_mut()
        .expect("GrowthAnimation not constructed");
    world.tick_result = Some(animation.tick(now_ms));
}

#[then(regex = r"^the tick returns (true|false)$")]
fn then_tick_returns(world: &mut GrowthAnimationWorld, expected: String) {
    let expected: bool = expected.parse().expect("regex only matches true/false");
    let result = world.tick_result.expect("tick not called");
    assert_eq!(result, expected);
}

#[then(regex = r"^the GrowthAnimation's current frame index is (\d+)$")]
fn then_current_frame_index(world: &mut GrowthAnimationWorld, expected: usize) {
    let animation = world
        .animation
        .as_ref()
        .expect("GrowthAnimation not constructed");
    assert_eq!(animation.current_frame_index(), expected);
}

#[tokio::main]
async fn main() {
    GrowthAnimationWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/growth_animation.feature")
        .await;
}
