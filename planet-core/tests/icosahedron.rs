use cucumber::{World as _, given, then};
use planet_core::geometry::mesh::Mesh;

#[derive(Debug, Default, cucumber::World)]
pub struct IcosahedronWorld {
    mesh: Option<Mesh>,
}

impl IcosahedronWorld {
    fn mesh(&self) -> &Mesh {
        self.mesh
            .as_ref()
            .expect("icosahedron mesh not constructed")
    }

    fn face_corner_indices(&self, face_index: usize) -> Vec<usize> {
        self.mesh().faces()[face_index]
            .edges
            .iter()
            .map(|&edge_index| self.mesh().edges()[edge_index].start)
            .collect()
    }
}

#[given("an icosahedron mesh")]
fn given_icosahedron(world: &mut IcosahedronWorld) {
    world.mesh = Some(Mesh::icosahedron().expect("Mesh::icosahedron() failed"));
}

#[then("the Mesh is constructed successfully")]
fn then_success(world: &mut IcosahedronWorld) {
    assert!(world.mesh.is_some());
}

#[then(regex = r"^the Mesh has (\d+) vertices$")]
fn then_vertex_count(world: &mut IcosahedronWorld, count: usize) {
    assert_eq!(world.mesh().vertices().len(), count);
}

#[then(regex = r"^the Mesh has (\d+) faces$")]
fn then_face_count(world: &mut IcosahedronWorld, count: usize) {
    assert_eq!(world.mesh().faces().len(), count);
}

#[then(regex = r"^every vertex of the Mesh has a radius of ([\d.]+)$")]
fn then_every_vertex_radius(world: &mut IcosahedronWorld, radius: f32) {
    for vertex in world.mesh().vertices() {
        assert!(
            (vertex.position.length() - radius).abs() < 1e-5,
            "expected radius {radius}, got {}",
            vertex.position.length()
        );
    }
}

#[then("every face in the Mesh has three distinct vertex indices")]
fn then_distinct_indices(world: &mut IcosahedronWorld) {
    for face_index in 0..world.mesh().faces().len() {
        let corners = world.face_corner_indices(face_index);
        assert_ne!(corners[0], corners[1]);
        assert_ne!(corners[1], corners[2]);
        assert_ne!(corners[0], corners[2]);
    }
}

#[then(regex = r"^every face's vertex index in the Mesh is less than (\d+)$")]
fn then_indices_less_than(world: &mut IcosahedronWorld, bound: usize) {
    for face_index in 0..world.mesh().faces().len() {
        for corner in world.face_corner_indices(face_index) {
            assert!(corner < bound);
        }
    }
}

#[then("every face's normal points away from the origin")]
fn then_wound_outward(world: &mut IcosahedronWorld) {
    for face_index in 0..world.mesh().faces().len() {
        let corners = world.face_corner_indices(face_index);
        let vertices = world.mesh().vertices();
        let pa = vertices[corners[0]].position;
        let pb = vertices[corners[1]].position;
        let pc = vertices[corners[2]].position;
        let centroid = pa.add(pb).add(pc).scale(1.0 / 3.0);
        let normal = pb.sub(pa).cross(pc.sub(pa));
        assert!(
            normal.dot(centroid) > 0.0,
            "face {corners:?} is wound inward"
        );
    }
}

#[tokio::main]
async fn main() {
    IcosahedronWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/icosahedron.feature")
        .await;
}
