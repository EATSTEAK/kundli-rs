use crate::kundli::astro::{AstroBody, AstroResult, ZodiacType};
use crate::kundli::derive::sign::normalize_longitude;
use crate::kundli::error::DeriveError;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProjectedBase {
    pub zodiac: ZodiacType,
    pub ascendant_longitude: f64,
    pub house_cusps: Vec<f64>,
    pub bodies: Vec<ProjectedBody>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProjectedBody {
    pub body: AstroBody,
    pub longitude: f64,
    pub is_retrograde: bool,
}

pub(crate) trait ProjectionOp {
    type Output;

    fn apply(&self, input: AstroResult) -> Result<Self::Output, DeriveError>;
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct IdentityProjection;

impl ProjectionOp for IdentityProjection {
    type Output = ProjectedBase;

    fn apply(&self, input: AstroResult) -> Result<Self::Output, DeriveError> {
        Ok(ProjectedBase {
            zodiac: input.meta.zodiac,
            ascendant_longitude: normalize_longitude(input.ascendant_longitude)?,
            house_cusps: input
                .house_cusps
                .into_iter()
                .map(normalize_longitude)
                .collect::<Result<Vec<_>, _>>()?,
            bodies: input
                .bodies
                .into_iter()
                .map(|body| {
                    Ok(ProjectedBody {
                        body: body.body,
                        longitude: normalize_longitude(body.longitude)?,
                        is_retrograde: body.speed_longitude < 0.0,
                    })
                })
                .collect::<Result<Vec<_>, DeriveError>>()?,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct SiderealProjection {
    pub ayanamsa: f64,
}

impl ProjectionOp for SiderealProjection {
    type Output = ProjectedBase;

    fn apply(&self, input: AstroResult) -> Result<Self::Output, DeriveError> {
        let project = |longitude: f64| normalize_longitude(longitude - self.ayanamsa);

        Ok(ProjectedBase {
            zodiac: ZodiacType::Sidereal,
            ascendant_longitude: project(input.ascendant_longitude)?,
            house_cusps: input
                .house_cusps
                .into_iter()
                .map(project)
                .collect::<Result<Vec<_>, _>>()?,
            bodies: input
                .bodies
                .into_iter()
                .map(|body| {
                    Ok(ProjectedBody {
                        body: body.body,
                        longitude: project(body.longitude)?,
                        is_retrograde: body.speed_longitude < 0.0,
                    })
                })
                .collect::<Result<Vec<_>, DeriveError>>()?,
        })
    }
}
