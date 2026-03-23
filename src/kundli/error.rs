use std::fmt;

use crate::kundli::astro::AstroError;

#[derive(Debug, Clone, PartialEq)]
pub enum DeriveError {
    MissingMoon,
    InvalidHouseCusps(usize),
    InvalidLongitude(f64),
}

impl fmt::Display for DeriveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingMoon => write!(f, "moon position is required for vimshottari dasha"),
            Self::InvalidHouseCusps(count) => {
                write!(f, "expected 12 house cusps, got {count}")
            }
            Self::InvalidLongitude(longitude) => write!(f, "invalid longitude: {longitude}"),
        }
    }
}

impl std::error::Error for DeriveError {}

#[derive(Debug)]
pub enum KundliError {
    Astro(AstroError),
    Derive(DeriveError),
}

impl fmt::Display for KundliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Astro(err) => err.fmt(f),
            Self::Derive(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for KundliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Astro(err) => Some(err),
            Self::Derive(err) => Some(err),
        }
    }
}

impl From<AstroError> for KundliError {
    fn from(value: AstroError) -> Self {
        Self::Astro(value)
    }
}

impl From<DeriveError> for KundliError {
    fn from(value: DeriveError) -> Self {
        Self::Derive(value)
    }
}
