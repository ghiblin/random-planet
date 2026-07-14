use std::fmt;

use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

use crate::geometry::mesh::{Mesh, MeshError, Vertex};
use crate::subdivision::seed::Seed;

pub(crate) const MIN_VERTEX_RADIUS: f32 = 0.05;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerrainNoise {
    pub(crate) frequency: f32,
    pub(crate) octaves: u32,
    pub(crate) persistence: f32,
    pub(crate) lacunarity: f32,
    pub(crate) amplitude: f32,
    pub(crate) redistribution_exponent: f32,
    pub(crate) terrace_levels: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TerrainNoiseError {
    InvalidFrequency { frequency: f32 },
    InvalidOctaves { octaves: u32 },
    InvalidPersistence { persistence: f32 },
    InvalidLacunarity { lacunarity: f32 },
    InvalidAmplitude { amplitude: f32 },
    InvalidRedistributionExponent { redistribution_exponent: f32 },
    InvalidTerraceLevels { terrace_levels: u32 },
}

impl fmt::Display for TerrainNoiseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for TerrainNoiseError {}

const MIN_OCTAVES: u32 = 1;
const MAX_OCTAVES: u32 = 8;
const MIN_LACUNARITY: f32 = 1.0;
const MAX_LACUNARITY: f32 = 4.0;
const MIN_TERRACE_LEVELS: u32 = 2;

impl TerrainNoise {
    pub fn new(
        frequency: f32,
        octaves: u32,
        persistence: f32,
        lacunarity: f32,
        amplitude: f32,
        redistribution_exponent: f32,
        terrace_levels: Option<u32>,
    ) -> Result<TerrainNoise, TerrainNoiseError> {
        if !(frequency.is_finite() && frequency > 0.0) {
            return Err(TerrainNoiseError::InvalidFrequency { frequency });
        }
        if !(MIN_OCTAVES..=MAX_OCTAVES).contains(&octaves) {
            return Err(TerrainNoiseError::InvalidOctaves { octaves });
        }
        if !(persistence.is_finite() && (0.0..=1.0).contains(&persistence)) {
            return Err(TerrainNoiseError::InvalidPersistence { persistence });
        }
        if !(lacunarity.is_finite() && lacunarity > MIN_LACUNARITY && lacunarity <= MAX_LACUNARITY)
        {
            return Err(TerrainNoiseError::InvalidLacunarity { lacunarity });
        }
        if !(amplitude.is_finite() && amplitude >= 0.0) {
            return Err(TerrainNoiseError::InvalidAmplitude { amplitude });
        }
        if !(redistribution_exponent.is_finite() && redistribution_exponent > 0.0) {
            return Err(TerrainNoiseError::InvalidRedistributionExponent {
                redistribution_exponent,
            });
        }
        if let Some(levels) = terrace_levels
            && levels < MIN_TERRACE_LEVELS
        {
            return Err(TerrainNoiseError::InvalidTerraceLevels {
                terrace_levels: levels,
            });
        }
        Ok(TerrainNoise {
            frequency,
            octaves,
            persistence,
            lacunarity,
            amplitude,
            redistribution_exponent,
            terrace_levels,
        })
    }

    pub fn frequency(&self) -> f32 {
        self.frequency
    }

    pub fn octaves(&self) -> u32 {
        self.octaves
    }

    pub fn persistence(&self) -> f32 {
        self.persistence
    }

    pub fn lacunarity(&self) -> f32 {
        self.lacunarity
    }

    pub fn amplitude(&self) -> f32 {
        self.amplitude
    }

    pub fn redistribution_exponent(&self) -> f32 {
        self.redistribution_exponent
    }

    pub fn terrace_levels(&self) -> Option<u32> {
        self.terrace_levels
    }
}

pub fn apply_terrain_noise(
    mesh: &Mesh,
    seed: Seed,
    terrain_noise: TerrainNoise,
) -> Result<Mesh, MeshError> {
    let noise = Fbm::<Perlin>::new(seed.value() as u32)
        .set_frequency(terrain_noise.frequency() as f64)
        .set_octaves(terrain_noise.octaves() as usize)
        .set_persistence(terrain_noise.persistence() as f64)
        .set_lacunarity(terrain_noise.lacunarity() as f64);

    let vertices = mesh
        .vertices()
        .iter()
        .map(|vertex| {
            let Some(direction) = vertex.position.normalized() else {
                return *vertex;
            };
            let raw = noise.get([direction.x as f64, direction.y as f64, direction.z as f64]);
            let clamped = (raw as f32).clamp(-1.0, 1.0);
            let mut signed =
                clamped.signum() * clamped.abs().powf(terrain_noise.redistribution_exponent());
            if let Some(levels) = terrain_noise.terrace_levels() {
                // Quantize into exactly `levels` bands (bin centers) rather than
                // rounding to the nearest multiple of 1/levels, which would produce
                // up to 2*levels+1 distinct values over the signed [-1, 1] range.
                let levels_f = levels as f32;
                let unit = (signed + 1.0) / 2.0;
                let bin = (unit * levels_f).floor().min(levels_f - 1.0);
                let bin_center_unit = (bin + 0.5) / levels_f;
                signed = bin_center_unit * 2.0 - 1.0;
            }
            let new_radius = (1.0 + signed * terrain_noise.amplitude()).max(MIN_VERTEX_RADIUS);
            Vertex {
                position: direction.scale(new_radius),
            }
        })
        .collect();

    Mesh::new(vertices, mesh.triangles().to_vec())
}
