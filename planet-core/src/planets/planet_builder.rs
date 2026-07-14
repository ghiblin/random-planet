use crate::geometry::mesh::Mesh;
use crate::presets::preset::Preset;
use crate::processor::vertex_scramble::scramble_vertices;
use crate::processor::vertex_scramble_range::VertexScrambleRange;
use crate::subdivision::seed::Seed;

use super::planet::{Planet, PlanetError};

#[derive(Default)]
pub struct PlanetBuilder {
    preset: Option<Preset>,
    seed: Option<Seed>,
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

    pub fn build(self) -> Result<Planet, PlanetError> {
        let preset = self.preset.unwrap_or_default();
        let seed = self.seed.unwrap_or_default();
        let mesh = Mesh::icosahedron()?;
        let mesh = scramble_vertices(&mesh, seed, VertexScrambleRange::default())?;
        let colors = mesh
            .vertices()
            .iter()
            .map(|vertex| {
                preset
                    .params()
                    .color_gradient()
                    .sample(vertex.position.length())
            })
            .collect();
        Ok(Planet {
            mesh,
            colors,
            preset,
            seed,
            max_depth: None,
        })
    }
}
