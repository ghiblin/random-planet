use cucumber::{World as _, given, then, when};
use planet_core::color::rgb::Rgb;
use planet_core::geometry::mesh::{Mesh, Triangle, Vertex as CoreVertex};
use planet_core::geometry::vec3::Vec3;
use planet_renderer::gpu::buffers::{Vertex, mesh_render_vertices};

#[derive(Debug, Default, cucumber::World)]
pub struct MeshRenderVerticesWorld {
    vertices: Vec<CoreVertex>,
    triangles: Vec<Triangle>,
    mesh: Option<Mesh>,
    colors: Vec<Rgb>,
    render_vertices: Vec<Vertex>,
}

fn normal_length(normal: [f32; 3]) -> f32 {
    (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt()
}

#[given(regex = r"^a Mesh constructed by Mesh::cube with side ([\d.]+)$")]
fn given_cube(world: &mut MeshRenderVerticesWorld, side: f32) {
    world.mesh = Some(Mesh::cube(side).expect("cube construction failed"));
}

#[given("a Mesh with 3 vertices at the same position")]
fn given_degenerate_vertices(world: &mut MeshRenderVerticesWorld) {
    let position = Vec3::new(1.0, 2.0, 3.0);
    world.vertices = vec![CoreVertex { position }; 3];
}

#[given(regex = r"^a Triangle referencing indices (\d+), (\d+), (\d+)$")]
fn given_triangle(world: &mut MeshRenderVerticesWorld, a: usize, b: usize, c: usize) {
    world.triangles.push(Triangle::new(a, b, c));
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
    let mesh = world.mesh.take().unwrap_or_else(|| {
        Mesh::new(world.vertices.clone(), world.triangles.clone())
            .expect("mesh construction failed")
    });
    let colors = vec![Rgb::new(1.0, 1.0, 1.0).expect("valid Rgb fixture"); mesh.vertices().len()];
    world.render_vertices = mesh_render_vertices(&mesh, &colors);
    world.mesh = Some(mesh);
}

#[when("the mesh is converted into render vertices using those colors")]
fn when_converted_with_colors(world: &mut MeshRenderVerticesWorld) {
    let mesh = world.mesh.take().expect("mesh not set");
    world.render_vertices = mesh_render_vertices(&mesh, &world.colors);
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

#[then("every triangle's three render vertices share an identical normal")]
fn then_shared_normal(world: &mut MeshRenderVerticesWorld) {
    for triangle in world.render_vertices.chunks(3) {
        assert_eq!(triangle[0].normal, triangle[1].normal);
        assert_eq!(triangle[1].normal, triangle[2].normal);
    }
}

#[then("every render vertex normal has unit length")]
fn then_unit_length(world: &mut MeshRenderVerticesWorld) {
    for vertex in &world.render_vertices {
        assert!((normal_length(vertex.normal) - 1.0).abs() < 1e-5);
    }
}

#[then("the +X face triangles have the normal (1.0, 0.0, 0.0)")]
fn then_plus_x_normal(world: &mut MeshRenderVerticesWorld) {
    // Mesh::cube emits triangles in -Z, +Z, -Y, +Y, -X, +X order, 2 triangles per
    // face; the +X face is therefore the last 2 triangles = last 6 render vertices.
    let plus_x = &world.render_vertices[world.render_vertices.len() - 6..];
    for vertex in plus_x {
        assert_eq!(vertex.normal, [1.0, 0.0, 0.0]);
    }
}

#[then("no panic occurs")]
fn then_no_panic(world: &mut MeshRenderVerticesWorld) {
    assert_eq!(world.render_vertices.len(), 3);
}

#[then("every render vertex normal is (0.0, 0.0, 0.0)")]
fn then_zero_normal(world: &mut MeshRenderVerticesWorld) {
    for vertex in &world.render_vertices {
        assert_eq!(vertex.normal, [0.0, 0.0, 0.0]);
    }
}

#[then("each render vertex's color equals its source vertex's Rgb")]
fn then_color_matches_source(world: &mut MeshRenderVerticesWorld) {
    let mesh = world.mesh.as_ref().expect("mesh not set");
    for (triangle, render_verts) in mesh.triangles().iter().zip(world.render_vertices.chunks(3)) {
        let expected = [triangle.a, triangle.b, triangle.c].map(|i| world.colors[i]);
        for (render_vertex, expected_color) in render_verts.iter().zip(expected) {
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
