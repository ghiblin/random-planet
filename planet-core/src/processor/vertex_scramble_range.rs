use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VertexScrambleRange {
    low: f32,
    high: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VertexScrambleRangeError {
    InvalidRange { low: f32, high: f32 },
    LowAtOrBelowNegativeOne { low: f32 },
}

impl fmt::Display for VertexScrambleRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VertexScrambleRangeError::InvalidRange { low, high } => {
                write!(f, "invalid vertex scramble range: low {low} high {high}")
            }
            VertexScrambleRangeError::LowAtOrBelowNegativeOne { low } => {
                write!(f, "vertex scramble range low {low} must be above -1.0")
            }
        }
    }
}

impl std::error::Error for VertexScrambleRangeError {}

const DEFAULT_VERTEX_SCRAMBLE_LOW: f32 = -0.05;
const DEFAULT_VERTEX_SCRAMBLE_HIGH: f32 = 0.05;

impl VertexScrambleRange {
    pub fn new(low: f32, high: f32) -> Result<VertexScrambleRange, VertexScrambleRangeError> {
        if low > -1.0 && low <= high {
            Ok(VertexScrambleRange { low, high })
        } else if low <= -1.0 {
            Err(VertexScrambleRangeError::LowAtOrBelowNegativeOne { low })
        } else {
            Err(VertexScrambleRangeError::InvalidRange { low, high })
        }
    }

    pub fn low(&self) -> f32 {
        self.low
    }

    pub fn high(&self) -> f32 {
        self.high
    }
}

impl Default for VertexScrambleRange {
    fn default() -> Self {
        VertexScrambleRange {
            low: DEFAULT_VERTEX_SCRAMBLE_LOW,
            high: DEFAULT_VERTEX_SCRAMBLE_HIGH,
        }
    }
}
