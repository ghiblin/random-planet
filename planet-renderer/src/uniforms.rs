pub fn pack_view_projection_uniform(matrix: &[[f32; 4]; 4]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(64);
    for column in matrix {
        for component in column {
            bytes.extend_from_slice(&component.to_le_bytes());
        }
    }
    bytes
}
