#![allow(dead_code)]

//! Sign-related longitude helpers for the derive layer.
//!
//! Provides pure functions for:
//! - Longitude normalization to [0, 360)
//! - Sign derivation from longitude
//! - Degrees-in-sign calculation

use crate::kundli::error::DeriveError;
use crate::kundli::model::Sign;

/// Degrees per zodiac sign.
const DEGREES_PER_SIGN: f64 = 30.0;

/// Normalizes a longitude to the range [0, 360).
///
/// Returns an error if the input is not finite (NaN or infinity).
///
/// # Examples
/// ```
/// # use kundli::kundli::derive::sign::normalize_longitude;
/// assert!((normalize_longitude(0.0).unwrap() - 0.0).abs() < 1e-10);
/// assert!((normalize_longitude(360.0).unwrap() - 0.0).abs() < 1e-10);
/// assert!((normalize_longitude(-30.0).unwrap() - 330.0).abs() < 1e-10);
/// assert!((normalize_longitude(390.0).unwrap() - 30.0).abs() < 1e-10);
/// ```
pub(crate) fn normalize_longitude(longitude: f64) -> Result<f64, DeriveError> {
    if !longitude.is_finite() {
        return Err(DeriveError::InvalidLongitude(longitude));
    }
    let normalized = longitude % 360.0;
    if normalized < 0.0 {
        Ok(normalized + 360.0)
    } else {
        Ok(normalized)
    }
}

/// Derives the zodiac sign from a longitude.
///
/// The longitude is first normalized to [0, 360) before determining the sign.
/// Returns an error if the input is not finite.
///
/// # Examples
/// ```
/// # use kundli::kundli::derive::sign::sign_from_longitude;
/// # use kundli::kundli::model::Sign;
/// assert_eq!(sign_from_longitude(0.0).unwrap(), Sign::Aries);
/// assert_eq!(sign_from_longitude(29.999).unwrap(), Sign::Aries);
/// assert_eq!(sign_from_longitude(30.0).unwrap(), Sign::Taurus);
/// assert_eq!(sign_from_longitude(180.0).unwrap(), Sign::Libra);
/// assert_eq!(sign_from_longitude(359.999).unwrap(), Sign::Pisces);
/// ```
pub(crate) fn sign_from_longitude(longitude: f64) -> Result<Sign, DeriveError> {
    let normalized = normalize_longitude(longitude)?;
    let sign_index = (normalized / DEGREES_PER_SIGN).floor() as usize;
    Ok(sign_from_index(sign_index))
}

/// Returns the degrees progressed within the current sign.
///
/// This is the position within the sign, in the range [0, 30).
/// Returns an error if the input is not finite.
///
/// # Examples
/// ```
/// # use kundli::kundli::derive::sign::degrees_in_sign;
/// assert!((degrees_in_sign(0.0).unwrap() - 0.0).abs() < 1e-10);
/// assert!((degrees_in_sign(15.0).unwrap() - 15.0).abs() < 1e-10);
/// assert!((degrees_in_sign(30.0).unwrap() - 0.0).abs() < 1e-10);
/// assert!((degrees_in_sign(45.5).unwrap() - 15.5).abs() < 1e-10);
/// ```
pub(crate) fn degrees_in_sign(longitude: f64) -> Result<f64, DeriveError> {
    let normalized = normalize_longitude(longitude)?;
    Ok(normalized % DEGREES_PER_SIGN)
}

/// Converts a sign index (0-11) to a Sign enum variant.
///
/// Index 0 = Aries, 1 = Taurus, ..., 11 = Pisces.
fn sign_from_index(index: usize) -> Sign {
    match index {
        0 => Sign::Aries,
        1 => Sign::Taurus,
        2 => Sign::Gemini,
        3 => Sign::Cancer,
        4 => Sign::Leo,
        5 => Sign::Virgo,
        6 => Sign::Libra,
        7 => Sign::Scorpio,
        8 => Sign::Sagittarius,
        9 => Sign::Capricorn,
        10 => Sign::Aquarius,
        11 => Sign::Pisces,
        // SAFETY: index is derived from normalized longitude / 30, so max is 11
        _ => unreachable!("sign index out of range: {index}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_longitude_positive() {
        assert!((normalize_longitude(0.0).unwrap() - 0.0).abs() < 1e-10);
        assert!((normalize_longitude(180.0).unwrap() - 180.0).abs() < 1e-10);
        assert!((normalize_longitude(359.999).unwrap() - 359.999).abs() < 1e-10);
    }

    #[test]
    fn test_normalize_longitude_exact_360() {
        assert!((normalize_longitude(360.0).unwrap() - 0.0).abs() < 1e-10);
        assert!((normalize_longitude(720.0).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_normalize_longitude_overflow() {
        assert!((normalize_longitude(390.0).unwrap() - 30.0).abs() < 1e-10);
        assert!((normalize_longitude(750.0).unwrap() - 30.0).abs() < 1e-10);
    }

    #[test]
    fn test_normalize_longitude_negative() {
        assert!((normalize_longitude(-30.0).unwrap() - 330.0).abs() < 1e-10);
        assert!((normalize_longitude(-360.0).unwrap() - 0.0).abs() < 1e-10);
        assert!((normalize_longitude(-390.0).unwrap() - 330.0).abs() < 1e-10);
    }

    #[test]
    fn test_normalize_longitude_invalid() {
        assert!(normalize_longitude(f64::NAN).is_err());
        assert!(normalize_longitude(f64::INFINITY).is_err());
        assert!(normalize_longitude(f64::NEG_INFINITY).is_err());
    }

    #[test]
    fn test_sign_from_longitude_boundaries() {
        assert_eq!(sign_from_longitude(0.0).unwrap(), Sign::Aries);
        assert_eq!(sign_from_longitude(29.999).unwrap(), Sign::Aries);
        assert_eq!(sign_from_longitude(30.0).unwrap(), Sign::Taurus);
        assert_eq!(sign_from_longitude(59.999).unwrap(), Sign::Taurus);
        assert_eq!(sign_from_longitude(60.0).unwrap(), Sign::Gemini);
        assert_eq!(sign_from_longitude(90.0).unwrap(), Sign::Cancer);
        assert_eq!(sign_from_longitude(120.0).unwrap(), Sign::Leo);
        assert_eq!(sign_from_longitude(150.0).unwrap(), Sign::Virgo);
        assert_eq!(sign_from_longitude(180.0).unwrap(), Sign::Libra);
        assert_eq!(sign_from_longitude(210.0).unwrap(), Sign::Scorpio);
        assert_eq!(sign_from_longitude(240.0).unwrap(), Sign::Sagittarius);
        assert_eq!(sign_from_longitude(270.0).unwrap(), Sign::Capricorn);
        assert_eq!(sign_from_longitude(300.0).unwrap(), Sign::Aquarius);
        assert_eq!(sign_from_longitude(330.0).unwrap(), Sign::Pisces);
        assert_eq!(sign_from_longitude(359.999).unwrap(), Sign::Pisces);
    }

    #[test]
    fn test_sign_from_longitude_negative() {
        assert_eq!(sign_from_longitude(-0.001).unwrap(), Sign::Pisces);
        assert_eq!(sign_from_longitude(-30.0).unwrap(), Sign::Pisces);
        assert_eq!(sign_from_longitude(-31.0).unwrap(), Sign::Aquarius);
    }

    #[test]
    fn test_sign_from_longitude_invalid() {
        assert!(sign_from_longitude(f64::NAN).is_err());
        assert!(sign_from_longitude(f64::INFINITY).is_err());
    }

    #[test]
    fn test_degrees_in_sign_basic() {
        assert!((degrees_in_sign(0.0).unwrap() - 0.0).abs() < 1e-10);
        assert!((degrees_in_sign(15.0).unwrap() - 15.0).abs() < 1e-10);
        assert!((degrees_in_sign(29.999).unwrap() - 29.999).abs() < 1e-10);
    }

    #[test]
    fn test_degrees_in_sign_across_boundary() {
        assert!((degrees_in_sign(30.0).unwrap() - 0.0).abs() < 1e-10);
        assert!((degrees_in_sign(45.5).unwrap() - 15.5).abs() < 1e-10);
        assert!((degrees_in_sign(90.0).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_degrees_in_sign_negative() {
        assert!((degrees_in_sign(-0.001).unwrap() - 29.999).abs() < 1e-6);
        assert!((degrees_in_sign(-15.0).unwrap() - 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_degrees_in_sign_invalid() {
        assert!(degrees_in_sign(f64::NAN).is_err());
        assert!(degrees_in_sign(f64::INFINITY).is_err());
    }
}
