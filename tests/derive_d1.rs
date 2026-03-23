use kundli_rs::kundli::astro::{
    AstroBody, AstroBodyPosition, AstroMeta, AstroResult, Ayanamsha, HouseSystem, ZodiacType,
};
use kundli_rs::kundli::config::KundliConfig;
use kundli_rs::kundli::derive::d1::{
    derive_d1_chart, derive_houses, derive_lagna, derive_planet_placements,
};
use kundli_rs::kundli::error::DeriveError;
use kundli_rs::kundli::model::{HouseNumber, Nakshatra, Pada, Sign};

const EPSILON: f64 = 1e-10;

fn sample_meta() -> AstroMeta {
    AstroMeta {
        jd_ut: 2451545.0,
        zodiac: ZodiacType::Sidereal,
        ayanamsha: Ayanamsha::Lahiri,
        ayanamsha_value: Some(24.0),
        sidereal_time: 12.0,
    }
}

fn sample_body(body: AstroBody, longitude: f64, speed_longitude: f64) -> AstroBodyPosition {
    AstroBodyPosition {
        body,
        longitude,
        latitude: 0.0,
        distance: 1.0,
        speed_longitude,
    }
}

#[test]
fn derive_lagna_normalizes_negative_ascendant() {
    let astro = AstroResult {
        bodies: vec![],
        ascendant_longitude: -15.0,
        mc_longitude: 90.0,
        house_cusps: vec![],
        meta: sample_meta(),
    };

    let lagna = derive_lagna(&astro).unwrap();

    assert_eq!(lagna.sign, Sign::Pisces);
    assert!((lagna.degrees_in_sign - 15.0).abs() < EPSILON);
    assert!((lagna.longitude - 345.0).abs() < EPSILON);
}

#[test]
fn derive_d1_chart_whole_sign_derives_lagna_planets_and_houses() {
    let astro = AstroResult {
        bodies: vec![
            sample_body(AstroBody::Sun, 50.0, 1.0),
            sample_body(AstroBody::Moon, 5.0, 13.0),
            sample_body(AstroBody::Saturn, 95.0, -0.1),
        ],
        ascendant_longitude: 45.0,
        mc_longitude: 135.0,
        house_cusps: vec![],
        meta: sample_meta(),
    };
    let config = KundliConfig {
        house_system: HouseSystem::WholeSign,
        ..KundliConfig::default()
    };

    let chart = derive_d1_chart(&astro, &config).unwrap();

    assert_eq!(chart.lagna.sign, Sign::Taurus);
    assert!((chart.lagna.degrees_in_sign - 15.0).abs() < EPSILON);
    assert!((chart.lagna.longitude - 45.0).abs() < EPSILON);

    assert_eq!(
        chart
            .planets
            .iter()
            .map(|planet| planet.body)
            .collect::<Vec<_>>(),
        vec![AstroBody::Sun, AstroBody::Moon, AstroBody::Saturn]
    );

    let sun = &chart.planets[0];
    assert_eq!(sun.sign, Sign::Taurus);
    assert!((sun.degrees_in_sign - 20.0).abs() < EPSILON);
    assert_eq!(sun.house, HouseNumber(1));
    assert!(!sun.is_retrograde);

    let moon = &chart.planets[1];
    assert_eq!(moon.sign, Sign::Aries);
    assert!((moon.degrees_in_sign - 5.0).abs() < EPSILON);
    assert_eq!(moon.house, HouseNumber(12));
    assert_eq!(moon.nakshatra.nakshatra, Nakshatra::Ashwini);
    assert_eq!(moon.nakshatra.pada, Pada::new(2).unwrap());

    let saturn = &chart.planets[2];
    assert_eq!(saturn.sign, Sign::Cancer);
    assert!((saturn.degrees_in_sign - 5.0).abs() < EPSILON);
    assert_eq!(saturn.house, HouseNumber(3));
    assert!(saturn.is_retrograde);

    assert_eq!(chart.houses.len(), 12);
    assert_eq!(chart.houses[0].house, HouseNumber(1));
    assert_eq!(chart.houses[0].sign, Sign::Taurus);
    assert!((chart.houses[0].cusp_longitude - 30.0).abs() < EPSILON);
    assert_eq!(chart.houses[11].house, HouseNumber(12));
    assert_eq!(chart.houses[11].sign, Sign::Aries);
    assert!(chart.houses[11].cusp_longitude.abs() < EPSILON);
}

#[test]
fn derive_planet_placements_and_houses_use_cusps_for_non_whole_sign_systems() {
    let astro = AstroResult {
        bodies: vec![sample_body(AstroBody::Mercury, 60.0, 0.5)],
        ascendant_longitude: 45.0,
        mc_longitude: 135.0,
        house_cusps: vec![
            45.0, 75.0, 105.0, 135.0, 165.0, 195.0, 225.0, 255.0, 285.0, 315.0, 345.0, 15.0,
        ],
        meta: sample_meta(),
    };
    let config = KundliConfig {
        house_system: HouseSystem::Equal,
        ..KundliConfig::default()
    };

    let planets = derive_planet_placements(&astro, &config).unwrap();
    let houses = derive_houses(&astro, &config).unwrap();

    assert_eq!(planets.len(), 1);
    assert_eq!(planets[0].sign, Sign::Gemini);
    assert_eq!(planets[0].house, HouseNumber(1));

    assert_eq!(houses.len(), 12);
    assert_eq!(houses[0].house, HouseNumber(1));
    assert_eq!(houses[0].sign, Sign::Taurus);
    assert!((houses[0].cusp_longitude - 45.0).abs() < EPSILON);
    assert_eq!(houses[11].house, HouseNumber(12));
    assert_eq!(houses[11].sign, Sign::Aries);
    assert!((houses[11].cusp_longitude - 15.0).abs() < EPSILON);
}

#[test]
fn derive_lagna_returns_error_for_invalid_ascendant() {
    let astro = AstroResult {
        bodies: vec![],
        ascendant_longitude: f64::NAN,
        mc_longitude: 90.0,
        house_cusps: vec![],
        meta: sample_meta(),
    };

    let error = derive_lagna(&astro).unwrap_err();

    assert!(matches!(error, DeriveError::InvalidLongitude(value) if value.is_nan()));
}

#[test]
fn derive_planet_placements_returns_error_for_invalid_body_longitude() {
    let astro = AstroResult {
        bodies: vec![sample_body(AstroBody::Sun, f64::INFINITY, 1.0)],
        ascendant_longitude: 45.0,
        mc_longitude: 135.0,
        house_cusps: vec![],
        meta: sample_meta(),
    };
    let config = KundliConfig {
        house_system: HouseSystem::WholeSign,
        ..KundliConfig::default()
    };

    let error = derive_planet_placements(&astro, &config).unwrap_err();

    assert!(matches!(error, DeriveError::InvalidLongitude(value) if value.is_infinite()));
}

#[test]
fn derive_houses_returns_error_for_invalid_ascendant_in_whole_sign() {
    let astro = AstroResult {
        bodies: vec![],
        ascendant_longitude: f64::NAN,
        mc_longitude: 90.0,
        house_cusps: vec![],
        meta: sample_meta(),
    };
    let config = KundliConfig {
        house_system: HouseSystem::WholeSign,
        ..KundliConfig::default()
    };

    let error = derive_houses(&astro, &config).unwrap_err();

    assert!(matches!(error, DeriveError::InvalidLongitude(value) if value.is_nan()));
}

#[test]
fn derive_d1_chart_returns_error_for_invalid_cusp_count() {
    let astro = AstroResult {
        bodies: vec![sample_body(AstroBody::Sun, 50.0, 1.0)],
        ascendant_longitude: 45.0,
        mc_longitude: 135.0,
        house_cusps: vec![0.0, 30.0, 60.0],
        meta: sample_meta(),
    };
    let config = KundliConfig {
        house_system: HouseSystem::Placidus,
        ..KundliConfig::default()
    };

    let error = derive_d1_chart(&astro, &config).unwrap_err();

    assert_eq!(error, DeriveError::InvalidHouseCusps(3));
}
