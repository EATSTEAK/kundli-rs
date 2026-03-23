use crate::kundli::astro::{AstroResult, HouseSystem};
use crate::kundli::config::KundliConfig;
use crate::kundli::derive::d1::derive_lagna_from_input;
use crate::kundli::derive::house::derive_house;
use crate::kundli::derive::input::KundliDeriveInput;
use crate::kundli::error::DeriveError;
use crate::kundli::model::{D9Chart, PlanetPlacement};

pub(crate) fn derive_d9_chart_from_input(
    input: &KundliDeriveInput,
    config: &KundliConfig,
) -> Result<D9Chart, DeriveError> {
    if input.meta.zodiac != crate::kundli::astro::ZodiacType::Sidereal {
        return Err(DeriveError::UnsupportedZodiac(input.meta.zodiac));
    }

    if config.house_system != HouseSystem::WholeSign {
        return Err(DeriveError::UnsupportedD9HouseSystem(config.house_system));
    }

    let navamsa = input.to_navamsa()?;

    Ok(D9Chart {
        lagna: derive_lagna_from_input(&navamsa)?,
        planets: derive_d9_planet_placements_from_input(&navamsa)?,
    })
}

pub fn derive_d9_chart(astro: &AstroResult, config: &KundliConfig) -> Result<D9Chart, DeriveError> {
    let input = KundliDeriveInput::from_astro(astro)?;
    derive_d9_chart_from_input(&input, config)
}

fn derive_d9_planet_placements_from_input(
    input: &KundliDeriveInput,
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
                    &[],
                    HouseSystem::WholeSign,
                )?,
                nakshatra: body.nakshatra,
                is_retrograde: body.is_retrograde,
            })
        })
        .collect()
}
