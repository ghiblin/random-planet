use cucumber::{World as _, given, then, when};
use planet_core::geometry::mesh::{Mesh, Triangle, Vertex};
use planet_core::geometry::vec3::Vec3;
use planet_core::processor::vertex_scramble::scramble_vertices;
use planet_core::processor::vertex_scramble_range::VertexScrambleRange;
use planet_core::subdivision::seed::Seed;

#[derive(Debug, Default, cucumber::World)]
pub struct VertexScrambleWorld {
    icosahedron_mesh: Option<Mesh>,
    vertices: Vec<Vertex>,
    triangles: Vec<Triangle>,
    result: Option<Mesh>,
    first_mesh: Option<Mesh>,
    second_mesh: Option<Mesh>,
}

impl VertexScrambleWorld {
    fn source_mesh(&self) -> Mesh {
        if let Some(mesh) = &self.icosahedron_mesh {
            mesh.clone()
        } else {
            Mesh::new(self.vertices.clone(), self.triangles.clone())
                .expect("source Mesh construction failed")
        }
    }

    fn result(&self) -> &Mesh {
        self.result
            .as_ref()
            .expect("scramble_vertices result not computed")
    }
}

#[given("an icosahedron mesh")]
fn given_icosahedron(world: &mut VertexScrambleWorld) {
    world.icosahedron_mesh = Some(Mesh::icosahedron().expect("Mesh::icosahedron() failed"));
}

#[given(
    regex = r"^a Mesh with a vertex at position (-?\d+(?:\.\d+)?), (-?\d+(?:\.\d+)?), (-?\d+(?:\.\d+)?)$"
)]
fn given_vertex_at_position(world: &mut VertexScrambleWorld, x: f32, y: f32, z: f32) {
    world.vertices = vec![Vertex {
        position: Vec3::new(x, y, z),
    }];
}

#[given("a Mesh with a vertex exactly at the origin")]
fn given_vertex_at_origin(world: &mut VertexScrambleWorld) {
    world.vertices = vec![Vertex {
        position: Vec3::new(0.0, 0.0, 0.0),
    }];
}

#[given("a Mesh with 3 vertices at the corners of an arbitrary triangle")]
fn given_arbitrary_triangle_vertices(world: &mut VertexScrambleWorld) {
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
fn given_triangle(world: &mut VertexScrambleWorld, a: usize, b: usize, c: usize) {
    world.triangles.push(Triangle::new(a, b, c));
}

#[when(
    regex = r"^the icosahedron mesh is scrambled with seed (\d+) and a VertexScrambleRange of low (-?\d+(?:\.\d+)?) and high (-?\d+(?:\.\d+)?)$"
)]
fn when_icosahedron_scrambled(world: &mut VertexScrambleWorld, seed: u64, low: f32, high: f32) {
    let source = world.source_mesh();
    let range = VertexScrambleRange::new(low, high).expect("VertexScrambleRange::new failed");
    world.result = Some(
        scramble_vertices(&source, Seed::from(seed), range).expect("scramble_vertices failed"),
    );
}

#[when(
    regex = r"^the icosahedron mesh is scrambled with seed (\d+) and a VertexScrambleRange of low (-?\d+(?:\.\d+)?) and high (-?\d+(?:\.\d+)?), producing the first Mesh$"
)]
fn when_icosahedron_scrambled_first(
    world: &mut VertexScrambleWorld,
    seed: u64,
    low: f32,
    high: f32,
) {
    let source = world.source_mesh();
    let range = VertexScrambleRange::new(low, high).expect("VertexScrambleRange::new failed");
    world.first_mesh = Some(
        scramble_vertices(&source, Seed::from(seed), range).expect("scramble_vertices failed"),
    );
}

#[when(
    regex = r"^the same icosahedron mesh is scrambled with seed (\d+) and a VertexScrambleRange of low (-?\d+(?:\.\d+)?) and high (-?\d+(?:\.\d+)?), producing the second Mesh$"
)]
fn when_icosahedron_scrambled_second(
    world: &mut VertexScrambleWorld,
    seed: u64,
    low: f32,
    high: f32,
) {
    let source = world.source_mesh();
    let range = VertexScrambleRange::new(low, high).expect("VertexScrambleRange::new failed");
    world.second_mesh = Some(
        scramble_vertices(&source, Seed::from(seed), range).expect("scramble_vertices failed"),
    );
}

#[when(
    regex = r"^that mesh is scrambled with seed (\d+) and a VertexScrambleRange of low (-?\d+(?:\.\d+)?) and high (-?\d+(?:\.\d+)?)$"
)]
fn when_mesh_scrambled(world: &mut VertexScrambleWorld, seed: u64, low: f32, high: f32) {
    let source = world.source_mesh();
    let range = VertexScrambleRange::new(low, high).expect("VertexScrambleRange::new failed");
    world.result = Some(
        scramble_vertices(&source, Seed::from(seed), range).expect("scramble_vertices failed"),
    );
}

#[then("the resulting Mesh is not identical to the icosahedron mesh")]
fn then_not_identical_to_icosahedron(world: &mut VertexScrambleWorld) {
    let source = world
        .icosahedron_mesh
        .as_ref()
        .expect("icosahedron mesh not given");
    assert_ne!(world.result(), source);
}

#[then("the resulting Mesh is identical to the icosahedron mesh")]
fn then_identical_to_icosahedron(world: &mut VertexScrambleWorld) {
    let source = world
        .icosahedron_mesh
        .as_ref()
        .expect("icosahedron mesh not given");
    assert_eq!(world.result(), source);
}

#[then(regex = r"^the resulting Mesh has (\d+) vertices$")]
fn then_vertex_count(world: &mut VertexScrambleWorld, count: usize) {
    assert_eq!(world.result().vertices().len(), count);
}

#[then("the resulting Mesh has the same triangles as the icosahedron mesh")]
fn then_same_triangles(world: &mut VertexScrambleWorld) {
    let source = world
        .icosahedron_mesh
        .as_ref()
        .expect("icosahedron mesh not given");
    assert_eq!(world.result().triangles(), source.triangles());
}

#[then("the first Mesh and the second Mesh are identical")]
fn then_first_and_second_identical(world: &mut VertexScrambleWorld) {
    let first = world.first_mesh.as_ref().expect("first Mesh not computed");
    let second = world
        .second_mesh
        .as_ref()
        .expect("second Mesh not computed");
    assert_eq!(first, second);
}

#[then("the first Mesh and the second Mesh are not identical")]
fn then_first_and_second_not_identical(world: &mut VertexScrambleWorld) {
    let first = world.first_mesh.as_ref().expect("first Mesh not computed");
    let second = world
        .second_mesh
        .as_ref()
        .expect("second Mesh not computed");
    assert_ne!(first, second);
}

#[then(
    regex = r"^every vertex of the resulting Mesh has a radius greater than or equal to (\d+(?:\.\d+)?)$"
)]
fn then_radius_lower_bound(world: &mut VertexScrambleWorld, bound: f32) {
    for vertex in world.result().vertices() {
        assert!(
            vertex.position.length() >= bound - 1e-5,
            "vertex radius {} is below {bound}",
            vertex.position.length()
        );
    }
}

#[then("no panic occurs")]
fn then_no_panic(world: &mut VertexScrambleWorld) {
    for vertex in world.result().vertices() {
        assert!(
            vertex.position.x.is_finite()
                && vertex.position.y.is_finite()
                && vertex.position.z.is_finite(),
            "vertex position {:?} is not finite",
            vertex.position
        );
    }
}

#[then("no vertex of the resulting Mesh has a coordinate equal to 0.0")]
fn then_no_zero_coordinate(world: &mut VertexScrambleWorld) {
    for vertex in world.result().vertices() {
        assert_ne!(
            vertex.position.x, 0.0,
            "vertex position {:?}",
            vertex.position
        );
        assert_ne!(
            vertex.position.y, 0.0,
            "vertex position {:?}",
            vertex.position
        );
        assert_ne!(
            vertex.position.z, 0.0,
            "vertex position {:?}",
            vertex.position
        );
    }
}

#[tokio::main]
async fn main() {
    VertexScrambleWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/vertex_scramble.feature")
        .await;
}
