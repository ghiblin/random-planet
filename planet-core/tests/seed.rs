use cucumber::{World as _, given, then, when};
use planet_core::subdivision::seed::Seed;

#[derive(Debug, Default, cucumber::World)]
pub struct SeedWorld {
    seed: Option<Seed>,
}

impl SeedWorld {
    fn seed(&self) -> Seed {
        self.seed.expect("Seed not constructed")
    }
}

#[when(regex = r"^a Seed is constructed with value (\d+)$")]
fn when_constructed(world: &mut SeedWorld, value: u64) {
    world.seed = Some(Seed::from(value));
}

#[given("the default Seed")]
fn given_default(world: &mut SeedWorld) {
    world.seed = Some(Seed::default());
}

#[then(regex = r"^the Seed has value (\d+)$")]
fn then_value(world: &mut SeedWorld, value: u64) {
    assert_eq!(world.seed().value(), value);
}

#[tokio::main]
async fn main() {
    SeedWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/seed.feature")
        .await;
}
