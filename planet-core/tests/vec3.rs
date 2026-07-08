use cucumber::{World as _, given, then, when};
use planet_core::geometry::vec3::Vec3;

#[derive(Debug, Default, cucumber::World)]
pub struct Vec3World {
    a: Option<Vec3>,
    b: Option<Vec3>,
    result_vec: Option<Vec3>,
    result_scalar: Option<f32>,
    result_normalized: Option<Option<Vec3>>,
}

impl Vec3World {
    fn a(&self) -> Vec3 {
        self.a.expect("first Vec3 not set")
    }

    fn b(&self) -> Vec3 {
        self.b.expect("second Vec3 not set")
    }
}

#[given(regex = r"^a Vec3 of \(([-\d.]+), ([-\d.]+), ([-\d.]+)\)$")]
fn given_a(world: &mut Vec3World, x: f32, y: f32, z: f32) {
    world.a = Some(Vec3::new(x, y, z));
}

#[given(regex = r"^a second Vec3 of \(([-\d.]+), ([-\d.]+), ([-\d.]+)\)$")]
fn given_b(world: &mut Vec3World, x: f32, y: f32, z: f32) {
    world.b = Some(Vec3::new(x, y, z));
}

#[when("the two vectors are added")]
fn when_added(world: &mut Vec3World) {
    world.result_vec = Some(world.a().add(world.b()));
}

#[when("the second vector is subtracted from the first")]
fn when_subtracted(world: &mut Vec3World) {
    world.result_vec = Some(world.a().sub(world.b()));
}

#[when(regex = r"^the vector is scaled by ([-\d.]+)$")]
fn when_scaled(world: &mut Vec3World, factor: f32) {
    world.result_vec = Some(world.a().scale(factor));
}

#[when("the dot product of the two vectors is computed")]
fn when_dot(world: &mut Vec3World) {
    world.result_scalar = Some(world.a().dot(world.b()));
}

#[when("the cross product of the two vectors is computed")]
fn when_cross(world: &mut Vec3World) {
    world.result_vec = Some(world.a().cross(world.b()));
}

#[when("the vector's length is computed")]
fn when_length(world: &mut Vec3World) {
    world.result_scalar = Some(world.a().length());
}

#[when("the vector is normalized")]
fn when_normalized(world: &mut Vec3World) {
    world.result_normalized = Some(world.a().normalized());
}

#[then(regex = r"^the resulting Vec3 is \(([-\d.]+), ([-\d.]+), ([-\d.]+)\)$")]
fn then_resulting_vec3(world: &mut Vec3World, x: f32, y: f32, z: f32) {
    assert_eq!(world.result_vec, Some(Vec3::new(x, y, z)));
}

#[then(regex = r"^the result is ([-\d.]+)$")]
fn then_result_is(world: &mut Vec3World, expected: f32) {
    assert_eq!(world.result_scalar, Some(expected));
}

#[then("the resulting Vec3 has a length of 1.0")]
fn then_length_one(world: &mut Vec3World) {
    let normalized = world
        .result_normalized
        .expect("normalization not computed")
        .expect("expected a normalized vector, got None");
    assert!((normalized.length() - 1.0).abs() < 1e-5);
}

#[then("normalization returns nothing")]
fn then_normalization_none(world: &mut Vec3World) {
    assert_eq!(world.result_normalized, Some(None));
}

#[tokio::main]
async fn main() {
    Vec3World::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/vec3.feature")
        .await;
}
