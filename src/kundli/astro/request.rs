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

/// Astronomical bodies included in a derivation-ready [`AstroResult`](crate::kundli::astro::AstroResult).
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

impl AstroBody {
    pub const ALL: [Self; 9] = [
        Self::Sun,
        Self::Moon,
        Self::Mars,
        Self::Mercury,
        Self::Jupiter,
        Self::Venus,
        Self::Saturn,
        Self::Rahu,
        Self::Ketu,
    ];

    pub const fn index(self) -> usize {
        match self {
            Self::Sun => 0,
            Self::Moon => 1,
            Self::Mars => 2,
            Self::Mercury => 3,
            Self::Jupiter => 4,
            Self::Venus => 5,
            Self::Saturn => 6,
            Self::Rahu => 7,
            Self::Ketu => 8,
        }
    }
}

/// Input required to calculate raw astronomical positions.
///
/// Use [`AstroRequest::new`] for the common construction path and the `with_*`
/// methods to override zodiac-related settings when needed.
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
}

impl AstroRequest {
    /// Creates a request with the crate's default kundli settings.
    ///
    /// Defaults:
    ///
    /// - [`ZodiacType::Sidereal`]
    /// - [`Ayanamsha::Lahiri`]
    /// - [`HouseSystem::WholeSign`]
    /// - [`NodeType::True`]
    pub fn new(jd_ut: f64, latitude: f64, longitude: f64) -> Self {
        Self {
            jd_ut,
            latitude,
            longitude,
            zodiac: ZodiacType::Sidereal,
            ayanamsha: Ayanamsha::Lahiri,
            house_system: HouseSystem::WholeSign,
            node_type: NodeType::True,
        }
    }

    /// Returns a copy with a different zodiac mode.
    pub fn with_zodiac(mut self, zodiac: ZodiacType) -> Self {
        self.zodiac = zodiac;
        self
    }

    /// Returns a copy with a different ayanamsha.
    pub fn with_ayanamsha(mut self, ayanamsha: Ayanamsha) -> Self {
        self.ayanamsha = ayanamsha;
        self
    }

    /// Returns a copy with a different house system.
    pub fn with_house_system(mut self, house_system: HouseSystem) -> Self {
        self.house_system = house_system;
        self
    }

    /// Returns a copy with a different node mode.
    pub fn with_node_type(mut self, node_type: NodeType) -> Self {
        self.node_type = node_type;
        self
    }

    /// Validates basic structural constraints on the request.
    ///
    /// This method checks only local input validity:
    ///
    /// - `jd_ut` must be finite,
    /// - coordinates must be finite and within supported ranges.
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

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> AstroRequest {
        AstroRequest::new(2451545.0, 37.5665, 126.978)
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
}
