#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edge {
    pub start: usize,
    pub end: usize,
    pub face: usize,
}
