use crate::geometry::mesh::{Mesh, MeshError};
use crate::geometry::vec3::Vec3;

pub(crate) fn cube(side: f32) -> Result<Mesh, MeshError> {
    if side < 0.0 {
        return Err(MeshError::NegativeCubeSide { side });
    }

    let half = side / 2.0;
    let positions = [
        (-half, -half, -half),
        (half, -half, -half),
        (half, half, -half),
        (-half, half, -half),
        (-half, -half, half),
        (half, -half, half),
        (half, half, half),
        (-half, half, half),
    ]
    .into_iter()
    .map(|(x, y, z)| Vec3::new(x, y, z))
    .collect();

    let triangles = vec![
        // -Z
        (0, 2, 1),
        (0, 3, 2),
        // +Z
        (4, 5, 6),
        (4, 6, 7),
        // -Y
        (0, 1, 5),
        (0, 5, 4),
        // +Y
        (3, 6, 2),
        (3, 7, 6),
        // -X
        (0, 4, 7),
        (0, 7, 3),
        // +X
        (1, 2, 6),
        (1, 6, 5),
    ];

    Mesh::new(positions, triangles)
}
