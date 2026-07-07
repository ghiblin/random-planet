use cucumber::{World as _, given, then, when};
use planet_core::mesh::Mesh;
use planet_renderer::buffers::mesh_render_line_indices;

#[derive(Debug, Default, cucumber::World)]
pub struct MeshRenderLineIndicesWorld {
    mesh: Option<Mesh>,
    line_indices: Vec<u16>,
}

#[given(regex = r"^a Mesh constructed by Mesh::cube with side ([\d.]+)$")]
fn given_cube(world: &mut MeshRenderLineIndicesWorld, side: f32) {
    world.mesh = Some(Mesh::cube(side).expect("cube construction failed"));
}

#[given("an empty Mesh with no vertices and no triangles")]
fn given_empty_mesh(world: &mut MeshRenderLineIndicesWorld) {
    world.mesh = Some(Mesh::new(vec![], vec![]).expect("mesh construction failed"));
}

#[when("the mesh is converted into wireframe line indices")]
fn when_converted(world: &mut MeshRenderLineIndicesWorld) {
    let mesh = world.mesh.as_ref().expect("mesh not set");
    world.line_indices = mesh_render_line_indices(mesh);
}

#[then(regex = r"^the wireframe line index list has (\d+) indices$")]
fn then_count(world: &mut MeshRenderLineIndicesWorld, count: usize) {
    assert_eq!(world.line_indices.len(), count);
}

#[then(
    regex = r"^the wireframe line indices for the first triangle are (\d+), (\d+), (\d+), (\d+), (\d+), (\d+)$"
)]
fn then_first_triangle(
    world: &mut MeshRenderLineIndicesWorld,
    a: u16,
    b: u16,
    c: u16,
    d: u16,
    e: u16,
    f: u16,
) {
    assert_eq!(&world.line_indices[0..6], &[a, b, c, d, e, f]);
}

#[then("the wireframe line index list is empty")]
fn then_empty(world: &mut MeshRenderLineIndicesWorld) {
    assert!(world.line_indices.is_empty());
}

#[tokio::main]
async fn main() {
    MeshRenderLineIndicesWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/mesh_render_line_indices.feature")
        .await;
}
