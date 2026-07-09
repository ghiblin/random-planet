use cucumber::{World as _, then, when};
use planet_core::color::rgb::{Rgb, RgbError};

#[derive(Debug, Default, cucumber::World)]
pub struct RgbWorld {
    result: Option<Result<Rgb, RgbError>>,
}

impl RgbWorld {
    fn rgb(&self) -> Rgb {
        *self
            .result
            .as_ref()
            .expect("Rgb not constructed")
            .as_ref()
            .expect("Rgb construction failed")
    }
}

#[when(
    regex = r"^an Rgb is constructed with r (-?\d+(?:\.\d+)?), g (-?\d+(?:\.\d+)?), b (-?\d+(?:\.\d+)?)$"
)]
fn when_constructed(world: &mut RgbWorld, r: f32, g: f32, b: f32) {
    world.result = Some(Rgb::new(r, g, b));
}

#[then("the Rgb is constructed successfully")]
fn then_success(world: &mut RgbWorld) {
    assert!(world.result.as_ref().expect("Rgb not constructed").is_ok());
}

#[then(regex = r"^the Rgb has r (-?\d+(?:\.\d+)?), g (-?\d+(?:\.\d+)?), b (-?\d+(?:\.\d+)?)$")]
fn then_has_channels(world: &mut RgbWorld, r: f32, g: f32, b: f32) {
    let rgb = world.rgb();
    assert_eq!(rgb.r(), r);
    assert_eq!(rgb.g(), g);
    assert_eq!(rgb.b(), b);
}

#[then(
    regex = r"^the construction fails with an out-of-range error of r (-?\d+(?:\.\d+)?), g (-?\d+(?:\.\d+)?), b (-?\d+(?:\.\d+)?)$"
)]
fn then_out_of_range(world: &mut RgbWorld, r: f32, g: f32, b: f32) {
    match world.result.as_ref().expect("Rgb not constructed") {
        Err(RgbError::OutOfRange {
            r: actual_r,
            g: actual_g,
            b: actual_b,
        }) => {
            assert_eq!(*actual_r, r);
            assert_eq!(*actual_g, g);
            assert_eq!(*actual_b, b);
        }
        other => panic!("expected OutOfRange, got {other:?}"),
    }
}

#[tokio::main]
async fn main() {
    RgbWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/rgb.feature")
        .await;
}
