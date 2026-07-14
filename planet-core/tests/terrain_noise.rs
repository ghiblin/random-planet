use cucumber::{World as _, then, when};
use planet_core::processor::terrain_noise::{TerrainNoise, TerrainNoiseError};

#[derive(Debug, Default, cucumber::World)]
pub struct TerrainNoiseWorld {
    result: Option<Result<TerrainNoise, TerrainNoiseError>>,
}

impl TerrainNoiseWorld {
    fn terrain_noise(&self) -> &TerrainNoise {
        self.result
            .as_ref()
            .expect("TerrainNoise not constructed")
            .as_ref()
            .expect("TerrainNoise construction failed")
    }

    fn error(&self) -> &TerrainNoiseError {
        self.result
            .as_ref()
            .expect("TerrainNoise not constructed")
            .as_ref()
            .expect_err("TerrainNoise construction unexpectedly succeeded")
    }
}

fn parse_terrace_levels(description: &str) -> Option<u32> {
    if description == "no terrace levels" {
        None
    } else {
        let count: u32 = description
            .split_whitespace()
            .next()
            .expect("terrace level count")
            .parse()
            .expect("terrace level count number");
        Some(count)
    }
}

#[when(
    regex = r"^a TerrainNoise is constructed with frequency (-?\d+(?:\.\d+)?|NaN), (\d+) octaves, persistence (-?\d+(?:\.\d+)?|NaN), lacunarity (-?\d+(?:\.\d+)?|NaN), amplitude (-?\d+(?:\.\d+)?|NaN), redistribution exponent (-?\d+(?:\.\d+)?|NaN), and (no terrace levels|\d+ terrace levels?)$"
)]
#[allow(clippy::too_many_arguments)]
fn when_constructed(
    world: &mut TerrainNoiseWorld,
    frequency: String,
    octaves: u32,
    persistence: String,
    lacunarity: String,
    amplitude: String,
    redistribution_exponent: String,
    terrace_levels: String,
) {
    let parse = |s: &str| -> f32 {
        if s == "NaN" {
            f32::NAN
        } else {
            s.parse().expect("numeric fixture")
        }
    };
    world.result = Some(TerrainNoise::new(
        parse(&frequency),
        octaves,
        parse(&persistence),
        parse(&lacunarity),
        parse(&amplitude),
        parse(&redistribution_exponent),
        parse_terrace_levels(&terrace_levels),
    ));
}

#[then("the TerrainNoise is constructed successfully")]
fn then_success(world: &mut TerrainNoiseWorld) {
    assert!(
        world
            .result
            .as_ref()
            .expect("TerrainNoise not constructed")
            .is_ok()
    );
}

#[then(regex = r"^the TerrainNoise has frequency (-?\d+(?:\.\d+)?)$")]
fn then_frequency(world: &mut TerrainNoiseWorld, value: f32) {
    assert_eq!(world.terrain_noise().frequency(), value);
}

#[then(regex = r"^the TerrainNoise has (\d+) octaves$")]
fn then_octaves(world: &mut TerrainNoiseWorld, value: u32) {
    assert_eq!(world.terrain_noise().octaves(), value);
}

#[then(regex = r"^the TerrainNoise has persistence (-?\d+(?:\.\d+)?)$")]
fn then_persistence(world: &mut TerrainNoiseWorld, value: f32) {
    assert_eq!(world.terrain_noise().persistence(), value);
}

#[then(regex = r"^the TerrainNoise has lacunarity (-?\d+(?:\.\d+)?)$")]
fn then_lacunarity(world: &mut TerrainNoiseWorld, value: f32) {
    assert_eq!(world.terrain_noise().lacunarity(), value);
}

#[then(regex = r"^the TerrainNoise has amplitude (-?\d+(?:\.\d+)?)$")]
fn then_amplitude(world: &mut TerrainNoiseWorld, value: f32) {
    assert_eq!(world.terrain_noise().amplitude(), value);
}

#[then(regex = r"^the TerrainNoise has redistribution exponent (-?\d+(?:\.\d+)?)$")]
fn then_redistribution_exponent(world: &mut TerrainNoiseWorld, value: f32) {
    assert_eq!(world.terrain_noise().redistribution_exponent(), value);
}

#[then("the TerrainNoise has no terrace levels")]
fn then_no_terrace_levels(world: &mut TerrainNoiseWorld) {
    assert_eq!(world.terrain_noise().terrace_levels(), None);
}

#[then(regex = r"^the TerrainNoise has (\d+) terrace levels$")]
fn then_terrace_levels(world: &mut TerrainNoiseWorld, value: u32) {
    assert_eq!(world.terrain_noise().terrace_levels(), Some(value));
}

#[then(
    regex = r"^the construction fails with an invalid-frequency error of (-?\d+(?:\.\d+)?|NaN)$"
)]
fn then_invalid_frequency(world: &mut TerrainNoiseWorld, value: String) {
    match world.error() {
        TerrainNoiseError::InvalidFrequency { frequency } => {
            if value == "NaN" {
                assert!(frequency.is_nan(), "expected NaN, got {frequency}");
            } else {
                let expected: f32 = value.parse().expect("numeric fixture");
                assert_eq!(*frequency, expected);
            }
        }
        other => panic!("expected InvalidFrequency, got {other:?}"),
    }
}

#[then(regex = r"^the construction fails with an invalid-octaves error of (\d+)$")]
fn then_invalid_octaves(world: &mut TerrainNoiseWorld, value: u32) {
    match world.error() {
        TerrainNoiseError::InvalidOctaves { octaves } => assert_eq!(*octaves, value),
        other => panic!("expected InvalidOctaves, got {other:?}"),
    }
}

#[then(regex = r"^the construction fails with an invalid-persistence error of (-?\d+(?:\.\d+)?)$")]
fn then_invalid_persistence(world: &mut TerrainNoiseWorld, value: f32) {
    match world.error() {
        TerrainNoiseError::InvalidPersistence { persistence } => assert_eq!(*persistence, value),
        other => panic!("expected InvalidPersistence, got {other:?}"),
    }
}

#[then(regex = r"^the construction fails with an invalid-lacunarity error of (-?\d+(?:\.\d+)?)$")]
fn then_invalid_lacunarity(world: &mut TerrainNoiseWorld, value: f32) {
    match world.error() {
        TerrainNoiseError::InvalidLacunarity { lacunarity } => assert_eq!(*lacunarity, value),
        other => panic!("expected InvalidLacunarity, got {other:?}"),
    }
}

#[then(regex = r"^the construction fails with an invalid-amplitude error of (-?\d+(?:\.\d+)?)$")]
fn then_invalid_amplitude(world: &mut TerrainNoiseWorld, value: f32) {
    match world.error() {
        TerrainNoiseError::InvalidAmplitude { amplitude } => assert_eq!(*amplitude, value),
        other => panic!("expected InvalidAmplitude, got {other:?}"),
    }
}

#[then(
    regex = r"^the construction fails with an invalid-redistribution-exponent error of (-?\d+(?:\.\d+)?)$"
)]
fn then_invalid_redistribution_exponent(world: &mut TerrainNoiseWorld, value: f32) {
    match world.error() {
        TerrainNoiseError::InvalidRedistributionExponent {
            redistribution_exponent,
        } => assert_eq!(*redistribution_exponent, value),
        other => panic!("expected InvalidRedistributionExponent, got {other:?}"),
    }
}

#[then(regex = r"^the construction fails with an invalid-terrace-levels error of (\d+)$")]
fn then_invalid_terrace_levels(world: &mut TerrainNoiseWorld, value: u32) {
    match world.error() {
        TerrainNoiseError::InvalidTerraceLevels { terrace_levels } => {
            assert_eq!(*terrace_levels, value)
        }
        other => panic!("expected InvalidTerraceLevels, got {other:?}"),
    }
}

#[tokio::main]
async fn main() {
    TerrainNoiseWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/terrain_noise.feature")
        .await;
}
