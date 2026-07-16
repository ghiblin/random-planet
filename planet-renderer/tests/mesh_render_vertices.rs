use cucumber::{World as _, given, then, when};
use planet_core::color::rgb::Rgb;
use planet_core::geometry::mesh::Mesh;
use planet_core::processor::finalize_normals::finalize_normals;
use planet_renderer::gpu::buffers::{Vertex, mesh_render_vertices};

#[derive(Debug, Default, cucumber::World)]
pub struct MeshRenderVerticesWorld {
    mesh: Option<Mesh>,
    positions: Vec<planet_core::geometry::vec3::Vec3>,
    triangles: Vec<(usize, usize, usize)>,
    colors: Vec<Rgb>,
    render_vertices: Vec<Vertex>,
}

#[given(regex = r"^a Mesh constructed by Mesh::cube with side ([\d.]+)$")]
fn given_cube(world: &mut MeshRenderVerticesWorld, side: f32) {
    world.mesh = Some(Mesh::cube(side).expect("cube construction failed"));
}

#[given("a Mesh with 3 vertices at the same position")]
fn given_degenerate_vertices(world: &mut MeshRenderVerticesWorld) {
    world.positions = vec![planet_core::geometry::vec3::Vec3::new(1.0, 2.0, 3.0); 3];
}

#[given(regex = r"^a triangle index-triple \((\d+), (\d+), (\d+)\)$")]
fn given_triangle(world: &mut MeshRenderVerticesWorld, a: usize, b: usize, c: usize) {
    world.triangles.push((a, b, c));
}

#[given("normals finalized for that mesh")]
fn given_normals_finalized(world: &mut MeshRenderVerticesWorld) {
    let mesh = world.mesh.take().unwrap_or_else(|| {
        Mesh::new(world.positions.clone(), world.triangles.clone())
            .expect("mesh construction failed")
    });
    world.mesh = Some(finalize_normals(&mesh));
}

#[given("an empty Mesh with no vertices and no triangles")]
fn given_empty_mesh(world: &mut MeshRenderVerticesWorld) {
    world.mesh = Some(Mesh::new(vec![], vec![]).expect("mesh construction failed"));
}

#[given("a distinct Rgb color for each of the mesh's vertices")]
fn given_distinct_colors(world: &mut MeshRenderVerticesWorld) {
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

#[when("the mesh is converted into render vertices")]
fn when_converted(world: &mut MeshRenderVerticesWorld) {
    let mesh = world.mesh.take().expect("mesh not set");
    let colors = vec![Rgb::new(1.0, 1.0, 1.0).expect("valid Rgb fixture"); mesh.vertices().len()];
    world.render_vertices = mesh_render_vertices(&mesh, &colors, false);
    world.mesh = Some(mesh);
}

#[when("the mesh is converted into render vertices with smooth shading")]
fn when_converted_smooth(world: &mut MeshRenderVerticesWorld) {
    let mesh = world.mesh.take().expect("mesh not set");
    let colors = vec![Rgb::new(1.0, 1.0, 1.0).expect("valid Rgb fixture"); mesh.vertices().len()];
    world.render_vertices = mesh_render_vertices(&mesh, &colors, false);
    world.mesh = Some(mesh);
}

#[when("the mesh is converted into render vertices with flat shading")]
fn when_converted_flat(world: &mut MeshRenderVerticesWorld) {
    let mesh = world.mesh.take().expect("mesh not set");
    let colors = vec![Rgb::new(1.0, 1.0, 1.0).expect("valid Rgb fixture"); mesh.vertices().len()];
    world.render_vertices = mesh_render_vertices(&mesh, &colors, true);
    world.mesh = Some(mesh);
}

#[when("the mesh is converted into render vertices using those colors")]
fn when_converted_with_colors(world: &mut MeshRenderVerticesWorld) {
    let mesh = world.mesh.take().expect("mesh not set");
    world.render_vertices = mesh_render_vertices(&mesh, &world.colors, false);
    world.mesh = Some(mesh);
}

#[then(regex = r"^the render vertex list has (\d+) vertices$")]
fn then_vertex_count(world: &mut MeshRenderVerticesWorld, count: usize) {
    assert_eq!(world.render_vertices.len(), count);
}

#[then("the render vertex list is empty")]
fn then_vertex_list_empty(world: &mut MeshRenderVerticesWorld) {
    assert!(world.render_vertices.is_empty());
}

#[then("each render vertex's normal equals its source vertex's normal")]
fn then_normal_matches_source(world: &mut MeshRenderVerticesWorld) {
    let mesh = world.mesh.as_ref().expect("mesh not set");
    for (face, render_verts) in mesh.faces().iter().zip(world.render_vertices.chunks(3)) {
        for (&edge_index, render_vertex) in face.edges.iter().zip(render_verts) {
            let source_index = mesh.edges()[edge_index].start;
            let expected = mesh.vertices()[source_index].normal;
            assert_eq!(render_vertex.normal, [expected.x, expected.y, expected.z]);
        }
    }
}

#[then("every render vertex belonging to the same face has that face's normal")]
fn then_normal_matches_face(world: &mut MeshRenderVerticesWorld) {
    let mesh = world.mesh.as_ref().expect("mesh not set");
    for (face, render_verts) in mesh.faces().iter().zip(world.render_vertices.chunks(3)) {
        let expected = face.normal;
        for render_vertex in render_verts {
            assert_eq!(render_vertex.normal, [expected.x, expected.y, expected.z]);
        }
    }
}

#[then(
    regex = r"^every render vertex belonging to that face has normal \((-?[\d.]+), (-?[\d.]+), (-?[\d.]+)\)$"
)]
fn then_render_vertices_have_normal(world: &mut MeshRenderVerticesWorld, x: f32, y: f32, z: f32) {
    let expected = [x, y, z];
    for render_vertex in &world.render_vertices {
        assert_eq!(render_vertex.normal, expected);
    }
}

#[then("each render vertex's color equals its source vertex's Rgb")]
fn then_color_matches_source(world: &mut MeshRenderVerticesWorld) {
    let mesh = world.mesh.as_ref().expect("mesh not set");
    for (face, render_verts) in mesh.faces().iter().zip(world.render_vertices.chunks(3)) {
        for (&edge_index, render_vertex) in face.edges.iter().zip(render_verts) {
            let source_index = mesh.edges()[edge_index].start;
            let expected_color = world.colors[source_index];
            assert_eq!(
                render_vertex.color,
                [expected_color.r(), expected_color.g(), expected_color.b()]
            );
        }
    }
}

#[tokio::main]
async fn main() {
    MeshRenderVerticesWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/mesh_render_vertices.feature")
        .await;
}
