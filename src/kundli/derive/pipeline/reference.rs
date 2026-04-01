use crate::kundli::config::{ReferenceKey, SpecialReference};
use crate::kundli::derive::reference_points::gulika_longitude;
use crate::kundli::error::DeriveError;

use super::ProjectedBase;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ReferencePoint {
    Lagna,
    Planet(crate::kundli::astro::AstroBody),
    Special(SpecialReference),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ResolvedReference {
    pub point: ReferencePoint,
    pub longitude: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ReferenceContext {
    pub projected: ProjectedBase,
    pub reference: ResolvedReference,
}

pub(crate) trait ReferenceOp<Input> {
    type Output;

    fn apply(&self, input: &Input) -> Result<Self::Output, DeriveError>;
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ReferenceTransform {
    reference: ReferenceKey,
}

impl ReferenceTransform {
    pub(crate) const fn new(reference: ReferenceKey) -> Self {
        Self { reference }
    }
}

impl ReferenceOp<ProjectedBase> for ReferenceTransform {
    type Output = ReferenceContext;

    fn apply(&self, input: &ProjectedBase) -> Result<Self::Output, DeriveError> {
        let resolved = match self.reference {
            ReferenceKey::Lagna => ResolvedReference {
                point: ReferencePoint::Lagna,
                longitude: input.ascendant_longitude,
            },
            ReferenceKey::Moon => {
                let moon = input
                    .bodies
                    .iter()
                    .find(|body| body.body == crate::kundli::astro::AstroBody::Moon)
                    .ok_or(DeriveError::MissingMoon)?;
                ResolvedReference {
                    point: ReferencePoint::Planet(crate::kundli::astro::AstroBody::Moon),
                    longitude: moon.longitude,
                }
            }
            ReferenceKey::Sun => {
                let sun = input
                    .bodies
                    .iter()
                    .find(|body| body.body == crate::kundli::astro::AstroBody::Sun)
                    .ok_or(DeriveError::MissingPlacementBody)?;
                ResolvedReference {
                    point: ReferencePoint::Planet(crate::kundli::astro::AstroBody::Sun),
                    longitude: sun.longitude,
                }
            }
            ReferenceKey::Planet(body) => {
                let placement = input
                    .bodies
                    .iter()
                    .find(|candidate| candidate.body == body)
                    .ok_or(DeriveError::MissingPlacementBody)?;
                ResolvedReference {
                    point: ReferencePoint::Planet(body),
                    longitude: placement.longitude,
                }
            }
            ReferenceKey::Special(SpecialReference::Gulika) => ResolvedReference {
                point: ReferencePoint::Special(SpecialReference::Gulika),
                longitude: gulika_longitude(input)?,
            },
        };

        Ok(ReferenceContext {
            projected: input.clone(),
            reference: resolved,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct LagnaReference;

impl ReferenceOp<ProjectedBase> for LagnaReference {
    type Output = ReferenceContext;

    fn apply(&self, input: &ProjectedBase) -> Result<Self::Output, DeriveError> {
        ReferenceTransform::new(ReferenceKey::Lagna).apply(input)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct MoonReference;

impl ReferenceOp<ProjectedBase> for MoonReference {
    type Output = ReferenceContext;

    fn apply(&self, input: &ProjectedBase) -> Result<Self::Output, DeriveError> {
        ReferenceTransform::new(ReferenceKey::Moon).apply(input)
    }
}
