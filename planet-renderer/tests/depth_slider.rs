use cucumber::{World as _, then, when};
use planet_renderer::controls::depth_slider::{DepthParseError, parse_depth};

#[derive(Debug, Default, cucumber::World)]
pub struct DepthSliderWorld {
    result: Option<Result<planet_core::subdivision::steps::Steps, DepthParseError>>,
}

#[when(regex = r#"^the depth-slider value "([^"]*)" is parsed$"#)]
fn when_value_parsed(world: &mut DepthSliderWorld, value: String) {
    world.result = Some(parse_depth(&value));
}

#[then(regex = r"^the parsed Steps has value (\d+)$")]
fn then_parsed_value(world: &mut DepthSliderWorld, expected: usize) {
    let result = world.result.take().expect("depth-slider value not parsed");
    let steps = result.expect("expected Ok(Steps)");
    assert_eq!(steps.value(), expected);
}

#[then("the parsing fails with an invalid-steps error")]
fn then_invalid_steps_error(world: &mut DepthSliderWorld) {
    let result = world.result.take().expect("depth-slider value not parsed");
    assert!(matches!(result, Err(DepthParseError::InvalidSteps(_))));
}

#[then("the parsing fails with a not-a-number error")]
fn then_not_a_number_error(world: &mut DepthSliderWorld) {
    let result = world.result.take().expect("depth-slider value not parsed");
    assert!(matches!(result, Err(DepthParseError::NotANumber { .. })));
}

#[tokio::main]
async fn main() {
    DepthSliderWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/depth_slider.feature")
        .await;
}
