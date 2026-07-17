use cucumber::{World as _, given, then, when};
use planet_core::color::rgb::Rgb;
use planet_core::geometry::mesh::Mesh;
use planet_core::processor::finalize_normals::finalize_normals;
use planet_renderer::gpu::buffers::{
    PackedFrame, mesh_render_indices, mesh_render_line_indices, mesh_render_vertices, pack_frame,
    pack_index_buffer, pack_vertex_buffer,
};

#[derive(Debug, Default, cucumber::World)]
pub struct PackFrameWorld {
    mesh: Option<Mesh>,
    colors: Vec<Rgb>,
    frame: Option<PackedFrame>,
}

#[given(regex = r"^a Mesh constructed by Mesh::cube with side ([\d.]+)$")]
fn given_cube(world: &mut PackFrameWorld, side: f32) {
    world.mesh = Some(Mesh::cube(side).expect("cube construction failed"));
}

#[given("normals finalized for that mesh")]
fn given_normals_finalized(world: &mut PackFrameWorld) {
    let mesh = world.mesh.take().expect("mesh not set");
    world.mesh = Some(finalize_normals(&mesh));
}

#[given("an empty Mesh with no vertices and no triangles")]
fn given_empty_mesh(world: &mut PackFrameWorld) {
    world.mesh = Some(Mesh::new(vec![], vec![]).expect("mesh construction failed"));
}

#[given("a distinct Rgb color for each of the mesh's vertices")]
fn given_distinct_colors(world: &mut PackFrameWorld) {
    let mesh = world.mesh.as_ref().expect("mesh not set");
    let count = mesh.vertices().len();
    world.colors = (0..count)
        .map(|i| {
            Rgb::new(
                i as f32 / (count - 1).max(1) as f32,
                0.5,
                1.0 - i as f32 / (count - 1).max(1) as f32,
            )
            .expect("valid Rgb fixture")
        })
        .collect();
}

#[when("the mesh and colors are packed into a PackedFrame")]
fn when_packed(world: &mut PackFrameWorld) {
    let mesh = world.mesh.as_ref().expect("mesh not set");
    world.frame = Some(pack_frame(mesh, &world.colors));
}

#[then(
    "the PackedFrame's vertex_bytes_smooth equals packing the mesh's smooth-shaded render vertices"
)]
fn then_vertex_bytes_smooth(world: &mut PackFrameWorld) {
    let mesh = world.mesh.as_ref().expect("mesh not set");
    let expected = pack_vertex_buffer(&mesh_render_vertices(mesh, &world.colors, false));
    let frame = world.frame.as_ref().expect("frame not packed");
    assert_eq!(frame.vertex_bytes_smooth, expected);
}

#[then("the PackedFrame's vertex_bytes_flat equals packing the mesh's flat-shaded render vertices")]
fn then_vertex_bytes_flat(world: &mut PackFrameWorld) {
    let mesh = world.mesh.as_ref().expect("mesh not set");
    let expected = pack_vertex_buffer(&mesh_render_vertices(mesh, &world.colors, true));
    let frame = world.frame.as_ref().expect("frame not packed");
    assert_eq!(frame.vertex_bytes_flat, expected);
}

#[then("the PackedFrame's index_bytes equals packing the mesh's render indices")]
fn then_index_bytes(world: &mut PackFrameWorld) {
    let mesh = world.mesh.as_ref().expect("mesh not set");
    let expected = pack_index_buffer(&mesh_render_indices(mesh));
    let frame = world.frame.as_ref().expect("frame not packed");
    assert_eq!(frame.index_bytes, expected);
}

#[then("the PackedFrame's line_index_bytes equals packing the mesh's render line indices")]
fn then_line_index_bytes(world: &mut PackFrameWorld) {
    let mesh = world.mesh.as_ref().expect("mesh not set");
    let expected = pack_index_buffer(&mesh_render_line_indices(mesh));
    let frame = world.frame.as_ref().expect("frame not packed");
    assert_eq!(frame.line_index_bytes, expected);
}

#[then("the PackedFrame's vertex_bytes_smooth is empty")]
fn then_vertex_bytes_smooth_empty(world: &mut PackFrameWorld) {
    let frame = world.frame.as_ref().expect("frame not packed");
    assert!(frame.vertex_bytes_smooth.is_empty());
}

#[then("the PackedFrame's index_bytes is empty")]
fn then_index_bytes_empty(world: &mut PackFrameWorld) {
    let frame = world.frame.as_ref().expect("frame not packed");
    assert!(frame.index_bytes.is_empty());
}

#[tokio::main]
async fn main() {
    PackFrameWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/pack_frame.feature")
        .await;
}
