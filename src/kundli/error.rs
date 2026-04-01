use std::fmt;

use crate::kundli::astro::{AstroError, HouseSystem, ZodiacType};
use crate::kundli::config::ChartKind;
use crate::kundli::model::DashaLord;

/// Invalid high-level chart selection declared in [`crate::kundli::config::KundliConfig`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChartSelectionError {
    /// At least one chart layer must be requested.
    Empty,
    /// A divisional chart requested division 0.
    InvalidDivision(u8),
    /// The selected chart kind does not support the given house mode.
    UnexpectedHouseMode(ChartKind),
    /// The selected chart kind requires a cusp-based house mode.
    CuspBasedHouseModeRequired(ChartKind),
}

impl fmt::Display for ChartSelectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "at least one chart must be requested"),
            Self::InvalidDivision(division) => {
                write!(
                    f,
                    "invalid divisional chart: D{division}; expected division >= 1"
                )
            }
            Self::UnexpectedHouseMode(kind) => {
                write!(
                    f,
                    "chart kind {kind:?} does not support the selected house mode"
                )
            }
            Self::CuspBasedHouseModeRequired(kind) => {
                write!(f, "chart kind {kind:?} requires a cusp-based house mode")
            }
        }
    }
}

impl std::error::Error for ChartSelectionError {}

#[derive(Debug, Clone, PartialEq)]
pub enum DeriveError {
    MissingMoon,
    MissingPlacementBody,
    InvalidHouseCusps(usize),
    InvalidHouseNumber(u8),
    InvalidLongitude(f64),
    InvalidPada(u8),
    InvalidDashaSequenceLord(DashaLord),
    InvalidDivision(u8),
    UnsupportedZodiac(ZodiacType),
    UnsupportedHouseSystem(HouseSystem),
    SpecialPointCalculationFailed(&'static str),
}

impl fmt::Display for DeriveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingMoon => write!(f, "moon position is required for vimshottari dasha"),
            Self::MissingPlacementBody => {
                write!(f, "pipeline placement is missing its body identifier")
            }
            Self::InvalidHouseCusps(count) => {
                write!(f, "expected 12 house cusps, got {count}")
            }
            Self::InvalidHouseNumber(value) => {
                write!(
                    f,
                    "invalid house number: {value}; expected a value in 1..=12"
                )
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
            Self::InvalidDashaSequenceLord(lord) => {
                write!(f, "invalid Vimshottari sequence lord: {lord:?}")
            }
            Self::InvalidDivision(division) => {
                write!(
                    f,
                    "invalid divisional chart: D{division}; expected division >= 1"
                )
            }
            Self::UnsupportedZodiac(zodiac) => {
                write!(
                    f,
                    "unsupported zodiac for derive operation: {zodiac:?}; expected sidereal data"
                )
            }
            Self::UnsupportedHouseSystem(house_system) => {
                write!(
                    f,
                    "unsupported house system for derive operation: {house_system:?}"
                )
            }
            Self::SpecialPointCalculationFailed(point) => {
                write!(f, "failed to calculate special reference point: {point}")
            }
        }
    }
}

impl std::error::Error for DeriveError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputConfigMismatchField {
    Zodiac,
    Ayanamsha,
    HouseSystem,
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

#[derive(Debug, Clone, PartialEq)]
pub enum KundliError {
    Astro(AstroError),
    Derive(DeriveError),
    ChartSelection(ChartSelectionError),
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
