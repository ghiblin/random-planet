use crate::color::color_gradient::ColorGradient;
use crate::color::rgb::Rgb;
use crate::processor::ocean_quota::OceanQuota;
use crate::processor::terrain_noise::TerrainNoise;
use crate::subdivision::subdivision_mode::SubdivisionMode;

use super::preset_params::PresetParams;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Preset {
    #[default]
    Earthy,
    Volcano,
    Rocky,
}

impl Preset {
    pub const ALL: [Preset; 3] = [Preset::Earthy, Preset::Volcano, Preset::Rocky];

    pub fn name(&self) -> &'static str {
        match self {
            Preset::Earthy => "Earthy",
            Preset::Volcano => "Volcano",
            Preset::Rocky => "Rocky",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Preset::Earthy => "Oceans, grasslands, and snow-capped peaks.",
            Preset::Volcano => "Charred basalt and glowing molten rock.",
            Preset::Rocky => "Barren gray stone, no water or lava.",
        }
    }

    pub fn params(&self) -> PresetParams {
        match self {
            Preset::Earthy => PresetParams::new(
                TerrainNoise {
                    frequency: 1.5,
                    octaves: 4,
                    persistence: 0.5,
                    lacunarity: 2.0,
                    amplitude: 0.50,
                    redistribution_exponent: 1.4,
                    terrace_levels: None,
                },
                ColorGradient {
                    stops: vec![
                        (
                            0.70,
                            Rgb {
                                r: 0.05,
                                g: 0.15,
                                b: 0.45,
                            },
                        ), // deep water
                        (
                            0.90,
                            Rgb {
                                r: 0.20,
                                g: 0.50,
                                b: 0.60,
                            },
                        ), // shallow water
                        (
                            1.00,
                            Rgb {
                                r: 0.82,
                                g: 0.76,
                                b: 0.50,
                            },
                        ), // sand
                        (
                            1.10,
                            Rgb {
                                r: 0.25,
                                g: 0.55,
                                b: 0.20,
                            },
                        ), // grassland
                        (
                            1.20,
                            Rgb {
                                r: 0.45,
                                g: 0.35,
                                b: 0.25,
                            },
                        ), // hills
                        (
                            1.30,
                            Rgb {
                                r: 0.95,
                                g: 0.95,
                                b: 0.95,
                            },
                        ), // snow cap
                    ],
                },
                Some(OceanQuota(0.4)),
                SubdivisionMode::UniformRedSplit,
            ),
            Preset::Volcano => PresetParams::new(
                TerrainNoise {
                    frequency: 2.5,
                    octaves: 5,
                    persistence: 0.55,
                    lacunarity: 2.2,
                    amplitude: 0.30,
                    redistribution_exponent: 2.2,
                    terrace_levels: Some(6),
                },
                ColorGradient {
                    stops: vec![
                        (
                            0.95,
                            Rgb {
                                r: 0.10,
                                g: 0.05,
                                b: 0.05,
                            },
                        ), // dark basalt
                        (
                            1.00,
                            Rgb {
                                r: 0.25,
                                g: 0.05,
                                b: 0.02,
                            },
                        ), // charred rock
                        (
                            1.15,
                            Rgb {
                                r: 0.55,
                                g: 0.10,
                                b: 0.02,
                            },
                        ), // glowing rock
                        (
                            1.25,
                            Rgb {
                                r: 0.95,
                                g: 0.35,
                                b: 0.05,
                            },
                        ), // molten orange
                        (
                            1.35,
                            Rgb {
                                r: 1.00,
                                g: 0.85,
                                b: 0.30,
                            },
                        ), // lava-yellow peak
                    ],
                },
                None,
                SubdivisionMode::UniformRedSplit,
            ),
            Preset::Rocky => PresetParams::new(
                TerrainNoise {
                    frequency: 3.0,
                    octaves: 4,
                    persistence: 0.5,
                    lacunarity: 2.0,
                    amplitude: 0.22,
                    redistribution_exponent: 1.8,
                    terrace_levels: Some(8),
                },
                ColorGradient {
                    stops: vec![
                        (
                            0.80,
                            Rgb {
                                r: 0.30,
                                g: 0.28,
                                b: 0.26,
                            },
                        ), // dark gray
                        (
                            0.95,
                            Rgb {
                                r: 0.45,
                                g: 0.42,
                                b: 0.38,
                            },
                        ), // gray
                        (
                            1.00,
                            Rgb {
                                r: 0.55,
                                g: 0.52,
                                b: 0.48,
                            },
                        ), // mid gray
                        (
                            1.10,
                            Rgb {
                                r: 0.68,
                                g: 0.64,
                                b: 0.58,
                            },
                        ), // light gray
                        (
                            1.20,
                            Rgb {
                                r: 0.80,
                                g: 0.78,
                                b: 0.74,
                            },
                        ), // pale peak
                    ],
                },
                None,
                SubdivisionMode::UniformRedSplit,
            ),
        }
    }
}
