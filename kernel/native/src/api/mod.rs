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
};

#[derive(Clone, Debug)]
pub enum KernelRequest {
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
pub enum KernelResponse {
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
