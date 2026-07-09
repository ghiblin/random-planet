use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    pub(crate) r: f32,
    pub(crate) g: f32,
    pub(crate) b: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RgbError {
    OutOfRange { r: f32, g: f32, b: f32 },
}

impl fmt::Display for RgbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RgbError::OutOfRange { r, g, b } => {
                write!(
                    f,
                    "rgb channels must be within 0.0..=1.0, got r {r} g {g} b {b}"
                )
            }
        }
    }
}

impl std::error::Error for RgbError {}

impl Rgb {
    pub fn new(r: f32, g: f32, b: f32) -> Result<Rgb, RgbError> {
        let in_range = |v: f32| (0.0..=1.0).contains(&v);
        if in_range(r) && in_range(g) && in_range(b) {
            Ok(Rgb { r, g, b })
        } else {
            Err(RgbError::OutOfRange { r, g, b })
        }
    }

    pub fn r(&self) -> f32 {
        self.r
    }

    pub fn g(&self) -> f32 {
        self.g
    }

    pub fn b(&self) -> f32 {
        self.b
    }
}
