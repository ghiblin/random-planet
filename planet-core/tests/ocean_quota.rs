use cucumber::{World as _, given, then, when};
use planet_core::processor::ocean_quota::{OceanQuota, OceanQuotaError};

#[derive(Debug, Default, cucumber::World)]
pub struct OceanQuotaWorld {
    result: Option<Result<OceanQuota, OceanQuotaError>>,
}

impl OceanQuotaWorld {
    fn value(&self) -> f32 {
        self.result
            .as_ref()
            .expect("OceanQuota not constructed")
            .as_ref()
            .expect("OceanQuota construction failed")
            .value()
    }
}

#[when(regex = r"^an OceanQuota is constructed with value (-?\d+(?:\.\d+)?|NaN)$")]
fn when_constructed(world: &mut OceanQuotaWorld, value: f32) {
    world.result = Some(OceanQuota::new(value));
}

#[given("the default OceanQuota")]
fn given_default(world: &mut OceanQuotaWorld) {
    world.result = Some(Ok(OceanQuota::default()));
}

#[then("the OceanQuota is constructed successfully")]
fn then_success(world: &mut OceanQuotaWorld) {
    assert!(
        world
            .result
            .as_ref()
            .expect("OceanQuota not constructed")
            .is_ok()
    );
}

#[then(regex = r"^the OceanQuota has value (-?\d+(?:\.\d+)?)$")]
fn then_value(world: &mut OceanQuotaWorld, value: f32) {
    assert_eq!(world.value(), value);
}

#[then(regex = r"^the construction fails with an out-of-range error of (-?\d+(?:\.\d+)?)$")]
fn then_out_of_range_error(world: &mut OceanQuotaWorld, value: f32) {
    match world.result.as_ref().expect("OceanQuota not constructed") {
        Err(OceanQuotaError::OutOfRange { value: actual }) => assert_eq!(*actual, value),
        other => panic!("expected OutOfRange, got {other:?}"),
    }
}

#[then("the construction fails with an out-of-range error of NaN")]
fn then_out_of_range_error_nan(world: &mut OceanQuotaWorld) {
    match world.result.as_ref().expect("OceanQuota not constructed") {
        Err(OceanQuotaError::OutOfRange { value: actual }) => {
            assert!(actual.is_nan(), "expected NaN, got {actual}")
        }
        other => panic!("expected OutOfRange, got {other:?}"),
    }
}

#[tokio::main]
async fn main() {
    OceanQuotaWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features/ocean_quota.feature")
        .await;
}
