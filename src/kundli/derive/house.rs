use crate::kundli::astro::HouseSystem;
use crate::kundli::derive::sign::normalize_longitude;
use crate::kundli::error::DeriveError;
use crate::kundli::model::HouseNumber;

const DEGREES_PER_SIGN: f64 = 30.0;
const NUM_HOUSES: usize = 12;

/// Derives the house number for a planet using the specified house system.
///
/// For WholeSign: uses ascendant sign anchoring.
/// For other systems: uses house cusps for placement.
///
/// # Arguments
/// * `planet_longitude` - The ecliptic longitude of the planet in degrees.
/// * `ascendant_longitude` - The ascendant longitude in degrees.
/// * `house_cusps` - House cusp longitudes (required for non-WholeSign systems).
/// * `house_system` - The house system to use.
///
/// # Errors
/// Returns `DeriveError::InvalidLongitude` if planet_longitude is not finite.
/// Returns `DeriveError::InvalidHouseCusps` if cusps are required but invalid.
pub(crate) fn derive_house(
    planet_longitude: f64,
    ascendant_longitude: f64,
    house_cusps: &[f64],
    house_system: HouseSystem,
) -> Result<HouseNumber, DeriveError> {
    let planet_longitude = normalize_longitude(planet_longitude)?;
    let ascendant_longitude = normalize_longitude(ascendant_longitude)?;

    match house_system {
        HouseSystem::WholeSign => derive_house_whole_sign(planet_longitude, ascendant_longitude),
        _ => derive_house_from_cusps(planet_longitude, house_cusps),
    }
}

/// Derives house placement using WholeSign system (ascendant sign anchoring).
///
/// In WholeSign, the ascendant's sign becomes the 1st house.
/// Each subsequent sign becomes the next house.
fn derive_house_whole_sign(
    planet_longitude: f64,
    ascendant_longitude: f64,
) -> Result<HouseNumber, DeriveError> {
    let planet_sign = longitude_to_sign_index(planet_longitude)?;
    let ascendant_sign = longitude_to_sign_index(ascendant_longitude)?;

    // House number = (planet_sign - ascendant_sign + 12) % 12 + 1
    let house_number =
        ((planet_sign as i32 - ascendant_sign as i32 + NUM_HOUSES as i32) % NUM_HOUSES as i32) + 1;

    // Safety: house_number is always in range 1-12 due to modulo
    debug_assert!((1..=12).contains(&house_number));
    HouseNumber::new(house_number as u8).ok_or(DeriveError::InvalidHouseNumber(house_number as u8))
}

/// Derives house placement using house cusps.
///
/// Finds which house contains the given planet longitude.
/// Handles wrap-around at 360 degrees correctly.
fn derive_house_from_cusps(
    planet_longitude: f64,
    house_cusps: &[f64],
) -> Result<HouseNumber, DeriveError> {
    if house_cusps.len() != NUM_HOUSES {
        return Err(DeriveError::InvalidHouseCusps(house_cusps.len()));
    }

    // Validate all cusps are finite
    for &cusp in house_cusps {
        if !cusp.is_finite() {
            return Err(DeriveError::InvalidLongitude(cusp));
        }
    }

    // Normalize planet longitude to [0, 360)
    let planet_lon = normalize_longitude(planet_longitude)?;

    // Find the house containing the planet
    // A planet is in house i if its longitude is >= cusp[i] and < cusp[i+1]
    // We need to handle wrap-around (when cusp[11] > cusp[0])
    for i in 0..NUM_HOUSES {
        let cusp_start = normalize_longitude(house_cusps[i])?;
        let cusp_end = normalize_longitude(house_cusps[(i + 1) % NUM_HOUSES])?;

        if is_in_house(planet_lon, cusp_start, cusp_end) {
            return HouseNumber::new((i + 1) as u8)
                .ok_or(DeriveError::InvalidHouseNumber((i + 1) as u8));
        }
    }

    Err(DeriveError::InvalidHouseCusps(house_cusps.len()))
}

/// Converts a longitude to a sign index (0-11).
fn longitude_to_sign_index(longitude: f64) -> Result<usize, DeriveError> {
    let normalized = normalize_longitude(longitude)?;
    Ok((normalized / DEGREES_PER_SIGN).floor() as usize % NUM_HOUSES)
}

/// Checks if a longitude falls within a house, handling wrap-around.
///
/// A planet is in a house if its longitude >= cusp_start and < cusp_end.
/// When cusp_end < cusp_start (wrap-around case), the house spans across 0 degrees.
fn is_in_house(planet_lon: f64, cusp_start: f64, cusp_end: f64) -> bool {
    if cusp_start <= cusp_end {
        // Normal case: house does not wrap around
        planet_lon >= cusp_start && planet_lon < cusp_end
    } else {
        // Wrap-around case: house spans from cusp_start to 360, then 0 to cusp_end
        planet_lon >= cusp_start || planet_lon < cusp_end
    }
}

const _: () = {
    let _ = DEGREES_PER_SIGN;
    let _ = NUM_HOUSES;
    let _ = derive_house as fn(f64, f64, &[f64], HouseSystem) -> Result<HouseNumber, DeriveError>;
    let _ = derive_house_whole_sign as fn(f64, f64) -> Result<HouseNumber, DeriveError>;
    let _ = derive_house_from_cusps as fn(f64, &[f64]) -> Result<HouseNumber, DeriveError>;
    let _ = longitude_to_sign_index as fn(f64) -> Result<usize, DeriveError>;
    let _ = is_in_house as fn(f64, f64, f64) -> bool;
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_constructor_rejects_invalid_house_number() {
        assert_eq!(HouseNumber::new(0), None);
        assert_eq!(HouseNumber::new(13), None);
    }

    fn house(number: u8) -> HouseNumber {
        HouseNumber::new(number).unwrap()
    }

    #[test]
    fn whole_sign_same_sign_as_ascendant_is_house_1() {
        // Ascendant at 15 degrees Aries (sign 0), planet at 20 degrees Aries
        let house_number = derive_house(20.0, 15.0, &[], HouseSystem::WholeSign);
        assert_eq!(house_number.unwrap(), house(1));
    }

    #[test]
    fn whole_sign_next_sign_is_house_2() {
        // Ascendant at 15 degrees Aries (sign 0), planet at 10 degrees Taurus (sign 1)
        let house_number = derive_house(40.0, 15.0, &[], HouseSystem::WholeSign);
        assert_eq!(house_number.unwrap(), house(2));
    }

    #[test]
    fn whole_sign_previous_sign_is_house_12() {
        // Ascendant at 15 degrees Aries (sign 0), planet at 10 degrees Pisces (sign 11)
        let house_number = derive_house(350.0, 15.0, &[], HouseSystem::WholeSign);
        assert_eq!(house_number.unwrap(), house(12));
    }

    #[test]
    fn whole_sign_multiple_houses_away() {
        // Ascendant at 5 degrees Aries (sign 0), planet at 95 degrees Cancer (sign 3)
        let house_number = derive_house(95.0, 5.0, &[], HouseSystem::WholeSign);
        assert_eq!(house_number.unwrap(), house(4));
    }

    #[test]
    fn whole_sign_at_sign_boundary() {
        // Ascendant at 0 degrees Aries, planet at exactly 30 degrees (0 Taurus)
        let house_number = derive_house(30.0, 0.0, &[], HouseSystem::WholeSign);
        assert_eq!(house_number.unwrap(), house(2));
    }

    #[test]
    fn whole_sign_with_negative_longitude() {
        // Ascendant at 15 degrees, planet at -10 degrees (equivalent to 350 degrees)
        let house_number = derive_house(-10.0, 15.0, &[], HouseSystem::WholeSign);
        assert_eq!(house_number.unwrap(), house(12));
    }

    #[test]
    fn cusp_based_planet_in_first_house() {
        // Simple case: cusps start at 0, 30, 60, etc. (Equal house)
        let cusps: Vec<f64> = (0..12).map(|i| (i * 30) as f64).collect();
        let house_number = derive_house(15.0, 0.0, &cusps, HouseSystem::Equal);
        assert_eq!(house_number.unwrap(), house(1));
    }

    #[test]
    fn cusp_based_planet_in_middle_house() {
        let cusps: Vec<f64> = (0..12).map(|i| (i * 30) as f64).collect();
        let house_number = derive_house(105.0, 0.0, &cusps, HouseSystem::Equal);
        assert_eq!(house_number.unwrap(), house(4));
    }

    #[test]
    fn cusp_based_planet_at_cusp_boundary() {
        // Planet exactly at a cusp belongs to that house
        let cusps: Vec<f64> = (0..12).map(|i| (i * 30) as f64).collect();
        let house_number = derive_house(60.0, 0.0, &cusps, HouseSystem::Equal);
        assert_eq!(house_number.unwrap(), house(3));
    }

    #[test]
    fn cusp_based_wrap_around_last_house() {
        // House 12 spans from 330 to 360/0 degrees
        let cusps: Vec<f64> = (0..12).map(|i| (i * 30) as f64).collect();
        let house_number = derive_house(345.0, 0.0, &cusps, HouseSystem::Equal);
        assert_eq!(house_number.unwrap(), house(12));
    }

    #[test]
    fn cusp_based_wrap_around_from_360_to_first_cusp() {
        // Ascendant at 300 degrees, cusps offset accordingly.
        let cusps: Vec<f64> = vec![
            300.0, 330.0, 0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 210.0, 240.0, 270.0,
        ];
        // Planet at 350 degrees should be in house 2 (330-0)
        let house_number = derive_house(350.0, 300.0, &cusps, HouseSystem::Placidus);
        assert_eq!(house_number.unwrap(), house(2));
    }

    #[test]
    fn non_whole_sign_validates_ascendant_longitude() {
        let cusps: Vec<f64> = (0..12).map(|i| (i * 30) as f64).collect();
        let house_number = derive_house(15.0, f64::NAN, &cusps, HouseSystem::Equal);
        assert!(
            matches!(house_number, Err(DeriveError::InvalidLongitude(value)) if value.is_nan())
        );
    }

    #[test]
    fn cusp_based_planet_in_wrap_around_house() {
        // House 2 spans 330 to 0 (wraps around 360)
        let cusps: Vec<f64> = vec![
            300.0, 330.0, 0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 210.0, 240.0, 270.0,
        ];
        // Planet at 5 degrees should be in house 3 (cusp at 0 to cusp at 30)
        let house_number = derive_house(5.0, 300.0, &cusps, HouseSystem::Placidus);
        assert_eq!(house_number.unwrap(), house(3));
    }

    #[test]
    fn invalid_longitude_returns_error() {
        let house_number = derive_house(f64::NAN, 15.0, &[], HouseSystem::WholeSign);
        assert!(matches!(
            house_number,
            Err(DeriveError::InvalidLongitude(_))
        ));
    }

    #[test]
    fn infinite_longitude_returns_error() {
        let house_number = derive_house(f64::INFINITY, 15.0, &[], HouseSystem::WholeSign);
        assert!(matches!(
            house_number,
            Err(DeriveError::InvalidLongitude(_))
        ));
    }

    #[test]
    fn wrong_number_of_cusps_returns_error() {
        let cusps = vec![0.0, 30.0, 60.0]; // Only 3 cusps
        let house_number = derive_house(45.0, 0.0, &cusps, HouseSystem::Equal);
        assert!(matches!(
            house_number,
            Err(DeriveError::InvalidHouseCusps(3))
        ));
    }

    #[test]
    fn empty_cusps_returns_error() {
        let house_number = derive_house(45.0, 0.0, &[], HouseSystem::Equal);
        assert!(matches!(
            house_number,
            Err(DeriveError::InvalidHouseCusps(0))
        ));
    }

    #[test]
    fn cusp_with_nan_returns_error() {
        let cusps = vec![
            0.0,
            30.0,
            60.0,
            90.0,
            120.0,
            150.0,
            f64::NAN,
            210.0,
            240.0,
            270.0,
            300.0,
            330.0,
        ];
        let house_number = derive_house(45.0, 0.0, &cusps, HouseSystem::Equal);
        assert!(matches!(
            house_number,
            Err(DeriveError::InvalidLongitude(_))
        ));
    }

    #[test]
    fn is_in_house_normal_case() {
        assert!(is_in_house(45.0, 30.0, 60.0));
        assert!(!is_in_house(25.0, 30.0, 60.0));
        assert!(!is_in_house(65.0, 30.0, 60.0));
    }

    #[test]
    fn is_in_house_wrap_around_case() {
        // House spans 350 to 10 (wraps around 0)
        assert!(is_in_house(355.0, 350.0, 10.0));
        assert!(is_in_house(5.0, 350.0, 10.0));
        assert!(!is_in_house(20.0, 350.0, 10.0));
        assert!(!is_in_house(340.0, 350.0, 10.0));
    }

    #[test]
    fn whole_sign_with_large_longitude() {
        // Planet at 725 degrees = 5 degrees Aries after normalization.
        let house_number = derive_house(725.0, 15.0, &[], HouseSystem::WholeSign);
        assert_eq!(house_number.unwrap(), house(1));
    }

    #[test]
    fn all_whole_sign_houses() {
        // Ascendant at 0 degrees Aries
        // Test all 12 houses
        for house_num in 1..=12 {
            let planet_sign = house_num - 1; // 0-indexed sign
            let planet_lon = (planet_sign as f64) * 30.0 + 15.0; // Middle of each sign
            let house_number = derive_house(planet_lon, 0.0, &[], HouseSystem::WholeSign);
            assert_eq!(
                house_number.unwrap(),
                house(house_num),
                "Failed for house {}",
                house_num
            );
        }
    }
}
