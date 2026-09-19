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
        KernelRequest::EvaluateAxisAlignedBox {
            evaluation_identity,
            result_identity,
            box_geometry,
            tolerance,
        } => evaluate_axis_aligned_box(
            evaluation_identity,
            result_identity,
            box_geometry,
            tolerance,
        ).map(KernelResponse::AxisAlignedBox),
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

fn evaluate_axis_aligned_box(
    evaluation_identity: String,
    result_identity: String,
    box_geometry: crate::functions::brep::AxisAlignedBox,
    tolerance: crate::functions::tolerance::Tolerance,
) -> Result<crate::api::AxisAlignedBoxEvaluationResult, ServiceError> {
    if evaluation_identity.trim().is_empty() {
        return Err(ServiceError("Box evaluation identity cannot be empty.".into()));
    }
    if result_identity.trim().is_empty() {
        return Err(ServiceError("Box result identity cannot be empty.".into()));
    }

    box_geometry
        .validate(tolerance)
        .map_err(|e| ServiceError(e.to_string()))?;

    let volume = box_geometry
        .volume(tolerance)
        .map_err(|e| ServiceError(e.to_string()))?;
    let surface_area = box_geometry
        .surface_area(tolerance)
        .map_err(|e| ServiceError(e.to_string()))?;
    let centroid = box_geometry
        .centroid(tolerance)
        .map_err(|e| ServiceError(e.to_string()))?;

    let result = crate::api::AxisAlignedBoxEvaluationResult {
        evaluation_identity,
        result_identity,
        min: [
            box_geometry.min.x,
            box_geometry.min.y,
            box_geometry.min.z,
        ],
        max: [
            box_geometry.max.x,
            box_geometry.max.y,
            box_geometry.max.z,
        ],
        volume,
        surface_area,
        centroid: [centroid.x, centroid.y, centroid.z],
    };

    if !result.min.iter().chain(result.max.iter()).chain(result.centroid.iter()).all(|x| x.is_finite())
        || !result.volume.is_finite()
        || !result.surface_area.is_finite()
    {
        return Err(ServiceError("AxisAlignedBox evaluation produced non-finite evidence.".into()));
    }

    Ok(result)
}
