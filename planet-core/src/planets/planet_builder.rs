use crate::geometry::mesh::Mesh;
use crate::presets::preset::Preset;
use crate::subdivision::seed::Seed;
use crate::subdivision::steps::Steps;
use crate::subdivision::subdivide::subdivide;
use crate::subdivision::subdivision_args::SubdivisionArgs;
use crate::subdivision::subdivision_mode::SubdivisionMode;

use super::planet::{Planet, PlanetError};

pub type GenerationProgress = Box<dyn FnMut(&Mesh, usize)>;

#[derive(Default)]
pub struct PlanetBuilder {
    preset: Option<Preset>,
    seed: Option<Seed>,
    max_depth: Option<Steps>,
    on_progress: Option<GenerationProgress>,
}

impl PlanetBuilder {
    pub fn with_preset(mut self, preset: Preset) -> Self {
        self.preset = Some(preset);
        self
    }

    pub fn with_seed(mut self, seed: Seed) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn with_max_depth(mut self, max_depth: Steps) -> Self {
        self.max_depth = Some(max_depth);
        self
    }

    pub fn with_on_progress(mut self, on_progress: GenerationProgress) -> Self {
        self.on_progress = Some(on_progress);
        self
    }

    pub fn build(self) -> Result<Planet, PlanetError> {
        let preset = self.preset.unwrap_or_default();
        let seed = self.seed.unwrap_or_default();
        let max_depth = self.max_depth.unwrap_or_default();
        let params = preset.params();
        let base = Mesh::icosahedron()?;
        let mut on_progress = self.on_progress;
        if let Some(callback) = on_progress.as_mut() {
            callback(&base, 0);
        }
        let args = SubdivisionArgs::new(
            Some(max_depth),
            Some(SubdivisionMode::RedGreenSplit {
                seed,
                elevation_noise_range: params.elevation_noise_range(),
                normal_noise_range: params.normal_noise_range(),
                min_edge_length: params.min_edge_length(),
                split_point_variance: params.split_point_variance(),
            }),
            on_progress,
        );
        let mesh = subdivide(&base, args)?;
        let colors = mesh
            .vertices()
            .iter()
            .map(|vertex| params.color_gradient().sample(vertex.position.length()))
            .collect();
        Ok(Planet {
            mesh,
            colors,
            preset,
        })
    }
}
