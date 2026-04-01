use crate::kundli::derive::sign::sign_from_longitude;
use crate::kundli::error::DeriveError;
use crate::kundli::model::{
    ChartResult, HouseResult, LagnaResult, PlanetPlacement, ReferenceResult,
};

use super::HouseContext;
use crate::kundli::derive::pipeline::reference::ReferencePoint;

pub(crate) trait Materialize {
    fn materialize(self) -> Result<ChartResult, DeriveError>;
}

impl Materialize for HouseContext {
    fn materialize(self) -> Result<ChartResult, DeriveError> {
        Ok(ChartResult {
            style: crate::kundli::model::ChartStyle::Standard,
            reference: Some(match self.reference.point {
                ReferencePoint::Lagna => ReferenceResult::Lagna {
                    longitude: self.reference.longitude,
                },
                ReferencePoint::Planet(body) => ReferenceResult::Planet {
                    body,
                    longitude: self.reference.longitude,
                },
                ReferencePoint::Special(kind) => ReferenceResult::Special {
                    kind,
                    longitude: self.reference.longitude,
                },
            }),
            lagna: LagnaResult {
                sign: self.ascendant.sign,
                degrees_in_sign: self.ascendant.degrees_in_sign,
                longitude: self.ascendant.longitude,
            },
            planets: self
                .bodies
                .into_iter()
                .map(|body| {
                    Ok(PlanetPlacement {
                        body: body
                            .placement
                            .body
                            .ok_or(DeriveError::MissingPlacementBody)?,
                        longitude: body.placement.longitude,
                        sign: body.placement.sign,
                        degrees_in_sign: body.placement.degrees_in_sign,
                        house: body.house,
                        nakshatra: body.placement.nakshatra,
                        is_retrograde: body.placement.is_retrograde,
                    })
                })
                .collect::<Result<Vec<_>, DeriveError>>()?,
            houses: self
                .houses
                .into_iter()
                .map(|house| {
                    Ok(HouseResult {
                        house: house.house,
                        cusp_longitude: house.cusp_longitude,
                        sign: sign_from_longitude(house.cusp_longitude)?,
                    })
                })
                .collect::<Result<Vec<_>, DeriveError>>()?,
        })
    }
}
