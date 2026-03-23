use crate::kundli::astro::{AstroBody, AstroResult};
use crate::kundli::derive::input::KundliDeriveInput;
use crate::kundli::derive::nakshatra::dasha_lord_for_nakshatra;
use crate::kundli::error::DeriveError;
use crate::kundli::model::{DashaLord, DashaPeriod, VimshottariDasha};

// Use a tropical year approximation for mapping Vimshottari year lengths onto Julian days.
const DAYS_PER_YEAR: f64 = 365.25;

pub(crate) fn derive_vimshottari_dasha_from_input(
    input: &KundliDeriveInput,
) -> Result<VimshottariDasha, DeriveError> {
    if input.meta.zodiac != crate::kundli::astro::ZodiacType::Sidereal {
        return Err(DeriveError::UnsupportedZodiac(input.meta.zodiac));
    }

    let moon = input.body(AstroBody::Moon).ok_or(DeriveError::MissingMoon)?;
    let current_lord = dasha_lord_for_nakshatra(moon.nakshatra.nakshatra);
    let current_duration_days = mahadasha_duration_days(current_lord);
    let current_start_jd_ut = input.meta.jd_ut - current_duration_days * moon.nakshatra_progress_ratio;
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
        moon_nakshatra: moon.nakshatra.nakshatra,
        current_mahadasha: mahadashas[0].clone(),
        mahadashas,
    })
}

pub fn derive_vimshottari_dasha(astro: &AstroResult) -> Result<VimshottariDasha, DeriveError> {
    let input = KundliDeriveInput::from_astro(astro)?;
    derive_vimshottari_dasha_from_input(&input)
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
