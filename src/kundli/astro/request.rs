use crate::kundli::astro::AstroError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZodiacType {
    Sidereal,
    Tropical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ayanamsha {
    Lahiri,
    Raman,
    Krishnamurti,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HouseSystem {
    Placidus,
    Koch,
    Equal,
    WholeSign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    Mean,
    True,
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct AstroRequest {
    pub jd_ut: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub zodiac: ZodiacType,
    pub ayanamsha: Ayanamsha,
    pub house_system: HouseSystem,
    pub node_type: NodeType,
    pub bodies: Vec<AstroBody>,
}

impl AstroRequest {
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
