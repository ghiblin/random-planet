#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Seed(u64);

impl Seed {
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl From<u64> for Seed {
    fn from(value: u64) -> Seed {
        Seed(value)
    }
}
