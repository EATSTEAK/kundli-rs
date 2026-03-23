use crate::kundli::astro::{AstroResult, HouseSystem};
use crate::kundli::config::KundliConfig;
use crate::kundli::derive::house::derive_house;
use crate::kundli::derive::nakshatra::nakshatra_placement_from_longitude;
use crate::kundli::derive::sign::{degrees_in_sign, normalize_longitude, sign_from_longitude};
use crate::kundli::error::DeriveError;
use crate::kundli::model::{D9Chart, LagnaResult, PlanetPlacement};

const NAVAMSAS_PER_SIGN: f64 = 9.0;

fn navamsa_longitude(longitude: f64) -> Result<f64, DeriveError> {
    let longitude = normalize_longitude(longitude)?;
    normalize_longitude(longitude * NAVAMSAS_PER_SIGN)
}

fn derive_d9_lagna(astro: &AstroResult) -> Result<LagnaResult, DeriveError> {
    let longitude = navamsa_longitude(astro.ascendant_longitude)?;

    Ok(LagnaResult {
        sign: sign_from_longitude(longitude)?,
        degrees_in_sign: degrees_in_sign(longitude)?,
        longitude,
    })
}

fn derive_d9_planet_placements(astro: &AstroResult) -> Result<Vec<PlanetPlacement>, DeriveError> {
    let ascendant_longitude = navamsa_longitude(astro.ascendant_longitude)?;

    astro
        .bodies
        .iter()
        .map(|body| {
            let longitude = navamsa_longitude(body.longitude)?;

            Ok(PlanetPlacement {
                body: body.body,
                longitude,
                sign: sign_from_longitude(longitude)?,
                degrees_in_sign: degrees_in_sign(longitude)?,
                // Raw D1 cusps do not map to D9, so derive divisional houses from the D9 lagna only.
                house: derive_house(longitude, ascendant_longitude, &[], HouseSystem::WholeSign)?,
                nakshatra: nakshatra_placement_from_longitude(longitude)?,
                is_retrograde: body.speed_longitude < 0.0,
            })
        })
        .collect()
}

pub fn derive_d9_chart(astro: &AstroResult, config: &KundliConfig) -> Result<D9Chart, DeriveError> {
    let _ = config;

    Ok(D9Chart {
        lagna: derive_d9_lagna(astro)?,
        planets: derive_d9_planet_placements(astro)?,
    })
}

const _: () = {
    let _ = NAVAMSAS_PER_SIGN;
    let _ = navamsa_longitude as fn(f64) -> Result<f64, DeriveError>;
    let _ = derive_d9_lagna as fn(&AstroResult) -> Result<LagnaResult, DeriveError>;
    let _ = derive_d9_planet_placements
        as fn(&AstroResult) -> Result<Vec<PlanetPlacement>, DeriveError>;
    let _ = derive_d9_chart as fn(&AstroResult, &KundliConfig) -> Result<D9Chart, DeriveError>;
};
