//! Nakshatra-related longitude helpers for the derive layer.
//!
//! Provides pure functions for:
//! - Nakshatra and pada derivation from longitude
//! - Degrees-in-nakshatra calculation
//! - Moon progress ratio for dasha calculations
//! - Dasha-lord mapping

use crate::kundli::derive::sign::normalize_longitude;
use crate::kundli::error::DeriveError;
use crate::kundli::model::{DashaLord, Nakshatra, NakshatraPlacement, Pada};

/// Degrees per nakshatra (360 / 27 = 13.333...).
const DEGREES_PER_NAKSHATRA: f64 = 360.0 / 27.0;

/// Degrees per pada (DEGREES_PER_NAKSHATRA / 4 = 3.333...).
const DEGREES_PER_PADA: f64 = DEGREES_PER_NAKSHATRA / 4.0;

/// Derives the nakshatra from a longitude.
///
/// The longitude is first normalized to [0, 360) before determining the nakshatra.
/// Returns an error if the input is not finite.
///
pub(crate) fn nakshatra_from_longitude(longitude: f64) -> Result<Nakshatra, DeriveError> {
    let normalized = normalize_longitude(longitude)?;
    let nakshatra_index = (normalized / DEGREES_PER_NAKSHATRA).floor() as usize;
    Ok(nakshatra_from_index(nakshatra_index))
}

/// Derives the pada (1-4) from a longitude.
///
/// The pada is determined by the position within the nakshatra.
/// Returns an error if the input is not finite.
///
pub(crate) fn pada_from_longitude(longitude: f64) -> Result<Pada, DeriveError> {
    let normalized = normalize_longitude(longitude)?;
    let degrees_in_nakshatra = normalized % DEGREES_PER_NAKSHATRA;
    let pada_number = (degrees_in_nakshatra / DEGREES_PER_PADA).floor() as u8 + 1;

    match Pada::new(pada_number) {
        Some(pada) => Ok(pada),
        None => Err(DeriveError::InvalidPada(pada_number)),
    }
}

/// Returns the degrees progressed within the current nakshatra.
///
/// This is the position within the nakshatra, in the range [0, 13.333...).
/// Returns an error if the input is not finite.
///
pub(crate) fn degrees_in_nakshatra(longitude: f64) -> Result<f64, DeriveError> {
    let normalized = normalize_longitude(longitude)?;
    Ok(normalized % DEGREES_PER_NAKSHATRA)
}

/// Returns the full nakshatra placement (nakshatra, pada, degrees-in-nakshatra).
///
/// This is a convenience function that combines the individual derivations.
/// Returns an error if the input is not finite.
pub(crate) fn nakshatra_placement_from_longitude(
    longitude: f64,
) -> Result<NakshatraPlacement, DeriveError> {
    let normalized = normalize_longitude(longitude)?;
    let nakshatra = nakshatra_from_longitude(normalized)?;
    let pada = pada_from_longitude(normalized)?;
    let degrees = degrees_in_nakshatra(normalized)?;
    Ok(NakshatraPlacement {
        nakshatra,
        pada,
        degrees_in_nakshatra: degrees,
    })
}

/// Returns the progress ratio through the current nakshatra.
///
/// This is a generic longitude helper that can be reused for any body.
/// Vimshottari dasha applies it specifically to the Moon.
/// The ratio is in the range [0.0, 1.0).
///
/// Returns an error if the input is not finite.
///
pub(crate) fn nakshatra_progress_ratio(longitude: f64) -> Result<f64, DeriveError> {
    let normalized = normalize_longitude(longitude)?;
    let degrees = normalized % DEGREES_PER_NAKSHATRA;
    Ok(degrees / DEGREES_PER_NAKSHATRA)
}

/// Returns the ruling dasha lord for a nakshatra.
///
/// In Vimshottari dasha, each nakshatra is ruled by one of the 9 planets.
/// The sequence starts with Ketu at Ashwini and cycles every 9 nakshatras.
///
pub(crate) fn dasha_lord_for_nakshatra(nakshatra: Nakshatra) -> DashaLord {
    let nakshatra_index = nakshatra_to_index(nakshatra);
    let lord_index = nakshatra_index % 9;
    DashaLord::SEQUENCE[lord_index]
}

/// Converts a nakshatra index (0-26) to a Nakshatra enum variant.
fn nakshatra_from_index(index: usize) -> Nakshatra {
    match index {
        0 => Nakshatra::Ashwini,
        1 => Nakshatra::Bharani,
        2 => Nakshatra::Krittika,
        3 => Nakshatra::Rohini,
        4 => Nakshatra::Mrigashira,
        5 => Nakshatra::Ardra,
        6 => Nakshatra::Punarvasu,
        7 => Nakshatra::Pushya,
        8 => Nakshatra::Ashlesha,
        9 => Nakshatra::Magha,
        10 => Nakshatra::PurvaPhalguni,
        11 => Nakshatra::UttaraPhalguni,
        12 => Nakshatra::Hasta,
        13 => Nakshatra::Chitra,
        14 => Nakshatra::Swati,
        15 => Nakshatra::Vishakha,
        16 => Nakshatra::Anuradha,
        17 => Nakshatra::Jyeshtha,
        18 => Nakshatra::Mula,
        19 => Nakshatra::PurvaAshadha,
        20 => Nakshatra::UttaraAshadha,
        21 => Nakshatra::Shravana,
        22 => Nakshatra::Dhanishta,
        23 => Nakshatra::Shatabhisha,
        24 => Nakshatra::PurvaBhadrapada,
        25 => Nakshatra::UttaraBhadrapada,
        26 => Nakshatra::Revati,
        // SAFETY: index is derived from normalized longitude / DEGREES_PER_NAKSHATRA, so max is 26
        _ => unreachable!("nakshatra index out of range: {index}"),
    }
}

/// Converts a Nakshatra enum variant to its index (0-26).
fn nakshatra_to_index(nakshatra: Nakshatra) -> usize {
    match nakshatra {
        Nakshatra::Ashwini => 0,
        Nakshatra::Bharani => 1,
        Nakshatra::Krittika => 2,
        Nakshatra::Rohini => 3,
        Nakshatra::Mrigashira => 4,
        Nakshatra::Ardra => 5,
        Nakshatra::Punarvasu => 6,
        Nakshatra::Pushya => 7,
        Nakshatra::Ashlesha => 8,
        Nakshatra::Magha => 9,
        Nakshatra::PurvaPhalguni => 10,
        Nakshatra::UttaraPhalguni => 11,
        Nakshatra::Hasta => 12,
        Nakshatra::Chitra => 13,
        Nakshatra::Swati => 14,
        Nakshatra::Vishakha => 15,
        Nakshatra::Anuradha => 16,
        Nakshatra::Jyeshtha => 17,
        Nakshatra::Mula => 18,
        Nakshatra::PurvaAshadha => 19,
        Nakshatra::UttaraAshadha => 20,
        Nakshatra::Shravana => 21,
        Nakshatra::Dhanishta => 22,
        Nakshatra::Shatabhisha => 23,
        Nakshatra::PurvaBhadrapada => 24,
        Nakshatra::UttaraBhadrapada => 25,
        Nakshatra::Revati => 26,
    }
}

const _: () = {
    let _ = DEGREES_PER_NAKSHATRA;
    let _ = DEGREES_PER_PADA;
    let _ = nakshatra_from_longitude as fn(f64) -> Result<Nakshatra, DeriveError>;
    let _ = pada_from_longitude as fn(f64) -> Result<Pada, DeriveError>;
    let _ = degrees_in_nakshatra as fn(f64) -> Result<f64, DeriveError>;
    let _ =
        nakshatra_placement_from_longitude as fn(f64) -> Result<NakshatraPlacement, DeriveError>;
    let _ = nakshatra_progress_ratio as fn(f64) -> Result<f64, DeriveError>;
    let _ = dasha_lord_for_nakshatra as fn(Nakshatra) -> DashaLord;
    let _ = nakshatra_from_index as fn(usize) -> Nakshatra;
    let _ = nakshatra_to_index as fn(Nakshatra) -> usize;
};

#[cfg(test)]
mod tests {
    use super::*;

    const DEGREES_PER_NAKSHATRA: f64 = 360.0 / 27.0;

    #[test]
    fn test_normalize_longitude_positive() {
        assert!((normalize_longitude(0.0).unwrap() - 0.0).abs() < 1e-10);
        assert!((normalize_longitude(180.0).unwrap() - 180.0).abs() < 1e-10);
        assert!((normalize_longitude(359.999).unwrap() - 359.999).abs() < 1e-10);
    }

    #[test]
    fn test_normalize_longitude_exact_360() {
        assert!((normalize_longitude(360.0).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_normalize_longitude_overflow() {
        assert!((normalize_longitude(390.0).unwrap() - 30.0).abs() < 1e-10);
    }

    #[test]
    fn test_normalize_longitude_negative() {
        assert!((normalize_longitude(-30.0).unwrap() - 330.0).abs() < 1e-10);
    }

    #[test]
    fn test_normalize_longitude_invalid() {
        assert!(normalize_longitude(f64::NAN).is_err());
        assert!(normalize_longitude(f64::INFINITY).is_err());
        assert!(normalize_longitude(f64::NEG_INFINITY).is_err());
    }

    #[test]
    fn test_nakshatra_from_longitude_boundaries() {
        // First nakshatra: Ashwini (0 - 13.333...)
        assert_eq!(nakshatra_from_longitude(0.0).unwrap(), Nakshatra::Ashwini);
        assert_eq!(
            nakshatra_from_longitude(DEGREES_PER_NAKSHATRA - 0.001).unwrap(),
            Nakshatra::Ashwini
        );

        // Second nakshatra: Bharani (13.333... - 26.666...)
        assert_eq!(
            nakshatra_from_longitude(DEGREES_PER_NAKSHATRA).unwrap(),
            Nakshatra::Bharani
        );

        // Last nakshatra: Revati (346.666... - 360)
        assert_eq!(
            nakshatra_from_longitude(360.0 - DEGREES_PER_NAKSHATRA / 2.0).unwrap(),
            Nakshatra::Revati
        );
        assert_eq!(
            nakshatra_from_longitude(359.999).unwrap(),
            Nakshatra::Revati
        );
    }

    #[test]
    fn test_nakshatra_from_longitude_all() {
        // Test each nakshatra at a safe midpoint (not at boundaries)
        for i in 0..27 {
            let expected = match i {
                0 => Nakshatra::Ashwini,
                1 => Nakshatra::Bharani,
                2 => Nakshatra::Krittika,
                3 => Nakshatra::Rohini,
                4 => Nakshatra::Mrigashira,
                5 => Nakshatra::Ardra,
                6 => Nakshatra::Punarvasu,
                7 => Nakshatra::Pushya,
                8 => Nakshatra::Ashlesha,
                9 => Nakshatra::Magha,
                10 => Nakshatra::PurvaPhalguni,
                11 => Nakshatra::UttaraPhalguni,
                12 => Nakshatra::Hasta,
                13 => Nakshatra::Chitra,
                14 => Nakshatra::Swati,
                15 => Nakshatra::Vishakha,
                16 => Nakshatra::Anuradha,
                17 => Nakshatra::Jyeshtha,
                18 => Nakshatra::Mula,
                19 => Nakshatra::PurvaAshadha,
                20 => Nakshatra::UttaraAshadha,
                21 => Nakshatra::Shravana,
                22 => Nakshatra::Dhanishta,
                23 => Nakshatra::Shatabhisha,
                24 => Nakshatra::PurvaBhadrapada,
                25 => Nakshatra::UttaraBhadrapada,
                26 => Nakshatra::Revati,
                _ => unreachable!(),
            };
            // Use midpoint of nakshatra to avoid boundary issues
            let longitude = DEGREES_PER_NAKSHATRA * (i as f64) + DEGREES_PER_NAKSHATRA / 2.0;
            assert_eq!(
                nakshatra_from_longitude(longitude).unwrap(),
                expected,
                "Failed for nakshatra index {i} at longitude {longitude}"
            );
        }
    }

    #[test]
    fn test_nakshatra_from_longitude_invalid() {
        assert!(nakshatra_from_longitude(f64::NAN).is_err());
        assert!(nakshatra_from_longitude(f64::INFINITY).is_err());
    }

    #[test]
    fn test_pada_from_longitude_boundaries() {
        // Pada 1: 0 - 3.333...
        assert_eq!(pada_from_longitude(0.0).unwrap().get(), 1);
        assert_eq!(pada_from_longitude(3.3).unwrap().get(), 1);

        // Pada 2: 3.333... - 6.666...
        assert_eq!(pada_from_longitude(3.4).unwrap().get(), 2);
        assert_eq!(pada_from_longitude(6.5).unwrap().get(), 2);

        // Pada 3: 6.666... - 10.0
        assert_eq!(pada_from_longitude(6.7).unwrap().get(), 3);
        assert_eq!(pada_from_longitude(9.9).unwrap().get(), 3);

        // Pada 4: 10.0 - 13.333...
        assert_eq!(pada_from_longitude(10.0).unwrap().get(), 4);
        assert_eq!(pada_from_longitude(13.2).unwrap().get(), 4);
    }

    #[test]
    fn test_pada_from_longitude_across_nakshatras() {
        // Each nakshatra should have the same pada pattern
        // Use small offsets from nakshatra start to avoid boundary floating-point issues
        // DEGREES_PER_PADA = 3.333..., so:
        // Pada 1: 0 - 3.333
        // Pada 2: 3.333 - 6.666
        // Pada 3: 6.666 - 10.0
        // Pada 4: 10.0 - 13.333
        for i in 0..27 {
            let base = DEGREES_PER_NAKSHATRA * (i as f64) + 0.1; // Small offset from boundary
            assert_eq!(pada_from_longitude(base).unwrap().get(), 1); // 0.1 is in pada 1
            assert_eq!(pada_from_longitude(base + 3.5).unwrap().get(), 2); // 3.6 is in pada 2
            assert_eq!(pada_from_longitude(base + 7.0).unwrap().get(), 3); // 7.1 is in pada 3
            assert_eq!(pada_from_longitude(base + 10.5).unwrap().get(), 4); // 10.6 is in pada 4
        }
    }

    #[test]
    fn test_pada_from_longitude_invalid() {
        assert!(pada_from_longitude(f64::NAN).is_err());
        assert!(pada_from_longitude(f64::INFINITY).is_err());
    }

    #[test]
    fn test_degrees_in_nakshatra_basic() {
        assert!((degrees_in_nakshatra(0.0).unwrap() - 0.0).abs() < 1e-10);
        assert!((degrees_in_nakshatra(6.666).unwrap() - 6.666).abs() < 1e-3);
    }

    #[test]
    fn test_degrees_in_nakshatra_across_boundary() {
        let result = degrees_in_nakshatra(DEGREES_PER_NAKSHATRA).unwrap();
        assert!(result.abs() < 1e-10);
    }

    #[test]
    fn test_degrees_in_nakshatra_invalid() {
        assert!(degrees_in_nakshatra(f64::NAN).is_err());
        assert!(degrees_in_nakshatra(f64::INFINITY).is_err());
    }

    #[test]
    fn test_nakshatra_progress_ratio_start() {
        assert!((nakshatra_progress_ratio(0.0).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_nakshatra_progress_ratio_middle() {
        let mid_degrees = DEGREES_PER_NAKSHATRA / 2.0;
        let ratio = nakshatra_progress_ratio(mid_degrees).unwrap();
        assert!((ratio - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_nakshatra_progress_ratio_end() {
        let end_degrees = DEGREES_PER_NAKSHATRA - 0.001;
        let ratio = nakshatra_progress_ratio(end_degrees).unwrap();
        assert!(ratio < 1.0);
        assert!(ratio > 0.99);
    }

    #[test]
    fn test_nakshatra_progress_ratio_across_nakshatras() {
        // Progress ratio should reset at each nakshatra boundary
        let ratio1 = nakshatra_progress_ratio(13.0).unwrap();
        let ratio2 = nakshatra_progress_ratio(26.5).unwrap();
        assert!((ratio1 - ratio2).abs() < 0.05); // Both near end of their nakshatras
    }

    #[test]
    fn test_nakshatra_progress_ratio_invalid() {
        assert!(nakshatra_progress_ratio(f64::NAN).is_err());
        assert!(nakshatra_progress_ratio(f64::INFINITY).is_err());
    }

    #[test]
    fn test_dasha_lord_for_nakshatra_sequence() {
        // First 9 nakshatras map directly to SEQUENCE
        assert_eq!(
            dasha_lord_for_nakshatra(Nakshatra::Ashwini),
            DashaLord::Ketu
        );
        assert_eq!(
            dasha_lord_for_nakshatra(Nakshatra::Bharani),
            DashaLord::Venus
        );
        assert_eq!(
            dasha_lord_for_nakshatra(Nakshatra::Krittika),
            DashaLord::Sun
        );
        assert_eq!(dasha_lord_for_nakshatra(Nakshatra::Rohini), DashaLord::Moon);
        assert_eq!(
            dasha_lord_for_nakshatra(Nakshatra::Mrigashira),
            DashaLord::Mars
        );
        assert_eq!(dasha_lord_for_nakshatra(Nakshatra::Ardra), DashaLord::Rahu);
        assert_eq!(
            dasha_lord_for_nakshatra(Nakshatra::Punarvasu),
            DashaLord::Jupiter
        );
        assert_eq!(
            dasha_lord_for_nakshatra(Nakshatra::Pushya),
            DashaLord::Saturn
        );
        assert_eq!(
            dasha_lord_for_nakshatra(Nakshatra::Ashlesha),
            DashaLord::Mercury
        );

        // Cycle repeats for next 9 nakshatras
        assert_eq!(dasha_lord_for_nakshatra(Nakshatra::Magha), DashaLord::Ketu);
        assert_eq!(
            dasha_lord_for_nakshatra(Nakshatra::PurvaPhalguni),
            DashaLord::Venus
        );

        // And again for last 9
        assert_eq!(dasha_lord_for_nakshatra(Nakshatra::Mula), DashaLord::Ketu);
        assert_eq!(
            dasha_lord_for_nakshatra(Nakshatra::Revati),
            DashaLord::Mercury
        );
    }

    #[test]
    fn test_nakshatra_placement_from_longitude() {
        let placement = nakshatra_placement_from_longitude(5.0).unwrap();
        assert_eq!(placement.nakshatra, Nakshatra::Ashwini);
        assert_eq!(placement.pada, Pada::new(2).unwrap());
        assert!((placement.degrees_in_nakshatra - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_nakshatra_placement_from_longitude_invalid() {
        assert!(nakshatra_placement_from_longitude(f64::NAN).is_err());
        assert!(nakshatra_placement_from_longitude(f64::INFINITY).is_err());
    }
}
