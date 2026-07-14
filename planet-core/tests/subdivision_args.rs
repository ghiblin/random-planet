use cucumber::{World as _, given, then, when};
use planet_core::subdivision::seed::Seed;
use planet_core::subdivision::steps::Steps;
use planet_core::subdivision::subdivision_args::SubdivisionArgs;
use planet_core::subdivision::subdivision_mode::SubdivisionMode;

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

#[when(
    regex = r"^SubdivisionArgs is constructed with those steps and the UniformRedSplit mode with seed (\d+)$"
)]
fn when_constructed_with_steps_and_mode(world: &mut SubdivisionArgsWorld, seed: u64) {
    let args = SubdivisionArgs::new(
        world.given_steps,
        Some(SubdivisionMode::UniformRedSplit {
            seed: Seed::from(seed),
        }),
        None,
    );
    world.resolved_steps = Some(args.steps());
    world.resolved_mode = Some(args.mode());
}

#[when(
    regex = r"^SubdivisionArgs is constructed with no steps and the UniformRedSplit mode with seed (\d+)$"
)]
fn when_constructed_with_no_steps(world: &mut SubdivisionArgsWorld, seed: u64) {
    let args = SubdivisionArgs::new(
        None,
        Some(SubdivisionMode::UniformRedSplit {
            seed: Seed::from(seed),
        }),
        None,
    );
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

#[then(regex = r"^the SubdivisionArgs has the UniformRedSplit mode with seed (\d+)$")]
fn then_mode_with_seed(world: &mut SubdivisionArgsWorld, seed: u64) {
    assert_eq!(
        world.resolved_mode.expect("mode not resolved"),
        SubdivisionMode::UniformRedSplit {
            seed: Seed::from(seed),
        }
    );
}

#[then("the SubdivisionArgs has the default UniformRedSplit mode")]
fn then_default_mode(world: &mut SubdivisionArgsWorld) {
    assert_eq!(
        world.resolved_mode.expect("mode not resolved"),
        SubdivisionMode::default()
    );
}

#[tokio::main]
async fn main() {
    SubdivisionArgsWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/subdivision_args.feature")
        .await;
}
