use cucumber::{World as _, given, then, when};
use planet_renderer::gpu::buffers::PackedFrame;
use planet_renderer::scene::growth_animation::GrowthAnimation;

fn frame_with_marker(marker: u8) -> PackedFrame {
    PackedFrame {
        vertex_bytes_smooth: vec![marker],
        vertex_bytes_flat: vec![marker],
        index_bytes: vec![marker],
        line_index_bytes: vec![marker],
    }
}

#[derive(Debug, Default, cucumber::World)]
pub struct GrowthAnimationWorld {
    animation: Option<GrowthAnimation>,
    first_frame: Option<PackedFrame>,
    second_frame: Option<PackedFrame>,
    tick_result: Option<bool>,
}

#[given("a new GrowthAnimation with no frames yet")]
fn given_new(world: &mut GrowthAnimationWorld) {
    world.animation = Some(GrowthAnimation::new());
}

#[given(regex = r"^a frame is pushed at ([\d.]+)ms$")]
fn given_frame_pushed(world: &mut GrowthAnimationWorld, now_ms: f64) {
    let animation = world
        .animation
        .as_mut()
        .expect("GrowthAnimation not constructed");
    let frame = frame_with_marker(1);
    animation.push_frame(frame.clone(), now_ms);
    world.first_frame = Some(frame);
}

#[given(regex = r"^a second, distinct frame is pushed at ([\d.]+)ms$")]
fn given_second_frame_pushed(world: &mut GrowthAnimationWorld, now_ms: f64) {
    let animation = world
        .animation
        .as_mut()
        .expect("GrowthAnimation not constructed");
    let frame = frame_with_marker(2);
    animation.push_frame(frame.clone(), now_ms);
    world.second_frame = Some(frame);
}

#[when(regex = r"^a frame is pushed at ([\d.]+)ms$")]
fn when_frame_pushed(world: &mut GrowthAnimationWorld, now_ms: f64) {
    given_frame_pushed(world, now_ms);
}

#[when(regex = r"^a second, distinct frame is pushed at ([\d.]+)ms$")]
fn when_second_frame_pushed(world: &mut GrowthAnimationWorld, now_ms: f64) {
    given_second_frame_pushed(world, now_ms);
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

#[then("the GrowthAnimation's current frame is that frame")]
#[then("the GrowthAnimation's current frame is still the first frame")]
fn then_current_is_first_frame(world: &mut GrowthAnimationWorld) {
    let animation = world
        .animation
        .as_ref()
        .expect("GrowthAnimation not constructed");
    let expected = world.first_frame.as_ref().expect("first frame not pushed");
    assert_eq!(animation.current(), Some(expected));
}

#[then("the GrowthAnimation's current frame is the second frame")]
fn then_current_is_second_frame(world: &mut GrowthAnimationWorld) {
    let animation = world
        .animation
        .as_ref()
        .expect("GrowthAnimation not constructed");
    let expected = world
        .second_frame
        .as_ref()
        .expect("second frame not pushed");
    assert_eq!(animation.current(), Some(expected));
}

#[then("the GrowthAnimation has no current frame")]
fn then_no_current_frame(world: &mut GrowthAnimationWorld) {
    let animation = world
        .animation
        .as_ref()
        .expect("GrowthAnimation not constructed");
    assert_eq!(animation.current(), None);
}

#[tokio::main]
async fn main() {
    GrowthAnimationWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/growth_animation.feature")
        .await;
}
