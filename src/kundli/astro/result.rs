use crate::kundli::astro::{AstroBody, Ayanamsha, ZodiacType};

const BODY_COUNT: usize = AstroBody::ALL.len();
const HOUSE_CUSP_COUNT: usize = 12;

/// A single astronomical body position returned by an [`AstroEngine`](crate::kundli::astro::AstroEngine).
#[derive(Debug, Clone, PartialEq)]
pub struct AstroBodyPosition {
    /// Body identifier matching the request order.
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

/// Raw astronomical output used as input to kundli derivation.
#[derive(Debug, Clone, PartialEq)]
pub struct AstroResult {
    /// Canonical body positions for Sun..Ketu, always in [`AstroBody::ALL`] order.
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
        self.bodies
            .iter()
            .find(|position| position.body == body)
            .expect("AstroResult must contain every AstroBody exactly once")
    }
}
