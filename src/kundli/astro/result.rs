use crate::kundli::astro::{AstroBody, Ayanamsha, ZodiacType};

#[derive(Debug, Clone, PartialEq)]
pub struct AstroBodyPosition {
    pub body: AstroBody,
    pub longitude: f64,
    pub latitude: f64,
    pub distance: f64,
    pub speed_longitude: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstroMeta {
    pub zodiac: ZodiacType,
    pub ayanamsha: Ayanamsha,
    pub ayanamsha_value: Option<f64>,
    pub sidereal_time: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstroResult {
    pub bodies: Vec<AstroBodyPosition>,
    pub ascendant_longitude: f64,
    pub mc_longitude: f64,
    pub house_cusps: Vec<f64>,
    pub meta: AstroMeta,
}
