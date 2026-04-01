use crate::kundli::astro::{AstroResult, HouseSystem};
use crate::kundli::config::KundliConfig;
use crate::kundli::derive::pipeline::{
    ChartPipeline, CuspBasedHouseTransform, IdentityProjection, IdentitySignTransform,
    LagnaReference, WholeSignHouseTransform,
};
use crate::kundli::error::DeriveError;
use crate::kundli::model::{ChartResult, D1Chart};

/// Derives a complete D1 chart from a raw astronomical result.
///
/// This is a lower-level helper than [`crate::calculate_kundli`]. Prefer the
/// high-level API unless you already have an [`AstroResult`] and only need the
/// D1 layer.
pub fn derive_d1_chart(astro: &AstroResult, config: &KundliConfig) -> Result<D1Chart, DeriveError> {
    derive_d1_chart_result(astro, config).map(Into::into)
}

pub(crate) fn derive_d1_chart_result(
    astro: &AstroResult,
    config: &KundliConfig,
) -> Result<ChartResult, DeriveError> {
    let house_transform = match config.house_system {
        HouseSystem::WholeSign => {
            return ChartPipeline::new(
                IdentityProjection,
                LagnaReference,
                IdentitySignTransform,
                WholeSignHouseTransform,
            )
            .execute(astro.clone());
        }
        house_system => CuspBasedHouseTransform { house_system },
    };

    ChartPipeline::new(
        IdentityProjection,
        LagnaReference,
        IdentitySignTransform,
        house_transform,
    )
    .execute(astro.clone())
}
