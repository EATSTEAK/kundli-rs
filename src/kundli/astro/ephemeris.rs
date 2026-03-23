use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
    sync::{Mutex, MutexGuard, OnceLock},
};

use swiss_eph::safe::{self, CalcFlags, HouseCusps, Planet, Position, SiderealMode};

use crate::kundli::astro::{
    AstroBody, AstroError, AstroRequest, Ayanamsha, HouseSystem, NodeType, SwissEphConfig,
    ZodiacType,
};

static SWISS_EPH_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
const HOUSE_ERROR_BUFFER_LEN: usize = 256;

pub(crate) struct Ephemeris;

pub(crate) struct EphemerisResult {
    pub(crate) bodies: Vec<Position>,
    pub(crate) houses: HouseCusps,
    pub(crate) sidereal_time: f64,
    pub(crate) ayanamsha_value: Option<f64>,
}

impl Ephemeris {
    pub(crate) fn calculate(
        request: &AstroRequest,
        config: &SwissEphConfig,
    ) -> Result<EphemerisResult, AstroError> {
        let _guard = lock_swiss_eph()?;

        configure_ephemeris(config, request)?;

        let flags = calc_flags(request.zodiac);
        let mut bodies = Vec::with_capacity(request.bodies.len());

        for body in request.bodies.iter().copied() {
            match body {
                AstroBody::Ketu => {
                    let node = node_body(request.node_type);
                    let rahu = safe::calc_ut(request.jd_ut, node.to_int(), flags.raw())?;
                    bodies.push(Position {
                        longitude: safe::normalize_degrees(rahu.longitude + 180.0),
                        latitude: -rahu.latitude,
                        distance: rahu.distance,
                        longitude_speed: rahu.longitude_speed,
                        latitude_speed: -rahu.latitude_speed,
                        distance_speed: rahu.distance_speed,
                    });
                }
                _ => {
                    let planet = to_planet(body, request.node_type);
                    bodies.push(safe::calc_ut(request.jd_ut, planet.to_int(), flags.raw())?);
                }
            }
        }

        let houses = if matches!(request.zodiac, ZodiacType::Sidereal) {
            houses_ex(
                request.jd_ut,
                request.latitude,
                request.longitude,
                request.house_system,
                flags.raw(),
            )?
        } else {
            safe::houses(
                request.jd_ut,
                request.latitude,
                request.longitude,
                to_house_system(request.house_system),
            )?
        };

        let ayanamsha_value = if matches!(request.zodiac, ZodiacType::Sidereal) {
            Some(safe::get_ayanamsa(request.jd_ut))
        } else {
            None
        };

        Ok(EphemerisResult {
            bodies,
            houses,
            sidereal_time: safe::sidereal_time(request.jd_ut),
            ayanamsha_value,
        })
    }
}

fn lock_swiss_eph() -> Result<MutexGuard<'static, ()>, AstroError> {
    SWISS_EPH_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|_| {
            AstroError::CalculationFailed("Swiss Ephemeris global state lock poisoned".to_string())
        })
}

fn configure_ephemeris(config: &SwissEphConfig, request: &AstroRequest) -> Result<(), AstroError> {
    set_ephemeris_path(config.ephemeris_path())?;

    if matches!(request.zodiac, ZodiacType::Sidereal) {
        safe::set_sidereal_mode(to_sidereal_mode(request.ayanamsha));
    }

    Ok(())
}

fn set_ephemeris_path(path: Option<&str>) -> Result<(), AstroError> {
    match path {
        Some(path) => {
            let _ = CString::new(path).map_err(|_| AstroError::InvalidEphemerisPath)?;
            safe::set_ephe_path(path);
        }
        None => safe::set_ephe_path(""),
    }

    Ok(())
}

fn calc_flags(zodiac: ZodiacType) -> CalcFlags {
    let mut flags = CalcFlags::new().with_speed().with_swiss_ephemeris();

    if matches!(zodiac, ZodiacType::Sidereal) {
        flags = flags.with_sidereal();
    }

    flags
}

fn to_sidereal_mode(ayanamsha: Ayanamsha) -> SiderealMode {
    match ayanamsha {
        Ayanamsha::Lahiri => SiderealMode::Lahiri,
        Ayanamsha::Raman => SiderealMode::Raman,
        Ayanamsha::Krishnamurti => SiderealMode::Krishnamurti,
    }
}

fn to_house_system(system: HouseSystem) -> safe::HouseSystem {
    match system {
        HouseSystem::Placidus => safe::HouseSystem::Placidus,
        HouseSystem::Koch => safe::HouseSystem::Koch,
        HouseSystem::Equal => safe::HouseSystem::Equal,
        HouseSystem::WholeSign => safe::HouseSystem::WholeSign,
    }
}

fn node_body(node_type: NodeType) -> Planet {
    match node_type {
        NodeType::Mean => Planet::MeanNode,
        NodeType::True => Planet::TrueNode,
    }
}

fn to_planet(body: AstroBody, node_type: NodeType) -> Planet {
    match body {
        AstroBody::Sun => Planet::Sun,
        AstroBody::Moon => Planet::Moon,
        AstroBody::Mars => Planet::Mars,
        AstroBody::Mercury => Planet::Mercury,
        AstroBody::Jupiter => Planet::Jupiter,
        AstroBody::Venus => Planet::Venus,
        AstroBody::Saturn => Planet::Saturn,
        AstroBody::Rahu => node_body(node_type),
        AstroBody::Ketu => unreachable!("ketu is derived from rahu"),
    }
}

fn houses_ex(
    jd_ut: f64,
    latitude: f64,
    longitude: f64,
    system: HouseSystem,
    flags: i32,
) -> Result<HouseCusps, AstroError> {
    let mut cusps = [0.0f64; 13];
    let mut ascmc = [0.0f64; 10];
    let mut cusp_speed = [0.0f64; 13];
    let mut ascmc_speed = [0.0f64; 10];
    let mut serr = [0 as c_char; HOUSE_ERROR_BUFFER_LEN];

    let ret = unsafe {
        swiss_eph::swe_houses_ex2(
            jd_ut,
            flags,
            latitude,
            longitude,
            to_house_system(system) as i32,
            cusps.as_mut_ptr(),
            ascmc.as_mut_ptr(),
            cusp_speed.as_mut_ptr(),
            ascmc_speed.as_mut_ptr(),
            serr.as_mut_ptr(),
        )
    };

    if ret < 0 {
        let message = unsafe { CStr::from_ptr(serr.as_ptr()) }
            .to_string_lossy()
            .trim()
            .to_owned();

        return Err(AstroError::CalculationFailed(if message.is_empty() {
            "sidereal house calculation failed".to_string()
        } else {
            message
        }));
    }

    let mut result_cusps = [0.0f64; 12];
    result_cusps.copy_from_slice(&cusps[1..13]);

    Ok(HouseCusps {
        cusps: result_cusps,
        ascendant: ascmc[0],
        mc: ascmc[1],
        armc: ascmc[2],
        vertex: ascmc[3],
        equatorial_ascendant: ascmc[4],
        co_ascendant_koch: ascmc[5],
        co_ascendant_munkasey: ascmc[6],
        polar_ascendant: ascmc[7],
    })
}
