use cucumber::{World as _, given, then, when};
use planet_core::geometry::mesh::{Mesh, Triangle, Vertex};
use planet_core::geometry::vec3::Vec3;
use planet_core::subdivision::elevation_noise_range::ElevationNoiseRange;
use planet_core::subdivision::min_edge_length::MinEdgeLength;
use planet_core::subdivision::seed::Seed;
use planet_core::subdivision::split_point_variance::SplitPointVariance;
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
    first_mesh: Option<Mesh>,
    second_mesh: Option<Mesh>,
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

#[given("a Mesh with an edge whose midpoint is the origin")]
fn given_antipodal_edge_vertices(world: &mut SubdivideWorld) {
    world.vertices = vec![
        Vertex {
            position: Vec3::new(1.0, 0.0, 0.0),
        },
        Vertex {
            position: Vec3::new(-1.0, 0.0, 0.0),
        },
        Vertex {
            position: Vec3::new(0.0, 1.0, 0.0),
        },
    ];
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

#[then("no panic occurs")]
fn then_no_panic(world: &mut SubdivideWorld) {
    let result = world.result();
    assert_eq!(result.triangles().len(), 4);
    for vertex in result.vertices() {
        assert!(
            vertex.position.x.is_finite()
                && vertex.position.y.is_finite()
                && vertex.position.z.is_finite(),
            "vertex position {:?} is not finite",
            vertex.position
        );
    }
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

fn radial_random_args(steps: usize, seed: u64, range: ElevationNoiseRange) -> SubdivisionArgs {
    SubdivisionArgs::new(
        Some(Steps::new(steps).expect("Steps::new failed")),
        Some(SubdivisionMode::RadialRandomSplit {
            seed: Seed::from(seed),
            elevation_noise_range: range,
        }),
        None,
    )
}

#[when(
    regex = r"^the mesh is subdivided with (\d+) steps? using SubdivisionMode::RadialRandomSplit with seed (\d+) and the default ElevationNoiseRange$"
)]
fn when_subdivided_radial_default_range(world: &mut SubdivideWorld, steps: usize, seed: u64) {
    let source = world.source_mesh();
    let args = radial_random_args(steps, seed, ElevationNoiseRange::default());
    world.result = Some(subdivide(&source, args).expect("subdivide() failed"));
}

#[when(
    regex = r"^the mesh is subdivided with (\d+) steps? using SubdivisionMode::RadialRandomSplit with seed (\d+) and an ElevationNoiseRange of low (-?\d+(?:\.\d+)?) and high (-?\d+(?:\.\d+)?)$"
)]
fn when_subdivided_radial_explicit_range(
    world: &mut SubdivideWorld,
    steps: usize,
    seed: u64,
    low: f32,
    high: f32,
) {
    let source = world.source_mesh();
    let range = ElevationNoiseRange::new(low, high).expect("ElevationNoiseRange::new failed");
    let args = radial_random_args(steps, seed, range);
    world.result = Some(subdivide(&source, args).expect("subdivide() failed"));
}

#[when(
    regex = r"^the mesh is subdivided with (\d+) steps? using SubdivisionMode::RadialRandomSplit with seed (\d+) and the default ElevationNoiseRange, producing the first Mesh$"
)]
fn when_subdivided_radial_default_range_first(world: &mut SubdivideWorld, steps: usize, seed: u64) {
    let source = world.source_mesh();
    let args = radial_random_args(steps, seed, ElevationNoiseRange::default());
    world.first_mesh = Some(subdivide(&source, args).expect("subdivide() failed"));
}

#[when(
    regex = r"^the same icosahedron mesh is subdivided with (\d+) steps? using SubdivisionMode::RadialRandomSplit with seed (\d+) and the default ElevationNoiseRange, producing the second Mesh$"
)]
fn when_subdivided_radial_default_range_second(
    world: &mut SubdivideWorld,
    steps: usize,
    seed: u64,
) {
    let source = world.source_mesh();
    let args = radial_random_args(steps, seed, ElevationNoiseRange::default());
    world.second_mesh = Some(subdivide(&source, args).expect("subdivide() failed"));
}

#[when(
    regex = r"^the mesh is subdivided with (\d+) steps? using SubdivisionMode::RadialRandomSplit with seed (\d+) and an ElevationNoiseRange of low (-?\d+(?:\.\d+)?) and high (-?\d+(?:\.\d+)?), producing the first Mesh$"
)]
fn when_subdivided_radial_explicit_range_first(
    world: &mut SubdivideWorld,
    steps: usize,
    seed: u64,
    low: f32,
    high: f32,
) {
    let source = world.source_mesh();
    let range = ElevationNoiseRange::new(low, high).expect("ElevationNoiseRange::new failed");
    let args = radial_random_args(steps, seed, range);
    world.first_mesh = Some(subdivide(&source, args).expect("subdivide() failed"));
}

#[when(
    regex = r"^the same icosahedron mesh is subdivided with (\d+) steps? using SubdivisionMode::RadialRandomSplit with seed (\d+) and an ElevationNoiseRange of low (-?\d+(?:\.\d+)?) and high (-?\d+(?:\.\d+)?), producing the second Mesh$"
)]
fn when_subdivided_radial_explicit_range_second(
    world: &mut SubdivideWorld,
    steps: usize,
    seed: u64,
    low: f32,
    high: f32,
) {
    let source = world.source_mesh();
    let range = ElevationNoiseRange::new(low, high).expect("ElevationNoiseRange::new failed");
    let args = radial_random_args(steps, seed, range);
    world.second_mesh = Some(subdivide(&source, args).expect("subdivide() failed"));
}

#[when(
    regex = r"^the same icosahedron mesh is subdivided with (\d+) steps? using SubdivisionMode::UniformRedSplit, producing the second Mesh$"
)]
fn when_subdivided_uniform_second(world: &mut SubdivideWorld, steps: usize) {
    let source = world.source_mesh();
    let args = SubdivisionArgs::new(
        Some(Steps::new(steps).expect("Steps::new failed")),
        Some(SubdivisionMode::UniformRedSplit),
        None,
    );
    world.second_mesh = Some(subdivide(&source, args).expect("subdivide() failed"));
}

#[then(
    regex = r"^the first (\d+) vertices of the resulting Mesh have the same positions as the icosahedron mesh's vertices$"
)]
fn then_original_vertices_unchanged(world: &mut SubdivideWorld, count: usize) {
    let source = world
        .icosahedron_mesh
        .as_ref()
        .expect("icosahedron mesh not given");
    let result = world.result();
    for index in 0..count {
        assert_eq!(
            result.vertices()[index].position,
            source.vertices()[index].position,
            "vertex {index} was displaced"
        );
    }
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

fn red_green_args(
    steps: usize,
    seed: u64,
    range: ElevationNoiseRange,
    min_edge_length: f32,
    split_point_variance: f32,
) -> SubdivisionArgs {
    SubdivisionArgs::new(
        Some(Steps::new(steps).expect("Steps::new failed")),
        Some(SubdivisionMode::RedGreenSplit {
            seed: Seed::from(seed),
            elevation_noise_range: range,
            min_edge_length: MinEdgeLength::new(min_edge_length)
                .expect("MinEdgeLength::new failed"),
            split_point_variance: SplitPointVariance::new(split_point_variance)
                .expect("SplitPointVariance::new failed"),
        }),
        None,
    )
}

#[when(
    regex = r"^the mesh is subdivided with (\d+) steps? using SubdivisionMode::RedGreenSplit with seed (\d+), the default ElevationNoiseRange, a MinEdgeLength of (-?\d+(?:\.\d+)?), and a SplitPointVariance of (-?\d+(?:\.\d+)?)$"
)]
fn when_subdivided_red_green_default_range(
    world: &mut SubdivideWorld,
    steps: usize,
    seed: u64,
    min_edge_length: f32,
    split_point_variance: f32,
) {
    let source = world.source_mesh();
    let args = red_green_args(
        steps,
        seed,
        ElevationNoiseRange::default(),
        min_edge_length,
        split_point_variance,
    );
    world.result = Some(subdivide(&source, args).expect("subdivide() failed"));
}

#[when(
    regex = r"^the mesh is subdivided with (\d+) steps? using SubdivisionMode::RedGreenSplit with seed (\d+), an ElevationNoiseRange of low (-?\d+(?:\.\d+)?) and high (-?\d+(?:\.\d+)?), a MinEdgeLength of (-?\d+(?:\.\d+)?), and a SplitPointVariance of (-?\d+(?:\.\d+)?)$"
)]
fn when_subdivided_red_green_explicit_range(
    world: &mut SubdivideWorld,
    steps: usize,
    seed: u64,
    low: f32,
    high: f32,
    min_edge_length: f32,
    split_point_variance: f32,
) {
    let source = world.source_mesh();
    let range = ElevationNoiseRange::new(low, high).expect("ElevationNoiseRange::new failed");
    let args = red_green_args(steps, seed, range, min_edge_length, split_point_variance);
    world.result = Some(subdivide(&source, args).expect("subdivide() failed"));
}

#[when(
    regex = r"^the mesh is subdivided with (\d+) steps? using SubdivisionMode::RedGreenSplit with seed (\d+), the default ElevationNoiseRange, a MinEdgeLength of (-?\d+(?:\.\d+)?), and a SplitPointVariance of (-?\d+(?:\.\d+)?), producing the first Mesh$"
)]
fn when_subdivided_red_green_default_range_first(
    world: &mut SubdivideWorld,
    steps: usize,
    seed: u64,
    min_edge_length: f32,
    split_point_variance: f32,
) {
    let source = world.source_mesh();
    let args = red_green_args(
        steps,
        seed,
        ElevationNoiseRange::default(),
        min_edge_length,
        split_point_variance,
    );
    world.first_mesh = Some(subdivide(&source, args).expect("subdivide() failed"));
}

#[when(
    regex = r"^the same icosahedron mesh is subdivided with (\d+) steps? using SubdivisionMode::RedGreenSplit with seed (\d+), the default ElevationNoiseRange, a MinEdgeLength of (-?\d+(?:\.\d+)?), and a SplitPointVariance of (-?\d+(?:\.\d+)?), producing the second Mesh$"
)]
fn when_subdivided_red_green_default_range_second(
    world: &mut SubdivideWorld,
    steps: usize,
    seed: u64,
    min_edge_length: f32,
    split_point_variance: f32,
) {
    let source = world.source_mesh();
    let args = red_green_args(
        steps,
        seed,
        ElevationNoiseRange::default(),
        min_edge_length,
        split_point_variance,
    );
    world.second_mesh = Some(subdivide(&source, args).expect("subdivide() failed"));
}

#[when(
    regex = r"^the mesh is subdivided with (\d+) steps? using SubdivisionMode::RedGreenSplit with seed (\d+), an ElevationNoiseRange of low (-?\d+(?:\.\d+)?) and high (-?\d+(?:\.\d+)?), a MinEdgeLength of (-?\d+(?:\.\d+)?), and a SplitPointVariance of (-?\d+(?:\.\d+)?), producing the first Mesh$"
)]
fn when_subdivided_red_green_explicit_range_first(
    world: &mut SubdivideWorld,
    steps: usize,
    seed: u64,
    low: f32,
    high: f32,
    min_edge_length: f32,
    split_point_variance: f32,
) {
    let source = world.source_mesh();
    let range = ElevationNoiseRange::new(low, high).expect("ElevationNoiseRange::new failed");
    let args = red_green_args(steps, seed, range, min_edge_length, split_point_variance);
    world.first_mesh = Some(subdivide(&source, args).expect("subdivide() failed"));
}

#[then("the resulting Mesh is identical to the source Mesh")]
fn then_identical_to_source(world: &mut SubdivideWorld) {
    let source = world.source_mesh();
    assert_eq!(*world.result(), source);
}

#[then("no vertex in the resulting Mesh sits at the exact midpoint of edge 0-1")]
fn then_no_vertex_at_edge_0_1_midpoint(world: &mut SubdivideWorld) {
    let source = world.source_mesh();
    let expected = source.vertices()[0]
        .position
        .add(source.vertices()[1].position)
        .scale(0.5);
    let found = world
        .result()
        .vertices()
        .iter()
        .any(|vertex| (vertex.position.sub(expected)).length() < 1e-5);
    assert!(!found, "unexpected vertex found at midpoint {expected:?}");
}

#[tokio::main]
async fn main() {
    SubdivideWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/subdivide.feature")
        .await;
}
