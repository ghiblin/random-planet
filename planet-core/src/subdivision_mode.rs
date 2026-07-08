use crate::subdivide::SubdivisionStrategy;
use crate::uniform_red_split::UniformRedSplit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SubdivisionMode {
    #[default]
    UniformRedSplit,
}

impl SubdivisionMode {
    pub(crate) fn strategy(&self) -> Box<dyn SubdivisionStrategy> {
        match self {
            SubdivisionMode::UniformRedSplit => Box::new(UniformRedSplit),
        }
    }
}
