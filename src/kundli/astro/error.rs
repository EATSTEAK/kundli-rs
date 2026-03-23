use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum AstroError {
    InvalidJulianDay(f64),
    InvalidCoordinates { latitude: f64, longitude: f64 },
    EmptyBodies,
    InvalidEphemerisPath,
    CalculationFailed(String),
}

impl fmt::Display for AstroError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJulianDay(jd_ut) => write!(f, "invalid Julian day: {jd_ut}"),
            Self::InvalidCoordinates {
                latitude,
                longitude,
            } => {
                write!(
                    f,
                    "invalid coordinates: latitude={latitude}, longitude={longitude}"
                )
            }
            Self::EmptyBodies => write!(f, "at least one astro body is required"),
            Self::InvalidEphemerisPath => {
                write!(f, "ephemeris path must not contain interior NUL bytes")
            }
            Self::CalculationFailed(message) => {
                write!(f, "astronomical calculation failed: {message}")
            }
        }
    }
}

impl std::error::Error for AstroError {}

impl From<swiss_eph::safe::SwissEphError> for AstroError {
    fn from(value: swiss_eph::safe::SwissEphError) -> Self {
        Self::CalculationFailed(value.to_string())
    }
}
