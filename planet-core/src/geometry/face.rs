use super::vec3::Vec3;

#[derive(Debug, Clone, PartialEq)]
pub struct Face {
    pub edges: Vec<usize>,
    pub order: usize,
    pub normal: Vec3,
}
