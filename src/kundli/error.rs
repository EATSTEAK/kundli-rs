use std::fmt;

use crate::kundli::astro::{AstroError, HouseSystem, ZodiacType};

/// Invalid high-level chart selection declared in [`crate::kundli::config::KundliConfig`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChartSelectionError {
    /// At least one chart layer must be requested.
    Empty,
}

impl fmt::Display for ChartSelectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "at least one chart must be requested"),
        }
    }
}

impl std::error::Error for ChartSelectionError {}

/// Errors returned while deriving kundli-specific structures from astronomical
/// output.
#[derive(Debug, Clone, PartialEq)]
pub enum DeriveError {
    /// Vimshottari dasha derivation requires a Moon position.
    MissingMoon,
    /// A cusp-based house derivation expected exactly 12 cusps.
    InvalidHouseCusps(usize),
    /// A longitude could not be interpreted as a finite degree value.
    InvalidLongitude(f64),
    /// A pada value fell outside the supported `1..=4` range.
    InvalidPada(u8),
    /// The derive operation requires sidereal input but received a different zodiac mode.
    UnsupportedZodiac(ZodiacType),
    /// D9 derivation currently supports only the whole-sign house system.
    UnsupportedD9HouseSystem(HouseSystem),
}

impl fmt::Display for DeriveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingMoon => write!(f, "moon position is required for vimshottari dasha"),
            Self::InvalidHouseCusps(count) => {
                write!(f, "expected 12 house cusps, got {count}")
            }
            Self::InvalidLongitude(longitude) => {
                write!(
                    f,
                    "invalid longitude: {longitude}; expected a finite degree value"
                )
            }
            Self::InvalidPada(value) => {
                write!(f, "invalid pada value: {value}; expected a value in 1..=4")
            }
            Self::UnsupportedZodiac(zodiac) => {
                write!(
                    f,
                    "unsupported zodiac for derive operation: {zodiac:?}; expected sidereal data"
                )
            }
            Self::UnsupportedD9HouseSystem(house_system) => {
                write!(
                    f,
                    "unsupported D9 house system: {house_system:?}; expected WholeSign"
                )
            }
        }
    }
}

impl std::error::Error for DeriveError {}

/// Duplicated settings that must match between the request and config.
///
/// Consumers can pattern match this enum to identify which public input field
/// needs to be aligned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputConfigMismatchField {
    /// `request.zodiac` and `config.zodiac` disagreed.
    Zodiac,
    /// `request.ayanamsha` and `config.ayanamsha` disagreed.
    Ayanamsha,
    /// `request.house_system` and `config.house_system` disagreed.
    HouseSystem,
    /// `request.node_type` and `config.node_type` disagreed.
    NodeType,
}

impl InputConfigMismatchField {
    const fn field_name(self) -> &'static str {
        match self {
            Self::Zodiac => "zodiac",
            Self::Ayanamsha => "ayanamsha",
            Self::HouseSystem => "house_system",
            Self::NodeType => "node_type",
        }
    }
}

impl fmt::Display for InputConfigMismatchField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let field_name = self.field_name();
        write!(f, "request.{field_name} must match config.{field_name}")
    }
}

/// Top-level error returned by high-level kundli calculation entrypoints.
#[derive(Debug, Clone, PartialEq)]
pub enum KundliError {
    /// The astronomical layer failed or rejected the request.
    Astro(AstroError),
    /// A kundli-specific derive step failed.
    Derive(DeriveError),
    /// The requested chart selection is invalid.
    ChartSelection(ChartSelectionError),
    /// Settings duplicated between request and config did not match.
    InputConfigMismatch(InputConfigMismatchField),
}

impl fmt::Display for KundliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Astro(err) => err.fmt(f),
            Self::Derive(err) => err.fmt(f),
            Self::ChartSelection(err) => write!(f, "invalid chart selection: {err}"),
            Self::InputConfigMismatch(field) => write!(f, "input/config mismatch: {field}"),
        }
    }
}

impl std::error::Error for KundliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Astro(err) => Some(err),
            Self::Derive(err) => Some(err),
            Self::ChartSelection(err) => Some(err),
            Self::InputConfigMismatch(_) => None,
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

impl From<ChartSelectionError> for KundliError {
    fn from(value: ChartSelectionError) -> Self {
        Self::ChartSelection(value)
    }
}
