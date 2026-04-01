mod core;
mod house;
mod materialize;
mod projection;
mod reference;
mod sign;

pub(crate) use core::ChartPipeline;
pub(crate) use house::{
    CuspBasedHouseTransform, HouseContext, HouseTransformOp, WholeSignHouseTransform,
};
pub(crate) use materialize::Materialize;
pub(crate) use projection::{IdentityProjection, ProjectedBase, ProjectionOp};
pub(crate) use reference::{
    LagnaReference, ReferenceContext, ReferenceOp, ReferenceTransform, ResolvedReference,
};
pub(crate) use sign::{
    D9Rule, DivisionalSignTransform, IdentitySignTransform, SignContext, SignPlacement,
    SignTransformOp, VargaTransform,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kundli::astro::{
        AstroBody, AstroBodyPosition, AstroMeta, AstroResult, Ayanamsha, ZodiacType,
    };
    use crate::kundli::config::ReferenceKey;
    use crate::kundli::derive::pipeline::reference::MoonReference;
    use crate::kundli::model::Sign;

    fn sample_result() -> AstroResult {
        let bodies = std::array::from_fn(|index| {
            let body = AstroBody::ALL[index];
            let longitude = match body {
                AstroBody::Sun => 15.0,
                AstroBody::Moon => 95.0,
                AstroBody::Saturn => 32.0,
                _ => 180.0 + index as f64,
            };

            AstroBodyPosition {
                body,
                longitude,
                latitude: 0.0,
                distance: 1.0,
                speed_longitude: if body == AstroBody::Saturn { -0.1 } else { 0.1 },
            }
        });

        AstroResult {
            bodies,
            ascendant_longitude: 45.0,
            mc_longitude: 135.0,
            house_cusps: [0.0; 12],
            meta: AstroMeta {
                jd_ut: 2451545.0,
                latitude: 37.5665,
                longitude: 126.9780,
                zodiac: ZodiacType::Sidereal,
                ayanamsha: Ayanamsha::Lahiri,
                ayanamsha_value: Some(24.0),
                sidereal_time: 12.0,
            },
        }
    }

    #[test]
    fn d1_pipeline_materializes_chart_result() {
        let pipeline = ChartPipeline::new(
            IdentityProjection,
            LagnaReference,
            IdentitySignTransform,
            WholeSignHouseTransform,
        );

        let chart = pipeline.execute(sample_result()).unwrap();

        assert_eq!(chart.lagna.sign, Sign::Taurus);
        assert_eq!(chart.planets.len(), AstroBody::ALL.len());
        assert_eq!(chart.planets[0].sign, Sign::Aries);
        assert_eq!(chart.planets[1].house.get(), 3);
        assert_eq!(chart.houses.len(), 12);
        assert_eq!(chart.houses[0].sign, Sign::Taurus);
    }

    #[test]
    fn d9_varga_transform_maps_longitudes() {
        let pipeline = ChartPipeline::new(
            IdentityProjection,
            LagnaReference,
            VargaTransform::<D9Rule>::new(),
            WholeSignHouseTransform,
        );

        let chart = pipeline.execute(sample_result()).unwrap();

        assert_eq!(chart.lagna.sign, Sign::Taurus);
        assert!((chart.planets[0].longitude - 135.0).abs() < 1e-10);
        assert_eq!(chart.planets[0].sign, Sign::Leo);
    }

    #[test]
    fn generic_divisional_transform_maps_d10() {
        let pipeline = ChartPipeline::new(
            IdentityProjection,
            LagnaReference,
            DivisionalSignTransform::new(10).unwrap(),
            WholeSignHouseTransform,
        );

        let chart = pipeline.execute(sample_result()).unwrap();

        assert!((chart.planets[0].longitude - 150.0).abs() < 1e-10);
        assert_eq!(chart.planets[0].sign, Sign::Virgo);
    }

    #[test]
    fn moon_reference_reanchors_whole_sign_houses() {
        let pipeline = ChartPipeline::new(
            IdentityProjection,
            MoonReference,
            IdentitySignTransform,
            WholeSignHouseTransform,
        );

        let chart = pipeline.execute(sample_result()).unwrap();

        assert_eq!(chart.planets[1].house.get(), 1);
        assert_eq!(chart.planets[0].house.get(), 10);
        assert_eq!(chart.houses[0].sign, Sign::Cancer);
    }

    #[test]
    fn generic_reference_transform_supports_sun_reference() {
        let pipeline = ChartPipeline::new(
            IdentityProjection,
            ReferenceTransform::new(ReferenceKey::Sun),
            IdentitySignTransform,
            WholeSignHouseTransform,
        );

        let chart = pipeline.execute(sample_result()).unwrap();

        assert_eq!(chart.planets[0].house.get(), 1);
        assert_eq!(chart.planets[1].house.get(), 4);
        assert_eq!(chart.houses[0].sign, Sign::Aries);
    }

    #[test]
    fn moon_reference_renumbers_cusp_based_houses() {
        let pipeline = ChartPipeline::new(
            IdentityProjection,
            MoonReference,
            IdentitySignTransform,
            CuspBasedHouseTransform {
                house_system: crate::kundli::astro::HouseSystem::Placidus,
            },
        );

        let mut result = sample_result();
        result.house_cusps = [
            0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 210.0, 240.0, 270.0, 300.0, 330.0,
        ];

        let chart = pipeline.execute(result).unwrap();

        assert_eq!(chart.planets[1].house.get(), 1);
        assert_eq!(chart.planets[0].house.get(), 10);
        assert_eq!(chart.houses[0].house.get(), 10);
        assert_eq!(chart.houses[3].house.get(), 1);
    }
}
