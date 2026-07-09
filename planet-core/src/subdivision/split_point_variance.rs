use std::fmt;

const DEFAULT_SPLIT_POINT_VARIANCE: f32 = 0.1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplitPointVariance(pub(crate) f32);

#[derive(Debug, Clone, PartialEq)]
pub enum SplitPointVarianceError {
    Negative { value: f32 },
}

impl fmt::Display for SplitPointVarianceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SplitPointVarianceError::Negative { value } => {
                write!(f, "split point variance must not be negative, got {value}")
            }
        }
    }
}

impl std::error::Error for SplitPointVarianceError {}

impl SplitPointVariance {
    pub fn new(value: f32) -> Result<SplitPointVariance, SplitPointVarianceError> {
        if value >= 0.0 {
            Ok(SplitPointVariance(value))
        } else {
            Err(SplitPointVarianceError::Negative { value })
        }
    }

    pub fn value(&self) -> f32 {
        self.0
    }
}

impl Default for SplitPointVariance {
    fn default() -> Self {
        SplitPointVariance(DEFAULT_SPLIT_POINT_VARIANCE)
    }
}
