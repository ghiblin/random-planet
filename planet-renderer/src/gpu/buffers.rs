use planet_core::color::rgb::Rgb;
use planet_core::geometry::mesh::Mesh;

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
}

pub fn mesh_render_vertices(mesh: &Mesh, colors: &[Rgb], flat_shading: bool) -> Vec<Vertex> {
    mesh.faces()
        .iter()
        .flat_map(|face| {
            let flat_normal = [face.normal.x, face.normal.y, face.normal.z];
            face.edges.iter().map(move |&edge_index| {
                let source_index = mesh.edges()[edge_index].start;
                let vertex = &mesh.vertices()[source_index];
                let color = colors[source_index];
                let normal = if flat_shading {
                    flat_normal
                } else {
                    [vertex.normal.x, vertex.normal.y, vertex.normal.z]
                };
                Vertex {
                    position: [vertex.position.x, vertex.position.y, vertex.position.z],
                    normal,
                    color: [color.r(), color.g(), color.b()],
                }
            })
        })
        .collect()
}

pub fn mesh_render_indices(mesh: &Mesh) -> Vec<u32> {
    (0..3 * mesh.faces().len())
        .map(|index| index as u32)
        .collect()
}

pub fn mesh_render_line_indices(mesh: &Mesh) -> Vec<u32> {
    (0..mesh.faces().len())
        .flat_map(|i| {
            let base = (3 * i) as u32;
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

pub fn pack_index_buffer(indices: &[u32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(std::mem::size_of_val(indices));
    for index in indices {
        bytes.extend_from_slice(&index.to_le_bytes());
    }
    bytes
}
