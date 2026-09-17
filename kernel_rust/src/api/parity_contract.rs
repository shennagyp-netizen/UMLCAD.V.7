use crate::functions::snapshot::SemanticSnapshot;

#[derive(Clone, Debug, PartialEq)]
pub enum ParityOperation {
    Validate,
    Analyze,
    Solve,
    EngineeringEvidence,
    ExportDxf,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParityRequest {
    pub operation: ParityOperation,
    pub snapshot: SemanticSnapshot,
}
