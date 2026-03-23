use crate::kundli::astro::{AstroBody, AstroBodyPosition, AstroMeta, AstroResult};
use crate::kundli::derive::nakshatra::{moon_progress_ratio, nakshatra_placement_from_longitude};
use crate::kundli::derive::sign::{degrees_in_sign, normalize_longitude, sign_from_longitude};
use crate::kundli::error::DeriveError;
use crate::kundli::model::{NakshatraPlacement, Sign};

const NAVAMSAS_PER_SIGN: f64 = 9.0;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PreparedAngle {
    pub longitude: f64,
    pub sign: Sign,
    pub degrees_in_sign: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PreparedBody {
    pub body: AstroBody,
    pub longitude: f64,
    pub sign: Sign,
    pub degrees_in_sign: f64,
    pub nakshatra: NakshatraPlacement,
    pub nakshatra_progress_ratio: f64,
    pub is_retrograde: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct KundliDeriveInput {
    pub meta: AstroMeta,
    pub ascendant: PreparedAngle,
    pub bodies: Vec<PreparedBody>,
    pub house_cusps: Vec<f64>,
}

impl KundliDeriveInput {
    pub(crate) fn from_astro(astro: &AstroResult) -> Result<Self, DeriveError> {
        Ok(Self {
            meta: astro.meta.clone(),
            ascendant: prepare_angle(astro.ascendant_longitude)?,
            bodies: astro
                .bodies
                .iter()
                .map(prepare_body)
                .collect::<Result<Vec<_>, _>>()?,
            house_cusps: astro.house_cusps.clone(),
        })
    }

    pub(crate) fn body(&self, body: AstroBody) -> Option<&PreparedBody> {
        self.bodies.iter().find(|candidate| candidate.body == body)
    }

    pub(crate) fn to_navamsa(&self) -> Result<Self, DeriveError> {
        Ok(Self {
            meta: self.meta.clone(),
            ascendant: prepare_angle(navamsa_longitude(self.ascendant.longitude)?)?,
            bodies: self
                .bodies
                .iter()
                .map(transform_body_to_navamsa)
                .collect::<Result<Vec<_>, _>>()?,
            house_cusps: self.house_cusps.clone(),
        })
    }
}

fn prepare_angle(longitude: f64) -> Result<PreparedAngle, DeriveError> {
    let longitude = normalize_longitude(longitude)?;

    Ok(PreparedAngle {
        longitude,
        sign: sign_from_longitude(longitude)?,
        degrees_in_sign: degrees_in_sign(longitude)?,
    })
}

fn prepare_body(body: &AstroBodyPosition) -> Result<PreparedBody, DeriveError> {
    let longitude = normalize_longitude(body.longitude)?;

    Ok(PreparedBody {
        body: body.body,
        longitude,
        sign: sign_from_longitude(longitude)?,
        degrees_in_sign: degrees_in_sign(longitude)?,
        nakshatra: nakshatra_placement_from_longitude(longitude)?,
        nakshatra_progress_ratio: moon_progress_ratio(longitude)?,
        is_retrograde: body.speed_longitude < 0.0,
    })
}

fn navamsa_longitude(longitude: f64) -> Result<f64, DeriveError> {
    let longitude = normalize_longitude(longitude)?;
    normalize_longitude(longitude * NAVAMSAS_PER_SIGN)
}

fn transform_body_to_navamsa(body: &PreparedBody) -> Result<PreparedBody, DeriveError> {
    let longitude = navamsa_longitude(body.longitude)?;

    Ok(PreparedBody {
        body: body.body,
        longitude,
        sign: sign_from_longitude(longitude)?,
        degrees_in_sign: degrees_in_sign(longitude)?,
        nakshatra: nakshatra_placement_from_longitude(longitude)?,
        nakshatra_progress_ratio: moon_progress_ratio(longitude)?,
        is_retrograde: body.is_retrograde,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kundli::astro::{AstroBody, Ayanamsha, ZodiacType};

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
    fn from_astro_precomputes_ascendant_and_body_snapshots() {
        let astro = AstroResult {
            bodies: vec![
                sample_body(AstroBody::Sun, 390.0, 1.0),
                sample_body(AstroBody::Moon, -10.0, -0.1),
            ],
            ascendant_longitude: -15.0,
            mc_longitude: 90.0,
            house_cusps: vec![],
            meta: sample_meta(),
        };

        let input = KundliDeriveInput::from_astro(&astro).unwrap();

        assert!((input.ascendant.longitude - 345.0).abs() < EPSILON);
        assert_eq!(input.ascendant.sign, Sign::Pisces);
        assert!((input.ascendant.degrees_in_sign - 15.0).abs() < EPSILON);

        assert_eq!(input.bodies.len(), 2);
        assert_eq!(input.bodies[0].body, AstroBody::Sun);
        assert!((input.bodies[0].longitude - 30.0).abs() < EPSILON);
        assert_eq!(input.bodies[0].sign, Sign::Taurus);
        assert!((input.bodies[0].degrees_in_sign - 0.0).abs() < EPSILON);
        assert!(!input.bodies[0].is_retrograde);

        assert_eq!(input.bodies[1].body, AstroBody::Moon);
        assert!((input.bodies[1].longitude - 350.0).abs() < EPSILON);
        assert_eq!(input.bodies[1].sign, Sign::Pisces);
        assert!(input.bodies[1].is_retrograde);
    }

    #[test]
    fn from_astro_preserves_order_and_supports_body_lookup() {
        let astro = AstroResult {
            bodies: vec![
                sample_body(AstroBody::Saturn, 95.0, -0.1),
                sample_body(AstroBody::Moon, 5.0, 13.0),
                sample_body(AstroBody::Sun, 50.0, 1.0),
            ],
            ascendant_longitude: 45.0,
            mc_longitude: 135.0,
            house_cusps: vec![],
            meta: sample_meta(),
        };

        let input = KundliDeriveInput::from_astro(&astro).unwrap();

        assert_eq!(
            input
                .bodies
                .iter()
                .map(|body| body.body)
                .collect::<Vec<_>>(),
            vec![AstroBody::Saturn, AstroBody::Moon, AstroBody::Sun]
        );
        assert_eq!(input.body(AstroBody::Moon).unwrap().body, AstroBody::Moon);
        assert_eq!(input.body(AstroBody::Rahu), None);
    }

    #[test]
    fn to_navamsa_transforms_ascendant_and_bodies() {
        let astro = AstroResult {
            bodies: vec![
                sample_body(AstroBody::Sun, 15.0, 1.0),
                sample_body(AstroBody::Saturn, 32.0, -0.1),
            ],
            ascendant_longitude: 45.0,
            mc_longitude: 135.0,
            house_cusps: vec![],
            meta: sample_meta(),
        };

        let navamsa = KundliDeriveInput::from_astro(&astro)
            .unwrap()
            .to_navamsa()
            .unwrap();

        assert!((navamsa.ascendant.longitude - 45.0).abs() < EPSILON);
        assert_eq!(navamsa.ascendant.sign, Sign::Taurus);
        assert!((navamsa.ascendant.degrees_in_sign - 15.0).abs() < EPSILON);

        assert_eq!(navamsa.bodies[0].body, AstroBody::Sun);
        assert!((navamsa.bodies[0].longitude - 135.0).abs() < EPSILON);
        assert_eq!(navamsa.bodies[0].sign, Sign::Leo);
        assert!((navamsa.bodies[0].degrees_in_sign - 15.0).abs() < EPSILON);
        assert!(!navamsa.bodies[0].is_retrograde);

        assert_eq!(navamsa.bodies[1].body, AstroBody::Saturn);
        assert!((navamsa.bodies[1].longitude - 288.0).abs() < EPSILON);
        assert_eq!(navamsa.bodies[1].sign, Sign::Capricorn);
        assert!(navamsa.bodies[1].is_retrograde);
    }
}
