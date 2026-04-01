use std::fmt;

/// Errors returned by the astronomical calculation layer.
#[derive(Debug, Clone, PartialEq)]
pub enum AstroError {
    /// The provided Julian day is not finite.
    InvalidJulianDay(f64),
    /// The provided latitude or longitude is out of range or not finite.
    InvalidCoordinates { latitude: f64, longitude: f64 },
    /// The configured ephemeris path cannot be passed to the backend.
    InvalidEphemerisPath,
    /// The underlying astronomical backend reported a calculation failure.
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
