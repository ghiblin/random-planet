use cucumber::{World as _, given, then, when};
use planet_core::mesh::{Mesh, MeshError, Triangle, Vertex};
use planet_core::vec3::Vec3;

#[derive(Debug, Default, cucumber::World)]
pub struct MeshWorld {
    vertices: Vec<Vertex>,
    triangles: Vec<Triangle>,
    result: Option<Result<Mesh, MeshError>>,
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

#[tokio::main]
async fn main() {
    MeshWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/mesh.feature")
        .await;
}
