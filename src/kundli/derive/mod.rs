//! Lower-level helpers for deriving kundli-specific structures from
//! [`AstroResult`](crate::AstroResult).
//!
//! Most consumers should use [`crate::calculate_kundli`] rather than these
//! functions directly. These modules are intended for advanced use cases where
//! you already have validated astronomical output and want to derive only a
//! specific chart or dasha layer without going through the full multi-chart API.

pub mod d1;
pub mod d9;
pub mod dasha;
pub(crate) mod house;
pub(crate) mod nakshatra;
pub(crate) mod pipeline;
pub(crate) mod reference_points;
pub(crate) mod sign;
