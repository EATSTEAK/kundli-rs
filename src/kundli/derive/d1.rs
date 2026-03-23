use crate::kundli::astro::{AstroResult, HouseSystem};
use crate::kundli::config::KundliConfig;
use crate::kundli::derive::house::derive_house;
use crate::kundli::derive::input::KundliDeriveInput;
use crate::kundli::derive::sign::{normalize_longitude, sign_from_longitude};
use crate::kundli::error::DeriveError;
use crate::kundli::model::{D1Chart, HouseNumber, HouseResult, LagnaResult, PlanetPlacement};

const DEGREES_PER_SIGN: f64 = 30.0;
const NUM_HOUSES: usize = 12;

pub(crate) fn derive_lagna_from_input(
    input: &KundliDeriveInput,
) -> Result<LagnaResult, DeriveError> {
    Ok(LagnaResult {
        sign: input.ascendant.sign,
        degrees_in_sign: input.ascendant.degrees_in_sign,
        longitude: input.ascendant.longitude,
    })
}

pub fn derive_lagna(astro: &AstroResult) -> Result<LagnaResult, DeriveError> {
    let input = KundliDeriveInput::from_astro(astro)?;
    derive_lagna_from_input(&input)
}

pub(crate) fn derive_planet_placements_from_input(
    input: &KundliDeriveInput,
    config: &KundliConfig,
) -> Result<Vec<PlanetPlacement>, DeriveError> {
    input
        .bodies
        .iter()
        .map(|body| {
            Ok(PlanetPlacement {
                body: body.body,
                longitude: body.longitude,
                sign: body.sign,
                degrees_in_sign: body.degrees_in_sign,
                house: derive_house(
                    body.longitude,
                    input.ascendant.longitude,
                    &input.house_cusps,
                    config.house_system,
                )?,
                nakshatra: body.nakshatra,
                is_retrograde: body.is_retrograde,
            })
        })
        .collect()
}

pub fn derive_planet_placements(
    astro: &AstroResult,
    config: &KundliConfig,
) -> Result<Vec<PlanetPlacement>, DeriveError> {
    let input = KundliDeriveInput::from_astro(astro)?;
    derive_planet_placements_from_input(&input, config)
}

pub(crate) fn derive_houses_from_input(
    input: &KundliDeriveInput,
    config: &KundliConfig,
) -> Result<Vec<HouseResult>, DeriveError> {
    match config.house_system {
        HouseSystem::WholeSign => {
            // WholeSign houses are anchored to sign boundaries, not the literal ascendant degree.
            let first_house_cusp =
                (input.ascendant.longitude / DEGREES_PER_SIGN).floor() * DEGREES_PER_SIGN;

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
            if input.house_cusps.len() != NUM_HOUSES {
                return Err(DeriveError::InvalidHouseCusps(input.house_cusps.len()));
            }

            input
                .house_cusps
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

pub fn derive_houses(
    astro: &AstroResult,
    config: &KundliConfig,
) -> Result<Vec<HouseResult>, DeriveError> {
    let input = KundliDeriveInput::from_astro(astro)?;
    derive_houses_from_input(&input, config)
}

pub(crate) fn derive_d1_chart_from_input(
    input: &KundliDeriveInput,
    config: &KundliConfig,
) -> Result<D1Chart, DeriveError> {
    Ok(D1Chart {
        lagna: derive_lagna_from_input(input)?,
        planets: derive_planet_placements_from_input(input, config)?,
        houses: derive_houses_from_input(input, config)?,
    })
}

pub fn derive_d1_chart(astro: &AstroResult, config: &KundliConfig) -> Result<D1Chart, DeriveError> {
    let input = KundliDeriveInput::from_astro(astro)?;
    derive_d1_chart_from_input(&input, config)
}
