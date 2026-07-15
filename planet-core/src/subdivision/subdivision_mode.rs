use super::seed::Seed;
use super::strategies::uniform_red_split::UniformRedSplit;
use super::subdivide::SubdivisionStrategy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SubdivisionMode {
    #[default]
    UniformRedSplit,
}

impl SubdivisionMode {
    pub(crate) fn strategy(&self, seed: Seed) -> Box<dyn SubdivisionStrategy> {
        match self {
            SubdivisionMode::UniformRedSplit => Box::new(UniformRedSplit::new(seed)),
        }
    }
}
