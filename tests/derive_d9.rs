use kundli_rs::kundli::astro::{
    AstroBody, AstroBodyPosition, AstroMeta, AstroResult, Ayanamsha, HouseSystem, ZodiacType,
};
use kundli_rs::kundli::config::KundliConfig;
use kundli_rs::kundli::derive::d9::derive_d9_chart;
use kundli_rs::kundli::error::DeriveError;
use kundli_rs::kundli::model::{HouseNumber, Nakshatra, Pada, Sign};

const EPSILON: f64 = 1e-10;
const NAVAMSA_BOUNDARY: f64 = 10.0 / 3.0;

fn house(number: u8) -> HouseNumber {
    HouseNumber::new(number).unwrap()
}

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

fn full_bodies(overrides: &[(AstroBody, f64, f64)]) -> [AstroBodyPosition; 9] {
    std::array::from_fn(|index| {
        let body = AstroBody::ALL[index];
        if let Some((_, longitude, speed_longitude)) = overrides.iter().find(|(candidate, _, _)| *candidate == body) {
            sample_body(body, *longitude, *speed_longitude)
        } else {
            sample_body(body, 180.0 + index as f64, 0.1)
        }
    })
}

#[test]
fn derive_d9_chart_transforms_lagna_and_planets_with_whole_sign_houses() {
    let astro = AstroResult {
        bodies: full_bodies(&[
            (AstroBody::Sun, 15.0, 1.0),
            (AstroBody::Moon, 0.0, 13.0),
            (AstroBody::Saturn, 32.0, -0.1),
        ]),
        ascendant_longitude: 45.0,
        mc_longitude: 135.0,
        house_cusps: [0.0; 12],
        meta: sample_meta(),
    };
    let config = KundliConfig::default().with_house_system(HouseSystem::WholeSign);

    let chart = derive_d9_chart(&astro, &config).unwrap();

    assert_eq!(chart.lagna.sign, Sign::Taurus);
    assert!((chart.lagna.degrees_in_sign - 15.0).abs() < EPSILON);
    assert!((chart.lagna.longitude - 45.0).abs() < EPSILON);

    let sun = chart.planets.iter().find(|planet| planet.body == AstroBody::Sun).unwrap();
    assert_eq!(sun.sign, Sign::Leo);
    assert!((sun.longitude - 135.0).abs() < EPSILON);
    assert!((sun.degrees_in_sign - 15.0).abs() < EPSILON);
    assert_eq!(sun.house, house(4));
    assert!(!sun.is_retrograde);

    let moon = chart.planets.iter().find(|planet| planet.body == AstroBody::Moon).unwrap();
    assert_eq!(moon.sign, Sign::Aries);
    assert!(moon.longitude.abs() < EPSILON);
    assert!(moon.degrees_in_sign.abs() < EPSILON);
    assert_eq!(moon.house, house(12));
    assert_eq!(moon.nakshatra.nakshatra, Nakshatra::Ashwini);
    assert_eq!(moon.nakshatra.pada, Pada::new(1).unwrap());

    let saturn = chart.planets.iter().find(|planet| planet.body == AstroBody::Saturn).unwrap();
    assert_eq!(saturn.sign, Sign::Capricorn);
    assert!((saturn.longitude - 288.0).abs() < EPSILON);
    assert!((saturn.degrees_in_sign - 18.0).abs() < EPSILON);
    assert_eq!(saturn.house, house(9));
    assert!(saturn.is_retrograde);
}

#[test]
fn derive_d9_chart_recomputes_nakshatra_progress_after_navamsa_transform() {
    let astro = AstroResult {
        bodies: full_bodies(&[(AstroBody::Moon, 5.0, 13.0)]),
        ascendant_longitude: 10.0,
        mc_longitude: 100.0,
        house_cusps: [0.0; 12],
        meta: sample_meta(),
    };

    let chart = derive_d9_chart(&astro, &KundliConfig::default()).unwrap();
    let moon = chart.planets.iter().find(|planet| planet.body == AstroBody::Moon).unwrap();

    assert!((moon.longitude - 45.0).abs() < EPSILON);
    assert_eq!(moon.nakshatra.nakshatra, Nakshatra::Rohini);
    assert_eq!(moon.nakshatra.pada, Pada::new(2).unwrap());
    assert!((moon.nakshatra.degrees_in_nakshatra - (45.0 - (360.0 / 27.0 * 3.0))).abs() < EPSILON);
}

#[test]
fn derive_d9_chart_handles_navamsa_boundaries_across_sign_modalities() {
    let astro = AstroResult {
        bodies: full_bodies(&[
            (AstroBody::Sun, NAVAMSA_BOUNDARY, 1.0),
            (AstroBody::Moon, 30.0, 13.0),
            (AstroBody::Mercury, 60.0, 0.5),
        ]),
        ascendant_longitude: 0.0,
        mc_longitude: 90.0,
        house_cusps: [0.0; 12],
        meta: sample_meta(),
    };

    let chart = derive_d9_chart(&astro, &KundliConfig::default()).unwrap();

    let sun = chart.planets.iter().find(|planet| planet.body == AstroBody::Sun).unwrap();
    assert_eq!(sun.sign, Sign::Taurus);
    assert!((sun.longitude - 30.0).abs() < EPSILON);
    assert!(sun.degrees_in_sign.abs() < EPSILON);
    assert_eq!(sun.house, house(2));

    let moon = chart.planets.iter().find(|planet| planet.body == AstroBody::Moon).unwrap();
    assert_eq!(moon.sign, Sign::Capricorn);
    assert!((moon.longitude - 270.0).abs() < EPSILON);
    assert!(moon.degrees_in_sign.abs() < EPSILON);
    assert_eq!(moon.house, house(10));

    let mercury = chart.planets.iter().find(|planet| planet.body == AstroBody::Mercury).unwrap();
    assert_eq!(mercury.sign, Sign::Libra);
    assert!((mercury.longitude - 180.0).abs() < EPSILON);
    assert!(mercury.degrees_in_sign.abs() < EPSILON);
    assert_eq!(mercury.house, house(7));
}

#[test]
fn derive_d9_chart_rejects_non_whole_sign_house_systems() {
    let astro = AstroResult {
        bodies: full_bodies(&[(AstroBody::Mercury, 50.0, 0.5)]),
        ascendant_longitude: 45.0,
        mc_longitude: 135.0,
        house_cusps: [0.0; 12],
        meta: sample_meta(),
    };
    let config = KundliConfig::default().with_house_system(HouseSystem::Equal);

    let error = derive_d9_chart(&astro, &config).unwrap_err();

    assert_eq!(error, DeriveError::UnsupportedD9HouseSystem(HouseSystem::Equal));
}

#[test]
fn derive_d9_chart_rejects_non_sidereal_astro_results() {
    let astro = AstroResult {
        bodies: full_bodies(&[(AstroBody::Mercury, 50.0, 0.5)]),
        ascendant_longitude: 45.0,
        mc_longitude: 135.0,
        house_cusps: [0.0; 12],
        meta: AstroMeta {
            zodiac: ZodiacType::Tropical,
            ..sample_meta()
        },
    };

    let error = derive_d9_chart(&astro, &KundliConfig::default()).unwrap_err();

    assert_eq!(error, DeriveError::UnsupportedZodiac(ZodiacType::Tropical));
}

#[test]
fn derive_d9_chart_returns_error_for_invalid_ascendant() {
    let astro = AstroResult {
        bodies: full_bodies(&[]),
        ascendant_longitude: f64::NAN,
        mc_longitude: 135.0,
        house_cusps: [0.0; 12],
        meta: sample_meta(),
    };

    let error = derive_d9_chart(&astro, &KundliConfig::default()).unwrap_err();

    assert!(matches!(error, DeriveError::InvalidLongitude(value) if value.is_nan()));
}

#[test]
fn derive_d9_chart_returns_error_for_invalid_longitude() {
    let astro = AstroResult {
        bodies: full_bodies(&[(AstroBody::Sun, f64::INFINITY, 1.0)]),
        ascendant_longitude: 45.0,
        mc_longitude: 135.0,
        house_cusps: [0.0; 12],
        meta: sample_meta(),
    };

    let error = derive_d9_chart(&astro, &KundliConfig::default()).unwrap_err();

    assert!(matches!(error, DeriveError::InvalidLongitude(value) if value.is_infinite()));
}
