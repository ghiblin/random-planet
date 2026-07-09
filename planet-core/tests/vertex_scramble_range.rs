use cucumber::{World as _, given, then, when};
use planet_core::processor::vertex_scramble_range::{
    VertexScrambleRange, VertexScrambleRangeError,
};

#[derive(Debug, Default, cucumber::World)]
pub struct VertexScrambleRangeWorld {
    result: Option<Result<VertexScrambleRange, VertexScrambleRangeError>>,
}

impl VertexScrambleRangeWorld {
    fn range(&self) -> VertexScrambleRange {
        *self
            .result
            .as_ref()
            .expect("VertexScrambleRange not constructed")
            .as_ref()
            .expect("VertexScrambleRange construction failed")
    }
}

#[when(
    regex = r"^a VertexScrambleRange is constructed with low (-?\d+(?:\.\d+)?) and high (-?\d+(?:\.\d+)?)$"
)]
fn when_constructed(world: &mut VertexScrambleRangeWorld, low: f32, high: f32) {
    world.result = Some(VertexScrambleRange::new(low, high));
}

#[given("the default VertexScrambleRange")]
fn given_default(world: &mut VertexScrambleRangeWorld) {
    world.result = Some(Ok(VertexScrambleRange::default()));
}

#[then("the VertexScrambleRange is constructed successfully")]
fn then_success(world: &mut VertexScrambleRangeWorld) {
    assert!(
        world
            .result
            .as_ref()
            .expect("VertexScrambleRange not constructed")
            .is_ok()
    );
}

#[then(regex = r"^the VertexScrambleRange has low (-?\d+(?:\.\d+)?)$")]
fn then_low(world: &mut VertexScrambleRangeWorld, low: f32) {
    assert_eq!(world.range().low(), low);
}

#[then(regex = r"^the VertexScrambleRange has high (-?\d+(?:\.\d+)?)$")]
fn then_high(world: &mut VertexScrambleRangeWorld, high: f32) {
    assert_eq!(world.range().high(), high);
}

#[then(
    regex = r"^the construction fails with an invalid-range error of low (-?\d+(?:\.\d+)?) and high (-?\d+(?:\.\d+)?)$"
)]
fn then_invalid_range(world: &mut VertexScrambleRangeWorld, low: f32, high: f32) {
    match world
        .result
        .as_ref()
        .expect("VertexScrambleRange not constructed")
    {
        Err(VertexScrambleRangeError::InvalidRange {
            low: actual_low,
            high: actual_high,
        }) => {
            assert_eq!(*actual_low, low);
            assert_eq!(*actual_high, high);
        }
        other => panic!("expected InvalidRange, got {other:?}"),
    }
}

#[then(
    regex = r"^the construction fails with a low-at-or-below-negative-one error of low (-?\d+(?:\.\d+)?)$"
)]
fn then_low_at_or_below_negative_one(world: &mut VertexScrambleRangeWorld, low: f32) {
    match world
        .result
        .as_ref()
        .expect("VertexScrambleRange not constructed")
    {
        Err(VertexScrambleRangeError::LowAtOrBelowNegativeOne { low: actual_low }) => {
            assert_eq!(*actual_low, low);
        }
        other => panic!("expected LowAtOrBelowNegativeOne, got {other:?}"),
    }
}

#[tokio::main]
async fn main() {
    VertexScrambleRangeWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/vertex_scramble_range.feature")
        .await;
}
