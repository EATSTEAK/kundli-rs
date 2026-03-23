use std::{fs, path::PathBuf};

use kundli_rs::kundli::astro::{
    AstroBody, AstroEngine, AstroRequest, Ayanamsha, HouseSystem, NodeType, SwissEphAstroEngine,
    SwissEphConfig, ZodiacType,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct SmokeFixture {
    jd_ut: f64,
    latitude: f64,
    longitude: f64,
    zodiac: String,
    ayanamsha: String,
    house_system: String,
    node_type: String,
    bodies: Vec<String>,
    expected_body_count: usize,
    expected_house_cusp_count: usize,
    expect_ayanamsha_value: bool,
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("astro")
        .join(name)
}

fn load_fixture() -> SmokeFixture {
    serde_json::from_str(&fs::read_to_string(fixture_path("smoke_case.json")).unwrap()).unwrap()
}

fn to_body(body: &str) -> AstroBody {
    match body {
        "Sun" => AstroBody::Sun,
        "Moon" => AstroBody::Moon,
        "Mars" => AstroBody::Mars,
        "Mercury" => AstroBody::Mercury,
        "Jupiter" => AstroBody::Jupiter,
        "Venus" => AstroBody::Venus,
        "Saturn" => AstroBody::Saturn,
        "Rahu" => AstroBody::Rahu,
        "Ketu" => AstroBody::Ketu,
        other => panic!("unsupported AstroBody fixture value: {other}"),
    }
}

fn to_zodiac(zodiac: &str) -> ZodiacType {
    match zodiac {
        "Sidereal" => ZodiacType::Sidereal,
        "Tropical" => ZodiacType::Tropical,
        other => panic!("unsupported ZodiacType fixture value: {other}"),
    }
}

fn to_ayanamsha(ayanamsha: &str) -> Ayanamsha {
    match ayanamsha {
        "Lahiri" => Ayanamsha::Lahiri,
        "Raman" => Ayanamsha::Raman,
        "Krishnamurti" => Ayanamsha::Krishnamurti,
        other => panic!("unsupported Ayanamsha fixture value: {other}"),
    }
}

fn to_house_system(system: &str) -> HouseSystem {
    match system {
        "Placidus" => HouseSystem::Placidus,
        "Koch" => HouseSystem::Koch,
        "Equal" => HouseSystem::Equal,
        "WholeSign" => HouseSystem::WholeSign,
        other => panic!("unsupported HouseSystem fixture value: {other}"),
    }
}

fn to_node_type(node_type: &str) -> NodeType {
    match node_type {
        "Mean" => NodeType::Mean,
        "True" => NodeType::True,
        other => panic!("unsupported NodeType fixture value: {other}"),
    }
}

#[test]
fn smoke_fixture_returns_requested_bodies_and_house_shape() {
    let fixture = load_fixture();
    let request = AstroRequest {
        jd_ut: fixture.jd_ut,
        latitude: fixture.latitude,
        longitude: fixture.longitude,
        zodiac: to_zodiac(&fixture.zodiac),
        ayanamsha: to_ayanamsha(&fixture.ayanamsha),
        house_system: to_house_system(&fixture.house_system),
        node_type: to_node_type(&fixture.node_type),
        bodies: fixture.bodies.iter().map(|body| to_body(body)).collect(),
    };
    let engine = SwissEphAstroEngine::new(SwissEphConfig::new());

    let result = engine.calculate(&request).unwrap();

    assert_eq!(result.bodies.len(), fixture.expected_body_count);
    assert_eq!(result.house_cusps.len(), fixture.expected_house_cusp_count);
    assert_eq!(
        result
            .bodies
            .iter()
            .map(|position| position.body)
            .collect::<Vec<_>>(),
        request.bodies
    );
    assert!(result.ascendant_longitude.is_finite());
    assert!(result.mc_longitude.is_finite());
    assert!(result.house_cusps.iter().all(|cusp| cusp.is_finite()));
    assert!(result.bodies.iter().all(|position| {
        position.longitude.is_finite()
            && position.latitude.is_finite()
            && position.distance.is_finite()
            && position.speed_longitude.is_finite()
    }));
    assert_eq!(result.meta.jd_ut, request.jd_ut);
    assert_eq!(
        result.meta.ayanamsha_value.is_some(),
        fixture.expect_ayanamsha_value
    );
}
