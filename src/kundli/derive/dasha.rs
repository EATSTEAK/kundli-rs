use crate::kundli::astro::{AstroBody, AstroResult, ZodiacType};
use crate::kundli::derive::nakshatra::{
    dasha_lord_for_nakshatra, nakshatra_placement_from_longitude, nakshatra_progress_ratio,
};
use crate::kundli::error::DeriveError;
use crate::kundli::model::{DashaLord, DashaPeriod, Nakshatra, VimshottariDasha};

// Use a tropical year approximation for mapping Vimshottari year lengths onto Julian days.
const DAYS_PER_YEAR: f64 = 365.25;

#[derive(Debug, Clone, PartialEq)]
struct MoonDashaSeed {
    jd_ut: f64,
    moon_nakshatra: Nakshatra,
    moon_nakshatra_progress_ratio: f64,
}

pub(crate) struct DashaPipeline;

impl DashaPipeline {
    pub(crate) fn execute(&self, astro: &AstroResult) -> Result<VimshottariDasha, DeriveError> {
        let seed = MoonDashaSeed::from_astro(astro)?;
        materialize_vimshottari_dasha(seed)
    }
}

impl MoonDashaSeed {
    fn from_astro(astro: &AstroResult) -> Result<Self, DeriveError> {
        if astro.meta.zodiac != ZodiacType::Sidereal {
            return Err(DeriveError::UnsupportedZodiac(astro.meta.zodiac));
        }

        let moon = astro.body(AstroBody::Moon);
        let moon_nakshatra = nakshatra_placement_from_longitude(moon.longitude)?.nakshatra;
        let moon_nakshatra_progress_ratio = nakshatra_progress_ratio(moon.longitude)?;

        Ok(Self {
            jd_ut: astro.meta.jd_ut,
            moon_nakshatra,
            moon_nakshatra_progress_ratio,
        })
    }
}

fn materialize_vimshottari_dasha(seed: MoonDashaSeed) -> Result<VimshottariDasha, DeriveError> {
    let current_lord = dasha_lord_for_nakshatra(seed.moon_nakshatra);
    let current_duration_days = mahadasha_duration_days(current_lord);
    let current_start_jd_ut =
        seed.jd_ut - current_duration_days * seed.moon_nakshatra_progress_ratio;
    let sequence_start = DashaLord::SEQUENCE
        .iter()
        .position(|&lord| lord == current_lord)
        .ok_or(DeriveError::InvalidDashaSequenceLord(current_lord))?;

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
        moon_nakshatra: seed.moon_nakshatra,
        current_mahadasha: mahadashas[0].clone(),
        mahadashas,
    })
}

/// Derives Vimshottari mahadasha periods from a raw astronomical result.
///
/// This is a lower-level helper than [`crate::calculate_kundli`]. The input
/// must be sidereal and must include the Moon.
pub fn derive_vimshottari_dasha(astro: &AstroResult) -> Result<VimshottariDasha, DeriveError> {
    DashaPipeline.execute(astro)
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
