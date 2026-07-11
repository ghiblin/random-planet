use cucumber::{World as _, given, then, when};
use planet_core::geometry::mesh::{Mesh, Vertex};
use planet_core::geometry::vec3::Vec3;
use planet_core::processor::ocean_quota::{OceanQuota, apply_ocean_quota};

#[derive(Debug, Default, cucumber::World)]
pub struct ApplyOceanQuotaWorld {
    icosahedron_mesh: Option<Mesh>,
    vertices: Vec<Vertex>,
    source: Option<Mesh>,
    result: Option<Mesh>,
}

impl ApplyOceanQuotaWorld {
    fn source_mesh(&self) -> Mesh {
        if let Some(mesh) = &self.icosahedron_mesh {
            mesh.clone()
        } else {
            Mesh::new(self.vertices.clone(), vec![]).expect("source Mesh construction failed")
        }
    }

    fn result(&self) -> &Mesh {
        self.result
            .as_ref()
            .expect("apply_ocean_quota result not computed")
    }
}

#[given("an icosahedron mesh")]
fn given_icosahedron(world: &mut ApplyOceanQuotaWorld) {
    world.icosahedron_mesh = Some(Mesh::icosahedron().expect("Mesh::icosahedron() failed"));
}

#[given(regex = r"^a Mesh with vertices at radii ([0-9., ]+)$")]
fn given_vertices_at_radii(world: &mut ApplyOceanQuotaWorld, radii: String) {
    world.vertices = radii
        .split(',')
        .map(|part| part.trim().parse::<f32>().expect("radius"))
        .map(|radius| Vertex {
            position: Vec3::new(radius, 0.0, 0.0),
        })
        .collect();
}

#[given("a Mesh with a vertex exactly at the origin")]
fn given_vertex_at_origin(world: &mut ApplyOceanQuotaWorld) {
    world.vertices = vec![Vertex {
        position: Vec3::new(0.0, 0.0, 0.0),
    }];
}

#[given("a Mesh with no vertices and no triangles")]
fn given_empty_mesh(world: &mut ApplyOceanQuotaWorld) {
    world.vertices = vec![];
}

#[when(
    regex = r"^(?:that mesh|the icosahedron mesh) is flattened with an OceanQuota of (\d+(?:\.\d+)?)$"
)]
fn when_flattened(world: &mut ApplyOceanQuotaWorld, quota: f32) {
    let source = world.source_mesh();
    world.source = Some(source.clone());
    let quota = OceanQuota::new(quota).expect("OceanQuota::new failed");
    world.result = Some(apply_ocean_quota(&source, quota).expect("apply_ocean_quota failed"));
}

#[then(regex = r"^the resulting Mesh has vertex radii ([0-9., ]+)$")]
fn then_vertex_radii(world: &mut ApplyOceanQuotaWorld, radii: String) {
    let expected: Vec<f32> = radii
        .split(',')
        .map(|part| part.trim().parse::<f32>().expect("radius"))
        .collect();
    let actual: Vec<f32> = world
        .result()
        .vertices()
        .iter()
        .map(|vertex| vertex.position.length())
        .collect();
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected.iter()) {
        assert!((a - e).abs() < 1e-5, "expected radius {e}, got {a}");
    }
}

#[then("the resulting Mesh is identical to the original mesh")]
fn then_identical_to_original(world: &mut ApplyOceanQuotaWorld) {
    let source = world.source.as_ref().expect("source mesh not recorded");
    assert_eq!(world.result(), source);
}

#[then("the resulting Mesh is identical to the icosahedron mesh")]
fn then_identical_to_icosahedron(world: &mut ApplyOceanQuotaWorld) {
    let source = world
        .icosahedron_mesh
        .as_ref()
        .expect("icosahedron mesh not given");
    assert_eq!(world.result(), source);
}

#[then(regex = r"^the resulting Mesh has (\d+) vertices$")]
fn then_vertex_count(world: &mut ApplyOceanQuotaWorld, count: usize) {
    assert_eq!(world.result().vertices().len(), count);
}

#[then("the resulting Mesh has the same triangles as the icosahedron mesh")]
fn then_same_triangles(world: &mut ApplyOceanQuotaWorld) {
    let source = world
        .icosahedron_mesh
        .as_ref()
        .expect("icosahedron mesh not given");
    assert_eq!(world.result().triangles(), source.triangles());
}

#[then("no panic occurs")]
fn then_no_panic(world: &mut ApplyOceanQuotaWorld) {
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

#[tokio::main]
async fn main() {
    ApplyOceanQuotaWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/apply_ocean_quota.feature")
        .await;
}
