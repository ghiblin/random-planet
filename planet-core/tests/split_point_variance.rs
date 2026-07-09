use cucumber::{World as _, given, then, when};
use planet_core::subdivision::split_point_variance::{SplitPointVariance, SplitPointVarianceError};

#[derive(Debug, Default, cucumber::World)]
pub struct SplitPointVarianceWorld {
    result: Option<Result<SplitPointVariance, SplitPointVarianceError>>,
}

impl SplitPointVarianceWorld {
    fn value(&self) -> f32 {
        self.result
            .as_ref()
            .expect("SplitPointVariance not constructed")
            .as_ref()
            .expect("SplitPointVariance construction failed")
            .value()
    }
}

#[when(regex = r"^a SplitPointVariance is constructed with value (-?\d+(?:\.\d+)?|NaN)$")]
fn when_constructed(world: &mut SplitPointVarianceWorld, value: f32) {
    world.result = Some(SplitPointVariance::new(value));
}

#[given("the default SplitPointVariance")]
fn given_default(world: &mut SplitPointVarianceWorld) {
    world.result = Some(Ok(SplitPointVariance::default()));
}

#[then("the SplitPointVariance is constructed successfully")]
fn then_success(world: &mut SplitPointVarianceWorld) {
    assert!(
        world
            .result
            .as_ref()
            .expect("SplitPointVariance not constructed")
            .is_ok()
    );
}

#[then(regex = r"^the SplitPointVariance has value (-?\d+(?:\.\d+)?)$")]
fn then_value(world: &mut SplitPointVarianceWorld, value: f32) {
    assert_eq!(world.value(), value);
}

#[then(regex = r"^the construction fails with a negative-value error of (-?\d+(?:\.\d+)?)$")]
fn then_negative_value_error(world: &mut SplitPointVarianceWorld, value: f32) {
    match world
        .result
        .as_ref()
        .expect("SplitPointVariance not constructed")
    {
        Err(SplitPointVarianceError::Negative { value: actual }) => assert_eq!(*actual, value),
        other => panic!("expected Negative, got {other:?}"),
    }
}

#[then("the construction fails with a negative-value error of NaN")]
fn then_negative_value_error_nan(world: &mut SplitPointVarianceWorld) {
    match world
        .result
        .as_ref()
        .expect("SplitPointVariance not constructed")
    {
        Err(SplitPointVarianceError::Negative { value: actual }) => {
            assert!(actual.is_nan(), "expected NaN, got {actual}")
        }
        other => panic!("expected Negative, got {other:?}"),
    }
}

#[tokio::main]
async fn main() {
    SplitPointVarianceWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/split_point_variance.feature")
        .await;
}
