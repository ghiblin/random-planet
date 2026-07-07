#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

const HALF: f32 = 0.5;

const FACES: [([f32; 3], [[f32; 3]; 4]); 6] = [
    (
        [1.0, 0.0, 0.0],
        [
            [HALF, -HALF, -HALF],
            [HALF, HALF, -HALF],
            [HALF, HALF, HALF],
            [HALF, -HALF, HALF],
        ],
    ),
    (
        [-1.0, 0.0, 0.0],
        [
            [-HALF, -HALF, HALF],
            [-HALF, HALF, HALF],
            [-HALF, HALF, -HALF],
            [-HALF, -HALF, -HALF],
        ],
    ),
    (
        [0.0, 1.0, 0.0],
        [
            [-HALF, HALF, -HALF],
            [-HALF, HALF, HALF],
            [HALF, HALF, HALF],
            [HALF, HALF, -HALF],
        ],
    ),
    (
        [0.0, -1.0, 0.0],
        [
            [-HALF, -HALF, HALF],
            [-HALF, -HALF, -HALF],
            [HALF, -HALF, -HALF],
            [HALF, -HALF, HALF],
        ],
    ),
    (
        [0.0, 0.0, 1.0],
        [
            [-HALF, -HALF, HALF],
            [HALF, -HALF, HALF],
            [HALF, HALF, HALF],
            [-HALF, HALF, HALF],
        ],
    ),
    (
        [0.0, 0.0, -1.0],
        [
            [HALF, -HALF, -HALF],
            [-HALF, -HALF, -HALF],
            [-HALF, HALF, -HALF],
            [HALF, HALF, -HALF],
        ],
    ),
];

pub fn cube_vertices() -> Vec<Vertex> {
    FACES
        .iter()
        .flat_map(|(normal, positions)| {
            positions.iter().map(|position| Vertex {
                position: *position,
                normal: *normal,
            })
        })
        .collect()
}

pub fn cube_indices() -> Vec<u16> {
    (0..6u16)
        .flat_map(|face| {
            let base = face * 4;
            [base, base + 1, base + 2, base, base + 2, base + 3]
        })
        .collect()
}

pub fn pack_vertex_buffer(vertices: &[Vertex]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(std::mem::size_of_val(vertices));
    for vertex in vertices {
        for component in vertex.position.iter().chain(vertex.normal.iter()) {
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
