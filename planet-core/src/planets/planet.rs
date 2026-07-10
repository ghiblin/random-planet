use crate::color::rgb::Rgb;
use crate::geometry::mesh::{Mesh, MeshError};
use crate::presets::preset::Preset;
use crate::subdivision::seed::Seed;
use crate::subdivision::steps::Steps;
use crate::subdivision::subdivide::subdivide;
use crate::subdivision::subdivision_args::SubdivisionArgs;
use crate::subdivision::subdivision_mode::SubdivisionMode;

pub type GenerationProgress = Box<dyn FnMut(&Mesh, usize)>;

#[derive(Debug, Clone, PartialEq)]
pub struct Planet {
    mesh: Mesh,
    colors: Vec<Rgb>,
    preset: Preset,
}

impl Planet {
    pub fn generate(
        preset: Preset,
        seed: Seed,
        max_depth: Steps,
        on_progress: Option<GenerationProgress>,
    ) -> Result<Planet, MeshError> {
        let params = preset.params();
        let base = Mesh::icosahedron()?;
        let mut on_progress = on_progress;
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

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn colors(&self) -> &[Rgb] {
        &self.colors
    }

    pub fn preset(&self) -> Preset {
        self.preset
    }
}
