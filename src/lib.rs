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
//! use kundli_rs::kundli::astro::AstroRequest;
//! use kundli_rs::kundli::config::{KnownChart, KundliConfig};
//!
//! let request = AstroRequest::new(2451545.0, 37.5665, 126.9780);
//!
//! let config = KundliConfig::from_request(&request).with_charts(&[
//!     KnownChart::D1,
//!     KnownChart::D9,
//!     KnownChart::VimshottariDasha,
//! ]);
//!
//! let result = calculate_kundli(request, config)?;
//! let d1 = result
//!     .chart(KnownChart::D1)
//!     .and_then(kundli_rs::kundli::model::ChartLayer::as_d1)
//!     .unwrap();
//!
//! println!("Lagna sign: {:?}", d1.lagna.sign);
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
//! ayanamsha, house-system, and node-type settings. Use
//! [`KundliConfig::from_request`](crate::kundli::config::KundliConfig::from_request)
//! when you want those duplicated settings to match by construction.

pub mod kundli;

pub use kundli::calculate::{calculate_kundli, calculate_kundli_with_engine};
