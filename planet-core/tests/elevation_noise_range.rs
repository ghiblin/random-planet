use cucumber::{World as _, given, then, when};
use planet_core::subdivision::elevation_noise_range::{
    ElevationNoiseRange, ElevationNoiseRangeError,
};

#[derive(Debug, Default, cucumber::World)]
pub struct ElevationNoiseRangeWorld {
    result: Option<Result<ElevationNoiseRange, ElevationNoiseRangeError>>,
}

impl ElevationNoiseRangeWorld {
    fn range(&self) -> ElevationNoiseRange {
        *self
            .result
            .as_ref()
            .expect("ElevationNoiseRange not constructed")
            .as_ref()
            .expect("ElevationNoiseRange construction failed")
    }
}

#[when(
    regex = r"^an ElevationNoiseRange is constructed with low (-?\d+(?:\.\d+)?) and high (-?\d+(?:\.\d+)?)$"
)]
fn when_constructed(world: &mut ElevationNoiseRangeWorld, low: f32, high: f32) {
    world.result = Some(ElevationNoiseRange::new(low, high));
}

#[given("the default ElevationNoiseRange")]
fn given_default(world: &mut ElevationNoiseRangeWorld) {
    world.result = Some(Ok(ElevationNoiseRange::default()));
}

#[then("the ElevationNoiseRange is constructed successfully")]
fn then_success(world: &mut ElevationNoiseRangeWorld) {
    assert!(
        world
            .result
            .as_ref()
            .expect("ElevationNoiseRange not constructed")
            .is_ok()
    );
}

#[then(regex = r"^the ElevationNoiseRange has low (-?\d+(?:\.\d+)?)$")]
fn then_low(world: &mut ElevationNoiseRangeWorld, low: f32) {
    assert_eq!(world.range().low(), low);
}

#[then(regex = r"^the ElevationNoiseRange has high (-?\d+(?:\.\d+)?)$")]
fn then_high(world: &mut ElevationNoiseRangeWorld, high: f32) {
    assert_eq!(world.range().high(), high);
}

#[then(
    regex = r"^the construction fails with an invalid-range error of low (-?\d+(?:\.\d+)?) and high (-?\d+(?:\.\d+)?)$"
)]
fn then_invalid_range(world: &mut ElevationNoiseRangeWorld, low: f32, high: f32) {
    match world
        .result
        .as_ref()
        .expect("ElevationNoiseRange not constructed")
    {
        Err(ElevationNoiseRangeError::InvalidRange {
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
    ElevationNoiseRangeWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/elevation_noise_range.feature")
        .await;
}
