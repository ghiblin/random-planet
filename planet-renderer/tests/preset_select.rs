use cucumber::{World as _, then, when};
use planet_core::presets::preset::Preset;
use planet_renderer::controls::preset_select::parse_preset;

fn name_to_preset(name: &str) -> Preset {
    match name {
        "Earthy" => Preset::Earthy,
        "Volcano" => Preset::Volcano,
        "Rocky" => Preset::Rocky,
        other => panic!("unknown preset: {other}"),
    }
}

#[derive(Debug, Default, cucumber::World)]
pub struct PresetSelectWorld {
    parsed: Option<Option<Preset>>,
    round_trip_failures: Vec<(Preset, Option<Preset>)>,
}

#[when(regex = r#"^the preset-select value "([^"]*)" is parsed$"#)]
fn when_value_parsed(world: &mut PresetSelectWorld, value: String) {
    world.parsed = Some(parse_preset(&value));
}

#[when("each of Preset::ALL's names is parsed")]
fn when_each_name_parsed(world: &mut PresetSelectWorld) {
    world.round_trip_failures = Preset::ALL
        .into_iter()
        .map(|preset| (preset, parse_preset(preset.name())))
        .filter(|(preset, parsed)| parsed != &Some(*preset))
        .collect();
}

#[then(regex = r#"^the parsed Preset is (Earthy|Volcano|Rocky)$"#)]
fn then_parsed_preset_is(world: &mut PresetSelectWorld, name: String) {
    let parsed = world.parsed.expect("preset-select value not parsed");
    assert_eq!(parsed, Some(name_to_preset(&name)));
}

#[then("no Preset is returned")]
fn then_no_preset_returned(world: &mut PresetSelectWorld) {
    let parsed = world.parsed.expect("preset-select value not parsed");
    assert_eq!(parsed, None);
}

#[then("every parsed Preset equals its source Preset")]
fn then_every_round_trips(world: &mut PresetSelectWorld) {
    assert!(
        world.round_trip_failures.is_empty(),
        "round-trip failures: {:?}",
        world.round_trip_failures
    );
}

#[tokio::main]
async fn main() {
    PresetSelectWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/preset_select.feature")
        .await;
}
