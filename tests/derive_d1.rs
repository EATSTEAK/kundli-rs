use kundli_rs::kundli::astro::{
    AstroBody, AstroBodyPosition, AstroMeta, AstroResult, Ayanamsha, HouseSystem, ZodiacType,
};
use kundli_rs::kundli::config::KundliConfig;
use kundli_rs::kundli::derive::d1::derive_d1_chart;
use kundli_rs::kundli::error::DeriveError;
use kundli_rs::kundli::model::{HouseNumber, Nakshatra, Pada, Sign};

const EPSILON: f64 = 1e-10;

fn house(number: u8) -> HouseNumber {
    HouseNumber::new(number).unwrap()
}

fn sample_meta() -> AstroMeta {
    AstroMeta {
        jd_ut: 2451545.0,
        latitude: 37.5665,
        longitude: 126.9780,
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

fn full_bodies(overrides: &[(AstroBody, f64, f64)]) -> [AstroBodyPosition; 9] {
    std::array::from_fn(|index| {
        let body = AstroBody::ALL[index];
        if let Some((_, longitude, speed_longitude)) = overrides
            .iter()
            .find(|(candidate, _, _)| *candidate == body)
        {
            sample_body(body, *longitude, *speed_longitude)
        } else {
            sample_body(body, 180.0 + index as f64, 0.1)
        }
    })
}

#[test]
fn derive_d1_chart_normalizes_negative_ascendant() {
    let astro = AstroResult {
        bodies: full_bodies(&[]),
        ascendant_longitude: -15.0,
        mc_longitude: 90.0,
        house_cusps: [0.0; 12],
        meta: sample_meta(),
    };

    let chart = derive_d1_chart(&astro, &KundliConfig::default()).unwrap();

    assert_eq!(chart.lagna.sign, Sign::Pisces);
    assert!((chart.lagna.degrees_in_sign - 15.0).abs() < EPSILON);
    assert!((chart.lagna.longitude - 345.0).abs() < EPSILON);
}

#[test]
fn derive_d1_chart_whole_sign_derives_lagna_planets_and_houses() {
    let astro = AstroResult {
        bodies: full_bodies(&[
            (AstroBody::Sun, 50.0, 1.0),
            (AstroBody::Moon, 5.0, 13.0),
            (AstroBody::Saturn, 95.0, -0.1),
        ]),
        ascendant_longitude: 45.0,
        mc_longitude: 135.0,
        house_cusps: [0.0; 12],
        meta: sample_meta(),
    };
    let config = KundliConfig::default().with_house_system(HouseSystem::WholeSign);

    let chart = derive_d1_chart(&astro, &config).unwrap();

    assert_eq!(chart.lagna.sign, Sign::Taurus);
    assert!((chart.lagna.degrees_in_sign - 15.0).abs() < EPSILON);
    assert!((chart.lagna.longitude - 45.0).abs() < EPSILON);
    assert_eq!(chart.planets.len(), AstroBody::ALL.len());

    let sun = chart
        .planets
        .iter()
        .find(|planet| planet.body == AstroBody::Sun)
        .unwrap();
    assert_eq!(sun.sign, Sign::Taurus);
    assert!((sun.degrees_in_sign - 20.0).abs() < EPSILON);
    assert_eq!(sun.house, house(1));
    assert!(!sun.is_retrograde);

    let moon = chart
        .planets
        .iter()
        .find(|planet| planet.body == AstroBody::Moon)
        .unwrap();
    assert_eq!(moon.sign, Sign::Aries);
    assert!((moon.degrees_in_sign - 5.0).abs() < EPSILON);
    assert_eq!(moon.house, house(12));
    assert_eq!(moon.nakshatra.nakshatra, Nakshatra::Ashwini);
    assert_eq!(moon.nakshatra.pada, Pada::new(2).unwrap());

    let saturn = chart
        .planets
        .iter()
        .find(|planet| planet.body == AstroBody::Saturn)
        .unwrap();
    assert_eq!(saturn.sign, Sign::Cancer);
    assert!((saturn.degrees_in_sign - 5.0).abs() < EPSILON);
    assert_eq!(saturn.house, house(3));
    assert!(saturn.is_retrograde);

    assert_eq!(chart.houses.len(), 12);
    assert_eq!(chart.houses[0].house, house(1));
    assert_eq!(chart.houses[0].sign, Sign::Taurus);
    assert!((chart.houses[0].cusp_longitude - 30.0).abs() < EPSILON);
    assert_eq!(chart.houses[11].house, house(12));
    assert_eq!(chart.houses[11].sign, Sign::Aries);
    assert!(chart.houses[11].cusp_longitude.abs() < EPSILON);
}

#[test]
fn derive_d1_chart_uses_cusps_for_non_whole_sign_systems() {
    let astro = AstroResult {
        bodies: full_bodies(&[(AstroBody::Mercury, 60.0, 0.5)]),
        ascendant_longitude: 45.0,
        mc_longitude: 135.0,
        house_cusps: [
            45.0, 75.0, 105.0, 135.0, 165.0, 195.0, 225.0, 255.0, 285.0, 315.0, 345.0, 15.0,
        ],
        meta: sample_meta(),
    };
    let config = KundliConfig::default().with_house_system(HouseSystem::Equal);

    let chart = derive_d1_chart(&astro, &config).unwrap();

    let mercury = chart
        .planets
        .iter()
        .find(|planet| planet.body == AstroBody::Mercury)
        .unwrap();
    assert_eq!(mercury.sign, Sign::Gemini);
    assert_eq!(mercury.house, house(1));

    assert_eq!(chart.houses.len(), 12);
    assert_eq!(chart.houses[0].house, house(1));
    assert_eq!(chart.houses[0].sign, Sign::Taurus);
    assert!((chart.houses[0].cusp_longitude - 45.0).abs() < EPSILON);
    assert_eq!(chart.houses[11].house, house(12));
    assert_eq!(chart.houses[11].sign, Sign::Aries);
    assert!((chart.houses[11].cusp_longitude - 15.0).abs() < EPSILON);
}

#[test]
fn derive_d1_chart_returns_error_for_invalid_ascendant() {
    let astro = AstroResult {
        bodies: full_bodies(&[]),
        ascendant_longitude: f64::NAN,
        mc_longitude: 90.0,
        house_cusps: [0.0; 12],
        meta: sample_meta(),
    };

    let error = derive_d1_chart(&astro, &KundliConfig::default()).unwrap_err();

    assert!(matches!(error, DeriveError::InvalidLongitude(value) if value.is_nan()));
}

#[test]
fn derive_d1_chart_returns_error_for_invalid_body_longitude() {
    let astro = AstroResult {
        bodies: full_bodies(&[(AstroBody::Sun, f64::INFINITY, 1.0)]),
        ascendant_longitude: 45.0,
        mc_longitude: 135.0,
        house_cusps: [0.0; 12],
        meta: sample_meta(),
    };
    let config = KundliConfig::default().with_house_system(HouseSystem::WholeSign);

    let error = derive_d1_chart(&astro, &config).unwrap_err();

    assert!(matches!(error, DeriveError::InvalidLongitude(value) if value.is_infinite()));
}
