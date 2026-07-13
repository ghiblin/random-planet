use crate::color::color_gradient::ColorGradient;
use crate::processor::ocean_quota::OceanQuota;
use crate::subdivision::elevation_noise_range::ElevationNoiseRange;
use crate::subdivision::min_edge_length::MinEdgeLength;
use crate::subdivision::normal_noise_range::NormalNoiseRange;
use crate::subdivision::split_point_variance::SplitPointVariance;

#[derive(Debug, Clone, PartialEq)]
pub struct PresetParams {
    min_edge_length: MinEdgeLength,
    elevation_noise_range: ElevationNoiseRange,
    normal_noise_range: NormalNoiseRange,
    split_point_variance: SplitPointVariance,
    color_gradient: ColorGradient,
    ocean_quota: Option<OceanQuota>,
}

impl PresetParams {
    pub fn new(
        min_edge_length: MinEdgeLength,
        elevation_noise_range: ElevationNoiseRange,
        normal_noise_range: NormalNoiseRange,
        split_point_variance: SplitPointVariance,
        color_gradient: ColorGradient,
        ocean_quota: Option<OceanQuota>,
    ) -> PresetParams {
        PresetParams {
            min_edge_length,
            elevation_noise_range,
            normal_noise_range,
            split_point_variance,
            color_gradient,
            ocean_quota,
        }
    }

    pub fn min_edge_length(&self) -> MinEdgeLength {
        self.min_edge_length
    }

    pub fn elevation_noise_range(&self) -> ElevationNoiseRange {
        self.elevation_noise_range
    }

    pub fn normal_noise_range(&self) -> NormalNoiseRange {
        self.normal_noise_range
    }

    pub fn split_point_variance(&self) -> SplitPointVariance {
        self.split_point_variance
    }

    pub fn color_gradient(&self) -> &ColorGradient {
        &self.color_gradient
    }

    pub fn ocean_quota(&self) -> Option<OceanQuota> {
        self.ocean_quota
    }
}
