//! Public modules for kundli calculation.
//!
//! Most consumers should prefer the crate-root exports such as
//! [`crate::calculate_kundli`], [`crate::AstroRequest`], and
//! [`crate::KundliConfig`].
//! These modules remain available for lower-level access, but internal pipeline
//! details are intentionally kept crate-private.

pub mod astro;
pub mod calculate;
pub mod config;
pub mod derive;
pub mod error;
pub mod model;
