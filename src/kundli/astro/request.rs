use crate::kundli::astro::AstroError;

/// Zodiac mode for astronomical and derivation calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZodiacType {
    /// Sidereal zodiac.
    Sidereal,
    /// Tropical zodiac.
    Tropical,
}

/// Ayanamsha used for sidereal calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ayanamsha {
    /// Lahiri ayanamsha.
    Lahiri,
    /// Raman ayanamsha.
    Raman,
    /// Krishnamurti ayanamsha.
    Krishnamurti,
}

/// House system used for house derivation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HouseSystem {
    /// Placidus house system.
    Placidus,
    /// Koch house system.
    Koch,
    /// Equal house system.
    Equal,
    /// Whole-sign house system.
    WholeSign,
}

/// Node mode used for Rahu and Ketu positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    /// Mean node positions.
    Mean,
    /// True node positions.
    True,
}

/// Astronomical bodies that can be requested from an [`AstroEngine`](crate::kundli::astro::AstroEngine).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AstroBody {
    Sun,
    Moon,
    Mars,
    Mercury,
    Jupiter,
    Venus,
    Saturn,
    Rahu,
    Ketu,
}

/// Input required to calculate raw astronomical positions.
///
/// The high-level kundli API expects several of these fields to match the
/// supplied [`crate::kundli::config::KundliConfig`].
#[derive(Debug, Clone, PartialEq)]
pub struct AstroRequest {
    /// Julian day in Universal Time.
    pub jd_ut: f64,
    /// Geographic latitude in degrees, expected in `-90.0..=90.0`.
    pub latitude: f64,
    /// Geographic longitude in degrees, expected in `-180.0..=180.0`.
    pub longitude: f64,
    /// Zodiac mode used by the backend.
    pub zodiac: ZodiacType,
    /// Ayanamsha used for sidereal calculations.
    pub ayanamsha: Ayanamsha,
    /// House system requested from the backend.
    pub house_system: HouseSystem,
    /// Node mode used for Rahu and Ketu positions.
    pub node_type: NodeType,
    /// Ordered list of bodies to calculate.
    pub bodies: Vec<AstroBody>,
}

impl AstroRequest {
    /// Validates basic structural constraints on the request.
    ///
    /// This method checks only local input validity:
    ///
    /// - `jd_ut` must be finite,
    /// - coordinates must be finite and within supported ranges,
    /// - `bodies` must not be empty.
    ///
    /// It does not verify whether duplicated settings match a separate
    /// [`crate::kundli::config::KundliConfig`].
    pub fn validate(&self) -> Result<(), AstroError> {
        if !self.jd_ut.is_finite() {
            return Err(AstroError::InvalidJulianDay(self.jd_ut));
        }

        if !self.latitude.is_finite()
            || !self.longitude.is_finite()
            || self.latitude < -90.0
            || self.latitude > 90.0
            || self.longitude < -180.0
            || self.longitude > 180.0
        {
            return Err(AstroError::InvalidCoordinates {
                latitude: self.latitude,
                longitude: self.longitude,
            });
        }

        if self.bodies.is_empty() {
            return Err(AstroError::EmptyBodies);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> AstroRequest {
        AstroRequest {
            jd_ut: 2451545.0,
            latitude: 37.5665,
            longitude: 126.978,
            zodiac: ZodiacType::Sidereal,
            ayanamsha: Ayanamsha::Lahiri,
            house_system: HouseSystem::WholeSign,
            node_type: NodeType::True,
            bodies: vec![AstroBody::Sun, AstroBody::Moon],
        }
    }

    #[test]
    fn validate_accepts_valid_request() {
        assert!(sample_request().validate().is_ok());
    }

    #[test]
    fn validate_rejects_out_of_range_coordinates() {
        let mut request = sample_request();
        request.latitude = 120.0;

        assert!(matches!(
            request.validate(),
            Err(AstroError::InvalidCoordinates { .. })
        ));
    }

    #[test]
    fn validate_rejects_empty_bodies() {
        let mut request = sample_request();
        request.bodies.clear();

        assert!(matches!(request.validate(), Err(AstroError::EmptyBodies)));
    }
}
