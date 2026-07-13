use cucumber::{World as _, given, then, when};
use planet_core::geometry::mesh::{Mesh, Triangle, Vertex as CoreVertex};
use planet_core::geometry::vec3::Vec3;
use planet_renderer::gpu::buffers::mesh_render_line_indices;

#[derive(Debug, Default, cucumber::World)]
pub struct MeshRenderLineIndicesWorld {
    mesh: Option<Mesh>,
    line_indices: Vec<u32>,
}

#[given(regex = r"^a Mesh constructed by Mesh::cube with side ([\d.]+)$")]
fn given_cube(world: &mut MeshRenderLineIndicesWorld, side: f32) {
    world.mesh = Some(Mesh::cube(side).expect("cube construction failed"));
}

#[given("an empty Mesh with no vertices and no triangles")]
fn given_empty_mesh(world: &mut MeshRenderLineIndicesWorld) {
    world.mesh = Some(Mesh::new(vec![], vec![]).expect("mesh construction failed"));
}

#[given(regex = r"^a Mesh with (\d+) triangles$")]
fn given_many_triangles(world: &mut MeshRenderLineIndicesWorld, count: usize) {
    let vertices = vec![
        CoreVertex {
            position: Vec3::new(0.0, 0.0, 0.0)
        };
        3
    ];
    let triangles = vec![Triangle::new(0, 1, 2); count];
    world.mesh = Some(Mesh::new(vertices, triangles).expect("mesh construction failed"));
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
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    e: u32,
    f: u32,
) {
    assert_eq!(&world.line_indices[0..6], &[a, b, c, d, e, f]);
}

#[then("the wireframe line index list is empty")]
fn then_empty(world: &mut MeshRenderLineIndicesWorld) {
    assert!(world.line_indices.is_empty());
}

#[then(regex = r"^the last wireframe line index is (\d+)$")]
fn then_last_index(world: &mut MeshRenderLineIndicesWorld, expected: u32) {
    assert_eq!(
        *world.line_indices.last().expect("line indices empty"),
        expected
    );
}

#[tokio::main]
async fn main() {
    MeshRenderLineIndicesWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/mesh_render_line_indices.feature")
        .await;
}
