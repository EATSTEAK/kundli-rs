//! High-level kundli calculation entrypoints.
//!
//! Most consumers should start with [`calculate_kundli`]. Use
//! [`calculate_kundli_with_engine`] when you want to provide a custom
//! [`AstroEngine`] implementation.

use crate::kundli::astro::{
    AstroEngine, AstroRequest, AstroResult, SwissEphAstroEngine, SwissEphConfig,
};
use crate::kundli::config::KundliConfig;
use crate::kundli::derive::d1::derive_d1_chart_result;
use crate::kundli::derive::d9::derive_d9_chart_result;
use crate::kundli::derive::dasha::derive_vimshottari_dasha;
use crate::kundli::error::{InputConfigMismatchField, KundliError};
use crate::kundli::model::{CalculationMeta, D1Chart, D9Chart, KundliResult};

/// Calculates a complete kundli using the default Swiss Ephemeris-backed
/// engine.
///
/// This is the most convenient entrypoint for consumers who do not need to
/// customize the astronomical backend. It validates the request, checks that
/// request-level settings match the provided [`KundliConfig`], runs the default
/// engine, and assembles the final [`KundliResult`].
///
/// Optional output sections are controlled by [`KundliConfig::include_d9`] and
/// [`KundliConfig::include_dasha`].
pub fn calculate_kundli(
    request: AstroRequest,
    config: KundliConfig,
) -> Result<KundliResult, KundliError> {
    let engine = SwissEphAstroEngine::new(SwissEphConfig::new());
    calculate_kundli_with_engine(&engine, &request, &config)
}

/// Calculates a complete kundli with an injected astronomical engine.
///
/// This advanced entrypoint is useful when you want to:
///
/// - reuse a custom [`AstroEngine`],
/// - test the derive pipeline with stubbed astro data, or
/// - source astronomical positions from a backend other than the default Swiss
///   Ephemeris implementation.
///
/// The function performs three steps:
///
/// 1. validates the [`AstroRequest`],
/// 2. verifies that request-level settings match the supplied [`KundliConfig`],
/// 3. derives the requested kundli layers from the returned [`AstroResult`].
///
/// Returns [`KundliError::InputConfigMismatch`] when duplicated settings on the
/// request and config disagree.
pub fn calculate_kundli_with_engine<E: AstroEngine>(
    engine: &E,
    request: &AstroRequest,
    config: &KundliConfig,
) -> Result<KundliResult, KundliError> {
    request.validate()?;
    validate_request_matches_config(request, config)?;

    let astro = engine.calculate(request)?;
    let d1: D1Chart = derive_d1_chart_result(&astro, config)?.into();
    let d9: Option<D9Chart> = config
        .include_d9
        .then(|| derive_d9_chart_result(&astro, config).map(Into::into))
        .transpose()?;
    let dasha = config
        .include_dasha
        .then(|| derive_vimshottari_dasha(&astro))
        .transpose()?;
    let lagna = d1.lagna.clone();
    let planets = d1.planets.clone();
    let houses = d1.houses.clone();

    Ok(KundliResult {
        meta: build_calculation_meta(&astro, config),
        lagna,
        planets,
        houses,
        d1,
        d9,
        dasha,
        warnings: vec![],
    })
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
        AstroBody, AstroBodyPosition, AstroError, AstroMeta, Ayanamsha, HouseSystem, NodeType,
        ZodiacType,
    };
    use crate::kundli::model::{Nakshatra, Sign};

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
        KundliConfig::from_request(request)
            .with_include_d9(true)
            .with_include_dasha(true)
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
        assert_eq!(result.lagna, result.d1.lagna);
        assert_eq!(result.planets, result.d1.planets);
        assert_eq!(result.houses, result.d1.houses);
        assert_eq!(result.d1.lagna.sign, Sign::Taurus);
        assert_eq!(result.d1.planets[1].nakshatra.nakshatra, Nakshatra::Ashwini);
        assert!(result.d9.is_some());
        assert!(result.dasha.is_some());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn calculate_with_engine_omits_optional_results_when_disabled() {
        let request = sample_request();
        let config = KundliConfig::from_request(&request);
        let engine = StubEngine {
            result: Ok(sample_astro()),
        };

        let result = calculate_kundli_with_engine(&engine, &request, &config).unwrap();

        assert!(result.d9.is_none());
        assert!(result.dasha.is_none());
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

        let error = calculate_kundli_with_engine(&engine, &request, &sample_config(&request)).unwrap_err();

        assert!(matches!(
            error,
            KundliError::Astro(AstroError::InvalidCoordinates { .. })
        ));
    }
}
