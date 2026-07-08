use std::fmt;

const DEFAULT_ELEVATION_NOISE_LOW: f32 = -0.05;
const DEFAULT_ELEVATION_NOISE_HIGH: f32 = 0.05;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElevationNoiseRange {
    low: f32,
    high: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElevationNoiseRangeError {
    InvalidRange { low: f32, high: f32 },
}

impl fmt::Display for ElevationNoiseRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ElevationNoiseRangeError::InvalidRange { low, high } => {
                write!(f, "invalid elevation noise range: low {low} high {high}")
            }
        }
    }
}

impl std::error::Error for ElevationNoiseRangeError {}

impl ElevationNoiseRange {
    pub fn new(low: f32, high: f32) -> Result<ElevationNoiseRange, ElevationNoiseRangeError> {
        if low <= high {
            Ok(ElevationNoiseRange { low, high })
        } else {
            Err(ElevationNoiseRangeError::InvalidRange { low, high })
        }
    }

    pub fn low(&self) -> f32 {
        self.low
    }

    pub fn high(&self) -> f32 {
        self.high
    }
}

impl Default for ElevationNoiseRange {
    fn default() -> Self {
        ElevationNoiseRange {
            low: DEFAULT_ELEVATION_NOISE_LOW,
            high: DEFAULT_ELEVATION_NOISE_HIGH,
        }
    }
}
