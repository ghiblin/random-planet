use cucumber::{World as _, then, when};
use planet_core::subdivision::seed::Seed;
use planet_renderer::controls::seed_from_timestamp::seed_from_timestamp;

#[derive(Debug, Default, cucumber::World)]
pub struct SeedFromTimestampWorld {
    seed: Option<Seed>,
    seed_pair: Option<(Seed, Seed)>,
}

#[when(regex = r"^the timestamp (\S+) is converted to a Seed$")]
fn when_converted(world: &mut SeedFromTimestampWorld, raw: String) {
    let timestamp: f64 = raw.parse().expect("timestamp fixture must parse as f64");
    world.seed = Some(seed_from_timestamp(timestamp));
}

#[when(regex = r"^the timestamp (\S+) is converted to a Seed twice$")]
fn when_converted_twice(world: &mut SeedFromTimestampWorld, raw: String) {
    let timestamp: f64 = raw.parse().expect("timestamp fixture must parse as f64");
    world.seed_pair = Some((
        seed_from_timestamp(timestamp),
        seed_from_timestamp(timestamp),
    ));
}

#[then(regex = r"^the resulting Seed has value (\d+)$")]
fn then_seed_value(world: &mut SeedFromTimestampWorld, expected: u64) {
    let seed = world.seed.expect("Seed not converted");
    assert_eq!(seed.value(), expected);
}

#[then("the resulting Seed has the maximum u64 value")]
fn then_seed_max(world: &mut SeedFromTimestampWorld) {
    let seed = world.seed.expect("Seed not converted");
    assert_eq!(seed.value(), u64::MAX);
}

#[then("both resulting Seeds are equal")]
fn then_seeds_equal(world: &mut SeedFromTimestampWorld) {
    let (first, second) = world.seed_pair.expect("Seed pair not converted");
    assert_eq!(first, second);
}

#[tokio::main]
async fn main() {
    SeedFromTimestampWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/seed_from_timestamp.feature")
        .await;
}
