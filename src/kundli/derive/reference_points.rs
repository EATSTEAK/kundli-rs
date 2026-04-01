use swiss_eph::safe::{self, GeoPos, HouseSystem as SwissHouseSystem, Planet, RiseTransFlags};

use crate::kundli::derive::pipeline::ProjectedBase;
use crate::kundli::derive::sign::normalize_longitude;
use crate::kundli::error::DeriveError;

const DAY_RULERS: [Planet; 7] = [
    Planet::Sun,
    Planet::Moon,
    Planet::Mars,
    Planet::Mercury,
    Planet::Jupiter,
    Planet::Venus,
    Planet::Saturn,
];

const NIGHT_RULERS: [Planet; 7] = [
    Planet::Sun,
    Planet::Venus,
    Planet::Mercury,
    Planet::Moon,
    Planet::Saturn,
    Planet::Jupiter,
    Planet::Mars,
];

pub(crate) fn gulika_longitude(projected: &ProjectedBase) -> Result<f64, DeriveError> {
    let geopos = GeoPos {
        longitude: projected.longitude,
        latitude: projected.latitude,
        altitude: 0.0,
    };
    let sunrise = safe::rise_trans(
        projected.jd_ut,
        Planet::Sun,
        None,
        geopos,
        RiseTransFlags::new().with_rise(),
    )
    .map_err(|_| DeriveError::SpecialPointCalculationFailed("gulika_sunrise"))?;
    let sunset = safe::rise_trans(
        projected.jd_ut,
        Planet::Sun,
        None,
        geopos,
        RiseTransFlags::new().with_set(),
    )
    .map_err(|_| DeriveError::SpecialPointCalculationFailed("gulika_sunset"))?;

    let is_day_birth = projected.jd_ut >= sunrise && projected.jd_ut < sunset;
    let day_index = weekday_index(projected.jd_ut);
    let segment = if is_day_birth {
        index_of(DAY_RULERS, Planet::Saturn, day_index)
    } else {
        index_of(NIGHT_RULERS, Planet::Saturn, day_index)
    };

    let span = if is_day_birth {
        sunset - sunrise
    } else {
        let next_sunrise = if projected.jd_ut >= sunset {
            safe::rise_trans(
                projected.jd_ut + 1.0,
                Planet::Sun,
                None,
                geopos,
                RiseTransFlags::new().with_rise(),
            )
            .map_err(|_| DeriveError::SpecialPointCalculationFailed("gulika_next_sunrise"))?
        } else {
            sunrise
        };
        next_sunrise - sunset
    };

    let start = if is_day_birth { sunrise } else { sunset };
    let gulika_jd = start + span * (segment as f64 / 8.0);
    let houses = safe::houses(
        gulika_jd,
        projected.latitude,
        projected.longitude,
        SwissHouseSystem::WholeSign,
    )
    .map_err(|_| DeriveError::SpecialPointCalculationFailed("gulika_ascendant"))?;

    normalize_longitude(houses.ascendant)
}

fn weekday_index(jd_ut: f64) -> usize {
    ((jd_ut.floor() as i64 + 1).rem_euclid(7)) as usize
}

fn index_of(sequence: [Planet; 7], target: Planet, offset: usize) -> usize {
    let rotated = std::array::from_fn::<_, 7, _>(|index| sequence[(offset + index) % 7]);
    rotated
        .iter()
        .position(|planet| *planet == target)
        .unwrap_or(0)
}
