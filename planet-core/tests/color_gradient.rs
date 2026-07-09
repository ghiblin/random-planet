use cucumber::{World as _, given, then, when};
use planet_core::color::color_gradient::{ColorGradient, ColorGradientError};
use planet_core::color::rgb::Rgb;

#[derive(Debug, Default, cucumber::World)]
pub struct ColorGradientWorld {
    construction_result: Option<Result<ColorGradient, ColorGradientError>>,
    gradient: Option<ColorGradient>,
    sampled: Option<Rgb>,
}

fn parse_color(description: &str) -> Rgb {
    if let Some(rest) = description.strip_prefix("with r ") {
        let mut parts = rest.splitn(2, ", g ");
        let r: f32 = parts
            .next()
            .expect("r channel")
            .trim()
            .parse()
            .expect("r channel number");
        let rest = parts.next().expect("g/b channels");
        let mut parts = rest.splitn(2, ", b ");
        let g: f32 = parts
            .next()
            .expect("g channel")
            .trim()
            .parse()
            .expect("g channel number");
        let b: f32 = parts
            .next()
            .expect("b channel")
            .trim()
            .parse()
            .expect("b channel number");
        return Rgb::new(r, g, b).expect("valid rgb fixture");
    }
    match description {
        "black" => Rgb::new(0.0, 0.0, 0.0).expect("valid rgb fixture"),
        "white" => Rgb::new(1.0, 1.0, 1.0).expect("valid rgb fixture"),
        "gray" => Rgb::new(0.5, 0.5, 0.5).expect("valid rgb fixture"),
        other => panic!("unknown color description: {other}"),
    }
}

fn parse_stop(part: &str) -> (f32, Rgb) {
    let mut split = part.splitn(2, " color ");
    let elevation: f32 = split
        .next()
        .expect("elevation")
        .trim()
        .parse()
        .expect("elevation number");
    let color_description = split.next().expect("color description").trim();
    (elevation, parse_color(color_description))
}

fn parse_stops(description: &str) -> Vec<(f32, Rgb)> {
    description
        .split("elevation ")
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            let part = part.trim_end_matches(" and").trim_end_matches(',').trim();
            parse_stop(part)
        })
        .collect()
}

#[when(regex = r"^a ColorGradient is constructed with stops at (.+)$")]
fn when_constructed_with_stops(world: &mut ColorGradientWorld, description: String) {
    world.construction_result = Some(ColorGradient::new(parse_stops(&description)));
}

#[when(regex = r"^a ColorGradient is constructed with a single stop at elevation (.+)$")]
fn when_constructed_with_single_stop(world: &mut ColorGradientWorld, description: String) {
    world.construction_result = Some(ColorGradient::new(vec![parse_stop(&description)]));
}

#[given(regex = r"^a ColorGradient with stops at (.+)$")]
fn given_gradient_with_stops(world: &mut ColorGradientWorld, description: String) {
    world.gradient =
        Some(ColorGradient::new(parse_stops(&description)).expect("valid gradient fixture"));
}

#[when(regex = r"^the ColorGradient is sampled at elevation (-?\d+(?:\.\d+)?)$")]
fn when_sampled(world: &mut ColorGradientWorld, elevation: f32) {
    let gradient = world
        .gradient
        .as_ref()
        .expect("ColorGradient fixture not set");
    world.sampled = Some(gradient.sample(elevation));
}

#[then("the ColorGradient is constructed successfully")]
fn then_success(world: &mut ColorGradientWorld) {
    assert!(
        world
            .construction_result
            .as_ref()
            .expect("ColorGradient not constructed")
            .is_ok()
    );
}

#[then(regex = r"^the construction fails with a too-few-stops error of count (\d+)$")]
fn then_too_few_stops(world: &mut ColorGradientWorld, count: usize) {
    match world
        .construction_result
        .as_ref()
        .expect("ColorGradient not constructed")
    {
        Err(ColorGradientError::TooFewStops { count: actual }) => assert_eq!(*actual, count),
        other => panic!("expected TooFewStops, got {other:?}"),
    }
}

#[then(
    regex = r"^the construction fails with a stops-not-strictly-ascending error at index (\d+)$"
)]
fn then_not_ascending(world: &mut ColorGradientWorld, index: usize) {
    match world
        .construction_result
        .as_ref()
        .expect("ColorGradient not constructed")
    {
        Err(ColorGradientError::StopsNotStrictlyAscending { index: actual }) => {
            assert_eq!(*actual, index)
        }
        other => panic!("expected StopsNotStrictlyAscending, got {other:?}"),
    }
}

#[then(regex = r"^the sampled Rgb equals (\w+)$")]
fn then_sampled_equals(world: &mut ColorGradientWorld, color_name: String) {
    let expected = parse_color(&color_name);
    assert_eq!(world.sampled.expect("ColorGradient not sampled"), expected);
}

#[then(
    regex = r"^the sampled Rgb has r (-?\d+(?:\.\d+)?), g (-?\d+(?:\.\d+)?), b (-?\d+(?:\.\d+)?)$"
)]
fn then_sampled_has(world: &mut ColorGradientWorld, r: f32, g: f32, b: f32) {
    let rgb = world.sampled.expect("ColorGradient not sampled");
    assert_eq!(rgb.r(), r);
    assert_eq!(rgb.g(), g);
    assert_eq!(rgb.b(), b);
}

#[tokio::main]
async fn main() {
    ColorGradientWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/color_gradient.feature")
        .await;
}
