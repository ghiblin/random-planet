use cucumber::{World as _, given, then, when};
use planet_core::geometry::mesh::{Mesh, MeshError};
use planet_core::geometry::vec3::Vec3;

#[derive(Debug, Default, cucumber::World)]
pub struct MeshWorld {
    positions: Vec<Vec3>,
    triangles: Vec<(usize, usize, usize)>,
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

    fn face_corner_indices(&self, face_index: usize) -> Vec<usize> {
        self.mesh().faces()[face_index]
            .edges
            .iter()
            .map(|&edge_index| self.mesh().edges()[edge_index].start)
            .collect()
    }
}

#[given(regex = r"^a list of (\d+) positions$")]
fn given_positions(world: &mut MeshWorld, count: usize) {
    world.positions = (0..count).map(|i| Vec3::new(i as f32, 0.0, 0.0)).collect();
}

#[given("an empty list of positions")]
fn given_empty_positions(world: &mut MeshWorld) {
    world.positions = vec![];
}

#[given(regex = r"^a triangle index-triple \((\d+), (\d+), (\d+)\)$")]
fn given_triangle(world: &mut MeshWorld, a: usize, b: usize, c: usize) {
    world.triangles.push((a, b, c));
}

#[given("an empty list of triangle index-triples")]
fn given_empty_triangles(world: &mut MeshWorld) {
    world.triangles = vec![];
}

#[when("a Mesh is constructed from the positions and the triangle index-triples")]
fn when_constructed(world: &mut MeshWorld) {
    world.result = Some(Mesh::new(world.positions.clone(), world.triangles.clone()));
}

#[then("the Mesh is constructed successfully")]
fn then_success(world: &mut MeshWorld) {
    assert!(world.result.as_ref().expect("Mesh not constructed").is_ok());
}

#[then("the Mesh's vertex positions match the given list")]
fn then_positions_match(world: &mut MeshWorld) {
    let positions: Vec<Vec3> = world.mesh().vertices().iter().map(|v| v.position).collect();
    assert_eq!(positions, world.positions);
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

#[then("the Mesh has zero faces")]
fn then_zero_faces(world: &mut MeshWorld) {
    assert_eq!(world.mesh().faces().len(), 0);
}

#[then(regex = r"^the Mesh has (\d+) faces?$")]
fn then_face_count(world: &mut MeshWorld, count: usize) {
    assert_eq!(world.mesh().faces().len(), count);
}

#[then(regex = r"^that face has order (\d+)$")]
fn then_face_order(world: &mut MeshWorld, order: usize) {
    assert_eq!(world.mesh().faces()[0].order, order);
}

#[then(regex = r"^the Mesh has (\d+) edges?$")]
fn then_edge_count(world: &mut MeshWorld, count: usize) {
    assert_eq!(world.mesh().edges().len(), count);
}

#[then("each of the 3 vertices has exactly 1 edge in its edges list")]
fn then_each_vertex_one_edge(world: &mut MeshWorld) {
    for vertex in world.mesh().vertices() {
        assert_eq!(vertex.edges.len(), 1);
    }
}

#[then("vertex 0 has exactly 6 edges, one per incident face")]
fn then_vertex_zero_six_edges(world: &mut MeshWorld) {
    assert_eq!(world.mesh().vertices()[0].edges.len(), 6);
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

#[then("every face in the Mesh has three distinct vertex indices")]
fn then_distinct_indices(world: &mut MeshWorld) {
    for face_index in 0..world.mesh().faces().len() {
        let corners = world.face_corner_indices(face_index);
        assert_ne!(corners[0], corners[1]);
        assert_ne!(corners[1], corners[2]);
        assert_ne!(corners[0], corners[2]);
    }
}

#[then("every face's vertex index in the Mesh is less than 8")]
fn then_indices_less_than_8(world: &mut MeshWorld) {
    for face_index in 0..world.mesh().faces().len() {
        for corner in world.face_corner_indices(face_index) {
            assert!(corner < 8);
        }
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
