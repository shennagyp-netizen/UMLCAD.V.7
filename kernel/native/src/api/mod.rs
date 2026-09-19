pub mod parity_contract;

#[derive(thiserror::Error, Debug)]
pub enum KernelError {
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("kernel operation failed: {0}")]
    Operation(String),
}

use crate::functions::{
    dimensions::DimensionSpec,
    engineering::EngineeringEvidence,
    snapshot::SemanticSnapshot,
    solver::{ConstraintAnalysis, ConstraintSolveResult, SolveOptions},
    tolerance::Tolerance,
    vec::Vec3,
};

#[derive(Clone, Debug)]
pub enum KernelRequest {
    EvaluateAxisAlignedBox {
        evaluation_identity: String,
        result_identity: String,
        box_geometry: AxisAlignedBox,
        tolerance: Tolerance,
    },
    Validate {
        snapshot: SemanticSnapshot,
    },
    AnalyzeLinearSystem {
        jacobian: Vec<Vec<f64>>,
        residuals: Vec<f64>,
        damping: f64,
        rank_tolerance: f64,
    },
    Solve {
        snapshot: SemanticSnapshot,
        options: SolveOptions,
    },
    Analyze {
        snapshot: SemanticSnapshot,
        options: SolveOptions,
    },
    EngineeringEvidence {
        snapshot: SemanticSnapshot,
    },
    Dimensions {
        snapshot: SemanticSnapshot,
        dimensions: Vec<DimensionSpec>,
    },
    ExportDxf {
        snapshot: SemanticSnapshot,
    },
}

#[derive(Clone, Debug, PartialEq)]
#[derive(Clone, Debug, PartialEq)]
pub struct AxisAlignedBoxEvaluationResult {
    pub evaluation_identity: String,
    pub result_identity: String,
    pub min: [f64; 3],
    pub max: [f64; 3],
    pub volume: f64,
    pub surface_area: f64,
    pub centroid: [f64; 3],
}

pub enum KernelResponse {
    AxisAlignedBox(AxisAlignedBoxEvaluationResult),
    Diagnostics(Vec<crate::functions::validation::Diagnostic>),
    Linear(crate::functions::solver::LinearSolveReport),
    Solve(ConstraintSolveResult),
    Analysis(ConstraintAnalysis),
    Engineering(EngineeringEvidence),
    Dimensions(Vec<crate::functions::dimensions::EvaluatedDimension>),
    Dxf(String),
}

pub fn dispatch(request: KernelRequest) -> Result<KernelResponse, KernelError> {
    crate::services::dispatch(request).map_err(|e| KernelError::Operation(e.to_string()))
}
