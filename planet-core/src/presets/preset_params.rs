use crate::color::color_gradient::ColorGradient;
use crate::processor::ocean_quota::OceanQuota;
use crate::processor::terrain_noise::TerrainNoise;

#[derive(Debug, Clone, PartialEq)]
pub struct PresetParams {
    terrain_noise: TerrainNoise,
    color_gradient: ColorGradient,
    ocean_quota: Option<OceanQuota>,
}

impl PresetParams {
    pub fn new(
        terrain_noise: TerrainNoise,
        color_gradient: ColorGradient,
        ocean_quota: Option<OceanQuota>,
    ) -> PresetParams {
        PresetParams {
            terrain_noise,
            color_gradient,
            ocean_quota,
        }
    }

    pub fn terrain_noise(&self) -> TerrainNoise {
        self.terrain_noise
    }

    pub fn color_gradient(&self) -> &ColorGradient {
        &self.color_gradient
    }

    pub fn ocean_quota(&self) -> Option<OceanQuota> {
        self.ocean_quota
    }
}
