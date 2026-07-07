use cucumber::{World as _, given, then, when};
use planet_renderer::buffers::{
    Vertex, cube_indices, cube_vertices, pack_index_buffer, pack_vertex_buffer,
};

#[derive(Debug, Default, cucumber::World)]
pub struct BuffersWorld {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    vertex_buffer: Vec<u8>,
    index_buffer: Vec<u8>,
}

#[given("the cube's fixed vertex list")]
fn cube_vertex_list(world: &mut BuffersWorld) {
    world.vertices = cube_vertices();
}

#[given("the cube's fixed index list")]
fn cube_index_list(world: &mut BuffersWorld) {
    world.indices = cube_indices();
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
    let size = std::mem::size_of::<u16>();
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
