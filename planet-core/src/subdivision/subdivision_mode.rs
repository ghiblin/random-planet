use super::seed::Seed;
use super::strategies::uniform_red_split::UniformRedSplit;
use super::subdivide::SubdivisionStrategy;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SubdivisionMode {
    UniformRedSplit { seed: Seed },
}

impl Default for SubdivisionMode {
    fn default() -> SubdivisionMode {
        SubdivisionMode::UniformRedSplit {
            seed: Seed::default(),
        }
    }
}

impl SubdivisionMode {
    pub(crate) fn strategy(&self) -> Box<dyn SubdivisionStrategy> {
        match self {
            SubdivisionMode::UniformRedSplit { seed } => Box::new(UniformRedSplit::new(*seed)),
        }
    }
}
