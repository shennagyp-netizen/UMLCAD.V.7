use std::ffi::CString;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use umlcad_v6_exchange_api::{ExchangeBackend, ExchangeDirection, ExchangeError, ExchangeEvidence, ExchangeFormat, ExchangeStatus};
use umlcad_v6_geometry_api::{GeometryBackend, GeometryEvidence, GeometryKind, GeometryResult, GeometryStatus, ToleranceContext};

use super::{NativeShape, OcctBackend, OcctShape, OCCT_CONSTRUCTION_FAILED, OCCT_INTERNAL_ERROR, OCCT_INVALID_ARGUMENT, OCCT_NULL_SHAPE, OCCT_OK};

unsafe extern "C" {
    fn umlcad_occt_shape_export_file(input: *const NativeShape, format: i32, path: *const std::ffi::c_char) -> i32;
    fn umlcad_occt_shape_import_file(format: i32, path: *const std::ffi::c_char, out_shape: *mut *mut NativeShape) -> i32;
}

static EXCHANGE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn exchange_lock() -> Result<std::sync::MutexGuard<'static, ()>, ExchangeError> {
    EXCHANGE_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|_| ExchangeError::TranslationFailure)
}

fn format_code(format: ExchangeFormat) -> i32 {
    match format {
        ExchangeFormat::Step => 0,
        ExchangeFormat::Iges => 1,
    }
}

fn path_cstring(path: &Path) -> Result<CString, ExchangeError> {
    let value = path.to_str().ok_or(ExchangeError::InvalidPath)?;
    if value.is_empty() || value.contains('\0') {
        return Err(ExchangeError::InvalidPath);
    }
    CString::new(value).map_err(|_| ExchangeError::InvalidPath)
}

fn exchange_status(status: i32) -> ExchangeError {
    match status {
        OCCT_INVALID_ARGUMENT => ExchangeError::InvalidPath,
        OCCT_NULL_SHAPE => ExchangeError::EmptyResult,
        OCCT_CONSTRUCTION_FAILED | OCCT_INTERNAL_ERROR => ExchangeError::TranslationFailure,
        _ => ExchangeError::TranslationFailure,
    }
}

fn imported_kind<S: Clone>(backend: &OcctBackend, shape: &S, tolerance: ToleranceContext) -> Result<GeometryKind, ExchangeError>
where
    OcctBackend: GeometryBackend<Shape = S>,
{
    let counts = backend
        .topology_counts(shape, tolerance)
        .map_err(|_| ExchangeError::TranslationFailure)?;
    if counts.solids > 0 {
        Ok(GeometryKind::Solid)
    } else if counts.faces > 0 {
        Ok(GeometryKind::Surface)
    } else if counts.edges > 0 || counts.vertices > 0 {
        Ok(GeometryKind::Curve)
    } else {
        Err(ExchangeError::EmptyResult)
    }
}

impl ExchangeBackend for OcctBackend {
    fn export_file(
        &self,
        shape: &Self::Shape,
        format: ExchangeFormat,
        path: &Path,
        tolerance: ToleranceContext,
    ) -> Result<ExchangeEvidence, ExchangeError> {
        tolerance
            .validate()
            .map_err(|_| ExchangeError::InvalidPath)?;
        let _exchange_guard = exchange_lock()?;
        let destination = path_cstring(path)?;
        let status = unsafe {
            umlcad_occt_shape_export_file(shape.raw.as_ptr(), format_code(format), destination.as_ptr())
        };
        if status != OCCT_OK {
            return Err(exchange_status(status));
        }
        let metadata = std::fs::metadata(path).map_err(|_| ExchangeError::IoFailure)?;
        if !metadata.is_file() {
            return Err(ExchangeError::IoFailure);
        }
        let bytes = metadata.len();
        if bytes == 0 {
            return Err(ExchangeError::EmptyResult);
        }
        Ok(ExchangeEvidence {
            format,
            direction: ExchangeDirection::Export,
            status: ExchangeStatus::Success,
            backend: self.backend_name(),
            bytes,
        })
    }

    fn import_file(
        &self,
        format: ExchangeFormat,
        path: &Path,
        tolerance: ToleranceContext,
    ) -> Result<GeometryResult<Self::Shape>, ExchangeError> {
        tolerance
            .validate()
            .map_err(|_| ExchangeError::InvalidPath)?;
        let _exchange_guard = exchange_lock()?;
        let metadata = std::fs::metadata(path).map_err(|_| ExchangeError::IoFailure)?;
        if !metadata.is_file() || metadata.len() == 0 {
            return Err(ExchangeError::EmptyResult);
        }
        let source = path_cstring(path)?;
        let mut raw = std::ptr::null_mut();
        let status = unsafe {
            umlcad_occt_shape_import_file(format_code(format), source.as_ptr(), &mut raw)
        };
        if status != OCCT_OK {
            return Err(exchange_status(status));
        }
        let raw = std::ptr::NonNull::new(raw).ok_or(ExchangeError::EmptyResult)?;
        let shape = OcctShape::from_raw(raw, GeometryKind::Solid);
        let validation = self
            .validate(&shape, tolerance)
            .map_err(|_| ExchangeError::TranslationFailure)?;
        if !validation.valid {
            return Err(ExchangeError::TranslationFailure);
        }
        let kind = imported_kind(self, &shape, tolerance)?;
        Ok(GeometryResult {
            shape,
            kind,
            evidence: GeometryEvidence {
                status: GeometryStatus::Success,
                backend: self.backend_name(),
                tolerance,
                message: None,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use umlcad_v6_geometry_api::GeometryBackend;

    const TOLERANCE: ToleranceContext = ToleranceContext {
        modeling: 1e-9,
        validation: 1e-9,
    };

    fn temp_path(extension: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("umlcad_v6_exchange_{}_{}.{}", std::process::id(), stamp, extension))
    }

    #[test]
    fn unsupported_extension_is_rejected_before_backend_io() {
        assert_eq!(
            ExchangeFormat::from_extension(Path::new("part.obj")),
            Err(ExchangeError::UnsupportedFormat)
        );
    }

    #[test]
    fn exchange_path_with_embedded_nul_is_rejected() {
        let backend = OcctBackend::new();
        let shape = backend.box_solid(10.0, 10.0, 10.0, TOLERANCE).unwrap().shape;
        let error = backend.export_file(&shape, ExchangeFormat::Step, Path::new("bad\0.step"), TOLERANCE);
        assert_eq!(error, Err(ExchangeError::InvalidPath));
    }

    #[test]
    fn step_round_trip_preserves_valid_geometry_and_kind() {
        let backend = OcctBackend::new();
        let source = backend.box_solid(10.0, 20.0, 30.0, TOLERANCE).unwrap().shape;
        let path = temp_path("step");
        let evidence = backend.export_file(&source, ExchangeFormat::Step, &path, TOLERANCE).unwrap();
        assert_eq!(evidence.status, ExchangeStatus::Success);
        assert!(evidence.bytes > 0);

        let imported = backend.import_file(ExchangeFormat::Step, &path, TOLERANCE).unwrap();
        assert_eq!(imported.kind, GeometryKind::Solid);
        assert!(backend.validate(&imported.shape, TOLERANCE).unwrap().valid);
        assert_eq!(backend.topology_counts(&source, TOLERANCE).unwrap(), backend.topology_counts(&imported.shape, TOLERANCE).unwrap());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn iges_round_trip_produces_valid_geometry() {
        let backend = OcctBackend::new();
        let source = backend.box_solid(5.0, 6.0, 7.0, TOLERANCE).unwrap().shape;
        let path = temp_path("iges");
        let evidence = backend.export_file(&source, ExchangeFormat::Iges, &path, TOLERANCE).unwrap();
        assert_eq!(evidence.status, ExchangeStatus::Success);
        assert!(evidence.bytes > 0);

        let imported = backend.import_file(ExchangeFormat::Iges, &path, TOLERANCE).unwrap();
        assert!(backend.validate(&imported.shape, TOLERANCE).unwrap().valid);
        assert!(backend.topology_counts(&imported.shape, TOLERANCE).unwrap().faces > 0);
        std::fs::remove_file(path).unwrap();
    }
}
