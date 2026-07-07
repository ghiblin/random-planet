use cucumber::{World as _, given, then, when};
use planet_renderer::camera::Camera;

#[derive(Debug, Default, cucumber::World)]
pub struct CameraWorld {
    camera: Option<Camera>,
    yaw_before: f32,
    pitch_before: f32,
    distance_before: f32,
}

impl CameraWorld {
    fn camera(&self) -> &Camera {
        self.camera.as_ref().expect("camera not constructed")
    }

    fn camera_mut(&mut self) -> &mut Camera {
        self.camera.as_mut().expect("camera not constructed")
    }
}

#[given("a Camera constructed with default orbit parameters")]
fn camera_default(world: &mut CameraWorld) {
    world.camera = Some(Camera::default());
}

#[given("a Camera constructed at the minimum distance")]
fn camera_at_min_distance(world: &mut CameraWorld) {
    world.camera = Some(Camera::at_distance(Camera::MIN_DISTANCE));
}

#[given("a Camera constructed at the maximum distance")]
fn camera_at_max_distance(world: &mut CameraWorld) {
    world.camera = Some(Camera::at_distance(Camera::MAX_DISTANCE));
}

#[when(regex = r"^the camera is orbited by a mouse delta of \(([-\d.]+), ([-\d.]+)\)$")]
fn orbit_by_delta(world: &mut CameraWorld, delta_yaw: f32, delta_pitch: f32) {
    world.yaw_before = world.camera().yaw;
    world.pitch_before = world.camera().pitch;
    world.camera_mut().orbit(delta_yaw, delta_pitch);
}

#[when("the camera is orbited upward past the maximum pitch")]
fn orbit_past_max_pitch(world: &mut CameraWorld) {
    world.camera_mut().orbit(0.0, f32::MAX);
}

#[when(regex = r"^the camera is zoomed in by a scroll delta of ([-\d.]+)$")]
fn zoom_in(world: &mut CameraWorld, scroll_delta: f32) {
    world.distance_before = world.camera().distance;
    world.camera_mut().zoom(-scroll_delta);
}

#[when(regex = r"^the camera is zoomed out by a scroll delta of ([-\d.]+)$")]
fn zoom_out(world: &mut CameraWorld, scroll_delta: f32) {
    world.distance_before = world.camera().distance;
    world.camera_mut().zoom(scroll_delta);
}

#[then(regex = r"^the camera's yaw increases by ([-\d.]+)$")]
fn yaw_increases_by(world: &mut CameraWorld, amount: f32) {
    assert_eq!(world.camera().yaw, world.yaw_before + amount);
}

#[then(regex = r"^the camera's pitch increases by ([-\d.]+)$")]
fn pitch_increases_by(world: &mut CameraWorld, amount: f32) {
    assert_eq!(world.camera().pitch, world.pitch_before + amount);
}

#[then("the camera's pitch stays at the maximum allowed pitch")]
fn pitch_at_max(world: &mut CameraWorld) {
    assert_eq!(world.camera().pitch, Camera::MAX_PITCH);
}

#[then("the camera's distance decreases")]
fn distance_decreases(world: &mut CameraWorld) {
    assert!(world.camera().distance < world.distance_before);
}

#[then("the camera's distance stays at or above the minimum distance")]
fn distance_at_or_above_min(world: &mut CameraWorld) {
    assert!(world.camera().distance >= Camera::MIN_DISTANCE);
}

#[then("the camera's distance stays at the minimum distance")]
fn distance_at_min(world: &mut CameraWorld) {
    assert_eq!(world.camera().distance, Camera::MIN_DISTANCE);
}

#[then("the camera's distance stays at the maximum distance")]
fn distance_at_max(world: &mut CameraWorld) {
    assert_eq!(world.camera().distance, Camera::MAX_DISTANCE);
}

#[tokio::main]
async fn main() {
    CameraWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/camera.feature")
        .await;
}
