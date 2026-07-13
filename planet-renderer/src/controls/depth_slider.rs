use std::fmt;

use planet_core::subdivision::steps::{MAX_SUBDIVISION_STEPS, Steps, StepsError};

pub const MIN_DEPTH: usize = 0;
pub const MAX_DEPTH: usize = MAX_SUBDIVISION_STEPS;

#[derive(Debug, Clone, PartialEq)]
pub enum DepthParseError {
    NotANumber { value: String },
    InvalidSteps(StepsError),
}

impl fmt::Display for DepthParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DepthParseError::NotANumber { value } => {
                write!(f, "depth value {value:?} is not a number")
            }
            DepthParseError::InvalidSteps(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for DepthParseError {}

impl From<StepsError> for DepthParseError {
    fn from(error: StepsError) -> DepthParseError {
        DepthParseError::InvalidSteps(error)
    }
}

pub fn parse_depth(value: &str) -> Result<Steps, DepthParseError> {
    let raw: usize = value.parse().map_err(|_| DepthParseError::NotANumber {
        value: value.to_string(),
    })?;
    Ok(Steps::new(raw)?)
}
