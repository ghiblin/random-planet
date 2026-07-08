use cucumber::{World as _, given, then, when};
use planet_core::subdivision::steps::{Steps, StepsError};

#[derive(Debug, Default, cucumber::World)]
pub struct StepsWorld {
    result: Option<Result<Steps, StepsError>>,
}

impl StepsWorld {
    fn steps(&self) -> Steps {
        *self
            .result
            .as_ref()
            .expect("Steps not constructed")
            .as_ref()
            .expect("Steps construction failed")
    }
}

#[when(regex = r"^Steps is constructed with (\d+)$")]
fn when_constructed(world: &mut StepsWorld, value: usize) {
    world.result = Some(Steps::new(value));
}

#[given("the default Steps")]
fn given_default(world: &mut StepsWorld) {
    world.result = Some(Ok(Steps::default()));
}

#[then("the Steps is constructed successfully")]
fn then_success(world: &mut StepsWorld) {
    assert!(
        world
            .result
            .as_ref()
            .expect("Steps not constructed")
            .is_ok()
    );
}

#[then(regex = r"^the Steps has value (\d+)$")]
fn then_value(world: &mut StepsWorld, value: usize) {
    assert_eq!(world.steps().value(), value);
}

#[then(
    regex = r"^the construction fails with an exceeds-maximum error of (\d+) steps and max (\d+)$"
)]
fn then_exceeds_maximum(world: &mut StepsWorld, steps: usize, max: usize) {
    match world.result.as_ref().expect("Steps not constructed") {
        Err(StepsError::ExceedsMaximum {
            steps: actual_steps,
            max: actual_max,
        }) => {
            assert_eq!(*actual_steps, steps);
            assert_eq!(*actual_max, max);
        }
        other => panic!("expected ExceedsMaximum, got {other:?}"),
    }
}

#[tokio::main]
async fn main() {
    StepsWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/steps.feature")
        .await;
}
