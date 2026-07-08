use cucumber::{World as _, given, then, when};
use planet_core::steps::Steps;
use planet_core::subdivision_args::SubdivisionArgs;
use planet_core::subdivision_mode::SubdivisionMode;

#[derive(Debug, Default, cucumber::World)]
pub struct SubdivisionArgsWorld {
    given_steps: Option<Steps>,
    resolved_steps: Option<Steps>,
    resolved_mode: Option<SubdivisionMode>,
}

#[given(regex = r"^Steps constructed with (\d+)$")]
fn given_steps(world: &mut SubdivisionArgsWorld, value: usize) {
    world.given_steps = Some(Steps::new(value).expect("Steps::new failed"));
}

#[when("SubdivisionArgs is constructed with those steps and the UniformRedSplit mode")]
fn when_constructed_with_steps_and_mode(world: &mut SubdivisionArgsWorld) {
    let args = SubdivisionArgs::new(
        world.given_steps,
        Some(SubdivisionMode::UniformRedSplit),
        None,
    );
    world.resolved_steps = Some(args.steps());
    world.resolved_mode = Some(args.mode());
}

#[when("SubdivisionArgs is constructed with no steps and the UniformRedSplit mode")]
fn when_constructed_with_no_steps(world: &mut SubdivisionArgsWorld) {
    let args = SubdivisionArgs::new(None, Some(SubdivisionMode::UniformRedSplit), None);
    world.resolved_steps = Some(args.steps());
    world.resolved_mode = Some(args.mode());
}

#[when("SubdivisionArgs is constructed with those steps and no mode")]
fn when_constructed_with_no_mode(world: &mut SubdivisionArgsWorld) {
    let args = SubdivisionArgs::new(world.given_steps, None, None);
    world.resolved_steps = Some(args.steps());
    world.resolved_mode = Some(args.mode());
}

#[then(regex = r"^the SubdivisionArgs has (\d+) steps$")]
fn then_steps(world: &mut SubdivisionArgsWorld, value: usize) {
    assert_eq!(
        world.resolved_steps.expect("steps not resolved").value(),
        value
    );
}

#[then("the SubdivisionArgs has the UniformRedSplit mode")]
fn then_mode(world: &mut SubdivisionArgsWorld) {
    assert_eq!(
        world.resolved_mode.expect("mode not resolved"),
        SubdivisionMode::UniformRedSplit
    );
}

#[tokio::main]
async fn main() {
    SubdivisionArgsWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/subdivision_args.feature")
        .await;
}
