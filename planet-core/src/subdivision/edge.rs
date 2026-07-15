use crate::geometry::vertex::Vertex;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct EdgeKey {
    pub(crate) low: usize,
    pub(crate) high: usize,
}

impl EdgeKey {
    pub(crate) fn new(a: usize, b: usize) -> EdgeKey {
        EdgeKey {
            low: a.min(b),
            high: a.max(b),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct EdgeCache {
    midpoints: HashMap<EdgeKey, usize>,
}

impl EdgeCache {
    pub(crate) fn new() -> EdgeCache {
        EdgeCache::default()
    }

    pub(crate) fn get_or_insert_with(
        &mut self,
        a: usize,
        b: usize,
        vertices: &mut Vec<Vertex>,
        compute: impl FnOnce(&Vertex, &Vertex) -> Vertex,
    ) -> usize {
        let key = EdgeKey::new(a, b);
        if let Some(&index) = self.midpoints.get(&key) {
            return index;
        }
        let vertex = compute(&vertices[a], &vertices[b]);
        let index = vertices.len();
        vertices.push(vertex);
        self.midpoints.insert(key, index);
        index
    }
}
