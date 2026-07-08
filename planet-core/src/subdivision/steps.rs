use std::fmt;

pub const MAX_SUBDIVISION_STEPS: usize = 8;
const DEFAULT_STEPS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Steps(usize);

#[derive(Debug, Clone, PartialEq)]
pub enum StepsError {
    ExceedsMaximum { steps: usize, max: usize },
}

impl fmt::Display for StepsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StepsError::ExceedsMaximum { steps, max } => {
                write!(f, "requested {steps} steps exceeds maximum of {max}")
            }
        }
    }
}

impl std::error::Error for StepsError {}

impl Steps {
    pub fn new(steps: usize) -> Result<Steps, StepsError> {
        if steps > MAX_SUBDIVISION_STEPS {
            return Err(StepsError::ExceedsMaximum {
                steps,
                max: MAX_SUBDIVISION_STEPS,
            });
        }
        Ok(Steps(steps))
    }

    pub fn value(&self) -> usize {
        self.0
    }
}

impl Default for Steps {
    fn default() -> Self {
        Steps(DEFAULT_STEPS)
    }
}
