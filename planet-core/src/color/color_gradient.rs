use std::fmt;

use super::rgb::Rgb;

#[derive(Debug, Clone, PartialEq)]
pub struct ColorGradient {
    pub(crate) stops: Vec<(f32, Rgb)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorGradientError {
    TooFewStops { count: usize },
    StopsNotStrictlyAscending { index: usize },
}

impl fmt::Display for ColorGradientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorGradientError::TooFewStops { count } => {
                write!(f, "color gradient needs at least 2 stops, got {count}")
            }
            ColorGradientError::StopsNotStrictlyAscending { index } => {
                write!(
                    f,
                    "color gradient stops must be strictly ascending by elevation, stop {index} is not"
                )
            }
        }
    }
}

impl std::error::Error for ColorGradientError {}

impl ColorGradient {
    pub fn new(stops: Vec<(f32, Rgb)>) -> Result<ColorGradient, ColorGradientError> {
        if stops.len() < 2 {
            return Err(ColorGradientError::TooFewStops { count: stops.len() });
        }
        for index in 1..stops.len() {
            let ascending =
                stops[index - 1].0.partial_cmp(&stops[index].0) == Some(std::cmp::Ordering::Less);
            if !ascending {
                return Err(ColorGradientError::StopsNotStrictlyAscending { index });
            }
        }
        Ok(ColorGradient { stops })
    }

    pub fn sample(&self, elevation: f32) -> Rgb {
        let last = self.stops.len() - 1;
        if elevation <= self.stops[0].0 {
            return self.stops[0].1;
        }
        if elevation >= self.stops[last].0 {
            return self.stops[last].1;
        }
        for index in 0..last {
            let (e0, c0) = self.stops[index];
            let (e1, c1) = self.stops[index + 1];
            if elevation == e1 {
                return c1;
            }
            if elevation >= e0 && elevation <= e1 {
                let t = (elevation - e0) / (e1 - e0);
                return Rgb {
                    r: c0.r() + (c1.r() - c0.r()) * t,
                    g: c0.g() + (c1.g() - c0.g()) * t,
                    b: c0.b() + (c1.b() - c0.b()) * t,
                };
            }
        }
        self.stops[last].1
    }
}
