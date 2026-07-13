use planet_core::subdivision::seed::Seed;

pub fn seed_from_timestamp(timestamp_millis: f64) -> Seed {
    Seed::from(timestamp_millis as u64)
}
