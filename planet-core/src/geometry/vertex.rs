use super::vec3::Vec3;

#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub edges: Vec<usize>,
}

impl Vertex {
    pub(crate) fn at(position: Vec3) -> Vertex {
        Vertex {
            position,
            normal: Vec3::new(0.0, 0.0, 0.0),
            edges: Vec::new(),
        }
    }
}
