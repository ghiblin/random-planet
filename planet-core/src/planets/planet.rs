use std::fmt;

use crate::color::rgb::Rgb;
use crate::geometry::mesh::{Mesh, MeshError};
use crate::presets::preset::Preset;
use crate::presets::preset_params::PresetParams;
use crate::processor::compose_mesh::compose_mesh;
use crate::processor::identity_mesh::identity_mesh;
use crate::processor::mesh_processor::MeshProcessor;
use crate::processor::ocean_quota::apply_ocean_quota;
use crate::processor::terrain_noise::apply_terrain_noise;
use crate::subdivision::seed::Seed;
use crate::subdivision::steps::Steps;
use crate::subdivision::subdivide::subdivide;
use crate::subdivision::subdivision_args::SubdivisionArgs;
use crate::subdivision::subdivision_mode::SubdivisionMode;

use super::planet_builder::PlanetBuilder;

pub type GenerationProgress = Box<dyn FnMut(&Mesh, usize)>;

#[derive(Debug, Clone, PartialEq)]
pub struct Planet {
    pub(crate) mesh: Mesh,
    pub(crate) colors: Vec<Rgb>,
    pub(crate) preset: Preset,
    pub(crate) seed: Seed,
    pub(crate) max_depth: Option<Steps>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlanetError {
    Mesh(MeshError),
}

impl fmt::Display for PlanetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlanetError::Mesh(error) => write!(f, "planet generation failed: {error}"),
        }
    }
}

impl std::error::Error for PlanetError {}

impl From<MeshError> for PlanetError {
    fn from(error: MeshError) -> PlanetError {
        PlanetError::Mesh(error)
    }
}

fn postprocessing_pipeline(params: &PresetParams, seed: Seed) -> MeshProcessor {
    let terrain_noise = params.terrain_noise();
    let mut pipeline = compose_mesh(
        identity_mesh(),
        Box::new(move |mesh: &Mesh| apply_terrain_noise(mesh, seed, terrain_noise)),
    );
    if let Some(quota) = params.ocean_quota() {
        pipeline = compose_mesh(
            pipeline,
            Box::new(move |mesh: &Mesh| apply_ocean_quota(mesh, quota)),
        );
    }
    pipeline
}

impl Planet {
    pub fn builder() -> PlanetBuilder {
        PlanetBuilder::default()
    }

    pub fn subdivide(
        &self,
        max_depth: Steps,
        on_progress: Option<GenerationProgress>,
    ) -> Result<Planet, PlanetError> {
        let params = self.preset.params();
        let mut on_progress = on_progress;
        if let Some(callback) = on_progress.as_mut() {
            callback(&self.mesh, 0);
        }
        let args = SubdivisionArgs::new(
            Some(max_depth),
            Some(SubdivisionMode::UniformRedSplit { seed: self.seed }),
            on_progress,
        );
        let mesh = subdivide(&self.mesh, args)?;
        let mesh = postprocessing_pipeline(&params, self.seed)(&mesh)?;
        let colors = mesh
            .vertices()
            .iter()
            .map(|vertex| params.color_gradient().sample(vertex.position.length()))
            .collect();
        Ok(Planet {
            mesh,
            colors,
            preset: self.preset,
            seed: self.seed,
            max_depth: Some(max_depth),
        })
    }

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn colors(&self) -> &[Rgb] {
        &self.colors
    }

    pub fn preset(&self) -> Preset {
        self.preset
    }

    pub fn seed(&self) -> Seed {
        self.seed
    }

    pub fn max_depth(&self) -> Option<Steps> {
        self.max_depth
    }
}
