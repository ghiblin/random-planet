use cucumber::{World as _, given, then, when};
use planet_core::geometry::mesh::Mesh;
use planet_renderer::gpu::buffers::mesh_render_indices;

#[derive(Debug, Default, cucumber::World)]
pub struct MeshRenderIndicesWorld {
    mesh: Option<Mesh>,
    render_indices: Vec<u16>,
}

#[given(regex = r"^a Mesh constructed by Mesh::cube with side ([\d.]+)$")]
fn given_cube(world: &mut MeshRenderIndicesWorld, side: f32) {
    world.mesh = Some(Mesh::cube(side).expect("cube construction failed"));
}

#[given("an empty Mesh with no vertices and no triangles")]
fn given_empty_mesh(world: &mut MeshRenderIndicesWorld) {
    world.mesh = Some(Mesh::new(vec![], vec![]).expect("mesh construction failed"));
}

#[when("the mesh is converted into render indices")]
fn when_converted(world: &mut MeshRenderIndicesWorld) {
    let mesh = world.mesh.as_ref().expect("mesh not set");
    world.render_indices = mesh_render_indices(mesh);
}

#[then("the render index list is 0 through 35 in order")]
fn then_sequential(world: &mut MeshRenderIndicesWorld) {
    let expected: Vec<u16> = (0..36).collect();
    assert_eq!(world.render_indices, expected);
}

#[then("the render index list is empty")]
fn then_empty(world: &mut MeshRenderIndicesWorld) {
    assert!(world.render_indices.is_empty());
}

#[tokio::main]
async fn main() {
    MeshRenderIndicesWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/mesh_render_indices.feature")
        .await;
}
