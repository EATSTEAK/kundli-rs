//! High-level public API for kundli calculation.
//!
//! Most consumers should start with
//! [`calculate::calculate_kundli`].
//! The submodules are exposed for users who want to work with lower-level
//! request, engine, derive, error, or result types directly.

pub mod astro;
pub mod calculate;
pub mod config;
pub mod derive;
pub mod error;
pub mod model;
