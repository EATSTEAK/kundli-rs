//! `kundli-rs` computes a kundli from astronomical inputs.
//!
//! The most ergonomic entrypoints are [`calculate_kundli`] and the crate-root
//! re-exported request/config/result types. Use [`calculate_kundli_with_engine`]
//! when you want to inject a custom [`AstroEngine`] implementation for testing
//! or alternative data sources.
//!
//! # Quick start
//!
//! ```no_run
//! use kundli_rs::{AstroRequest, ChartLayer, ChartSpec, KundliConfig, calculate_kundli};
//!
//! let request = AstroRequest::new(2451545.0, 37.5665, 126.9780);
//!
//! let config = KundliConfig::from_request(&request).with_charts(&[
//!     ChartSpec::d1(),
//!     ChartSpec::d9(),
//!     ChartSpec::vimshottari_dasha(),
//! ]);
//!
//! let result = calculate_kundli(request, config)?;
//! let d1 = result
//!     .chart(ChartSpec::d1())
//!     .and_then(ChartLayer::as_chart)
//!     .unwrap();
//!
//! println!("Lagna sign: {:?}", d1.lagna.sign);
//! # Ok::<(), kundli_rs::KundliError>(())
//! ```
//!
//! # API overview
//!
//! A typical workflow is:
//!
//! 1. Build an [`AstroRequest`].
//! 2. Choose a [`KundliConfig`].
//! 3. Call [`calculate_kundli`] or [`calculate_kundli_with_engine`].
//! 4. Inspect the returned [`KundliResult`].
//!
//! [`AstroRequest`] and [`KundliConfig`] both carry zodiac, ayanamsha,
//! house-system, and node-type settings. Use [`KundliConfig::from_request`]
//! when you want those duplicated settings to match by construction.

pub mod kundli;

pub use kundli::astro::{
    AstroBody, AstroBodyPosition, AstroEngine, AstroError, AstroMeta, AstroRequest, AstroResult,
    Ayanamsha, HouseSystem, NodeType, SwissEphAstroEngine, SwissEphConfig, ZodiacType,
};
pub use kundli::calculate::{calculate_kundli, calculate_kundli_with_engine};
pub use kundli::config::{
    ChartKind, ChartSpec, HouseMode, KundliConfig, ReferenceKey, SpecialReference,
};
pub use kundli::error::{ChartSelectionError, DeriveError, InputConfigMismatchField, KundliError};
pub use kundli::model::{
    CalculationMeta, CalculationWarning, ChartLayer, ChartResult, ChartStyle, D1Chart, D9Chart,
    DashaLord, DashaPeriod, HouseNumber, HouseResult, KundliResult, LagnaResult, Nakshatra,
    NakshatraPlacement, Pada, PlanetPlacement, ReferenceResult, Sign, VimshottariDasha,
};
