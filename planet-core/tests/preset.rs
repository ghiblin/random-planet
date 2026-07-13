use cucumber::{World as _, given, then, when};
use planet_core::presets::preset::Preset;
use planet_core::presets::preset_params::PresetParams;

#[derive(Debug, Default, cucumber::World)]
pub struct PresetWorld {
    params: Option<PresetParams>,
    params_pair: Option<(PresetParams, PresetParams)>,
    preset: Option<Preset>,
    all: Option<[Preset; 3]>,
    names_requested: bool,
    descriptions_requested: bool,
}

fn parse_preset(name: &str) -> Preset {
    match name {
        "Earthy" => Preset::Earthy,
        "Volcano" => Preset::Volcano,
        "Rocky" => Preset::Rocky,
        other => panic!("unknown preset: {other}"),
    }
}

#[when(regex = r"^Preset::(Earthy|Volcano|Rocky)'s params are requested$")]
fn when_params_requested(world: &mut PresetWorld, name: String) {
    world.params = Some(parse_preset(&name).params());
}

#[when(regex = r"^Preset::(Earthy|Volcano|Rocky)'s params are requested twice$")]
fn when_params_requested_twice(world: &mut PresetWorld, name: String) {
    let preset = parse_preset(&name);
    world.params_pair = Some((preset.params(), preset.params()));
}

#[given("the default Preset")]
fn given_default_preset(world: &mut PresetWorld) {
    world.preset = Some(Preset::default());
}

#[then(regex = r"^the Preset equals Preset::(Earthy|Volcano|Rocky)$")]
fn then_preset_equals(world: &mut PresetWorld, name: String) {
    assert_eq!(world.preset.expect("Preset not set"), parse_preset(&name));
}

#[then(regex = r"^the PresetParams has a MinEdgeLength of (-?\d+(?:\.\d+)?)$")]
fn then_min_edge_length(world: &mut PresetWorld, value: f32) {
    let params = world.params.as_ref().expect("PresetParams not requested");
    assert_eq!(params.min_edge_length().value(), value);
}

#[then(
    regex = r"^the PresetParams has an ElevationNoiseRange of low (-?\d+(?:\.\d+)?) and high (-?\d+(?:\.\d+)?)$"
)]
fn then_elevation_noise_range(world: &mut PresetWorld, low: f32, high: f32) {
    let params = world.params.as_ref().expect("PresetParams not requested");
    assert_eq!(params.elevation_noise_range().low(), low);
    assert_eq!(params.elevation_noise_range().high(), high);
}

#[then(
    regex = r"^the PresetParams has a NormalNoiseRange of low (-?\d+(?:\.\d+)?) and high (-?\d+(?:\.\d+)?)$"
)]
fn then_normal_noise_range(world: &mut PresetWorld, low: f32, high: f32) {
    let params = world.params.as_ref().expect("PresetParams not requested");
    assert_eq!(params.normal_noise_range().low(), low);
    assert_eq!(params.normal_noise_range().high(), high);
}

#[then(regex = r"^the PresetParams has a SplitPointVariance of (-?\d+(?:\.\d+)?)$")]
fn then_split_point_variance(world: &mut PresetWorld, value: f32) {
    let params = world.params.as_ref().expect("PresetParams not requested");
    assert_eq!(params.split_point_variance().value(), value);
}

#[then(
    regex = r"^sampling its color gradient at elevation (-?\d+(?:\.\d+)?) returns Rgb r (-?\d+(?:\.\d+)?), g (-?\d+(?:\.\d+)?), b (-?\d+(?:\.\d+)?)$"
)]
fn then_color_gradient_sample(world: &mut PresetWorld, elevation: f32, r: f32, g: f32, b: f32) {
    let params = world.params.as_ref().expect("PresetParams not requested");
    let sampled = params.color_gradient().sample(elevation);
    assert_eq!(sampled.r(), r);
    assert_eq!(sampled.g(), g);
    assert_eq!(sampled.b(), b);
}

#[then(regex = r"^the PresetParams has an OceanQuota of (-?\d+(?:\.\d+)?)$")]
fn then_ocean_quota(world: &mut PresetWorld, value: f32) {
    let params = world.params.as_ref().expect("PresetParams not requested");
    assert_eq!(
        params
            .ocean_quota()
            .expect("PresetParams has no OceanQuota")
            .value(),
        value
    );
}

#[then("the PresetParams has no OceanQuota")]
fn then_no_ocean_quota(world: &mut PresetWorld) {
    let params = world.params.as_ref().expect("PresetParams not requested");
    assert_eq!(params.ocean_quota(), None);
}

#[then("both PresetParams are identical")]
fn then_both_identical(world: &mut PresetWorld) {
    let (first, second) = world
        .params_pair
        .as_ref()
        .expect("PresetParams pair not requested");
    assert_eq!(first, second);
}

#[when("Preset::ALL is requested")]
fn when_all_requested(world: &mut PresetWorld) {
    world.all = Some(Preset::ALL);
}

#[then("Preset::ALL equals Earthy, Volcano, Rocky in that order")]
fn then_all_in_order(world: &mut PresetWorld) {
    let all = world.all.expect("Preset::ALL not requested");
    assert_eq!(all, [Preset::Earthy, Preset::Volcano, Preset::Rocky]);
}

#[when("each Preset's name is requested")]
fn when_names_requested(world: &mut PresetWorld) {
    world.names_requested = true;
}

#[then(regex = r#"^Preset::(Earthy|Volcano|Rocky)'s name is "([^"]+)"$"#)]
fn then_name_is(world: &mut PresetWorld, name: String, expected: String) {
    assert!(world.names_requested, "Preset names not requested");
    assert_eq!(parse_preset(&name).name(), expected);
}

#[when("each Preset's description is requested")]
fn when_descriptions_requested(world: &mut PresetWorld) {
    world.descriptions_requested = true;
}

#[then(regex = r"^Preset::(Earthy|Volcano|Rocky)'s description is non-empty$")]
fn then_description_non_empty(world: &mut PresetWorld, name: String) {
    assert!(
        world.descriptions_requested,
        "Preset descriptions not requested"
    );
    assert!(!parse_preset(&name).description().is_empty());
}

#[tokio::main]
async fn main() {
    PresetWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/preset.feature")
        .await;
}
