mod core;
mod house;
mod materialize;
mod projection;
mod reference;
mod sign;

pub(crate) use core::Pipeline;
pub(crate) use house::{CuspBasedHouseTransform, HouseContext, HouseTransformOp, WholeSignHouseTransform};
pub(crate) use materialize::Materialize;
pub(crate) use projection::{IdentityProjection, ProjectedBase, ProjectionOp};
pub(crate) use reference::{LagnaReference, ReferenceContext, ReferenceOp, ResolvedReference};
pub(crate) use sign::{D9Rule, IdentitySignTransform, SignContext, SignPlacement, SignTransformOp, VargaTransform};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kundli::astro::{AstroBody, AstroBodyPosition, AstroMeta, AstroResult, Ayanamsha, ZodiacType};
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
                zodiac: ZodiacType::Sidereal,
                ayanamsha: Ayanamsha::Lahiri,
                ayanamsha_value: Some(24.0),
                sidereal_time: 12.0,
            },
        }
    }

    #[test]
    fn d1_pipeline_materializes_chart_result() {
        let pipeline = Pipeline::new(
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
        let pipeline = Pipeline::new(
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
}
