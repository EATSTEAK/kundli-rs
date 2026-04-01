use crate::kundli::astro::HouseSystem;
use crate::kundli::derive::sign::normalize_longitude;
use crate::kundli::error::DeriveError;
use crate::kundli::model::HouseNumber;

use super::{SignContext, SignPlacement};

const DEGREES_PER_SIGN: f64 = 30.0;
const NUM_HOUSES: usize = 12;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct HouseContext {
    pub reference: super::ResolvedReference,
    pub ascendant: SignPlacement,
    pub ascendant_house: HouseNumber,
    pub bodies: Vec<HousedPlacement>,
    pub houses: Vec<HouseSeed>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct HousedPlacement {
    pub placement: SignPlacement,
    pub house: HouseNumber,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct HouseSeed {
    pub house: HouseNumber,
    pub cusp_longitude: f64,
}

pub(crate) trait HouseTransformOp<Input> {
    type Output;

    fn apply(&self, input: &Input) -> Result<Self::Output, DeriveError>;
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct WholeSignHouseTransform;

impl HouseTransformOp<SignContext> for WholeSignHouseTransform {
    type Output = HouseContext;

    fn apply(&self, input: &SignContext) -> Result<Self::Output, DeriveError> {
        let ascendant_house =
            whole_sign_house(input.reference.longitude, input.ascendant.longitude)?;
        let first_house_cusp = sign_start_longitude(input.reference.longitude)?;

        Ok(HouseContext {
            reference: input.reference.clone(),
            ascendant: input.ascendant.clone(),
            ascendant_house,
            bodies: input
                .bodies
                .iter()
                .map(|placement| {
                    Ok(HousedPlacement {
                        house: whole_sign_house(input.reference.longitude, placement.longitude)?,
                        placement: placement.clone(),
                    })
                })
                .collect::<Result<Vec<_>, DeriveError>>()?,
            houses: (0..NUM_HOUSES)
                .map(|index| {
                    let house = HouseNumber::new((index + 1) as u8)
                        .ok_or(DeriveError::InvalidHouseNumber((index + 1) as u8))?;
                    let cusp_longitude =
                        normalize_longitude(first_house_cusp + index as f64 * DEGREES_PER_SIGN)?;
                    Ok(HouseSeed {
                        house,
                        cusp_longitude,
                    })
                })
                .collect::<Result<Vec<_>, DeriveError>>()?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CuspBasedHouseTransform {
    pub house_system: HouseSystem,
}

impl HouseTransformOp<SignContext> for CuspBasedHouseTransform {
    type Output = HouseContext;

    fn apply(&self, input: &SignContext) -> Result<Self::Output, DeriveError> {
        let _ = self.house_system;

        if input.house_cusps.len() != NUM_HOUSES {
            return Err(DeriveError::InvalidHouseCusps(input.house_cusps.len()));
        }

        let first_house = derive_house_from_cusps(input.reference.longitude, &input.house_cusps)?;
        let ascendant_house =
            derive_house_from_cusps(input.ascendant.longitude, &input.house_cusps)?;

        let ascendant_house = renumber_house(ascendant_house, first_house)?;
        let bodies = input
            .bodies
            .iter()
            .map(|placement| {
                let absolute_house =
                    derive_house_from_cusps(placement.longitude, &input.house_cusps)?;
                let house = renumber_house(absolute_house, first_house)?;
                Ok(HousedPlacement {
                    house,
                    placement: placement.clone(),
                })
            })
            .collect::<Result<Vec<_>, DeriveError>>()?;
        let houses = input
            .house_cusps
            .iter()
            .enumerate()
            .map(|(index, cusp)| {
                let absolute_house = HouseNumber::new((index + 1) as u8)
                    .ok_or(DeriveError::InvalidHouseNumber((index + 1) as u8))?;
                let house = renumber_house(absolute_house, first_house)?;
                Ok(HouseSeed {
                    house,
                    cusp_longitude: normalize_longitude(*cusp)?,
                })
            })
            .collect::<Result<Vec<_>, DeriveError>>()?;

        Ok(HouseContext {
            reference: input.reference.clone(),
            ascendant: input.ascendant.clone(),
            ascendant_house,
            bodies,
            houses,
        })
    }
}

fn sign_start_longitude(longitude: f64) -> Result<f64, DeriveError> {
    let longitude = normalize_longitude(longitude)?;
    normalize_longitude((longitude / DEGREES_PER_SIGN).floor() * DEGREES_PER_SIGN)
}

fn whole_sign_house(
    reference_longitude: f64,
    target_longitude: f64,
) -> Result<HouseNumber, DeriveError> {
    let reference_sign = sign_index(reference_longitude)?;
    let target_sign = sign_index(target_longitude)?;
    let house = ((target_sign + NUM_HOUSES - reference_sign) % NUM_HOUSES + 1) as u8;
    HouseNumber::new(house).ok_or(DeriveError::InvalidHouseNumber(house))
}

fn sign_index(longitude: f64) -> Result<usize, DeriveError> {
    let longitude = normalize_longitude(longitude)?;
    Ok((longitude / DEGREES_PER_SIGN).floor() as usize % NUM_HOUSES)
}

fn renumber_house(
    absolute_house: HouseNumber,
    first_house: HouseNumber,
) -> Result<HouseNumber, DeriveError> {
    let renumbered =
        ((absolute_house.get() + NUM_HOUSES as u8 - first_house.get()) % NUM_HOUSES as u8) + 1;
    HouseNumber::new(renumbered).ok_or(DeriveError::InvalidHouseNumber(renumbered))
}

fn derive_house_from_cusps(
    planet_longitude: f64,
    house_cusps: &[f64],
) -> Result<HouseNumber, DeriveError> {
    if house_cusps.len() != NUM_HOUSES {
        return Err(DeriveError::InvalidHouseCusps(house_cusps.len()));
    }

    let planet_longitude = normalize_longitude(planet_longitude)?;

    for index in 0..NUM_HOUSES {
        let start = normalize_longitude(house_cusps[index])?;
        let end = normalize_longitude(house_cusps[(index + 1) % NUM_HOUSES])?;
        if is_in_range(planet_longitude, start, end) {
            return HouseNumber::new((index + 1) as u8)
                .ok_or(DeriveError::InvalidHouseNumber((index + 1) as u8));
        }
    }

    Err(DeriveError::InvalidHouseCusps(house_cusps.len()))
}

fn is_in_range(longitude: f64, start: f64, end: f64) -> bool {
    if start <= end {
        longitude >= start && longitude < end
    } else {
        longitude >= start || longitude < end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_sign_house_always_returns_a_valid_house_number() {
        for reference in [0.0, 29.999, 30.0, 180.0, 359.999] {
            for target in [0.0, 15.0, 30.0, 90.0, 180.0, 359.999] {
                let house = whole_sign_house(reference, target).unwrap();
                assert!((1..=12).contains(&house.get()));
            }
        }
    }
}
