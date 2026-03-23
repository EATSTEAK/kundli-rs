//! `kundli-rs` computes a kundli from astronomical inputs.
//!
//! The most ergonomic entrypoint is [`calculate_kundli`], which uses the default
//! Swiss Ephemeris-backed engine. Use [`calculate_kundli_with_engine`] when you
//! want to inject a custom [`AstroEngine`](crate::kundli::astro::AstroEngine)
//! implementation for testing or alternative data sources.
//!
//! # Quick start
//!
//! ```no_run
//! use kundli_rs::calculate_kundli;
//! use kundli_rs::kundli::astro::{
//!     AstroBody, AstroRequest, Ayanamsha, HouseSystem, NodeType, ZodiacType,
//! };
//! use kundli_rs::kundli::config::KundliConfig;
//!
//! let request = AstroRequest {
//!     jd_ut: 2451545.0,
//!     latitude: 37.5665,
//!     longitude: 126.9780,
//!     zodiac: ZodiacType::Sidereal,
//!     ayanamsha: Ayanamsha::Lahiri,
//!     house_system: HouseSystem::WholeSign,
//!     node_type: NodeType::True,
//!     bodies: vec![AstroBody::Sun, AstroBody::Moon, AstroBody::Saturn],
//! };
//!
//! let config = KundliConfig {
//!     include_d9: true,
//!     include_dasha: true,
//!     ..KundliConfig::default()
//! };
//!
//! let result = calculate_kundli(request, config)?;
//!
//! println!("Lagna sign: {:?}", result.lagna.sign);
//! # Ok::<(), kundli_rs::kundli::error::KundliError>(())
//! ```
//!
//! # API overview
//!
//! A typical workflow is:
//!
//! 1. Build an [`AstroRequest`](crate::kundli::astro::AstroRequest).
//! 2. Choose a [`KundliConfig`](crate::kundli::config::KundliConfig).
//! 3. Call [`calculate_kundli`] or [`calculate_kundli_with_engine`].
//! 4. Inspect the returned [`KundliResult`](crate::kundli::model::KundliResult).
//!
//! [`AstroRequest`](crate::kundli::astro::AstroRequest) and
//! [`KundliConfig`](crate::kundli::config::KundliConfig) both carry zodiac,
//! ayanamsha, house-system, and node-type settings. These values must match or
//! calculation will fail with
//! [`KundliError::InputConfigMismatch`](crate::kundli::error::KundliError::InputConfigMismatch).

pub mod kundli;

pub use kundli::calculate::{calculate_kundli, calculate_kundli_with_engine};
