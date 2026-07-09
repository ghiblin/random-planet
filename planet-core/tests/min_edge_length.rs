use cucumber::{World as _, given, then, when};
use planet_core::subdivision::min_edge_length::{MinEdgeLength, MinEdgeLengthError};

#[derive(Debug, Default, cucumber::World)]
pub struct MinEdgeLengthWorld {
    result: Option<Result<MinEdgeLength, MinEdgeLengthError>>,
}

impl MinEdgeLengthWorld {
    fn value(&self) -> f32 {
        self.result
            .as_ref()
            .expect("MinEdgeLength not constructed")
            .as_ref()
            .expect("MinEdgeLength construction failed")
            .value()
    }
}

#[when(regex = r"^a MinEdgeLength is constructed with value (-?\d+(?:\.\d+)?|NaN)$")]
fn when_constructed(world: &mut MinEdgeLengthWorld, value: f32) {
    world.result = Some(MinEdgeLength::new(value));
}

#[given("the default MinEdgeLength")]
fn given_default(world: &mut MinEdgeLengthWorld) {
    world.result = Some(Ok(MinEdgeLength::default()));
}

#[then("the MinEdgeLength is constructed successfully")]
fn then_success(world: &mut MinEdgeLengthWorld) {
    assert!(
        world
            .result
            .as_ref()
            .expect("MinEdgeLength not constructed")
            .is_ok()
    );
}

#[then(regex = r"^the MinEdgeLength has value (-?\d+(?:\.\d+)?)$")]
fn then_value(world: &mut MinEdgeLengthWorld, value: f32) {
    assert_eq!(world.value(), value);
}

#[then(regex = r"^the construction fails with a negative-value error of (-?\d+(?:\.\d+)?)$")]
fn then_negative_value_error(world: &mut MinEdgeLengthWorld, value: f32) {
    match world
        .result
        .as_ref()
        .expect("MinEdgeLength not constructed")
    {
        Err(MinEdgeLengthError::Negative { value: actual }) => assert_eq!(*actual, value),
        other => panic!("expected Negative, got {other:?}"),
    }
}

#[then("the construction fails with a negative-value error of NaN")]
fn then_negative_value_error_nan(world: &mut MinEdgeLengthWorld) {
    match world
        .result
        .as_ref()
        .expect("MinEdgeLength not constructed")
    {
        Err(MinEdgeLengthError::Negative { value: actual }) => {
            assert!(actual.is_nan(), "expected NaN, got {actual}")
        }
        other => panic!("expected Negative, got {other:?}"),
    }
}

#[tokio::main]
async fn main() {
    MinEdgeLengthWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/min_edge_length.feature")
        .await;
}
