use std::{fs, path::PathBuf};

use kundli_rs::kundli::astro::{AstroError, AstroRequest};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CoordinateValidationFixture {
    cases: Vec<CoordinateValidationCase>,
}

#[derive(Debug, Deserialize)]
struct CoordinateValidationCase {
    name: String,
    jd_ut: f64,
    latitude: f64,
    longitude: f64,
    expected: String,
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("astro")
        .join(name)
}

fn load_fixture() -> CoordinateValidationFixture {
    serde_json::from_str(&fs::read_to_string(fixture_path("coordinate_validation.json")).unwrap())
        .unwrap()
}

fn sample_request() -> AstroRequest {
    AstroRequest::new(2451545.0, 37.5665, 126.978)
}

#[test]
fn coordinate_validation_fixture_matches_expected_outcomes() {
    let fixture = load_fixture();

    for case in fixture.cases {
        let mut request = sample_request();
        request.jd_ut = case.jd_ut;
        request.latitude = case.latitude;
        request.longitude = case.longitude;

        match case.expected.as_str() {
            "ok" => assert!(
                request.validate().is_ok(),
                "fixture case failed: {}",
                case.name
            ),
            "invalid_coordinates" => assert!(
                matches!(
                    request.validate(),
                    Err(AstroError::InvalidCoordinates { .. })
                ),
                "fixture case failed: {}",
                case.name
            ),
            other => panic!("unsupported expectation: {other}"),
        }
    }
}

#[test]
fn validate_rejects_non_finite_values() {
    let mut request = sample_request();
    request.jd_ut = f64::NAN;
    assert!(matches!(
        request.validate(),
        Err(AstroError::InvalidJulianDay(value)) if value.is_nan()
    ));

    let mut request = sample_request();
    request.latitude = f64::INFINITY;
    assert!(matches!(
        request.validate(),
        Err(AstroError::InvalidCoordinates { latitude, .. }) if latitude.is_infinite()
    ));

    let mut request = sample_request();
    request.longitude = f64::NEG_INFINITY;
    assert!(matches!(
        request.validate(),
        Err(AstroError::InvalidCoordinates { longitude, .. }) if longitude.is_infinite()
    ));
}
