use crate::kundli::astro::{AstroBody, Ayanamsha, ZodiacType};

const BODY_COUNT: usize = AstroBody::ALL.len();
const HOUSE_CUSP_COUNT: usize = 12;

/// A single canonical astronomical body position returned by an [`AstroEngine`](crate::kundli::astro::AstroEngine).
#[derive(Debug, Clone, PartialEq)]
pub struct AstroBodyPosition {
    /// Body identifier matching the canonical [`AstroBody::ALL`] order.
    pub body: AstroBody,
    /// Ecliptic longitude in degrees.
    pub longitude: f64,
    /// Ecliptic latitude in degrees.
    pub latitude: f64,
    /// Distance reported by the backend.
    pub distance: f64,
    /// Instantaneous longitudinal speed in degrees per day.
    pub speed_longitude: f64,
}

/// Metadata accompanying an [`AstroResult`].
#[derive(Debug, Clone, PartialEq)]
pub struct AstroMeta {
    /// The requested Julian day in Universal Time.
    pub jd_ut: f64,
    /// Zodiac mode used for the calculation.
    pub zodiac: ZodiacType,
    /// Ayanamsha used for sidereal calculations.
    pub ayanamsha: Ayanamsha,
    /// Numeric ayanamsha value reported by the backend, when applicable.
    pub ayanamsha_value: Option<f64>,
    /// Sidereal time reported by the backend.
    pub sidereal_time: f64,
}

/// Derivation-ready astronomical snapshot used as input to kundli derivation.
#[derive(Debug, Clone, PartialEq)]
pub struct AstroResult {
    /// Canonical body positions for Sun..Ketu, always in [`AstroBody::ALL`] order.
    /// Use [`AstroResult::body`] for enum-addressed lookup by canonical slot.
    pub bodies: [AstroBodyPosition; BODY_COUNT],
    /// Ascendant longitude in degrees.
    pub ascendant_longitude: f64,
    /// Midheaven longitude in degrees.
    pub mc_longitude: f64,
    /// House cusp longitudes in degrees.
    pub house_cusps: [f64; HOUSE_CUSP_COUNT],
    /// Calculation metadata.
    pub meta: AstroMeta,
}

impl AstroResult {
    pub fn body(&self, body: AstroBody) -> &AstroBodyPosition {
        let position = &self.bodies[body.index()];
        debug_assert_eq!(position.body, body);
        position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_body(body: AstroBody, longitude: f64) -> AstroBodyPosition {
        AstroBodyPosition {
            body,
            longitude,
            latitude: 0.0,
            distance: 1.0,
            speed_longitude: 0.0,
        }
    }

    #[test]
    fn body_uses_canonical_slot_lookup() {
        let bodies = std::array::from_fn(|index| {
            let body = AstroBody::ALL[index];
            sample_body(body, index as f64 * 10.0)
        });
        let result = AstroResult {
            bodies,
            ascendant_longitude: 0.0,
            mc_longitude: 0.0,
            house_cusps: [0.0; HOUSE_CUSP_COUNT],
            meta: AstroMeta {
                jd_ut: 2451545.0,
                zodiac: ZodiacType::Sidereal,
                ayanamsha: Ayanamsha::Lahiri,
                ayanamsha_value: Some(24.0),
                sidereal_time: 12.0,
            },
        };

        let moon = result.body(AstroBody::Moon);
        let ketu = result.body(AstroBody::Ketu);

        assert_eq!(moon.body, AstroBody::Moon);
        assert_eq!(moon.longitude, 10.0);
        assert_eq!(ketu.body, AstroBody::Ketu);
        assert_eq!(ketu.longitude, 80.0);
    }
}
