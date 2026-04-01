use crate::kundli::astro::{AstroResult, HouseSystem, ZodiacType};
use crate::kundli::config::KundliConfig;
use crate::kundli::derive::pipeline::{
    ChartPipeline, D9Rule, IdentityProjection, LagnaReference, VargaTransform,
    WholeSignHouseTransform,
};
use crate::kundli::error::DeriveError;
use crate::kundli::model::{ChartResult, D9Chart};

/// Derives a Navamsa (D9) chart from a raw astronomical result.
pub fn derive_d9_chart(astro: &AstroResult, config: &KundliConfig) -> Result<D9Chart, DeriveError> {
    derive_d9_chart_result(astro, config).map(Into::into)
}

pub(crate) fn derive_d9_chart_result(
    astro: &AstroResult,
    config: &KundliConfig,
) -> Result<ChartResult, DeriveError> {
    if astro.meta.zodiac != ZodiacType::Sidereal {
        return Err(DeriveError::UnsupportedZodiac(astro.meta.zodiac));
    }

    if config.house_system != HouseSystem::WholeSign {
        return Err(DeriveError::UnsupportedHouseSystem(config.house_system));
    }

    ChartPipeline::new(
        IdentityProjection,
        LagnaReference,
        VargaTransform::<D9Rule>::new(),
        WholeSignHouseTransform,
    )
    .execute(astro.clone())
}
