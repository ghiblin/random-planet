use planet_core::color::rgb::Rgb;
use planet_core::geometry::mesh::Mesh;
use planet_core::geometry::vec3::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
}

pub fn mesh_render_vertices(mesh: &Mesh, colors: &[Rgb]) -> Vec<Vertex> {
    mesh.triangles()
        .iter()
        .flat_map(|triangle| {
            let a = mesh.vertices()[triangle.a].position;
            let b = mesh.vertices()[triangle.b].position;
            let c = mesh.vertices()[triangle.c].position;
            let normal = b
                .sub(a)
                .cross(c.sub(a))
                .normalized()
                .unwrap_or(Vec3::new(0.0, 0.0, 0.0));
            let normal = [normal.x, normal.y, normal.z];
            let corner_colors = [colors[triangle.a], colors[triangle.b], colors[triangle.c]];
            [a, b, c]
                .into_iter()
                .zip(corner_colors)
                .map(move |(position, color)| Vertex {
                    position: [position.x, position.y, position.z],
                    normal,
                    color: [color.r(), color.g(), color.b()],
                })
        })
        .collect()
}

pub fn mesh_render_indices(mesh: &Mesh) -> Vec<u16> {
    (0..3 * mesh.triangles().len())
        .map(|index| index as u16)
        .collect()
}

pub fn mesh_render_line_indices(mesh: &Mesh) -> Vec<u16> {
    (0..mesh.triangles().len())
        .flat_map(|i| {
            let base = 3 * i as u16;
            [base, base + 1, base + 1, base + 2, base + 2, base]
        })
        .collect()
}

pub fn pack_vertex_buffer(vertices: &[Vertex]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(std::mem::size_of_val(vertices));
    for vertex in vertices {
        for component in vertex
            .position
            .iter()
            .chain(vertex.normal.iter())
            .chain(vertex.color.iter())
        {
            bytes.extend_from_slice(&component.to_le_bytes());
        }
    }
    bytes
}

pub fn pack_index_buffer(indices: &[u16]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(std::mem::size_of_val(indices));
    for index in indices {
        bytes.extend_from_slice(&index.to_le_bytes());
    }
    bytes
}
