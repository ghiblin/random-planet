use cucumber::{World as _, given, then, when};
use planet_core::subdivision::normal_noise_range::{NormalNoiseRange, NormalNoiseRangeError};

#[derive(Debug, Default, cucumber::World)]
pub struct NormalNoiseRangeWorld {
    result: Option<Result<NormalNoiseRange, NormalNoiseRangeError>>,
}

impl NormalNoiseRangeWorld {
    fn range(&self) -> NormalNoiseRange {
        *self
            .result
            .as_ref()
            .expect("NormalNoiseRange not constructed")
            .as_ref()
            .expect("NormalNoiseRange construction failed")
    }
}

#[when(
    regex = r"^a NormalNoiseRange is constructed with low (-?\d+(?:\.\d+)?) and high (-?\d+(?:\.\d+)?)$"
)]
fn when_constructed(world: &mut NormalNoiseRangeWorld, low: f32, high: f32) {
    world.result = Some(NormalNoiseRange::new(low, high));
}

#[given("the default NormalNoiseRange")]
fn given_default(world: &mut NormalNoiseRangeWorld) {
    world.result = Some(Ok(NormalNoiseRange::default()));
}

#[then("the NormalNoiseRange is constructed successfully")]
fn then_success(world: &mut NormalNoiseRangeWorld) {
    assert!(
        world
            .result
            .as_ref()
            .expect("NormalNoiseRange not constructed")
            .is_ok()
    );
}

#[then(regex = r"^the NormalNoiseRange has low (-?\d+(?:\.\d+)?)$")]
fn then_low(world: &mut NormalNoiseRangeWorld, low: f32) {
    assert_eq!(world.range().low(), low);
}

#[then(regex = r"^the NormalNoiseRange has high (-?\d+(?:\.\d+)?)$")]
fn then_high(world: &mut NormalNoiseRangeWorld, high: f32) {
    assert_eq!(world.range().high(), high);
}

#[then(
    regex = r"^the construction fails with an invalid-range error of low (-?\d+(?:\.\d+)?) and high (-?\d+(?:\.\d+)?)$"
)]
fn then_invalid_range(world: &mut NormalNoiseRangeWorld, low: f32, high: f32) {
    match world
        .result
        .as_ref()
        .expect("NormalNoiseRange not constructed")
    {
        Err(NormalNoiseRangeError::InvalidRange {
            low: actual_low,
            high: actual_high,
        }) => {
            assert_eq!(*actual_low, low);
            assert_eq!(*actual_high, high);
        }
        other => panic!("expected InvalidRange, got {other:?}"),
    }
}

#[tokio::main]
async fn main() {
    NormalNoiseRangeWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/normal_noise_range.feature")
        .await;
}
