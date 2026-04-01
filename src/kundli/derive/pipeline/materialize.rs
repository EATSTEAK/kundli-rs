use crate::kundli::derive::sign::sign_from_longitude;
use crate::kundli::model::{ChartResult, HouseResult, LagnaResult, PlanetPlacement};

use super::HouseContext;

pub(crate) trait Materialize {
    fn materialize(self) -> ChartResult;
}

impl Materialize for HouseContext {
    fn materialize(self) -> ChartResult {
        ChartResult {
            lagna: LagnaResult {
                sign: self.ascendant.sign,
                degrees_in_sign: self.ascendant.degrees_in_sign,
                longitude: self.ascendant.longitude,
            },
            planets: self
                .bodies
                .into_iter()
                .map(|body| PlanetPlacement {
                    body: body
                        .placement
                        .body
                        .expect("planet placements in HouseContext must contain a body"),
                    longitude: body.placement.longitude,
                    sign: body.placement.sign,
                    degrees_in_sign: body.placement.degrees_in_sign,
                    house: body.house,
                    nakshatra: body.placement.nakshatra,
                    is_retrograde: body.placement.is_retrograde,
                })
                .collect(),
            houses: self
                .houses
                .into_iter()
                .map(|house| HouseResult {
                    house: house.house,
                    cusp_longitude: house.cusp_longitude,
                    sign: sign_from_longitude(house.cusp_longitude)
                        .expect("materialized house cusp longitudes must be finite"),
                })
                .collect(),
        }
    }
}
