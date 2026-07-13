use crate::color::color_gradient::ColorGradient;
use crate::color::rgb::Rgb;
use crate::processor::ocean_quota::OceanQuota;
use crate::subdivision::elevation_noise_range::ElevationNoiseRange;
use crate::subdivision::min_edge_length::MinEdgeLength;
use crate::subdivision::normal_noise_range::NormalNoiseRange;
use crate::subdivision::split_point_variance::SplitPointVariance;

use super::preset_params::PresetParams;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Preset {
    #[default]
    Earthy,
    Volcano,
    Rocky,
}

impl Preset {
    pub fn params(&self) -> PresetParams {
        match self {
            Preset::Earthy => PresetParams::new(
                MinEdgeLength(0.35),
                ElevationNoiseRange {
                    low: -0.05,
                    high: 0.15,
                },
                NormalNoiseRange {
                    low: -0.05,
                    high: 0.05,
                },
                SplitPointVariance(0.1),
                ColorGradient {
                    stops: vec![
                        (
                            0.85,
                            Rgb {
                                r: 0.05,
                                g: 0.15,
                                b: 0.45,
                            },
                        ), // deep water
                        (
                            0.95,
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
                            1.05,
                            Rgb {
                                r: 0.25,
                                g: 0.55,
                                b: 0.20,
                            },
                        ), // grassland
                        (
                            1.10,
                            Rgb {
                                r: 0.45,
                                g: 0.35,
                                b: 0.25,
                            },
                        ), // hills
                        (
                            1.15,
                            Rgb {
                                r: 0.95,
                                g: 0.95,
                                b: 0.95,
                            },
                        ), // snow cap
                    ],
                },
                Some(OceanQuota(0.4)),
            ),
            Preset::Volcano => PresetParams::new(
                MinEdgeLength(0.25),
                ElevationNoiseRange {
                    low: -0.05,
                    high: 0.35,
                },
                NormalNoiseRange {
                    low: -0.10,
                    high: 0.10,
                },
                SplitPointVariance(0.2),
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
            ),
            Preset::Rocky => PresetParams::new(
                MinEdgeLength(0.3),
                ElevationNoiseRange {
                    low: -0.2,
                    high: 0.2,
                },
                NormalNoiseRange {
                    low: -0.15,
                    high: 0.15,
                },
                SplitPointVariance(0.25),
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
            ),
        }
    }
}
