//! High-level kundli calculation entrypoints.
//!
//! Most consumers should start with [`calculate_kundli`]. Use
//! [`calculate_kundli_with_engine`] when you want to provide a custom
//! [`AstroEngine`] implementation.

use crate::kundli::astro::{
    AstroEngine, AstroRequest, AstroResult, HouseSystem, SwissEphAstroEngine, SwissEphConfig,
    ZodiacType,
};
use crate::kundli::config::{ChartKind, ChartSpec, HouseMode, KundliConfig};
use crate::kundli::derive::dasha::derive_vimshottari_dasha;
use crate::kundli::derive::pipeline::{
    ChartPipeline, CuspBasedHouseTransform, DivisionalSignTransform, IdentityProjection,
    IdentitySignTransform, ReferenceTransform, WholeSignHouseTransform,
};
use crate::kundli::error::{DeriveError, InputConfigMismatchField, KundliError};
use std::collections::BTreeMap;

use crate::kundli::model::{CalculationMeta, ChartLayer, ChartResult, KundliResult};

pub fn calculate_kundli(
    request: AstroRequest,
    config: KundliConfig,
) -> Result<KundliResult, KundliError> {
    let engine = SwissEphAstroEngine::new(SwissEphConfig::new());
    calculate_kundli_with_engine(&engine, &request, &config)
}

pub fn calculate_kundli_with_engine<E: AstroEngine>(
    engine: &E,
    request: &AstroRequest,
    config: &KundliConfig,
) -> Result<KundliResult, KundliError> {
    request.validate()?;
    let mut config = config.clone();
    config.validate()?;
    validate_request_matches_config(request, &config)?;

    let astro = engine.calculate(request)?;
    let mut charts = BTreeMap::new();

    for chart in &config.charts {
        let layer = derive_chart_layer(&astro, &config, *chart)?;
        charts.insert(*chart, layer);
    }

    Ok(KundliResult {
        meta: build_calculation_meta(&astro, &config),
        charts,
        warnings: vec![],
    })
}

fn derive_chart_layer(
    astro: &AstroResult,
    config: &KundliConfig,
    chart: ChartSpec,
) -> Result<ChartLayer, KundliError> {
    match chart.kind {
        ChartKind::VimshottariDasha => Ok(ChartLayer::VimshottariDasha(derive_vimshottari_dasha(
            astro,
        )?)),
        _ => Ok(ChartLayer::Chart(derive_chart_result(
            astro, config, chart,
        )?)),
    }
}

fn derive_chart_result(
    astro: &AstroResult,
    config: &KundliConfig,
    chart: ChartSpec,
) -> Result<ChartResult, DeriveError> {
    // Bhava/Chalit currently share the same pipeline assembly as Rasi; the
    // distinct chart kinds remain semantic placeholders until they gain their
    // own result-shaping behavior.
    if matches!(
        chart.kind,
        ChartKind::Varga { .. } | ChartKind::DivisionalBhava { .. }
    ) && astro.meta.zodiac != ZodiacType::Sidereal
    {
        return Err(DeriveError::UnsupportedZodiac(astro.meta.zodiac));
    }

    let reference = ReferenceTransform::new(chart.reference);

    match chart.kind {
        ChartKind::Rasi | ChartKind::Bhava | ChartKind::Chalit => {
            match resolve_house_mode(chart, config)? {
                ResolvedHouseMode::WholeSign => ChartPipeline::new(
                    IdentityProjection,
                    reference,
                    IdentitySignTransform,
                    WholeSignHouseTransform,
                )
                .execute(astro.clone()),
                ResolvedHouseMode::CuspBased(house_system) => ChartPipeline::new(
                    IdentityProjection,
                    reference,
                    IdentitySignTransform,
                    CuspBasedHouseTransform { house_system },
                )
                .execute(astro.clone()),
                ResolvedHouseMode::None => unreachable!("non-dasha charts must expose houses"),
            }
        }
        ChartKind::Varga { division } => match resolve_house_mode(chart, config)? {
            ResolvedHouseMode::WholeSign => ChartPipeline::new(
                IdentityProjection,
                reference,
                DivisionalSignTransform::new(division)?,
                WholeSignHouseTransform,
            )
            .execute(astro.clone()),
            ResolvedHouseMode::CuspBased(house_system) => ChartPipeline::new(
                IdentityProjection,
                reference,
                DivisionalSignTransform::new(division)?,
                CuspBasedHouseTransform { house_system },
            )
            .execute(astro.clone()),
            ResolvedHouseMode::None => unreachable!("varga charts must expose houses"),
        },
        ChartKind::DivisionalBhava { division } => match resolve_house_mode(chart, config)? {
            ResolvedHouseMode::WholeSign => ChartPipeline::new(
                IdentityProjection,
                reference,
                DivisionalSignTransform::new(division)?,
                WholeSignHouseTransform,
            )
            .execute(astro.clone()),
            ResolvedHouseMode::CuspBased(house_system) => ChartPipeline::new(
                IdentityProjection,
                reference,
                DivisionalSignTransform::new(division)?,
                CuspBasedHouseTransform { house_system },
            )
            .execute(astro.clone()),
            ResolvedHouseMode::None => unreachable!("divisional bhava charts must expose houses"),
        },
        ChartKind::VimshottariDasha => unreachable!("handled separately"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResolvedHouseMode {
    WholeSign,
    CuspBased(HouseSystem),
    None,
}

fn resolve_house_mode(
    chart: ChartSpec,
    config: &KundliConfig,
) -> Result<ResolvedHouseMode, DeriveError> {
    let configured = match config.house_system {
        HouseSystem::WholeSign => ResolvedHouseMode::WholeSign,
        other => ResolvedHouseMode::CuspBased(other),
    };

    let resolved = match chart.house_mode {
        HouseMode::Configured => configured,
        HouseMode::WholeSign => ResolvedHouseMode::WholeSign,
        HouseMode::CuspBased(system) => ResolvedHouseMode::CuspBased(system),
        HouseMode::None => ResolvedHouseMode::None,
    };

    debug_assert!(
        !matches!(
            chart.kind,
            ChartKind::Bhava | ChartKind::Chalit | ChartKind::DivisionalBhava { .. }
        ) || matches!(resolved, ResolvedHouseMode::CuspBased(_)),
        "bhava-style charts must be validated as cusp-based before derivation"
    );

    Ok(resolved)
}

fn validate_request_matches_config(
    request: &AstroRequest,
    config: &KundliConfig,
) -> Result<(), KundliError> {
    if request.zodiac != config.zodiac {
        return Err(KundliError::InputConfigMismatch(
            InputConfigMismatchField::Zodiac,
        ));
    }

    if request.ayanamsha != config.ayanamsha {
        return Err(KundliError::InputConfigMismatch(
            InputConfigMismatchField::Ayanamsha,
        ));
    }

    if request.house_system != config.house_system {
        return Err(KundliError::InputConfigMismatch(
            InputConfigMismatchField::HouseSystem,
        ));
    }

    if request.node_type != config.node_type {
        return Err(KundliError::InputConfigMismatch(
            InputConfigMismatchField::NodeType,
        ));
    }

    Ok(())
}

fn build_calculation_meta(astro: &AstroResult, config: &KundliConfig) -> CalculationMeta {
    CalculationMeta {
        jd_ut: astro.meta.jd_ut,
        zodiac: astro.meta.zodiac,
        ayanamsha: astro.meta.ayanamsha,
        ayanamsha_value: astro.meta.ayanamsha_value,
        house_system: config.house_system,
        node_type: config.node_type,
        sidereal_time: astro.meta.sidereal_time,
        body_count: astro.bodies.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kundli::astro::{
        AstroBody, AstroBodyPosition, AstroError, AstroMeta, Ayanamsha, NodeType,
    };
    use crate::kundli::model::Nakshatra;

    #[derive(Debug, Clone)]
    struct StubEngine {
        result: Result<AstroResult, AstroError>,
    }

    impl AstroEngine for StubEngine {
        fn calculate(&self, _request: &AstroRequest) -> Result<AstroResult, AstroError> {
            self.result.clone()
        }
    }

    fn sample_request() -> AstroRequest {
        AstroRequest::new(2451545.0, 37.5665, 126.9780)
    }

    fn sample_config(request: &AstroRequest) -> KundliConfig {
        KundliConfig::from_request(request).with_charts(&[
            ChartSpec::d1(),
            ChartSpec::d9(),
            ChartSpec::vimshottari_dasha(),
        ])
    }

    fn sample_astro() -> AstroResult {
        let bodies = std::array::from_fn(|index| {
            let body = AstroBody::ALL[index];
            match body {
                AstroBody::Sun => AstroBodyPosition {
                    body,
                    longitude: 15.0,
                    latitude: 0.0,
                    distance: 1.0,
                    speed_longitude: 1.0,
                },
                AstroBody::Moon => AstroBodyPosition {
                    body,
                    longitude: 5.0,
                    latitude: 0.0,
                    distance: 1.0,
                    speed_longitude: 13.0,
                },
                AstroBody::Saturn => AstroBodyPosition {
                    body,
                    longitude: 32.0,
                    latitude: 0.0,
                    distance: 1.0,
                    speed_longitude: -0.1,
                },
                _ => AstroBodyPosition {
                    body,
                    longitude: 180.0 + index as f64,
                    latitude: 0.0,
                    distance: 1.0,
                    speed_longitude: 0.1,
                },
            }
        });

        AstroResult {
            bodies,
            ascendant_longitude: 45.0,
            mc_longitude: 135.0,
            house_cusps: [0.0; 12],
            meta: AstroMeta {
                jd_ut: 2451545.0,
                zodiac: ZodiacType::Sidereal,
                ayanamsha: Ayanamsha::Lahiri,
                ayanamsha_value: Some(24.0),
                sidereal_time: 12.0,
            },
        }
    }

    #[test]
    fn calculate_with_engine_assembles_full_kundli_result() {
        let request = sample_request();
        let config = sample_config(&request);
        let engine = StubEngine {
            result: Ok(sample_astro()),
        };

        let result = calculate_kundli_with_engine(&engine, &request, &config).unwrap();

        assert_eq!(result.meta.jd_ut, 2451545.0);
        assert_eq!(result.meta.zodiac, ZodiacType::Sidereal);
        assert_eq!(result.meta.ayanamsha, Ayanamsha::Lahiri);
        assert_eq!(result.meta.house_system, HouseSystem::WholeSign);
        assert_eq!(result.meta.node_type, NodeType::True);
        assert_eq!(result.meta.body_count, AstroBody::ALL.len());
        let d1 = result
            .chart(ChartSpec::d1())
            .and_then(ChartLayer::as_chart)
            .unwrap();
        assert_eq!(d1.lagna.sign, crate::kundli::model::Sign::Taurus);
        assert_eq!(d1.planets[1].nakshatra.nakshatra, Nakshatra::Ashwini);
        assert!(result.chart(ChartSpec::d9()).is_some());
        assert!(result.chart(ChartSpec::vimshottari_dasha()).is_some());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn calculate_with_engine_rejects_empty_chart_requests() {
        let request = sample_request();
        let config = KundliConfig::from_request(&request);
        let engine = StubEngine {
            result: Ok(sample_astro()),
        };

        let error = calculate_kundli_with_engine(&engine, &request, &config).unwrap_err();

        assert_eq!(
            error,
            KundliError::ChartSelection(crate::kundli::error::ChartSelectionError::Empty)
        );
    }

    #[test]
    fn calculate_with_engine_deduplicates_duplicate_chart_requests_via_config_validation() {
        let request = sample_request();
        let config = KundliConfig::from_request(&request).with_charts(&[
            ChartSpec::d1(),
            ChartSpec::d1(),
            ChartSpec::vimshottari_dasha(),
        ]);
        let engine = StubEngine {
            result: Ok(sample_astro()),
        };

        let result = calculate_kundli_with_engine(&engine, &request, &config).unwrap();

        assert_eq!(result.charts.len(), 2);
        assert!(result.chart(ChartSpec::d1()).is_some());
        assert!(result.chart(ChartSpec::vimshottari_dasha()).is_some());
    }

    #[test]
    fn calculate_with_engine_rejects_request_config_mismatch() {
        let request = sample_request();
        let config = sample_config(&request).with_house_system(HouseSystem::Equal);
        let engine = StubEngine {
            result: Ok(sample_astro()),
        };

        let error = calculate_kundli_with_engine(&engine, &request, &config).unwrap_err();

        assert_eq!(
            error,
            KundliError::InputConfigMismatch(InputConfigMismatchField::HouseSystem)
        );
    }

    #[test]
    fn calculate_with_engine_propagates_request_validation_error() {
        let mut request = sample_request();
        request.latitude = 120.0;
        let engine = StubEngine {
            result: Ok(sample_astro()),
        };

        let error =
            calculate_kundli_with_engine(&engine, &request, &sample_config(&request)).unwrap_err();

        assert!(matches!(
            error,
            KundliError::Astro(AstroError::InvalidCoordinates { .. })
        ));
    }

    #[test]
    fn calculate_with_engine_rejects_bhava_selection_before_derivation() {
        let request = sample_request();
        let config = KundliConfig::from_request(&request).with_charts(&[ChartSpec::bhava()]);
        let engine = StubEngine {
            result: Ok(sample_astro()),
        };

        let error = calculate_kundli_with_engine(&engine, &request, &config).unwrap_err();

        assert_eq!(
            error,
            KundliError::ChartSelection(
                crate::kundli::error::ChartSelectionError::CuspBasedHouseModeRequired(
                    ChartKind::Bhava,
                ),
            )
        );
    }

    #[test]
    fn resolve_house_mode_keeps_bhava_style_specs_cusp_based_after_validation() {
        let config = KundliConfig::default().with_house_system(HouseSystem::WholeSign);

        assert_eq!(
            resolve_house_mode(
                ChartSpec::bhava().with_house_mode(HouseMode::CuspBased(HouseSystem::Placidus)),
                &config,
            )
            .unwrap(),
            ResolvedHouseMode::CuspBased(HouseSystem::Placidus)
        );
    }
}
