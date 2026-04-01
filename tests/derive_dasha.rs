use kundli_rs::kundli::astro::{
    AstroBody, AstroBodyPosition, AstroMeta, AstroResult, Ayanamsha, ZodiacType,
};
use kundli_rs::kundli::derive::dasha::derive_vimshottari_dasha;
use kundli_rs::kundli::error::DeriveError;
use kundli_rs::kundli::model::{DashaLord, Nakshatra};

const EPSILON: f64 = 1e-9;
const DAYS_PER_YEAR: f64 = 365.25;
const DEGREES_PER_NAKSHATRA: f64 = 360.0 / 27.0;

fn sample_meta(jd_ut: f64) -> AstroMeta {
    AstroMeta {
        jd_ut,
        zodiac: ZodiacType::Sidereal,
        ayanamsha: Ayanamsha::Lahiri,
        ayanamsha_value: Some(24.0),
        sidereal_time: 12.0,
    }
}

fn sample_body(body: AstroBody, longitude: f64) -> AstroBodyPosition {
    AstroBodyPosition {
        body,
        longitude,
        latitude: 0.0,
        distance: 1.0,
        speed_longitude: 1.0,
    }
}

fn full_bodies(overrides: &[(AstroBody, f64)]) -> [AstroBodyPosition; 9] {
    std::array::from_fn(|index| {
        let body = AstroBody::ALL[index];
        if let Some((_, longitude)) = overrides.iter().find(|(candidate, _)| *candidate == body) {
            sample_body(body, *longitude)
        } else {
            sample_body(body, 180.0 + index as f64)
        }
    })
}

#[test]
fn derive_vimshottari_dasha_derives_current_period_and_full_sequence() {
    let birth_jd_ut = 2451545.0;
    let astro = AstroResult {
        bodies: full_bodies(&[(AstroBody::Moon, DEGREES_PER_NAKSHATRA / 2.0)]),
        ascendant_longitude: 0.0,
        mc_longitude: 90.0,
        house_cusps: [0.0; 12],
        meta: sample_meta(birth_jd_ut),
    };

    let dasha = derive_vimshottari_dasha(&astro).unwrap();
    let half_ketu_days = 7.0 * DAYS_PER_YEAR / 2.0;

    assert_eq!(dasha.moon_nakshatra, Nakshatra::Ashwini);
    assert_eq!(dasha.current_mahadasha.lord, DashaLord::Ketu);
    assert!((dasha.current_mahadasha.start_jd_ut - (birth_jd_ut - half_ketu_days)).abs() < EPSILON);
    assert!((dasha.current_mahadasha.end_jd_ut - (birth_jd_ut + half_ketu_days)).abs() < EPSILON);
    assert_eq!(dasha.mahadashas.len(), 9);
    assert_eq!(dasha.mahadashas[0], dasha.current_mahadasha);
    assert_eq!(
        dasha.mahadashas.iter().map(|period| period.lord).collect::<Vec<_>>(),
        vec![
            DashaLord::Ketu,
            DashaLord::Venus,
            DashaLord::Sun,
            DashaLord::Moon,
            DashaLord::Mars,
            DashaLord::Rahu,
            DashaLord::Jupiter,
            DashaLord::Saturn,
            DashaLord::Mercury,
        ]
    );

    for periods in dasha.mahadashas.windows(2) {
        assert!((periods[0].end_jd_ut - periods[1].start_jd_ut).abs() < EPSILON);
    }
}

#[test]
fn derive_vimshottari_dasha_wraps_sequence_after_mercury() {
    let astro = AstroResult {
        bodies: full_bodies(&[(AstroBody::Moon, 350.0)]),
        ascendant_longitude: 0.0,
        mc_longitude: 90.0,
        house_cusps: [0.0; 12],
        meta: sample_meta(2451545.0),
    };

    let dasha = derive_vimshottari_dasha(&astro).unwrap();

    assert_eq!(dasha.moon_nakshatra, Nakshatra::Revati);
    assert_eq!(dasha.mahadashas[0].lord, DashaLord::Mercury);
    assert_eq!(dasha.mahadashas[1].lord, DashaLord::Ketu);
    assert_eq!(dasha.mahadashas[2].lord, DashaLord::Venus);
}

#[test]
fn derive_vimshottari_dasha_rejects_non_sidereal_astro_results() {
    let astro = AstroResult {
        bodies: full_bodies(&[(AstroBody::Moon, 10.0)]),
        ascendant_longitude: 0.0,
        mc_longitude: 90.0,
        house_cusps: [0.0; 12],
        meta: AstroMeta {
            zodiac: ZodiacType::Tropical,
            ..sample_meta(2451545.0)
        },
    };

    let error = derive_vimshottari_dasha(&astro).unwrap_err();

    assert_eq!(error, DeriveError::UnsupportedZodiac(ZodiacType::Tropical));
}
