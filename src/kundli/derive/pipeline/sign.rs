use crate::kundli::astro::AstroBody;
use crate::kundli::derive::nakshatra::{
    nakshatra_placement_from_longitude, nakshatra_progress_ratio,
};
use crate::kundli::derive::sign::{degrees_in_sign, normalize_longitude, sign_from_longitude};
use crate::kundli::error::DeriveError;
use crate::kundli::model::{NakshatraPlacement, Sign};

use super::{ReferenceContext, ResolvedReference};

const DEGREES_PER_SIGN: f64 = 30.0;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SignPlacement {
    pub body: Option<AstroBody>,
    pub longitude: f64,
    pub sign: Sign,
    pub degrees_in_sign: f64,
    pub nakshatra: NakshatraPlacement,
    pub nakshatra_progress_ratio: f64,
    pub is_retrograde: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SignContext {
    pub reference: ResolvedReference,
    pub ascendant: SignPlacement,
    pub bodies: Vec<SignPlacement>,
    pub house_cusps: Vec<f64>,
}

pub(crate) trait SignTransformOp<Input> {
    type Output;

    fn apply(&self, input: &Input) -> Result<Self::Output, DeriveError>;
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct IdentitySignTransform;

impl SignTransformOp<ReferenceContext> for IdentitySignTransform {
    type Output = SignContext;

    fn apply(&self, input: &ReferenceContext) -> Result<Self::Output, DeriveError> {
        Ok(SignContext {
            reference: input.reference.clone(),
            ascendant: build_sign_placement(None, input.projected.ascendant_longitude, false)?,
            bodies: input
                .projected
                .bodies
                .iter()
                .map(|body| {
                    build_sign_placement(Some(body.body), body.longitude, body.is_retrograde)
                })
                .collect::<Result<Vec<_>, _>>()?,
            house_cusps: input.projected.house_cusps.to_vec(),
        })
    }
}

pub(crate) trait VargaRule {
    fn map(longitude: f64) -> Result<f64, DeriveError>;
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct D9Rule;

impl VargaRule for D9Rule {
    fn map(longitude: f64) -> Result<f64, DeriveError> {
        map_divisional_longitude(longitude, 9)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct VargaTransform<R> {
    _marker: std::marker::PhantomData<R>,
}

impl<R> VargaTransform<R> {
    pub(crate) fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<R> SignTransformOp<ReferenceContext> for VargaTransform<R>
where
    R: VargaRule,
{
    type Output = SignContext;

    fn apply(&self, input: &ReferenceContext) -> Result<Self::Output, DeriveError> {
        Ok(SignContext {
            reference: ResolvedReference {
                point: input.reference.point,
                longitude: R::map(input.reference.longitude)?,
            },
            ascendant: build_sign_placement(
                None,
                R::map(input.projected.ascendant_longitude)?,
                false,
            )?,
            bodies: input
                .projected
                .bodies
                .iter()
                .map(|body| {
                    build_sign_placement(
                        Some(body.body),
                        R::map(body.longitude)?,
                        body.is_retrograde,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
            house_cusps: input.projected.house_cusps.to_vec(),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DivisionalSignTransform {
    division: u8,
}

impl DivisionalSignTransform {
    pub(crate) fn new(division: u8) -> Result<Self, DeriveError> {
        if division == 0 {
            return Err(DeriveError::InvalidDivision(division));
        }

        Ok(Self { division })
    }
}

impl SignTransformOp<ReferenceContext> for DivisionalSignTransform {
    type Output = SignContext;

    fn apply(&self, input: &ReferenceContext) -> Result<Self::Output, DeriveError> {
        let division = self.division;

        Ok(SignContext {
            reference: ResolvedReference {
                point: input.reference.point,
                longitude: map_divisional_longitude(input.reference.longitude, division)?,
            },
            ascendant: build_sign_placement(
                None,
                map_divisional_longitude(input.projected.ascendant_longitude, division)?,
                false,
            )?,
            bodies: input
                .projected
                .bodies
                .iter()
                .map(|body| {
                    build_sign_placement(
                        Some(body.body),
                        map_divisional_longitude(body.longitude, division)?,
                        body.is_retrograde,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
            house_cusps: input.projected.house_cusps.to_vec(),
        })
    }
}

fn map_divisional_longitude(longitude: f64, division: u8) -> Result<f64, DeriveError> {
    if division == 0 {
        return Err(DeriveError::InvalidDivision(division));
    }

    let longitude = normalize_longitude(longitude)?;
    let sign_index = (longitude / DEGREES_PER_SIGN).floor() as usize;
    let degrees_in_sign = longitude % DEGREES_PER_SIGN;
    let segment_size = DEGREES_PER_SIGN / f64::from(division);
    let segment_index = (degrees_in_sign / segment_size).floor() as usize;
    let target_sign = divisional_target_sign(sign_index, degrees_in_sign, division, segment_index)?;
    let degrees_in_segment = degrees_in_sign % segment_size;
    normalize_longitude(
        target_sign as f64 * DEGREES_PER_SIGN + degrees_in_segment * f64::from(division),
    )
}

fn divisional_target_sign(
    sign_index: usize,
    degrees_in_sign: f64,
    division: u8,
    segment_index: usize,
) -> Result<usize, DeriveError> {
    let result = match division {
        2 => {
            let odd = is_odd_sign(sign_index);
            let first_half = degrees_in_sign < 15.0;
            match (odd, first_half) {
                (true, true) => 4,
                (true, false) => 3,
                (false, true) => 3,
                (false, false) => 4,
            }
        }
        3 => (sign_index + segment_index * 4) % 12,
        4 => (sign_index + segment_index * 3) % 12,
        7 => {
            let start = if is_odd_sign(sign_index) {
                sign_index
            } else {
                (sign_index + 6) % 12
            };
            (start + segment_index) % 12
        }
        9 => {
            let start = match sign_modality(sign_index) {
                Modality::Movable => sign_index,
                Modality::Fixed => (sign_index + 8) % 12,
                Modality::Dual => (sign_index + 4) % 12,
            };
            (start + segment_index) % 12
        }
        10 => {
            let start = if is_odd_sign(sign_index) {
                sign_index
            } else {
                (sign_index + 8) % 12
            };
            (start + segment_index) % 12
        }
        12 => (sign_index + segment_index) % 12,
        16 => {
            let start = match sign_modality(sign_index) {
                Modality::Movable => 0,
                Modality::Fixed => 4,
                Modality::Dual => 8,
            };
            (start + segment_index) % 12
        }
        20 => {
            let start = match sign_modality(sign_index) {
                Modality::Movable => 0,
                Modality::Fixed => 8,
                Modality::Dual => 4,
            };
            (start + segment_index) % 12
        }
        24 => {
            let start = if is_odd_sign(sign_index) { 4 } else { 3 };
            (start + segment_index) % 12
        }
        27 => {
            let start = if is_odd_sign(sign_index) { 0 } else { 6 };
            (start + segment_index) % 12
        }
        30 => trimsamsa_target_sign(sign_index, degrees_in_sign),
        40 => {
            let start = if is_odd_sign(sign_index) { 0 } else { 6 };
            (start + segment_index) % 12
        }
        45 => {
            let start = match sign_modality(sign_index) {
                Modality::Movable => 0,
                Modality::Fixed => 4,
                Modality::Dual => 8,
            };
            (start + segment_index) % 12
        }
        60 => {
            let start = if is_odd_sign(sign_index) { 0 } else { 6 };
            (start + segment_index) % 12
        }
        _ => (sign_index + segment_index) % 12,
    };

    Ok(result)
}

fn trimsamsa_target_sign(sign_index: usize, degrees_in_sign: f64) -> usize {
    let odd = is_odd_sign(sign_index);
    if odd {
        if degrees_in_sign < 5.0 {
            0
        } else if degrees_in_sign < 10.0 {
            10
        } else if degrees_in_sign < 18.0 {
            8
        } else if degrees_in_sign < 25.0 {
            2
        } else {
            6
        }
    } else if degrees_in_sign < 5.0 {
        1
    } else if degrees_in_sign < 12.0 {
        5
    } else if degrees_in_sign < 20.0 {
        11
    } else if degrees_in_sign < 25.0 {
        9
    } else {
        7
    }
}

#[derive(Debug, Clone, Copy)]
enum Modality {
    Movable,
    Fixed,
    Dual,
}

fn sign_modality(sign_index: usize) -> Modality {
    match sign_index % 3 {
        0 => Modality::Movable,
        1 => Modality::Fixed,
        _ => Modality::Dual,
    }
}

fn is_odd_sign(sign_index: usize) -> bool {
    sign_index.is_multiple_of(2)
}

fn build_sign_placement(
    body: Option<AstroBody>,
    longitude: f64,
    is_retrograde: bool,
) -> Result<SignPlacement, DeriveError> {
    let longitude = normalize_longitude(longitude)?;

    Ok(SignPlacement {
        body,
        longitude,
        sign: sign_from_longitude(longitude)?,
        degrees_in_sign: degrees_in_sign(longitude)?,
        nakshatra: nakshatra_placement_from_longitude(longitude)?,
        nakshatra_progress_ratio: nakshatra_progress_ratio(longitude)?,
        is_retrograde,
    })
}
