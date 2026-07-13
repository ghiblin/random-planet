use crate::processor::mesh_processor::MeshProcessor;

pub(crate) fn identity_mesh() -> MeshProcessor {
    Box::new(|mesh| Ok(mesh.clone()))
}

#[cfg(test)]
mod tests {
    use super::identity_mesh;
    use crate::geometry::mesh::{Mesh, Vertex};
    use crate::geometry::vec3::Vec3;

    #[test]
    fn identity_mesh_returns_the_mesh_unchanged() {
        let mesh = Mesh::new(
            vec![Vertex {
                position: Vec3::new(1.0, 2.0, 3.0),
            }],
            vec![],
        )
        .expect("valid mesh fixture");

        let result = identity_mesh()(&mesh).expect("identity never fails");

        assert_eq!(result, mesh);
    }
}
