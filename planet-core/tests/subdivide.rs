use cucumber::{World as _, given, then, when};
use planet_core::icosahedron::icosahedron;
use planet_core::mesh::{Mesh, Triangle, Vertex};
use planet_core::subdivide::subdivide;
use planet_core::uniform_red_split::UniformRedSplit;
use planet_core::vec3::Vec3;

#[derive(Debug, Default, cucumber::World)]
pub struct SubdivideWorld {
    icosahedron_mesh: Option<Mesh>,
    vertices: Vec<Vertex>,
    triangles: Vec<Triangle>,
    edge_endpoints: Option<(Vec3, Vec3)>,
    result: Option<Mesh>,
}

impl SubdivideWorld {
    fn source_mesh(&self) -> Mesh {
        if let Some(mesh) = &self.icosahedron_mesh {
            mesh.clone()
        } else {
            Mesh::new(self.vertices.clone(), self.triangles.clone())
                .expect("source Mesh construction failed")
        }
    }

    fn result(&self) -> &Mesh {
        self.result.as_ref().expect("subdivide result not computed")
    }
}

#[given("an icosahedron mesh")]
fn given_icosahedron(world: &mut SubdivideWorld) {
    world.icosahedron_mesh = Some(icosahedron().expect("icosahedron() failed"));
}

#[given("a Mesh with 3 vertices at the corners of an arbitrary triangle")]
fn given_arbitrary_triangle_vertices(world: &mut SubdivideWorld) {
    world.vertices = vec![
        Vertex {
            position: Vec3::new(0.0, 0.0, 0.0),
        },
        Vertex {
            position: Vec3::new(2.0, 0.0, 0.0),
        },
        Vertex {
            position: Vec3::new(0.0, 2.0, 1.0),
        },
    ];
}

#[given(regex = r"^a Triangle referencing indices (\d+), (\d+), (\d+)$")]
fn given_triangle(world: &mut SubdivideWorld, a: usize, b: usize, c: usize) {
    world.triangles.push(Triangle::new(a, b, c));
}

#[given("the two vertices of the first triangle's first edge in the icosahedron mesh")]
fn given_first_edge_endpoints(world: &mut SubdivideWorld) {
    let mesh = world
        .icosahedron_mesh
        .as_ref()
        .expect("icosahedron mesh not given");
    let triangle = mesh.triangles()[0];
    let a = mesh.vertices()[triangle.a].position;
    let b = mesh.vertices()[triangle.b].position;
    world.edge_endpoints = Some((a, b));
}

#[when(regex = r"^the mesh is subdivided to depth (\d+) using the uniform red-split strategy$")]
fn when_subdivided(world: &mut SubdivideWorld, depth: u32) {
    let source = world.source_mesh();
    world.result =
        Some(subdivide(&source, depth, &mut UniformRedSplit).expect("subdivide() failed"));
}

#[then(regex = r"^the resulting Mesh has (\d+) triangles$")]
fn then_triangle_count(world: &mut SubdivideWorld, count: usize) {
    assert_eq!(world.result().triangles().len(), count);
}

#[then(regex = r"^the resulting Mesh has (\d+) vertices$")]
fn then_vertex_count(world: &mut SubdivideWorld, count: usize) {
    assert_eq!(world.result().vertices().len(), count);
}

#[then("no two vertices in the resulting Mesh have the same position")]
fn then_no_duplicate_positions(world: &mut SubdivideWorld) {
    let vertices = world.result().vertices();
    for i in 0..vertices.len() {
        for j in (i + 1)..vertices.len() {
            assert_ne!(
                vertices[i].position, vertices[j].position,
                "vertices {i} and {j} share a position"
            );
        }
    }
}

#[then("every vertex of the resulting Mesh has a radius less than or equal to 1.0")]
fn then_radius_bound(world: &mut SubdivideWorld) {
    for vertex in world.result().vertices() {
        assert!(
            vertex.position.length() <= 1.0 + 1e-5,
            "vertex radius {} exceeds 1.0",
            vertex.position.length()
        );
    }
}

#[then("a vertex exists in the resulting Mesh at the exact midpoint of the two given vertices")]
fn then_midpoint_exists(world: &mut SubdivideWorld) {
    let (a, b) = world.edge_endpoints.expect("edge endpoints not given");
    let expected = a.add(b).scale(0.5);
    let found = world
        .result()
        .vertices()
        .iter()
        .any(|vertex| (vertex.position.sub(expected)).length() < 1e-5);
    assert!(found, "no vertex found at expected midpoint {expected:?}");
}

#[then("the resulting Mesh is identical to the icosahedron mesh")]
fn then_identical_to_icosahedron(world: &mut SubdivideWorld) {
    let source = world
        .icosahedron_mesh
        .as_ref()
        .expect("icosahedron mesh not given");
    assert_eq!(world.result(), source);
}

#[tokio::main]
async fn main() {
    SubdivideWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/subdivide.feature")
        .await;
}
