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

#[then(regex = r"^the Mesh has (\d+) triangles$")]
fn then_triangle_count(world: &mut IcosahedronWorld, count: usize) {
    assert_eq!(world.mesh().triangles().len(), count);
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

#[then("every triangle in the Mesh has three distinct vertex indices")]
fn then_distinct_indices(world: &mut IcosahedronWorld) {
    for triangle in world.mesh().triangles() {
        assert_ne!(triangle.a, triangle.b);
        assert_ne!(triangle.b, triangle.c);
        assert_ne!(triangle.a, triangle.c);
    }
}

#[then(regex = r"^every triangle index in the Mesh is less than (\d+)$")]
fn then_indices_less_than(world: &mut IcosahedronWorld, bound: usize) {
    for triangle in world.mesh().triangles() {
        assert!(triangle.a < bound);
        assert!(triangle.b < bound);
        assert!(triangle.c < bound);
    }
}

#[then("every triangle's face normal points away from the origin")]
fn then_wound_outward(world: &mut IcosahedronWorld) {
    for triangle in world.mesh().triangles() {
        let vertices = world.mesh().vertices();
        let pa = vertices[triangle.a].position;
        let pb = vertices[triangle.b].position;
        let pc = vertices[triangle.c].position;
        let centroid = pa.add(pb).add(pc).scale(1.0 / 3.0);
        let normal = pb.sub(pa).cross(pc.sub(pa));
        assert!(
            normal.dot(centroid) > 0.0,
            "triangle {triangle:?} is wound inward"
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
