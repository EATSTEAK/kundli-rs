use std::{fs, path::PathBuf};

use kundli_rs::kundli::astro::{
    AstroBody, AstroEngine, AstroRequest, Ayanamsha, HouseSystem, NodeType, SwissEphAstroEngine,
    SwissEphConfig, ZodiacType,
};
use kundli_rs::kundli::calculate::calculate_kundli_with_engine;
use kundli_rs::kundli::config::{KnownChart, KundliConfig};
use kundli_rs::kundli::derive::d1::derive_d1_chart;
use kundli_rs::kundli::derive::d9::derive_d9_chart;
use kundli_rs::kundli::derive::dasha::derive_vimshottari_dasha;
use kundli_rs::kundli::model::ChartLayer;
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

fn build_request(fixture: &SmokeFixture) -> AstroRequest {
    AstroRequest::new(fixture.jd_ut, fixture.latitude, fixture.longitude)
        .with_zodiac(to_zodiac(&fixture.zodiac))
        .with_ayanamsha(to_ayanamsha(&fixture.ayanamsha))
        .with_house_system(to_house_system(&fixture.house_system))
        .with_node_type(to_node_type(&fixture.node_type))
}

fn build_config(request: &AstroRequest) -> KundliConfig {
    KundliConfig::from_request(request).with_charts(&[
        KnownChart::D1,
        KnownChart::D9,
        KnownChart::VimshottariDasha,
    ])
}

#[test]
fn smoke_fixture_returns_full_bodies_and_house_shape() {
    let fixture = load_fixture();
    let request = build_request(&fixture);
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
        AstroBody::ALL.to_vec()
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

#[test]
fn smoke_fixture_derives_d1_d9_and_dasha_from_astro_result() {
    let fixture = load_fixture();
    let request = build_request(&fixture);
    let config = build_config(&request);
    let engine = SwissEphAstroEngine::new(SwissEphConfig::new());

    let result = engine.calculate(&request).unwrap();
    let d1 = derive_d1_chart(&result, &config).unwrap();
    let d9 = derive_d9_chart(&result, &config).unwrap();
    let dasha = derive_vimshottari_dasha(&result).unwrap();

    assert_eq!(d1.planets.len(), fixture.expected_body_count);
    assert_eq!(d1.houses.len(), fixture.expected_house_cusp_count);
    assert_eq!(d9.planets.len(), fixture.expected_body_count);
    assert_eq!(dasha.mahadashas.len(), 9);
    assert_eq!(dasha.current_mahadasha, dasha.mahadashas[0]);
    assert!(dasha.current_mahadasha.start_jd_ut <= result.meta.jd_ut);
    assert!(dasha.current_mahadasha.end_jd_ut > result.meta.jd_ut);
    assert_eq!(
        d1.planets
            .iter()
            .map(|planet| planet.body)
            .collect::<Vec<_>>(),
        AstroBody::ALL.to_vec()
    );
    assert_eq!(
        d9.planets
            .iter()
            .map(|planet| planet.body)
            .collect::<Vec<_>>(),
        AstroBody::ALL.to_vec()
    );
    assert!(d1.lagna.longitude.is_finite());
    assert!(d9.lagna.longitude.is_finite());
    assert!(
        d1.planets
            .iter()
            .all(|planet| planet.longitude.is_finite() && (1..=12).contains(&planet.house.get()))
    );
    assert!(
        d9.planets
            .iter()
            .all(|planet| planet.longitude.is_finite() && (1..=12).contains(&planet.house.get()))
    );
}

#[test]
fn smoke_fixture_final_api_matches_manual_pipeline() {
    let fixture = load_fixture();
    let request = build_request(&fixture);
    let config = build_config(&request);
    let engine = SwissEphAstroEngine::new(SwissEphConfig::new());

    let astro = engine.calculate(&request).unwrap();
    let manual_d1 = derive_d1_chart(&astro, &config).unwrap();
    let manual_d9 = derive_d9_chart(&astro, &config).unwrap();
    let manual_dasha = derive_vimshottari_dasha(&astro).unwrap();
    let final_result = calculate_kundli_with_engine(&engine, &request, &config).unwrap();

    assert_eq!(
        final_result
            .chart(KnownChart::D1)
            .and_then(ChartLayer::as_d1),
        Some(&manual_d1)
    );
    assert_eq!(
        final_result
            .chart(KnownChart::D9)
            .and_then(ChartLayer::as_d9),
        Some(&manual_d9)
    );
    assert_eq!(
        final_result
            .chart(KnownChart::VimshottariDasha)
            .and_then(ChartLayer::as_vimshottari_dasha),
        Some(&manual_dasha)
    );
    assert_eq!(final_result.meta.jd_ut, request.jd_ut);
    assert_eq!(final_result.meta.zodiac, request.zodiac);
    assert_eq!(final_result.meta.ayanamsha, request.ayanamsha);
    assert_eq!(final_result.meta.house_system, request.house_system);
    assert_eq!(final_result.meta.node_type, request.node_type);
    assert_eq!(final_result.meta.body_count, fixture.expected_body_count);
    assert!(final_result.warnings.is_empty());
}

#[test]
fn smoke_fixture_final_api_omits_optional_results_when_disabled() {
    let fixture = load_fixture();
    let request = build_request(&fixture);
    let config = KundliConfig::from_request(&request).with_charts(&[KnownChart::D1]);
    let engine = SwissEphAstroEngine::new(SwissEphConfig::new());

    let final_result = calculate_kundli_with_engine(&engine, &request, &config).unwrap();
    let d1 = final_result
        .chart(KnownChart::D1)
        .and_then(ChartLayer::as_d1)
        .unwrap();

    assert!(final_result.chart(KnownChart::D1).is_some());
    assert!(final_result.chart(KnownChart::D9).is_none());
    assert!(final_result.chart(KnownChart::VimshottariDasha).is_none());
    assert_eq!(d1.planets.len(), fixture.expected_body_count);
    assert_eq!(d1.houses.len(), fixture.expected_house_cusp_count);
}
