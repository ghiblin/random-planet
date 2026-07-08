use cucumber::{World as _, given, then, when};
use planet_core::geometry::mesh::{Mesh, MeshError, Triangle, Vertex};
use planet_core::geometry::vec3::Vec3;

#[derive(Debug, Default, cucumber::World)]
pub struct MeshWorld {
    vertices: Vec<Vertex>,
    triangles: Vec<Triangle>,
    result: Option<Result<Mesh, MeshError>>,
    cube_meshes: Vec<Mesh>,
}

impl MeshWorld {
    fn mesh(&self) -> &Mesh {
        self.result
            .as_ref()
            .expect("Mesh not constructed")
            .as_ref()
            .expect("Mesh construction failed")
    }
}

#[given(regex = r"^a list of (\d+) vertices$")]
fn given_vertices(world: &mut MeshWorld, count: usize) {
    world.vertices = (0..count)
        .map(|i| Vertex {
            position: Vec3::new(i as f32, 0.0, 0.0),
        })
        .collect();
}

#[given("an empty list of vertices")]
fn given_empty_vertices(world: &mut MeshWorld) {
    world.vertices = vec![];
}

#[given(regex = r"^a Triangle referencing indices (\d+), (\d+), (\d+)$")]
fn given_triangle(world: &mut MeshWorld, a: usize, b: usize, c: usize) {
    world.triangles.push(Triangle::new(a, b, c));
}

#[given("an empty list of triangles")]
fn given_empty_triangles(world: &mut MeshWorld) {
    world.triangles = vec![];
}

#[when("a Mesh is constructed from the vertices and the triangle")]
#[when("a Mesh is constructed from the vertices and the triangles")]
fn when_constructed(world: &mut MeshWorld) {
    world.result = Some(Mesh::new(world.vertices.clone(), world.triangles.clone()));
}

#[then("the Mesh is constructed successfully")]
fn then_success(world: &mut MeshWorld) {
    assert!(world.result.as_ref().expect("Mesh not constructed").is_ok());
}

#[then("the Mesh's vertices match the given list")]
fn then_vertices_match(world: &mut MeshWorld) {
    assert_eq!(world.mesh().vertices(), world.vertices.as_slice());
}

#[then("the Mesh's triangles match the given list")]
fn then_triangles_match(world: &mut MeshWorld) {
    assert_eq!(world.mesh().triangles(), world.triangles.as_slice());
}

#[then("the construction fails with a vertex-index-out-of-bounds error")]
fn then_out_of_bounds(world: &mut MeshWorld) {
    match world.result.as_ref().expect("Mesh not constructed") {
        Err(MeshError::VertexIndexOutOfBounds { .. }) => {}
        other => panic!("expected VertexIndexOutOfBounds, got {other:?}"),
    }
}

#[then("the Mesh has zero vertices")]
fn then_zero_vertices(world: &mut MeshWorld) {
    assert_eq!(world.mesh().vertices().len(), 0);
}

#[then("the Mesh has zero triangles")]
fn then_zero_triangles(world: &mut MeshWorld) {
    assert_eq!(world.mesh().triangles().len(), 0);
}

#[given(regex = r"^a Mesh constructed by Mesh::cube with side (-?[\d.]+)$")]
#[when(regex = r"^a Mesh is constructed by Mesh::cube with side (-?[\d.]+)$")]
fn cube_with_side(world: &mut MeshWorld, side: f32) {
    let result = Mesh::cube(side);
    if let Ok(mesh) = &result {
        world.cube_meshes.push(mesh.clone());
    }
    world.result = Some(result);
}

#[then(regex = r"^the Mesh has (\d+) vertices$")]
fn then_vertex_count(world: &mut MeshWorld, count: usize) {
    assert_eq!(world.mesh().vertices().len(), count);
}

#[then(regex = r"^the Mesh has (\d+) triangles$")]
fn then_triangle_count(world: &mut MeshWorld, count: usize) {
    assert_eq!(world.mesh().triangles().len(), count);
}

#[then("every triangle in the Mesh has three distinct vertex indices")]
fn then_distinct_indices(world: &mut MeshWorld) {
    for triangle in world.mesh().triangles() {
        assert_ne!(triangle.a, triangle.b);
        assert_ne!(triangle.b, triangle.c);
        assert_ne!(triangle.a, triangle.c);
    }
}

#[then("every triangle index in the Mesh is less than 8")]
fn then_indices_less_than_8(world: &mut MeshWorld) {
    for triangle in world.mesh().triangles() {
        assert!(triangle.a < 8);
        assert!(triangle.b < 8);
        assert!(triangle.c < 8);
    }
}

#[then(
    "every vertex of the side-2.0 Mesh is twice as far from the origin as the corresponding vertex of the side-1.0 Mesh"
)]
fn then_double_distance(world: &mut MeshWorld) {
    let side_1 = &world.cube_meshes[0];
    let side_2 = &world.cube_meshes[1];
    for (v1, v2) in side_1.vertices().iter().zip(side_2.vertices()) {
        assert_eq!(v2.position, v1.position.scale(2.0));
    }
}

#[then("the construction fails with a negative-cube-side error")]
fn then_negative_cube_side(world: &mut MeshWorld) {
    match world.result.as_ref().expect("construction not attempted") {
        Err(MeshError::NegativeCubeSide { .. }) => {}
        other => panic!("expected NegativeCubeSide, got {other:?}"),
    }
}

#[then("every vertex of the Mesh is at the origin")]
fn then_all_at_origin(world: &mut MeshWorld) {
    for vertex in world.mesh().vertices() {
        assert_eq!(vertex.position, Vec3::new(0.0, 0.0, 0.0));
    }
}

#[tokio::main]
async fn main() {
    MeshWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/mesh.feature")
        .await;
}
