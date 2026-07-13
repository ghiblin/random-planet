use crate::geometry::mesh::Mesh;
use crate::processor::mesh_processor::MeshProcessor;

pub(crate) fn compose_mesh(first: MeshProcessor, second: MeshProcessor) -> MeshProcessor {
    Box::new(move |mesh: &Mesh| {
        let mesh = first(mesh)?;
        second(&mesh)
    })
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use super::compose_mesh;
    use crate::geometry::mesh::{Mesh, MeshError, Vertex};
    use crate::geometry::vec3::Vec3;
    use crate::processor::mesh_processor::MeshProcessor;

    #[test]
    fn applies_first_then_second() {
        let mesh = Mesh::new(
            vec![Vertex {
                position: Vec3::new(1.0, 0.0, 0.0),
            }],
            vec![],
        )
        .expect("valid mesh fixture");
        let double: MeshProcessor = Box::new(|mesh: &Mesh| {
            let vertices = mesh
                .vertices()
                .iter()
                .map(|vertex| Vertex {
                    position: vertex.position.scale(2.0),
                })
                .collect();
            Mesh::new(vertices, mesh.triangles().to_vec())
        });
        let add_one_x: MeshProcessor = Box::new(|mesh: &Mesh| {
            let vertices = mesh
                .vertices()
                .iter()
                .map(|vertex| Vertex {
                    position: vertex.position.add(Vec3::new(1.0, 0.0, 0.0)),
                })
                .collect();
            Mesh::new(vertices, mesh.triangles().to_vec())
        });

        let result = compose_mesh(double, add_one_x)(&mesh).expect("compose_mesh should succeed");

        // first `double` (1 -> 2), then `add_one_x` (2 -> 3) — not 4, which is what
        // applying `add_one_x` before `double` would produce.
        assert_eq!(result.vertices()[0].position, Vec3::new(3.0, 0.0, 0.0));
    }

    #[test]
    fn short_circuits_when_first_fails() {
        let mesh = Mesh::new(vec![], vec![]).expect("valid mesh fixture");
        let failing: MeshProcessor = Box::new(|_mesh: &Mesh| {
            Err(MeshError::VertexIndexOutOfBounds {
                index: 0,
                vertex_count: 0,
            })
        });
        let second_called = Rc::new(Cell::new(false));
        let second_called_handle = Rc::clone(&second_called);
        let second: MeshProcessor = Box::new(move |mesh: &Mesh| {
            second_called_handle.set(true);
            Ok(mesh.clone())
        });

        let result = compose_mesh(failing, second)(&mesh);

        assert!(result.is_err());
        assert!(
            !second_called.get(),
            "second stage must not run when first fails"
        );
    }
}
