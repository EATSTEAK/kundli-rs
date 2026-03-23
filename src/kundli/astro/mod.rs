//! Raw astronomical request, engine, error, and result types.
//!
//! This module contains the lower-level building blocks used by the higher-level
//! kundli calculation API. Most consumers only need
//! [`crate::calculate_kundli`] together with [`AstroRequest`] and
//! [`crate::kundli::config::KundliConfig`], but advanced users can work with
//! [`AstroEngine`] and [`AstroResult`] directly.

mod engine;
mod ephemeris;
mod error;
mod request;
mod result;

pub use engine::{AstroEngine, SwissEphAstroEngine, SwissEphConfig};
pub use error::AstroError;
pub use request::{AstroBody, AstroRequest, Ayanamsha, HouseSystem, NodeType, ZodiacType};
pub use result::{AstroBodyPosition, AstroMeta, AstroResult};
