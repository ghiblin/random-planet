use cucumber::{World as _, given, then, when};
use planet_core::geometry::mesh::{Mesh, Triangle, Vertex};
use planet_core::geometry::vec3::Vec3;
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
    vertices: Vec<Vertex>,
    triangles: Vec<Triangle>,
    edge_endpoints: Option<(Vec3, Vec3)>,
    result: Option<Mesh>,
    callback_invocations: Option<Invocations>,
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

    fn invocations(&self) -> std::cell::Ref<'_, Vec<(Mesh, usize)>> {
        self.callback_invocations
            .as_ref()
            .expect("no recording update callback given")
            .borrow()
    }
}

#[given("an icosahedron mesh")]
fn given_icosahedron(world: &mut SubdivideWorld) {
    world.icosahedron_mesh = Some(Mesh::icosahedron().expect("Mesh::icosahedron() failed"));
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

#[when(
    regex = r"^the mesh is subdivided with (\d+) steps? using SubdivisionMode::UniformRedSplit$"
)]
fn when_subdivided(world: &mut SubdivideWorld, steps: usize) {
    let source = world.source_mesh();
    let args = SubdivisionArgs::new(
        Some(Steps::new(steps).expect("Steps::new failed")),
        Some(SubdivisionMode::UniformRedSplit),
        None,
    );
    world.result = Some(subdivide(&source, args).expect("subdivide() failed"));
}

#[when("the mesh is subdivided with default SubdivisionArgs")]
fn when_subdivided_default(world: &mut SubdivideWorld) {
    let source = world.source_mesh();
    let args = SubdivisionArgs::new(None, None, None);
    world.result = Some(subdivide(&source, args).expect("subdivide() failed"));
}

#[when(
    regex = r"^the mesh is subdivided with (\d+) steps? using SubdivisionMode::UniformRedSplit and a recording update callback$"
)]
fn when_subdivided_with_callback(world: &mut SubdivideWorld, steps: usize) {
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
        Some(update_cb),
    );
    world.result = Some(subdivide(&source, args).expect("subdivide() failed"));
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

#[then(regex = r"^the update callback was invoked (\d+) times$")]
fn then_callback_invocation_count(world: &mut SubdivideWorld, count: usize) {
    assert_eq!(world.invocations().len(), count);
}

#[then(
    regex = r"^the update callback's (\d+)(?:st|nd|rd|th) invocation received a Mesh with (\d+) triangles$"
)]
fn then_callback_invocation_triangles(world: &mut SubdivideWorld, index: usize, count: usize) {
    let invocations = world.invocations();
    let (mesh, round) = &invocations[index - 1];
    assert_eq!(mesh.triangles().len(), count);
    assert_eq!(*round, index);
}

#[tokio::main]
async fn main() {
    SubdivideWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/subdivide.feature")
        .await;
}
