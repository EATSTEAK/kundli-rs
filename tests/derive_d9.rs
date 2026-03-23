#[path = "../src/kundli/derive/house.rs"]
mod house;
#[path = "../src/kundli/derive/nakshatra.rs"]
mod nakshatra;
#[path = "../src/kundli/derive/sign.rs"]
mod sign;

mod kundli {
    pub mod astro {
        pub use kundli_rs::kundli::astro::*;
    }

    pub mod config {
        pub use kundli_rs::kundli::config::*;
    }

    pub mod error {
        pub use kundli_rs::kundli::error::*;
    }

    pub mod model {
        pub use kundli_rs::kundli::model::*;
    }

    pub mod derive {
        pub mod sign {
            pub(crate) use crate::sign::*;
        }

        pub mod nakshatra {
            pub(crate) use crate::nakshatra::*;
        }

        pub mod house {
            pub(crate) use crate::house::*;
        }
    }
}

#[path = "../src/kundli/derive/d9.rs"]
mod d9;

use d9::derive_d9_chart;
use kundli_rs::kundli::astro::{
    AstroBody, AstroBodyPosition, AstroMeta, AstroResult, Ayanamsha, HouseSystem, ZodiacType,
};
use kundli_rs::kundli::config::KundliConfig;
use kundli_rs::kundli::error::DeriveError;
use kundli_rs::kundli::model::{HouseNumber, Nakshatra, Pada, Sign};

const EPSILON: f64 = 1e-10;
const NAVAMSA_BOUNDARY: f64 = 10.0 / 3.0;

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
fn derive_d9_chart_transforms_lagna_and_planets_with_whole_sign_houses() {
    let astro = AstroResult {
        bodies: vec![
            sample_body(AstroBody::Sun, 15.0, 1.0),
            sample_body(AstroBody::Moon, 0.0, 13.0),
            sample_body(AstroBody::Saturn, 32.0, -0.1),
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

    let chart = derive_d9_chart(&astro, &config).unwrap();

    assert_eq!(chart.lagna.sign, Sign::Taurus);
    assert!((chart.lagna.degrees_in_sign - 15.0).abs() < EPSILON);
    assert!((chart.lagna.longitude - 45.0).abs() < EPSILON);

    let sun = &chart.planets[0];
    assert_eq!(sun.sign, Sign::Leo);
    assert!((sun.longitude - 135.0).abs() < EPSILON);
    assert!((sun.degrees_in_sign - 15.0).abs() < EPSILON);
    assert_eq!(sun.house, HouseNumber(4));
    assert!(!sun.is_retrograde);

    let moon = &chart.planets[1];
    assert_eq!(moon.sign, Sign::Aries);
    assert!(moon.longitude.abs() < EPSILON);
    assert!(moon.degrees_in_sign.abs() < EPSILON);
    assert_eq!(moon.house, HouseNumber(12));
    assert_eq!(moon.nakshatra.nakshatra, Nakshatra::Ashwini);
    assert_eq!(moon.nakshatra.pada, Pada::new(1).unwrap());

    let saturn = &chart.planets[2];
    assert_eq!(saturn.sign, Sign::Capricorn);
    assert!((saturn.longitude - 288.0).abs() < EPSILON);
    assert!((saturn.degrees_in_sign - 18.0).abs() < EPSILON);
    assert_eq!(saturn.house, HouseNumber(9));
    assert!(saturn.is_retrograde);
}

#[test]
fn derive_d9_chart_handles_navamsa_boundaries_across_sign_modalities() {
    let astro = AstroResult {
        bodies: vec![
            sample_body(AstroBody::Sun, NAVAMSA_BOUNDARY, 1.0),
            sample_body(AstroBody::Moon, 30.0, 13.0),
            sample_body(AstroBody::Mercury, 60.0, 0.5),
        ],
        ascendant_longitude: 0.0,
        mc_longitude: 90.0,
        house_cusps: vec![],
        meta: sample_meta(),
    };

    let chart = derive_d9_chart(&astro, &KundliConfig::default()).unwrap();

    let sun = &chart.planets[0];
    assert_eq!(sun.sign, Sign::Taurus);
    assert!((sun.longitude - 30.0).abs() < EPSILON);
    assert!(sun.degrees_in_sign.abs() < EPSILON);
    assert_eq!(sun.house, HouseNumber(2));

    let moon = &chart.planets[1];
    assert_eq!(moon.sign, Sign::Capricorn);
    assert!((moon.longitude - 270.0).abs() < EPSILON);
    assert!(moon.degrees_in_sign.abs() < EPSILON);
    assert_eq!(moon.house, HouseNumber(10));

    let mercury = &chart.planets[2];
    assert_eq!(mercury.sign, Sign::Libra);
    assert!((mercury.longitude - 180.0).abs() < EPSILON);
    assert!(mercury.degrees_in_sign.abs() < EPSILON);
    assert_eq!(mercury.house, HouseNumber(7));
}

#[test]
fn derive_d9_chart_rejects_non_whole_sign_house_systems() {
    let astro = AstroResult {
        bodies: vec![sample_body(AstroBody::Mercury, 50.0, 0.5)],
        ascendant_longitude: 45.0,
        mc_longitude: 135.0,
        house_cusps: vec![0.0, 30.0, 60.0],
        meta: sample_meta(),
    };
    let config = KundliConfig {
        house_system: HouseSystem::Equal,
        ..KundliConfig::default()
    };

    let error = derive_d9_chart(&astro, &config).unwrap_err();

    assert_eq!(
        error,
        DeriveError::UnsupportedD9HouseSystem(HouseSystem::Equal)
    );
}

#[test]
fn derive_d9_chart_rejects_non_sidereal_astro_results() {
    let astro = AstroResult {
        bodies: vec![sample_body(AstroBody::Mercury, 50.0, 0.5)],
        ascendant_longitude: 45.0,
        mc_longitude: 135.0,
        house_cusps: vec![],
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
        bodies: vec![],
        ascendant_longitude: f64::NAN,
        mc_longitude: 135.0,
        house_cusps: vec![],
        meta: sample_meta(),
    };

    let error = derive_d9_chart(&astro, &KundliConfig::default()).unwrap_err();

    assert!(matches!(error, DeriveError::InvalidLongitude(value) if value.is_nan()));
}

#[test]
fn derive_d9_chart_returns_error_for_invalid_longitude() {
    let astro = AstroResult {
        bodies: vec![sample_body(AstroBody::Sun, f64::INFINITY, 1.0)],
        ascendant_longitude: 45.0,
        mc_longitude: 135.0,
        house_cusps: vec![],
        meta: sample_meta(),
    };

    let error = derive_d9_chart(&astro, &KundliConfig::default()).unwrap_err();

    assert!(matches!(error, DeriveError::InvalidLongitude(value) if value.is_infinite()));
}
