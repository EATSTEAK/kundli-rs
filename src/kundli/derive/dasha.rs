use crate::kundli::astro::{AstroBody, AstroResult};
use crate::kundli::derive::nakshatra::{
    dasha_lord_for_nakshatra, moon_progress_ratio, nakshatra_placement_from_longitude,
};
use crate::kundli::error::DeriveError;
use crate::kundli::model::{DashaLord, DashaPeriod, VimshottariDasha};

// Use a tropical year approximation for mapping Vimshottari year lengths onto Julian days.
const DAYS_PER_YEAR: f64 = 365.25;

pub fn derive_vimshottari_dasha(astro: &AstroResult) -> Result<VimshottariDasha, DeriveError> {
    if astro.meta.zodiac != crate::kundli::astro::ZodiacType::Sidereal {
        return Err(DeriveError::UnsupportedZodiac(astro.meta.zodiac));
    }

    let moon = astro
        .bodies
        .iter()
        .find(|body| body.body == AstroBody::Moon)
        .ok_or(DeriveError::MissingMoon)?;

    let moon_nakshatra = nakshatra_placement_from_longitude(moon.longitude)?;
    let current_lord = dasha_lord_for_nakshatra(moon_nakshatra.nakshatra);
    let elapsed_ratio = moon_progress_ratio(moon.longitude)?;
    let current_duration_days = mahadasha_duration_days(current_lord);
    let current_start_jd_ut = astro.meta.jd_ut - current_duration_days * elapsed_ratio;
    let sequence_start = DashaLord::SEQUENCE
        .iter()
        .position(|&lord| lord == current_lord)
        .expect("current dasha lord must be part of the Vimshottari sequence");

    let mut next_start_jd_ut = current_start_jd_ut;
    let mut mahadashas = Vec::with_capacity(DashaLord::SEQUENCE.len());

    for offset in 0..DashaLord::SEQUENCE.len() {
        let lord = DashaLord::SEQUENCE[(sequence_start + offset) % DashaLord::SEQUENCE.len()];
        let end_jd_ut = next_start_jd_ut + mahadasha_duration_days(lord);

        mahadashas.push(DashaPeriod {
            lord,
            start_jd_ut: next_start_jd_ut,
            end_jd_ut,
        });

        next_start_jd_ut = end_jd_ut;
    }

    Ok(VimshottariDasha {
        moon_nakshatra: moon_nakshatra.nakshatra,
        current_mahadasha: mahadashas[0].clone(),
        mahadashas,
    })
}

fn mahadasha_duration_days(lord: DashaLord) -> f64 {
    mahadasha_years(lord) * DAYS_PER_YEAR
}

fn mahadasha_years(lord: DashaLord) -> f64 {
    match lord {
        DashaLord::Ketu => 7.0,
        DashaLord::Venus => 20.0,
        DashaLord::Sun => 6.0,
        DashaLord::Moon => 10.0,
        DashaLord::Mars => 7.0,
        DashaLord::Rahu => 18.0,
        DashaLord::Jupiter => 16.0,
        DashaLord::Saturn => 19.0,
        DashaLord::Mercury => 17.0,
    }
}
