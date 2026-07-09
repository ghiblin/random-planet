use super::elevation_noise_range::ElevationNoiseRange;
use super::min_edge_length::MinEdgeLength;
use super::seed::Seed;
use super::split_point_variance::SplitPointVariance;
use super::strategies::radial_random_split::RadialRandomSplit;
use super::strategies::red_green_split::RedGreenSplit;
use super::strategies::uniform_red_split::UniformRedSplit;
use super::subdivide::SubdivisionStrategy;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SubdivisionMode {
    #[default]
    UniformRedSplit,
    RadialRandomSplit {
        seed: Seed,
        elevation_noise_range: ElevationNoiseRange,
    },
    RedGreenSplit {
        seed: Seed,
        elevation_noise_range: ElevationNoiseRange,
        min_edge_length: MinEdgeLength,
        split_point_variance: SplitPointVariance,
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
            SubdivisionMode::RedGreenSplit {
                seed,
                elevation_noise_range,
                min_edge_length,
                split_point_variance,
            } => Box::new(RedGreenSplit::new(
                *seed,
                *elevation_noise_range,
                *min_edge_length,
                *split_point_variance,
            )),
        }
    }
}
