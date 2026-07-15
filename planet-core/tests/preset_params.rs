use cucumber::{World as _, given, then, when};
use planet_core::color::color_gradient::ColorGradient;
use planet_core::color::rgb::Rgb;
use planet_core::presets::preset_params::PresetParams;
use planet_core::processor::ocean_quota::OceanQuota;
use planet_core::processor::terrain_noise::TerrainNoise;
use planet_core::subdivision::subdivision_mode::SubdivisionMode;

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
        TerrainNoise,
        ColorGradient,
        Option<OceanQuota>,
        SubdivisionMode,
    )>,
    params: Option<PresetParams>,
    params_pair: Option<(PresetParams, PresetParams)>,
}

fn default_terrain_noise(amplitude: f32) -> TerrainNoise {
    TerrainNoise::new(1.5, 4, 0.5, 2.0, amplitude, 1.4, None).expect("valid terrain noise fixture")
}

#[given(
    regex = r"^a TerrainNoise with amplitude (-?\d+(?:\.\d+)?), a ColorGradient with stops at (.+), an OceanQuota of (-?\d+(?:\.\d+)?), and SubdivisionMode::UniformRedSplit$"
)]
fn given_fixture(
    world: &mut PresetParamsWorld,
    amplitude: f32,
    stops_description: String,
    ocean_quota: f32,
) {
    world.fixture = Some((
        default_terrain_noise(amplitude),
        ColorGradient::new(parse_stops(&stops_description)).expect("valid color gradient fixture"),
        Some(OceanQuota::new(ocean_quota).expect("valid ocean quota fixture")),
        SubdivisionMode::UniformRedSplit,
    ));
}

#[when("a PresetParams is constructed from those 4 values")]
fn when_constructed(world: &mut PresetParamsWorld) {
    let (terrain_noise, color_gradient, ocean_quota, subdivision_mode) =
        world.fixture.clone().expect("fixture not set");
    world.params = Some(PresetParams::new(
        terrain_noise,
        color_gradient,
        ocean_quota,
        subdivision_mode,
    ));
}

#[when("two PresetParams are constructed from those same 4 values, separately")]
fn when_constructed_twice(world: &mut PresetParamsWorld) {
    let (terrain_noise, color_gradient, ocean_quota, subdivision_mode) =
        world.fixture.clone().expect("fixture not set");
    let first = PresetParams::new(
        terrain_noise,
        color_gradient.clone(),
        ocean_quota,
        subdivision_mode,
    );
    let second = PresetParams::new(terrain_noise, color_gradient, ocean_quota, subdivision_mode);
    world.params_pair = Some((first, second));
}

#[then(regex = r"^the PresetParams has a TerrainNoise with amplitude (-?\d+(?:\.\d+)?)$")]
fn then_terrain_noise(world: &mut PresetParamsWorld, value: f32) {
    let params = world.params.as_ref().expect("PresetParams not constructed");
    assert_eq!(params.terrain_noise().amplitude(), value);
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

#[then("the PresetParams has subdivision mode SubdivisionMode::UniformRedSplit")]
fn then_subdivision_mode(world: &mut PresetParamsWorld) {
    let params = world.params.as_ref().expect("PresetParams not constructed");
    assert_eq!(params.subdivision_mode(), SubdivisionMode::UniformRedSplit);
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
