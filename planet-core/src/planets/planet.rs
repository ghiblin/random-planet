use std::fmt;

use crate::color::color_gradient::ColorGradient;
use crate::color::rgb::Rgb;
use crate::geometry::mesh::{Mesh, MeshError};
use crate::presets::preset::Preset;
use crate::processor::finalize_normals::finalize_normals;
use crate::processor::ocean_quota::apply_ocean_quota;
use crate::processor::terrain_noise::apply_terrain_noise_for_round;
use crate::subdivision::seed::Seed;
use crate::subdivision::steps::Steps;
use crate::subdivision::subdivide::subdivide;
use crate::subdivision::subdivision_args::{SubdivisionArgs, UpdateCallback};

use super::planet_builder::PlanetBuilder;
use super::postprocess_stage::PostprocessStage;

pub type GenerationProgress = Box<dyn FnMut(&Mesh, usize)>;
pub type PostprocessProgress = Box<dyn FnMut(PostprocessStage)>;

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

fn vertex_color(radius: f32, sea_level: Option<f32>, gradient: &ColorGradient) -> Rgb {
    const SEA_LEVEL_TOLERANCE: f32 = 1e-4;
    match sea_level {
        Some(sea_level) if (radius - sea_level).abs() <= SEA_LEVEL_TOLERANCE => {
            gradient.sample(f32::NEG_INFINITY)
        }
        _ => gradient.sample(radius),
    }
}

impl Planet {
    pub fn builder() -> PlanetBuilder {
        PlanetBuilder::default()
    }

    pub fn subdivide(
        &self,
        max_depth: Steps,
        on_progress: Option<GenerationProgress>,
        on_postprocess: Option<PostprocessProgress>,
    ) -> Result<Planet, PlanetError> {
        let params = self.preset.params();
        let terrain_noise = params.terrain_noise();
        let seed = self.seed;
        let mut on_progress = on_progress;
        if let Some(callback) = on_progress.as_mut() {
            callback(&self.mesh, 0);
        }
        let update_cb: UpdateCallback = Box::new(move |mesh, round| {
            let revealed_octaves = (round as u32).min(terrain_noise.octaves());
            let noised =
                apply_terrain_noise_for_round(&mesh, seed, terrain_noise, revealed_octaves)?;
            if let Some(callback) = on_progress.as_mut() {
                callback(&noised, round);
            }
            Ok(noised)
        });
        let args = SubdivisionArgs::new(
            Some(max_depth),
            Some(params.subdivision_mode()),
            Some(seed),
            Some(update_cb),
        );
        let mesh = subdivide(&self.mesh, args)?;

        let mut on_postprocess = on_postprocess;
        let mesh = match params.ocean_quota() {
            Some(quota) => {
                if let Some(callback) = on_postprocess.as_mut() {
                    callback(PostprocessStage::OceanQuota);
                }
                apply_ocean_quota(&mesh, quota)?
            }
            None => mesh,
        };
        let sea_level = params.ocean_quota().map(|_| {
            mesh.vertices()
                .iter()
                .map(|vertex| vertex.position.length())
                .fold(f32::INFINITY, f32::min)
        });
        let colors = mesh
            .vertices()
            .iter()
            .map(|vertex| {
                vertex_color(vertex.position.length(), sea_level, params.color_gradient())
            })
            .collect();
        let mesh = finalize_normals(&mesh);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::rgb::Rgb;

    fn gradient() -> ColorGradient {
        ColorGradient::new(vec![
            (0.7, Rgb::new(0.0, 0.0, 1.0).unwrap()),
            (1.3, Rgb::new(1.0, 1.0, 1.0).unwrap()),
        ])
        .unwrap()
    }

    #[test]
    fn samples_first_stop_when_at_sea_level() {
        let gradient = gradient();
        let color = vertex_color(0.997, Some(0.997), &gradient);
        assert_eq!(color, gradient.sample(f32::NEG_INFINITY));
    }

    #[test]
    fn samples_first_stop_when_within_tolerance_of_sea_level() {
        let gradient = gradient();
        let color = vertex_color(0.99705, Some(0.9970), &gradient);
        assert_eq!(color, gradient.sample(f32::NEG_INFINITY));
    }

    #[test]
    fn samples_own_radius_when_outside_tolerance_of_sea_level() {
        let gradient = gradient();
        let color = vertex_color(1.1, Some(0.997), &gradient);
        assert_eq!(color, gradient.sample(1.1));
    }

    #[test]
    fn samples_own_radius_when_no_sea_level() {
        let gradient = gradient();
        let color = vertex_color(0.997, None, &gradient);
        assert_eq!(color, gradient.sample(0.997));
    }
}
