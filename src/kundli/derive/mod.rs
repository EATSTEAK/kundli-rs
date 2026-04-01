//! Lower-level helpers for deriving kundli-specific structures from
//! [`AstroResult`](crate::kundli::astro::AstroResult).
//!
//! Most consumers should use [`crate::calculate_kundli`] rather than these
//! functions directly. These modules are useful when you already have validated
//! astronomical output and want to derive only a specific chart or dasha layer.

pub mod d1;
pub mod d9;
pub mod dasha;
pub mod pipeline;
pub(crate) mod house;
pub(crate) mod input;
pub(crate) mod nakshatra;
pub(crate) mod sign;
