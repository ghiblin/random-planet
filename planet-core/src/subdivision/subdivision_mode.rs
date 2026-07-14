use super::strategies::uniform_red_split::UniformRedSplit;
use super::subdivide::SubdivisionStrategy;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SubdivisionMode {
    #[default]
    UniformRedSplit,
}

impl SubdivisionMode {
    pub(crate) fn strategy(&self) -> Box<dyn SubdivisionStrategy> {
        match self {
            SubdivisionMode::UniformRedSplit => Box::new(UniformRedSplit::new()),
        }
    }
}
