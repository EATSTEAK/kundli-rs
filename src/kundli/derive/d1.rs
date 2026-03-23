use crate::kundli::astro::{AstroResult, HouseSystem};
use crate::kundli::config::KundliConfig;
use crate::kundli::derive::input::KundliDeriveInput;
use crate::kundli::derive::sign::{normalize_longitude, sign_from_longitude};
use crate::kundli::error::DeriveError;
use crate::kundli::model::{D1Chart, HouseNumber, HouseResult, LagnaResult, PlanetPlacement};

const DEGREES_PER_SIGN: f64 = 30.0;
const NUM_HOUSES: usize = 12;

pub(crate) fn derive_lagna_from_input(input: &KundliDeriveInput) -> Result<LagnaResult, DeriveError> {
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
                house: derive_house_from_input(body.longitude, input, config)?,
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

fn derive_house_from_input(
    planet_longitude: f64,
    input: &KundliDeriveInput,
    config: &KundliConfig,
) -> Result<HouseNumber, DeriveError> {
    match config.house_system {
        HouseSystem::WholeSign => derive_house_whole_sign(planet_longitude, input.ascendant.longitude),
        _ => derive_house_from_cusps(planet_longitude, &input.house_cusps),
    }
}

fn derive_house_whole_sign(
    planet_longitude: f64,
    ascendant_longitude: f64,
) -> Result<HouseNumber, DeriveError> {
    let planet_sign = longitude_to_sign_index(planet_longitude);
    let ascendant_sign = longitude_to_sign_index(ascendant_longitude);
    let house_number = ((planet_sign as i32 - ascendant_sign as i32 + NUM_HOUSES as i32)
        % NUM_HOUSES as i32)
        + 1;

    debug_assert!((1..=12).contains(&house_number));
    Ok(HouseNumber(house_number as u8))
}

fn derive_house_from_cusps(
    planet_longitude: f64,
    house_cusps: &[f64],
) -> Result<HouseNumber, DeriveError> {
    if house_cusps.len() != NUM_HOUSES {
        return Err(DeriveError::InvalidHouseCusps(house_cusps.len()));
    }

    for &cusp in house_cusps {
        if !cusp.is_finite() {
            return Err(DeriveError::InvalidLongitude(cusp));
        }
    }

    let planet_lon = normalize_longitude(planet_longitude)?;

    for i in 0..NUM_HOUSES {
        let cusp_start = normalize_longitude(house_cusps[i])?;
        let cusp_end = normalize_longitude(house_cusps[(i + 1) % NUM_HOUSES])?;

        if is_in_house(planet_lon, cusp_start, cusp_end) {
            return Ok(HouseNumber((i + 1) as u8));
        }
    }

    Err(DeriveError::InvalidHouseCusps(house_cusps.len()))
}

fn longitude_to_sign_index(longitude: f64) -> usize {
    let normalized = normalize_longitude(longitude)
        .expect("longitude_to_sign_index is only used after finite longitude validation");
    (normalized / DEGREES_PER_SIGN).floor() as usize % NUM_HOUSES
}

fn is_in_house(planet_lon: f64, cusp_start: f64, cusp_end: f64) -> bool {
    if cusp_start <= cusp_end {
        planet_lon >= cusp_start && planet_lon < cusp_end
    } else {
        planet_lon >= cusp_start || planet_lon < cusp_end
    }
}
