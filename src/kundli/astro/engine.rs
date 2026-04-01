use crate::kundli::astro::{
    AstroBodyPosition, AstroError, AstroMeta, AstroRequest, AstroResult, ephemeris::Ephemeris,
};

/// Backend abstraction for astronomical calculations.
pub trait AstroEngine {
    /// Calculates raw astronomical output for a validated request.
    fn calculate(&self, request: &AstroRequest) -> Result<AstroResult, AstroError>;
}

/// Configuration for [`SwissEphAstroEngine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SwissEphConfig {
    ephemeris_path: Option<String>,
}

impl SwissEphConfig {
    /// Creates an empty Swiss Ephemeris configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the ephemeris path used by the backend.
    pub fn with_ephemeris_path(mut self, ephemeris_path: impl Into<String>) -> Self {
        self.ephemeris_path = Some(ephemeris_path.into());
        self
    }

    pub(crate) fn ephemeris_path(&self) -> Option<&str> {
        self.ephemeris_path.as_deref()
    }
}

/// Default [`AstroEngine`] implementation backed by Swiss Ephemeris.
#[derive(Debug, Clone, Default)]
pub struct SwissEphAstroEngine {
    config: SwissEphConfig,
}

impl SwissEphAstroEngine {
    /// Creates a Swiss Ephemeris-backed engine with the provided config.
    pub fn new(config: SwissEphConfig) -> Self {
        Self { config }
    }
}

impl AstroEngine for SwissEphAstroEngine {
    fn calculate(&self, request: &AstroRequest) -> Result<AstroResult, AstroError> {
        request.validate()?;

        let raw = Ephemeris::calculate(request, &self.config)?;
        let bodies = std::array::from_fn(|index| {
            let body = crate::kundli::astro::AstroBody::ALL[index];
            let position = raw.bodies[index];
            AstroBodyPosition {
                body,
                longitude: position.longitude,
                latitude: position.latitude,
                distance: position.distance,
                speed_longitude: position.longitude_speed,
            }
        });

        Ok(AstroResult {
            bodies,
            ascendant_longitude: raw.houses.ascendant,
            mc_longitude: raw.houses.mc,
            house_cusps: raw.houses.cusps,
            meta: AstroMeta {
                jd_ut: request.jd_ut,
                zodiac: request.zodiac,
                ayanamsha: request.ayanamsha,
                ayanamsha_value: raw.ayanamsha_value,
                sidereal_time: raw.sidereal_time,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample_request() -> AstroRequest {
        AstroRequest::new(2451545.0, 37.5665, 126.978)
    }

    #[test]
    fn calculate_returns_full_body_set_and_house_shape() {
        let engine = SwissEphAstroEngine::default();
        let request = sample_request();
        let result = engine.calculate(&request).unwrap();

        assert_eq!(
            result.bodies.len(),
            crate::kundli::astro::AstroBody::ALL.len()
        );
        assert_eq!(result.house_cusps.len(), 12);
        assert!(result.ascendant_longitude >= 0.0);
        assert!(result.mc_longitude >= 0.0);
        assert_eq!(result.meta.jd_ut, request.jd_ut);
        assert!(result.meta.ayanamsha_value.is_some());
    }

    #[test]
    fn engine_accepts_declarative_config() {
        let engine = SwissEphAstroEngine::new(SwissEphConfig::new().with_ephemeris_path(""));

        let result = engine.calculate(&sample_request()).unwrap();

        assert_eq!(
            result.bodies.len(),
            crate::kundli::astro::AstroBody::ALL.len()
        );
    }
}
