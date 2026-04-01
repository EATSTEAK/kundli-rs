use crate::kundli::astro::AstroBody;
use crate::kundli::error::DeriveError;

use super::ProjectedBase;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ReferencePoint {
    Lagna,
    Planet(AstroBody),
    CustomLongitude(f64),
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
pub(crate) struct LagnaReference;

impl ReferenceOp<ProjectedBase> for LagnaReference {
    type Output = ReferenceContext;

    fn apply(&self, input: &ProjectedBase) -> Result<Self::Output, DeriveError> {
        Ok(ReferenceContext {
            projected: input.clone(),
            reference: ResolvedReference {
                point: ReferencePoint::Lagna,
                longitude: input.ascendant_longitude,
            },
        })
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct MoonReference;

impl ReferenceOp<ProjectedBase> for MoonReference {
    type Output = ReferenceContext;

    fn apply(&self, input: &ProjectedBase) -> Result<Self::Output, DeriveError> {
        let moon = input
            .bodies
            .iter()
            .find(|body| body.body == AstroBody::Moon)
            .ok_or(DeriveError::MissingMoon)?;

        Ok(ReferenceContext {
            projected: input.clone(),
            reference: ResolvedReference {
                point: ReferencePoint::Planet(AstroBody::Moon),
                longitude: moon.longitude,
            },
        })
    }
}
