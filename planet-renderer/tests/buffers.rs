use cucumber::{World as _, given, then, when};
use planet_renderer::gpu::buffers::{Vertex, pack_index_buffer, pack_vertex_buffer};

#[derive(Debug, Default, cucumber::World)]
pub struct BuffersWorld {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    vertex_buffer: Vec<u8>,
    index_buffer: Vec<u8>,
}

#[given(regex = r"^a vertex list with (\d+) vertices$")]
fn vertex_list(world: &mut BuffersWorld, count: usize) {
    world.vertices = vec![
        Vertex {
            position: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            color: [0.0, 0.0, 0.0],
        };
        count
    ];
}

#[given(regex = r"^an index list with (\d+) indices$")]
fn index_list(world: &mut BuffersWorld, count: u32) {
    world.indices = (0..count).collect();
}

#[given("an empty vertex list")]
fn empty_vertex_list(world: &mut BuffersWorld) {
    world.vertices = Vec::new();
}

#[when("the vertex list is packed into a vertex buffer")]
fn pack_vertices(world: &mut BuffersWorld) {
    world.vertex_buffer = pack_vertex_buffer(&world.vertices);
}

#[when("the index list is packed into an index buffer")]
fn pack_indices(world: &mut BuffersWorld) {
    world.index_buffer = pack_index_buffer(&world.indices);
}

#[then("the buffer's byte length equals the vertex count times the vertex stride")]
fn assert_vertex_buffer_len(world: &mut BuffersWorld) {
    let stride = std::mem::size_of::<Vertex>();
    assert_eq!(world.vertex_buffer.len(), world.vertices.len() * stride);
}

#[then("the buffer's byte length equals the index count times the index size")]
fn assert_index_buffer_len(world: &mut BuffersWorld) {
    let size = std::mem::size_of::<u32>();
    assert_eq!(world.index_buffer.len(), world.indices.len() * size);
}

#[then("the buffer is empty")]
fn assert_buffer_empty(world: &mut BuffersWorld) {
    assert!(world.vertex_buffer.is_empty());
}

#[tokio::main]
async fn main() {
    BuffersWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/buffers.feature")
        .await;
}
