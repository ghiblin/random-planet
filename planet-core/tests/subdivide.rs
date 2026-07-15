use cucumber::{World as _, given, then, when};
use planet_core::geometry::mesh::Mesh;
use planet_core::geometry::vec3::Vec3;
use planet_core::subdivision::seed::Seed;
use planet_core::subdivision::steps::Steps;
use planet_core::subdivision::subdivide::subdivide;
use planet_core::subdivision::subdivision_args::{SubdivisionArgs, UpdateCallback};
use planet_core::subdivision::subdivision_mode::SubdivisionMode;
use std::cell::RefCell;
use std::rc::Rc;

type Invocations = Rc<RefCell<Vec<(Mesh, usize)>>>;

#[derive(Debug, Default, cucumber::World)]
pub struct SubdivideWorld {
    icosahedron_mesh: Option<Mesh>,
    positions: Vec<Vec3>,
    triangles: Vec<(usize, usize, usize)>,
    edge_endpoints: Option<(Vec3, Vec3)>,
    result: Option<Mesh>,
    first_mesh: Option<Mesh>,
    second_mesh: Option<Mesh>,
    callback_invocations: Option<Invocations>,
}

impl SubdivideWorld {
    fn source_mesh(&self) -> Mesh {
        if let Some(mesh) = &self.icosahedron_mesh {
            mesh.clone()
        } else {
            Mesh::new(self.positions.clone(), self.triangles.clone())
                .expect("source Mesh construction failed")
        }
    }

    fn result(&self) -> &Mesh {
        self.result.as_ref().expect("subdivide result not computed")
    }

    fn invocations(&self) -> std::cell::Ref<'_, Vec<(Mesh, usize)>> {
        self.callback_invocations
            .as_ref()
            .expect("no recording update callback given")
            .borrow()
    }
}

fn subdivided(source: &Mesh, steps: usize, seed: u64) -> Mesh {
    let args = SubdivisionArgs::new(
        Some(Steps::new(steps).expect("Steps::new failed")),
        Some(SubdivisionMode::UniformRedSplit),
        Some(Seed::from(seed)),
        None,
    );
    subdivide(source, args).expect("subdivide() failed")
}

#[given("an icosahedron mesh")]
fn given_icosahedron(world: &mut SubdivideWorld) {
    world.icosahedron_mesh = Some(Mesh::icosahedron().expect("Mesh::icosahedron() failed"));
}

#[given("a Mesh with 3 vertices at the corners of an arbitrary triangle")]
fn given_arbitrary_triangle_vertices(world: &mut SubdivideWorld) {
    world.positions = vec![
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(0.0, 2.0, 1.0),
    ];
}

#[given(regex = r"^a triangle index-triple \((\d+), (\d+), (\d+)\)$")]
fn given_triangle(world: &mut SubdivideWorld, a: usize, b: usize, c: usize) {
    world.triangles.push((a, b, c));
}

#[given("the two vertices of the first face's first edge in the icosahedron mesh")]
fn given_first_edge_endpoints(world: &mut SubdivideWorld) {
    let mesh = world
        .icosahedron_mesh
        .as_ref()
        .expect("icosahedron mesh not given");
    let first_edge = mesh.edges()[mesh.faces()[0].edges[0]];
    let a = mesh.vertices()[first_edge.start].position;
    let b = mesh.vertices()[first_edge.end].position;
    world.edge_endpoints = Some((a, b));
}

#[when(
    regex = r"^the mesh is subdivided with (\d+) steps? using SubdivisionMode::UniformRedSplit with seed (\d+)$"
)]
fn when_subdivided(world: &mut SubdivideWorld, steps: usize, seed: u64) {
    let source = world.source_mesh();
    world.result = Some(subdivided(&source, steps, seed));
}

#[when("the mesh is subdivided with default SubdivisionArgs")]
fn when_subdivided_default(world: &mut SubdivideWorld) {
    let source = world.source_mesh();
    let args = SubdivisionArgs::new(None, None, None, None);
    world.result = Some(subdivide(&source, args).expect("subdivide() failed"));
}

#[when(
    regex = r"^the mesh is subdivided with (\d+) steps? using SubdivisionMode::UniformRedSplit with seed (\d+), producing the first Mesh$"
)]
fn when_subdivided_first(world: &mut SubdivideWorld, steps: usize, seed: u64) {
    let source = world.source_mesh();
    world.first_mesh = Some(subdivided(&source, steps, seed));
}

#[when(
    regex = r"^the same icosahedron mesh is subdivided with (\d+) steps? using SubdivisionMode::UniformRedSplit with seed (\d+), producing the second Mesh$"
)]
fn when_subdivided_second(world: &mut SubdivideWorld, steps: usize, seed: u64) {
    let source = world
        .icosahedron_mesh
        .as_ref()
        .expect("icosahedron mesh not given")
        .clone();
    world.second_mesh = Some(subdivided(&source, steps, seed));
}

#[when(
    regex = r"^the mesh is subdivided with (\d+) steps? using SubdivisionMode::UniformRedSplit with seed (\d+) and a recording update callback$"
)]
fn when_subdivided_with_callback(world: &mut SubdivideWorld, steps: usize, seed: u64) {
    let source = world.source_mesh();
    let invocations = Rc::new(RefCell::new(Vec::new()));
    let recorder = invocations.clone();
    let update_cb: UpdateCallback = Box::new(move |mesh, round| {
        recorder.borrow_mut().push((mesh.clone(), round));
    });
    world.callback_invocations = Some(invocations);
    let args = SubdivisionArgs::new(
        Some(Steps::new(steps).expect("Steps::new failed")),
        Some(SubdivisionMode::UniformRedSplit),
        Some(Seed::from(seed)),
        Some(update_cb),
    );
    world.result = Some(subdivide(&source, args).expect("subdivide() failed"));
}

#[then(regex = r"^the resulting Mesh has (\d+) faces$")]
fn then_face_count(world: &mut SubdivideWorld, count: usize) {
    assert_eq!(world.result().faces().len(), count);
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

#[then(
    regex = r"^every vertex of the resulting Mesh has a radius less than or equal to (\d+(?:\.\d+)?)$"
)]
fn then_radius_upper_bound(world: &mut SubdivideWorld, bound: f32) {
    for vertex in world.result().vertices() {
        assert!(
            vertex.position.length() <= bound + 1e-5,
            "vertex radius {} exceeds {bound}",
            vertex.position.length()
        );
    }
}

#[then(
    regex = r"^every vertex of the resulting Mesh has a radius greater than or equal to (\d+(?:\.\d+)?)$"
)]
fn then_radius_lower_bound(world: &mut SubdivideWorld, bound: f32) {
    for vertex in world.result().vertices() {
        assert!(
            vertex.position.length() >= bound - 1e-5,
            "vertex radius {} is below {bound}",
            vertex.position.length()
        );
    }
}

#[then(
    regex = r"^a vertex exists in the resulting Mesh within (\d+(?:\.\d+)?) times the edge's length of the exact midpoint of the two given vertices$"
)]
fn then_midpoint_within_bound(world: &mut SubdivideWorld, factor: f32) {
    let (a, b) = world.edge_endpoints.expect("edge endpoints not given");
    let expected = a.add(b).scale(0.5);
    let edge_length = b.sub(a).length();
    let bound = factor * edge_length;
    let found = world
        .result()
        .vertices()
        .iter()
        .any(|vertex| (vertex.position.sub(expected)).length() <= bound);
    assert!(
        found,
        "no vertex found within {bound} of expected midpoint {expected:?}"
    );
}

#[then("no vertex in the resulting Mesh sits at the exact midpoint of the two given vertices")]
fn then_no_vertex_at_exact_midpoint(world: &mut SubdivideWorld) {
    let (a, b) = world.edge_endpoints.expect("edge endpoints not given");
    let expected = a.add(b).scale(0.5);
    let found_exact = world
        .result()
        .vertices()
        .iter()
        .any(|vertex| (vertex.position.sub(expected)).length() < 1e-6);
    assert!(
        !found_exact,
        "a vertex was found at the exact midpoint {expected:?}, expected jitter to move it away"
    );
}

#[then("the resulting Mesh is identical to the icosahedron mesh")]
fn then_identical_to_icosahedron(world: &mut SubdivideWorld) {
    let source = world
        .icosahedron_mesh
        .as_ref()
        .expect("icosahedron mesh not given");
    assert_eq!(world.result(), source);
}

#[then("the first Mesh and the second Mesh are identical")]
fn then_first_and_second_identical(world: &mut SubdivideWorld) {
    let first = world.first_mesh.as_ref().expect("first Mesh not computed");
    let second = world
        .second_mesh
        .as_ref()
        .expect("second Mesh not computed");
    assert_eq!(first, second);
}

#[then("the first Mesh and the second Mesh are not identical")]
fn then_first_and_second_not_identical(world: &mut SubdivideWorld) {
    let first = world.first_mesh.as_ref().expect("first Mesh not computed");
    let second = world
        .second_mesh
        .as_ref()
        .expect("second Mesh not computed");
    assert_ne!(first, second);
}

#[then(regex = r"^the update callback was invoked (\d+) times$")]
fn then_callback_invocation_count(world: &mut SubdivideWorld, count: usize) {
    assert_eq!(world.invocations().len(), count);
}

#[then(
    regex = r"^the update callback's (\d+)(?:st|nd|rd|th) invocation received a Mesh with (\d+) faces$"
)]
fn then_callback_invocation_faces(world: &mut SubdivideWorld, index: usize, count: usize) {
    let invocations = world.invocations();
    let (mesh, round) = &invocations[index - 1];
    assert_eq!(mesh.faces().len(), count);
    assert_eq!(*round, index);
}

#[tokio::main]
async fn main() {
    SubdivideWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/subdivide.feature")
        .await;
}
