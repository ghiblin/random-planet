use std::fmt;

const DEFAULT_MIN_EDGE_LENGTH: f32 = 0.1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinEdgeLength(pub(crate) f32);

#[derive(Debug, Clone, PartialEq)]
pub enum MinEdgeLengthError {
    Negative { value: f32 },
}

impl fmt::Display for MinEdgeLengthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MinEdgeLengthError::Negative { value } => {
                write!(f, "min edge length must not be negative, got {value}")
            }
        }
    }
}

impl std::error::Error for MinEdgeLengthError {}

impl MinEdgeLength {
    pub fn new(value: f32) -> Result<MinEdgeLength, MinEdgeLengthError> {
        if value >= 0.0 {
            Ok(MinEdgeLength(value))
        } else {
            Err(MinEdgeLengthError::Negative { value })
        }
    }

    pub fn value(&self) -> f32 {
        self.0
    }
}

impl Default for MinEdgeLength {
    fn default() -> Self {
        MinEdgeLength(DEFAULT_MIN_EDGE_LENGTH)
    }
}
