use crate::api::{KernelRequest, KernelResponse};
use crate::functions::{
    dimensions::evaluate_dimensions,
    dxf::export_dxf,
    engineering::validate_engineering,
    solver::{scaled_damped_qr, solve_snapshot},
    validation::validate_snapshot,
};
use std::fmt;

#[derive(Debug)]
pub struct ServiceError(pub String);
impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for ServiceError {}

pub fn dispatch(request: KernelRequest) -> Result<KernelResponse, ServiceError> {
    match request {
        KernelRequest::Validate { snapshot } => {
            Ok(KernelResponse::Diagnostics(validate_snapshot(&snapshot)))
        }
        KernelRequest::AnalyzeLinearSystem {
            jacobian,
            residuals,
            damping,
            rank_tolerance,
        } => scaled_damped_qr(&jacobian, &residuals, damping, rank_tolerance)
            .map(KernelResponse::Linear)
            .map_err(ServiceError),
        KernelRequest::Solve { snapshot, options } => solve_snapshot(&snapshot, options)
            .map(KernelResponse::Solve)
            .map_err(ServiceError),
        KernelRequest::Analyze { snapshot, options } => solve_snapshot(&snapshot, options)
            .map(|r| KernelResponse::Analysis(r.analysis))
            .map_err(ServiceError),
        KernelRequest::EngineeringEvidence { snapshot } => {
            Ok(KernelResponse::Engineering(validate_engineering(&snapshot)))
        }
        KernelRequest::Dimensions {
            snapshot,
            dimensions,
        } => evaluate_dimensions(&snapshot, &dimensions)
            .map(KernelResponse::Dimensions)
            .map_err(ServiceError),
        KernelRequest::ExportDxf { snapshot } => Ok(KernelResponse::Dxf(export_dxf(&snapshot))),
    }
}
