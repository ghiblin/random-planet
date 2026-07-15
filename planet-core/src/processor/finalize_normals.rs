use crate::geometry::face::Face;
use crate::geometry::mesh::Mesh;
use crate::geometry::vec3::Vec3;
use crate::geometry::vertex::Vertex;

fn face_corners(mesh: &Mesh, face: &Face) -> [usize; 3] {
    let mut corners = [0usize; 3];
    for (slot, &edge_index) in corners.iter_mut().zip(&face.edges) {
        *slot = mesh.edges()[edge_index].start;
    }
    corners
}

/// Computes each `Face`'s flat normal and each `Vertex`'s area-weighted normal from
/// the mesh's final geometry, once every position-mutating step (subdivision,
/// terrain noise, ocean quota) has completed. An unnormalized face normal
/// `(b - a) x (c - a)` already has magnitude `2 * area`, so summing that raw vector
/// across a vertex's incident faces and normalizing once at the end gives the
/// area-weighted average for free, with no separate area computation.
pub fn finalize_normals(mesh: &Mesh) -> Mesh {
    let zero = Vec3::new(0.0, 0.0, 0.0);
    let mut raw_normal_sums = vec![zero; mesh.vertices().len()];
    let mut face_normals = Vec::with_capacity(mesh.faces().len());

    for face in mesh.faces() {
        let corners = face_corners(mesh, face);
        let a = mesh.vertices()[corners[0]].position;
        let b = mesh.vertices()[corners[1]].position;
        let c = mesh.vertices()[corners[2]].position;
        let raw_normal = b.sub(a).cross(c.sub(a));
        for &corner in &corners {
            raw_normal_sums[corner] = raw_normal_sums[corner].add(raw_normal);
        }
        face_normals.push(raw_normal.normalized().unwrap_or(zero));
    }

    let faces = mesh
        .faces()
        .iter()
        .zip(face_normals)
        .map(|(face, normal)| Face {
            edges: face.edges.clone(),
            order: face.order,
            normal,
        })
        .collect();

    let vertices = mesh
        .vertices()
        .iter()
        .zip(raw_normal_sums)
        .map(|(vertex, raw_sum)| Vertex {
            position: vertex.position,
            normal: raw_sum.normalized().unwrap_or(zero),
            edges: vertex.edges.clone(),
        })
        .collect();

    Mesh::from_parts(vertices, mesh.edges().to_vec(), faces)
}
