use std::path::Path;
use umlcad_v6_geometry_api::{GeometryBackend, GeometryResult, ToleranceContext};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExchangeFormat {
    Step,
    Iges,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExchangeDirection {
    Import,
    Export,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExchangeStatus {
    Success,
    Failed,
    Unsupported,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExchangeEvidence {
    pub format: ExchangeFormat,
    pub direction: ExchangeDirection,
    pub status: ExchangeStatus,
    pub backend: &'static str,
    pub bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExchangeError {
    InvalidPath,
    UnsupportedFormat,
    IoFailure,
    TranslationFailure,
    EmptyResult,
}

impl ExchangeFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Step => "step",
            Self::Iges => "iges",
        }
    }

    pub fn from_extension(path: &Path) -> Result<Self, ExchangeError> {
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase)
            .ok_or(ExchangeError::InvalidPath)?;
        match extension.as_str() {
            "step" | "stp" => Ok(Self::Step),
            "iges" | "igs" => Ok(Self::Iges),
            _ => Err(ExchangeError::UnsupportedFormat),
        }
    }
}

pub trait ExchangeBackend: GeometryBackend {
    fn export_file(
        &self,
        shape: &Self::Shape,
        format: ExchangeFormat,
        path: &Path,
        tolerance: ToleranceContext,
    ) -> Result<ExchangeEvidence, ExchangeError>;

    fn import_file(
        &self,
        format: ExchangeFormat,
        path: &Path,
        tolerance: ToleranceContext,
    ) -> Result<GeometryResult<Self::Shape>, ExchangeError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_exchange_extensions_are_explicit() {
        assert_eq!(ExchangeFormat::from_extension(Path::new("part.step")).unwrap(), ExchangeFormat::Step);
        assert_eq!(ExchangeFormat::from_extension(Path::new("part.stp")).unwrap(), ExchangeFormat::Step);
        assert_eq!(ExchangeFormat::from_extension(Path::new("part.iges")).unwrap(), ExchangeFormat::Iges);
        assert_eq!(ExchangeFormat::from_extension(Path::new("part.igs")).unwrap(), ExchangeFormat::Iges);
        assert_eq!(ExchangeFormat::from_extension(Path::new("part.obj")), Err(ExchangeError::UnsupportedFormat));
    }

    #[test]
    fn exchange_evidence_carries_direction_and_backend_identity() {
        let evidence = ExchangeEvidence {
            format: ExchangeFormat::Step,
            direction: ExchangeDirection::Export,
            status: ExchangeStatus::Success,
            backend: "occt",
            bytes: 1024,
        };
        assert_eq!(evidence.direction, ExchangeDirection::Export);
        assert_eq!(evidence.backend, "occt");
    }
}
