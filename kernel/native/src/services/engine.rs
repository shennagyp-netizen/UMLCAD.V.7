use crate::api::{BoxSolidReport, BoxSolidTopology, KernelRequest, KernelResponse};
use crate::functions::{
    dimensions::evaluate_dimensions,
    dxf::export_dxf,
    engineering::validate_engineering,
    solver::{scaled_damped_qr, solve_snapshot},
    validation::validate_snapshot,
};
use sha2::{Digest, Sha256};
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
        KernelRequest::BuildAxisAlignedBoxSolid {
            operation_identity,
            bounds,
            tolerance,
        } => build_axis_aligned_box_solid_report(
            &operation_identity,
            bounds,
            tolerance,
        )
        .map(KernelResponse::BoxSolid)
        .map_err(|error| ServiceError(error.to_string())),
    }
}

fn build_axis_aligned_box_solid_report(
    operation_identity: &str,
    bounds: crate::functions::brep::AxisAlignedBox,
    tolerance: crate::functions::tolerance::Tolerance,
) -> Result<BoxSolidReport, crate::functions::brep::BRepError> {
    if operation_identity.trim().is_empty() {
        return Err(crate::functions::brep::BRepError::MissingReference);
    }

    let solid = crate::functions::brep::build_axis_aligned_box_solid(bounds, tolerance)?;
    let mut hasher = Sha256::new();
    hasher.update(b"uml-cad-axis-aligned-box-solid/1|");
    hasher.update(operation_identity.as_bytes());
    hasher.update(b"|");
    hasher.update(bounds.min.x.to_bits().to_le_bytes());
    hasher.update(bounds.min.y.to_bits().to_le_bytes());
    hasher.update(bounds.min.z.to_bits().to_le_bytes());
    hasher.update(bounds.max.x.to_bits().to_le_bytes());
    hasher.update(bounds.max.y.to_bits().to_le_bytes());
    hasher.update(bounds.max.z.to_bits().to_le_bytes());
    hasher.update(tolerance.absolute.to_bits().to_le_bytes());
    hasher.update(tolerance.relative.to_bits().to_le_bytes());
    let operation_hash = format!("{:x}", hasher.finalize());
    let result_id = format!("solid:{operation_hash}");

    let evidence_hash = {
        let mut evidence = Sha256::new();
        evidence.update(b"uml-cad-axis-aligned-box-solid-evidence/1|");
        evidence.update(result_id.as_bytes());
        format!("{:x}", evidence.finalize())
    };

    let topology = solid
        .faces
        .iter()
        .map(|face| BoxSolidTopology {
            kind: "Face".to_string(),
            key: face.id.clone(),
        })
        .collect::<Vec<_>>();

    Ok(BoxSolidReport {
        result_id,
        evidence_hash,
        topology,
        volume: solid.volume(tolerance)?,
        surface_area: solid.surface_area(tolerance)?,
        centroid: solid.centroid(tolerance)?,
        solid,
    })
}
}
