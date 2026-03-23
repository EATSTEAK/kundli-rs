use crate::kundli::astro::{AstroResult, HouseSystem};
use crate::kundli::config::KundliConfig;
use crate::kundli::derive::house::derive_house;
use crate::kundli::derive::nakshatra::nakshatra_placement_from_longitude;
use crate::kundli::derive::sign::{degrees_in_sign, normalize_longitude, sign_from_longitude};
use crate::kundli::error::DeriveError;
use crate::kundli::model::{D1Chart, HouseNumber, HouseResult, LagnaResult, PlanetPlacement};

const DEGREES_PER_SIGN: f64 = 30.0;
const NUM_HOUSES: usize = 12;

pub fn derive_lagna(astro: &AstroResult) -> Result<LagnaResult, DeriveError> {
    let longitude = normalize_longitude(astro.ascendant_longitude)?;

    Ok(LagnaResult {
        sign: sign_from_longitude(longitude)?,
        degrees_in_sign: degrees_in_sign(longitude)?,
        longitude,
    })
}

pub fn derive_planet_placements(
    astro: &AstroResult,
    config: &KundliConfig,
) -> Result<Vec<PlanetPlacement>, DeriveError> {
    astro.bodies
        .iter()
        .map(|body| {
            let longitude = normalize_longitude(body.longitude)?;

            Ok(PlanetPlacement {
                body: body.body,
                longitude,
                sign: sign_from_longitude(longitude)?,
                degrees_in_sign: degrees_in_sign(longitude)?,
                house: derive_house(
                    body.longitude,
                    astro.ascendant_longitude,
                    &astro.house_cusps,
                    config.house_system,
                )?,
                nakshatra: nakshatra_placement_from_longitude(longitude)?,
                is_retrograde: body.speed_longitude < 0.0,
            })
        })
        .collect()
}

pub fn derive_houses(
    astro: &AstroResult,
    config: &KundliConfig,
) -> Result<Vec<HouseResult>, DeriveError> {
    match config.house_system {
        HouseSystem::WholeSign => {
            let ascendant_longitude = normalize_longitude(astro.ascendant_longitude)?;
            let first_house_cusp = (ascendant_longitude / DEGREES_PER_SIGN).floor() * DEGREES_PER_SIGN;

            (0..NUM_HOUSES)
                .map(|index| {
                    let cusp_longitude =
                        normalize_longitude(first_house_cusp + index as f64 * DEGREES_PER_SIGN)?;

                    Ok(HouseResult {
                        house: HouseNumber((index + 1) as u8),
                        cusp_longitude,
                        sign: sign_from_longitude(cusp_longitude)?,
                    })
                })
                .collect()
        }
        _ => {
            if astro.house_cusps.len() != NUM_HOUSES {
                return Err(DeriveError::InvalidHouseCusps(astro.house_cusps.len()));
            }

            astro.house_cusps
                .iter()
                .enumerate()
                .map(|(index, &cusp)| {
                    let cusp_longitude = normalize_longitude(cusp)?;

                    Ok(HouseResult {
                        house: HouseNumber((index + 1) as u8),
                        cusp_longitude,
                        sign: sign_from_longitude(cusp_longitude)?,
                    })
                })
                .collect()
        }
    }
}

pub fn derive_d1_chart(astro: &AstroResult, config: &KundliConfig) -> Result<D1Chart, DeriveError> {
    Ok(D1Chart {
        lagna: derive_lagna(astro)?,
        planets: derive_planet_placements(astro, config)?,
        houses: derive_houses(astro, config)?,
    })
}
