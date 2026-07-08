use super::elevation_noise_range::ElevationNoiseRange;
use super::radial_random_split::RadialRandomSplit;
use super::seed::Seed;
use super::subdivide::SubdivisionStrategy;
use super::uniform_red_split::UniformRedSplit;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SubdivisionMode {
    #[default]
    UniformRedSplit,
    RadialRandomSplit {
        seed: Seed,
        elevation_noise_range: ElevationNoiseRange,
    },
}

impl SubdivisionMode {
    pub(crate) fn strategy(&self) -> Box<dyn SubdivisionStrategy> {
        match self {
            SubdivisionMode::UniformRedSplit => Box::new(UniformRedSplit),
            SubdivisionMode::RadialRandomSplit {
                seed,
                elevation_noise_range,
            } => Box::new(RadialRandomSplit::new(*seed, *elevation_noise_range)),
        }
    }
}
