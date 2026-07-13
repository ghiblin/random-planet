use cucumber::{World as _, given, then, when};
use planet_core::color::color_gradient::ColorGradient;
use planet_core::color::rgb::Rgb;
use planet_core::presets::preset_params::PresetParams;
use planet_core::processor::ocean_quota::OceanQuota;
use planet_core::subdivision::elevation_noise_range::ElevationNoiseRange;
use planet_core::subdivision::min_edge_length::MinEdgeLength;
use planet_core::subdivision::normal_noise_range::NormalNoiseRange;
use planet_core::subdivision::split_point_variance::SplitPointVariance;

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

#[derive(Debug, Default, cucumber::World)]
pub struct PresetParamsWorld {
    fixture: Option<(
        MinEdgeLength,
        ElevationNoiseRange,
        NormalNoiseRange,
        SplitPointVariance,
        ColorGradient,
        Option<OceanQuota>,
    )>,
    params: Option<PresetParams>,
    params_pair: Option<(PresetParams, PresetParams)>,
}

fn parse_range(description: &str) -> (f32, f32) {
    let mut parts = description.splitn(2, " and high ");
    let low: f32 = parts
        .next()
        .expect("low bound")
        .trim_start_matches("low ")
        .trim()
        .parse()
        .expect("low bound number");
    let high: f32 = parts
        .next()
        .expect("high bound")
        .trim()
        .parse()
        .expect("high bound number");
    (low, high)
}

#[given(
    regex = r"^a MinEdgeLength of (-?\d+(?:\.\d+)?), an ElevationNoiseRange of (low -?\d+(?:\.\d+)? and high -?\d+(?:\.\d+)?), a NormalNoiseRange of (low -?\d+(?:\.\d+)? and high -?\d+(?:\.\d+)?), a SplitPointVariance of (-?\d+(?:\.\d+)?), a ColorGradient with stops at (.+), and an OceanQuota of (-?\d+(?:\.\d+)?)$"
)]
fn given_fixture(
    world: &mut PresetParamsWorld,
    min_edge_length: f32,
    elevation_range: String,
    normal_range: String,
    split_point_variance: f32,
    stops_description: String,
    ocean_quota: f32,
) {
    let (elevation_low, elevation_high) = parse_range(&elevation_range);
    let (normal_low, normal_high) = parse_range(&normal_range);
    world.fixture = Some((
        MinEdgeLength::new(min_edge_length).expect("valid min edge length fixture"),
        ElevationNoiseRange::new(elevation_low, elevation_high)
            .expect("valid elevation noise range fixture"),
        NormalNoiseRange::new(normal_low, normal_high).expect("valid normal noise range fixture"),
        SplitPointVariance::new(split_point_variance).expect("valid split point variance fixture"),
        ColorGradient::new(parse_stops(&stops_description)).expect("valid color gradient fixture"),
        Some(OceanQuota::new(ocean_quota).expect("valid ocean quota fixture")),
    ));
}

#[when("a PresetParams is constructed from those 6 values")]
fn when_constructed(world: &mut PresetParamsWorld) {
    let (
        min_edge_length,
        elevation_noise_range,
        normal_noise_range,
        split_point_variance,
        color_gradient,
        ocean_quota,
    ) = world.fixture.clone().expect("fixture not set");
    world.params = Some(PresetParams::new(
        min_edge_length,
        elevation_noise_range,
        normal_noise_range,
        split_point_variance,
        color_gradient,
        ocean_quota,
    ));
}

#[when("two PresetParams are constructed from those same 6 values, separately")]
fn when_constructed_twice(world: &mut PresetParamsWorld) {
    let (
        min_edge_length,
        elevation_noise_range,
        normal_noise_range,
        split_point_variance,
        color_gradient,
        ocean_quota,
    ) = world.fixture.clone().expect("fixture not set");
    let first = PresetParams::new(
        min_edge_length,
        elevation_noise_range,
        normal_noise_range,
        split_point_variance,
        color_gradient.clone(),
        ocean_quota,
    );
    let second = PresetParams::new(
        min_edge_length,
        elevation_noise_range,
        normal_noise_range,
        split_point_variance,
        color_gradient,
        ocean_quota,
    );
    world.params_pair = Some((first, second));
}

#[then(regex = r"^the PresetParams has a MinEdgeLength of (-?\d+(?:\.\d+)?)$")]
fn then_min_edge_length(world: &mut PresetParamsWorld, value: f32) {
    let params = world.params.as_ref().expect("PresetParams not constructed");
    assert_eq!(params.min_edge_length().value(), value);
}

#[then(
    regex = r"^the PresetParams has an ElevationNoiseRange of low (-?\d+(?:\.\d+)?) and high (-?\d+(?:\.\d+)?)$"
)]
fn then_elevation_noise_range(world: &mut PresetParamsWorld, low: f32, high: f32) {
    let params = world.params.as_ref().expect("PresetParams not constructed");
    assert_eq!(params.elevation_noise_range().low(), low);
    assert_eq!(params.elevation_noise_range().high(), high);
}

#[then(
    regex = r"^the PresetParams has a NormalNoiseRange of low (-?\d+(?:\.\d+)?) and high (-?\d+(?:\.\d+)?)$"
)]
fn then_normal_noise_range(world: &mut PresetParamsWorld, low: f32, high: f32) {
    let params = world.params.as_ref().expect("PresetParams not constructed");
    assert_eq!(params.normal_noise_range().low(), low);
    assert_eq!(params.normal_noise_range().high(), high);
}

#[then(regex = r"^the PresetParams has a SplitPointVariance of (-?\d+(?:\.\d+)?)$")]
fn then_split_point_variance(world: &mut PresetParamsWorld, value: f32) {
    let params = world.params.as_ref().expect("PresetParams not constructed");
    assert_eq!(params.split_point_variance().value(), value);
}

#[then(regex = r"^the PresetParams's ColorGradient samples elevation (-?\d+(?:\.\d+)?) to (\w+)$")]
fn then_color_gradient_samples(world: &mut PresetParamsWorld, elevation: f32, color_name: String) {
    let params = world.params.as_ref().expect("PresetParams not constructed");
    let expected = parse_color(&color_name);
    assert_eq!(params.color_gradient().sample(elevation), expected);
}

#[then(regex = r"^the PresetParams has an OceanQuota of (-?\d+(?:\.\d+)?)$")]
fn then_ocean_quota(world: &mut PresetParamsWorld, value: f32) {
    let params = world.params.as_ref().expect("PresetParams not constructed");
    assert_eq!(
        params
            .ocean_quota()
            .expect("PresetParams has no OceanQuota")
            .value(),
        value
    );
}

#[then("the two PresetParams are identical")]
fn then_identical(world: &mut PresetParamsWorld) {
    let (first, second) = world
        .params_pair
        .as_ref()
        .expect("PresetParams pair not constructed");
    assert_eq!(first, second);
}

#[tokio::main]
async fn main() {
    PresetParamsWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/preset_params.feature")
        .await;
}
