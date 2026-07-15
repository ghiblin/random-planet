use cucumber::{World as _, given, then, when};
use planet_core::geometry::mesh::Mesh;
use planet_core::geometry::vec3::Vec3;
use planet_core::processor::finalize_normals::finalize_normals;

#[derive(Debug, Default, cucumber::World)]
pub struct FinalizeNormalsWorld {
    mesh: Option<Mesh>,
    triangles: Vec<(usize, usize, usize)>,
    positions: Vec<Vec3>,
    result: Option<Mesh>,
}

impl FinalizeNormalsWorld {
    fn result(&self) -> &Mesh {
        self.result.as_ref().expect("normals not finalized")
    }
}

fn normal_length(normal: Vec3) -> f32 {
    normal.dot(normal).sqrt()
}

#[given(regex = r"^a Mesh constructed by Mesh::cube with side ([\d.]+)$")]
fn given_cube(world: &mut FinalizeNormalsWorld, side: f32) {
    world.mesh = Some(Mesh::cube(side).expect("cube construction failed"));
}

#[given("a Mesh with 3 vertices at the same position")]
fn given_degenerate_vertices(world: &mut FinalizeNormalsWorld) {
    world.positions = vec![Vec3::new(1.0, 2.0, 3.0); 3];
}

#[given(regex = r"^a triangle index-triple \((\d+), (\d+), (\d+)\)$")]
fn given_triangle(world: &mut FinalizeNormalsWorld, a: usize, b: usize, c: usize) {
    world.triangles.push((a, b, c));
}

#[given(
    "a Mesh where vertex 0 is shared by one large face facing (0.0, 0.0, 1.0) and one small face facing (1.0, 0.0, 0.0)"
)]
fn given_unequal_area_fixture(world: &mut FinalizeNormalsWorld) {
    let positions = vec![
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(10.0, 0.0, 0.0),
        Vec3::new(0.0, 10.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
    ];
    let triangles = vec![(0, 1, 2), (0, 3, 4)];
    world.mesh = Some(Mesh::new(positions, triangles).expect("mesh construction failed"));
}

#[when("normals are finalized")]
fn when_finalized(world: &mut FinalizeNormalsWorld) {
    let mesh = world.mesh.take().unwrap_or_else(|| {
        Mesh::new(world.positions.clone(), world.triangles.clone())
            .expect("mesh construction failed")
    });
    world.result = Some(finalize_normals(&mesh));
    world.mesh = Some(mesh);
}

#[then("every face's normal has unit length")]
fn then_every_face_unit_normal(world: &mut FinalizeNormalsWorld) {
    for face in world.result().faces() {
        let length = normal_length(face.normal);
        assert!(
            (length - 1.0).abs() < 1e-5,
            "expected unit length, got {length}"
        );
    }
}

#[then(regex = r"^vertex 0's normal is approximately \((-?[\d.]+), (-?[\d.]+), (-?[\d.]+)\)$")]
fn then_vertex_zero_normal_approx(world: &mut FinalizeNormalsWorld, x: f32, y: f32, z: f32) {
    let normal = world.result().vertices()[0].normal;
    let expected = Vec3::new(x, y, z);
    let distance = normal.sub(expected).length();
    assert!(
        distance < 1e-2,
        "expected normal approximately {expected:?}, got {normal:?}"
    );
}

#[then("no panic occurs")]
fn then_no_panic(world: &mut FinalizeNormalsWorld) {
    assert!(world.result.is_some());
}

#[then("every vertex's normal is (0.0, 0.0, 0.0)")]
fn then_every_vertex_zero_normal(world: &mut FinalizeNormalsWorld) {
    for vertex in world.result().vertices() {
        assert_eq!(vertex.normal, Vec3::new(0.0, 0.0, 0.0));
    }
}

#[tokio::main]
async fn main() {
    FinalizeNormalsWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/finalize_normals.feature")
        .await;
}
