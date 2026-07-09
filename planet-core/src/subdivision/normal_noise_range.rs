use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NormalNoiseRange {
    low: f32,
    high: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NormalNoiseRangeError {
    InvalidRange { low: f32, high: f32 },
}

impl fmt::Display for NormalNoiseRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NormalNoiseRangeError::InvalidRange { low, high } => {
                write!(f, "invalid normal noise range: low {low} high {high}")
            }
        }
    }
}

impl std::error::Error for NormalNoiseRangeError {}

const DEFAULT_NORMAL_NOISE_LOW: f32 = -0.05;
const DEFAULT_NORMAL_NOISE_HIGH: f32 = 0.05;

impl NormalNoiseRange {
    pub fn new(low: f32, high: f32) -> Result<NormalNoiseRange, NormalNoiseRangeError> {
        if low <= high {
            Ok(NormalNoiseRange { low, high })
        } else {
            Err(NormalNoiseRangeError::InvalidRange { low, high })
        }
    }

    pub fn low(&self) -> f32 {
        self.low
    }

    pub fn high(&self) -> f32 {
        self.high
    }
}

impl Default for NormalNoiseRange {
    fn default() -> Self {
        NormalNoiseRange {
            low: DEFAULT_NORMAL_NOISE_LOW,
            high: DEFAULT_NORMAL_NOISE_HIGH,
        }
    }
}
