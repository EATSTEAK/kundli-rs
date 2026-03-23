use crate::kundli::astro::{
    AstroEngine, AstroRequest, AstroResult, SwissEphAstroEngine, SwissEphConfig,
};
use crate::kundli::config::KundliConfig;
use crate::kundli::derive::d1::derive_d1_chart;
use crate::kundli::derive::d9::derive_d9_chart;
use crate::kundli::derive::dasha::derive_vimshottari_dasha;
use crate::kundli::error::KundliError;
use crate::kundli::model::{CalculationMeta, KundliResult};

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
    validate_request_matches_config(request, config)?;

    let astro = engine.calculate(request)?;
    let d1 = derive_d1_chart(&astro, config)?;
    let d9 = config
        .include_d9
        .then(|| derive_d9_chart(&astro, config))
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
            "request.zodiac must match config.zodiac",
        ));
    }

    if request.ayanamsha != config.ayanamsha {
        return Err(KundliError::InputConfigMismatch(
            "request.ayanamsha must match config.ayanamsha",
        ));
    }

    if request.house_system != config.house_system {
        return Err(KundliError::InputConfigMismatch(
            "request.house_system must match config.house_system",
        ));
    }

    if request.node_type != config.node_type {
        return Err(KundliError::InputConfigMismatch(
            "request.node_type must match config.node_type",
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
        AstroRequest {
            jd_ut: 2451545.0,
            latitude: 37.5665,
            longitude: 126.9780,
            zodiac: ZodiacType::Sidereal,
            ayanamsha: Ayanamsha::Lahiri,
            house_system: HouseSystem::WholeSign,
            node_type: NodeType::True,
            bodies: vec![AstroBody::Sun, AstroBody::Moon, AstroBody::Saturn],
        }
    }

    fn sample_config() -> KundliConfig {
        KundliConfig {
            include_d9: true,
            include_dasha: true,
            ..KundliConfig::default()
        }
    }

    fn sample_astro() -> AstroResult {
        AstroResult {
            bodies: vec![
                AstroBodyPosition {
                    body: AstroBody::Sun,
                    longitude: 15.0,
                    latitude: 0.0,
                    distance: 1.0,
                    speed_longitude: 1.0,
                },
                AstroBodyPosition {
                    body: AstroBody::Moon,
                    longitude: 5.0,
                    latitude: 0.0,
                    distance: 1.0,
                    speed_longitude: 13.0,
                },
                AstroBodyPosition {
                    body: AstroBody::Saturn,
                    longitude: 32.0,
                    latitude: 0.0,
                    distance: 1.0,
                    speed_longitude: -0.1,
                },
            ],
            ascendant_longitude: 45.0,
            mc_longitude: 135.0,
            house_cusps: vec![],
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
        let config = sample_config();
        let engine = StubEngine {
            result: Ok(sample_astro()),
        };

        let result = calculate_kundli_with_engine(&engine, &request, &config).unwrap();

        assert_eq!(result.meta.jd_ut, 2451545.0);
        assert_eq!(result.meta.zodiac, ZodiacType::Sidereal);
        assert_eq!(result.meta.ayanamsha, Ayanamsha::Lahiri);
        assert_eq!(result.meta.house_system, HouseSystem::WholeSign);
        assert_eq!(result.meta.node_type, NodeType::True);
        assert_eq!(result.meta.body_count, 3);
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
        let config = KundliConfig::default();
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
        let config = KundliConfig {
            house_system: HouseSystem::Equal,
            ..sample_config()
        };
        let engine = StubEngine {
            result: Ok(sample_astro()),
        };

        let error = calculate_kundli_with_engine(&engine, &request, &config).unwrap_err();

        assert_eq!(
            error,
            KundliError::InputConfigMismatch("request.house_system must match config.house_system",)
        );
    }

    #[test]
    fn calculate_with_engine_propagates_request_validation_error() {
        let mut request = sample_request();
        request.latitude = 120.0;
        let engine = StubEngine {
            result: Ok(sample_astro()),
        };

        let error = calculate_kundli_with_engine(&engine, &request, &sample_config()).unwrap_err();

        assert!(matches!(
            error,
            KundliError::Astro(AstroError::InvalidCoordinates { .. })
        ));
    }
}
