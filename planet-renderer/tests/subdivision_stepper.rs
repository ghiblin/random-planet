use cucumber::{World as _, given, then, when};
use planet_core::icosahedron::icosahedron;
use planet_core::uniform_red_split::UniformRedSplit;
use planet_renderer::subdivision_stepper::SubdivisionStepper;

#[derive(Debug, Default, cucumber::World)]
pub struct SubdivisionStepperWorld {
    stepper: Option<SubdivisionStepper>,
    last_step_result: Option<bool>,
}

impl SubdivisionStepperWorld {
    fn stepper(&self) -> &SubdivisionStepper {
        self.stepper.as_ref().expect("stepper not constructed")
    }
}

#[given(
    regex = r"^a SubdivisionStepper constructed from the icosahedron mesh with max depth (\d+)$"
)]
fn given_stepper(world: &mut SubdivisionStepperWorld, max_depth: u32) {
    let mesh = icosahedron().expect("icosahedron() failed");
    world.stepper = Some(SubdivisionStepper::new(mesh, max_depth));
}

#[when("the stepper is stepped once using the uniform red-split strategy")]
#[when("the stepper is stepped again using the uniform red-split strategy")]
fn when_stepped(world: &mut SubdivisionStepperWorld) {
    let result = world
        .stepper
        .as_mut()
        .expect("stepper not constructed")
        .step(&mut UniformRedSplit)
        .expect("step() failed");
    world.last_step_result = Some(result);
}

#[then("the step succeeds")]
fn then_step_succeeds(world: &mut SubdivisionStepperWorld) {
    assert_eq!(world.last_step_result, Some(true));
}

#[then("the second step does not succeed")]
fn then_step_fails(world: &mut SubdivisionStepperWorld) {
    assert_eq!(world.last_step_result, Some(false));
}

#[then(regex = r"^the stepper has completed (\d+) rounds$")]
fn then_rounds_completed(world: &mut SubdivisionStepperWorld, rounds: u32) {
    assert_eq!(world.stepper().rounds_completed(), rounds);
}

#[then(regex = r"^the stepper's mesh has (\d+) triangles$")]
fn then_triangle_count(world: &mut SubdivisionStepperWorld, count: usize) {
    assert_eq!(world.stepper().mesh().triangles().len(), count);
}

#[tokio::main]
async fn main() {
    SubdivisionStepperWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/subdivision_stepper.feature")
        .await;
}
