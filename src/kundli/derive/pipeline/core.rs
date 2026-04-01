use crate::kundli::astro::AstroResult;
use crate::kundli::error::DeriveError;
use crate::kundli::model::ChartResult;

use super::{HouseTransformOp, Materialize, ProjectionOp, ReferenceOp, SignTransformOp};

pub(crate) struct ChartPipeline<P, R, ST, HT>
where
    P: ProjectionOp,
    R: ReferenceOp<P::Output>,
    ST: SignTransformOp<R::Output>,
    HT: HouseTransformOp<ST::Output>,
    HT::Output: Materialize,
{
    projection: P,
    reference: R,
    sign_transform: ST,
    house_transform: HT,
}

impl<P, R, ST, HT> ChartPipeline<P, R, ST, HT>
where
    P: ProjectionOp,
    R: ReferenceOp<P::Output>,
    ST: SignTransformOp<R::Output>,
    HT: HouseTransformOp<ST::Output>,
    HT::Output: Materialize,
{
    pub(crate) fn new(
        projection: P,
        reference: R,
        sign_transform: ST,
        house_transform: HT,
    ) -> Self {
        Self {
            projection,
            reference,
            sign_transform,
            house_transform,
        }
    }

    pub(crate) fn execute(&self, input: AstroResult) -> Result<ChartResult, DeriveError> {
        let projected = self.projection.apply(input)?;
        let referenced = self.reference.apply(&projected)?;
        let signed = self.sign_transform.apply(&referenced)?;
        let housed = self.house_transform.apply(&signed)?;
        housed.materialize()
    }
}
