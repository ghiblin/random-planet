use planet_core::presets::preset::Preset;

pub fn parse_preset(value: &str) -> Option<Preset> {
    Preset::ALL
        .into_iter()
        .find(|preset| preset.name() == value)
}
