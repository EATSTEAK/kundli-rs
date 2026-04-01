use crate::kundli::astro::AstroBody;
use crate::kundli::derive::nakshatra::{
    nakshatra_placement_from_longitude, nakshatra_progress_ratio,
};
use crate::kundli::derive::sign::{degrees_in_sign, normalize_longitude, sign_from_longitude};
use crate::kundli::error::DeriveError;
use crate::kundli::model::{NakshatraPlacement, Sign};

use super::{ReferenceContext, ResolvedReference};

const NAVAMSAS_PER_SIGN: f64 = 9.0;

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
            house_cusps: input.projected.house_cusps.clone(),
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
        let longitude = normalize_longitude(longitude)?;
        normalize_longitude(longitude * NAVAMSAS_PER_SIGN)
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
            house_cusps: input.projected.house_cusps.clone(),
        })
    }
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
