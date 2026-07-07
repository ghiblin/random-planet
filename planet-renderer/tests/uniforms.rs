use cucumber::{World as _, given, then, when};
use planet_renderer::camera::Camera;
use planet_renderer::uniforms::pack_view_projection_uniform;

#[derive(Debug, Default, cucumber::World)]
pub struct UniformsWorld {
    matrix: Option<[[f32; 4]; 4]>,
    uniform_buffer: Vec<u8>,
}

#[given("a view-projection matrix computed from a Camera")]
fn view_projection_matrix(world: &mut UniformsWorld) {
    world.matrix = Some(Camera::default().view_projection_matrix(16.0 / 9.0));
}

#[when("the matrix is packed into a uniform buffer")]
fn pack_matrix(world: &mut UniformsWorld) {
    let matrix = world.matrix.expect("matrix not computed");
    world.uniform_buffer = pack_view_projection_uniform(&matrix);
}

#[then(regex = r"^the buffer's byte length equals (\d+) bytes$")]
fn assert_uniform_buffer_len(world: &mut UniformsWorld, expected_len: usize) {
    assert_eq!(world.uniform_buffer.len(), expected_len);
}

#[tokio::main]
async fn main() {
    UniformsWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/uniforms.feature")
        .await;
}
