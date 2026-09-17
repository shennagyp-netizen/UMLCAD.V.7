use umlcad_v6_geometry_api::{GeometryBackend, GeometryError, GeometryResult, ToleranceContext};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point3 { pub x: f64, pub y: f64, pub z: f64 }
impl Point3 {
    pub fn sub_point(self, other: Self) -> Self { Self { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z } }
    pub fn add_point(self, other: Self) -> Self { Self { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z } }
    pub fn scale(self, factor: f64) -> Self { Self { x: self.x * factor, y: self.y * factor, z: self.z * factor } }
    pub fn dot(self, other: Self) -> f64 { self.x * other.x + self.y * other.y + self.z * other.z }
    pub fn cross(self, other: Self) -> Self { Self { x: self.y * other.z - self.z * other.y, y: self.z * other.x - self.x * other.z, z: self.x * other.y - self.y * other.x } }
    pub fn norm(self) -> f64 { self.dot(self).sqrt() }
    pub fn normalized(self, label: &'static str) -> Result<Self, GeometryError> { let norm = self.norm(); if !norm.is_finite() || norm == 0.0 { return Err(GeometryError::InvalidInput(label)); } Ok(self.scale(1.0 / norm)) }
    fn finite(self) -> bool { self.x.is_finite() && self.y.is_finite() && self.z.is_finite() }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlanarLineSegment3D { pub start: Point3, pub end: Point3, pub plane_normal: Point3 }
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlanarSurfacePatch3D { pub origin: Point3, pub u_dir: Point3, pub v_dir: Point3, pub width: f64, pub height: f64 }

impl PlanarLineSegment3D {
    pub fn validate(self, tolerance: ToleranceContext) -> Result<(), GeometryError> {
        tolerance.validate()?;
        if !self.start.finite() || !self.end.finite() || !self.plane_normal.finite() { return Err(GeometryError::InvalidInput("planar line offset contains non-finite values")); }
        let tangent = self.end.sub_point(self.start);
        if tangent.norm() <= tolerance.modeling { return Err(GeometryError::InvalidInput("planar line offset requires a non-degenerate segment")); }
        if self.plane_normal.norm() <= tolerance.modeling { return Err(GeometryError::InvalidInput("planar line offset requires a non-zero plane normal")); }
        let normal = self.plane_normal.normalized("planar line offset requires a non-zero plane normal")?;
        let tangent_unit = tangent.normalized("planar line offset requires a non-degenerate segment")?;
        if normal.dot(tangent_unit).abs() > tolerance.validation { return Err(GeometryError::InvalidInput("line tangent must lie in the declared offset plane")); }
        Ok(())
    }
    pub fn offset(self, distance: f64, tolerance: ToleranceContext) -> Result<Self, GeometryError> {
        self.validate(tolerance)?;
        if !distance.is_finite() { return Err(GeometryError::InvalidInput("line offset distance must be finite")); }
        let tangent = self.end.sub_point(self.start).normalized("planar line offset requires a non-degenerate segment")?;
        let normal = self.plane_normal.normalized("planar line offset requires a non-zero plane normal")?;
        let offset_direction = normal.cross(tangent).normalized("declared offset plane is incompatible with the line tangent")?;
        let translation = offset_direction.scale(distance);
        let start = self.start.add_point(translation);
        let end = self.end.add_point(translation);
        if !start.finite() || !end.finite() { return Err(GeometryError::InvalidInput("line offset result is not finite")); }
        Ok(Self { start, end, plane_normal: self.plane_normal })
    }
}

impl PlanarSurfacePatch3D {
    pub fn validate(self, tolerance: ToleranceContext) -> Result<(), GeometryError> {
        tolerance.validate()?;
        if !self.origin.finite() || !self.u_dir.finite() || !self.v_dir.finite() { return Err(GeometryError::InvalidInput("planar surface offset contains non-finite values")); }
        if !self.width.is_finite() || !self.height.is_finite() || self.width <= 0.0 || self.height <= 0.0 { return Err(GeometryError::InvalidInput("planar surface patch dimensions must be finite and positive")); }
        let u_norm = self.u_dir.norm(); let v_norm = self.v_dir.norm();
        if !u_norm.is_finite() || u_norm <= tolerance.modeling { return Err(GeometryError::InvalidInput("planar surface requires a non-zero U direction")); }
        if !v_norm.is_finite() || v_norm <= tolerance.modeling { return Err(GeometryError::InvalidInput("planar surface requires a non-zero V direction")); }
        if (u_norm - 1.0).abs() > tolerance.validation || (v_norm - 1.0).abs() > tolerance.validation { return Err(GeometryError::InvalidInput("planar surface directions must be unit vectors")); }
        if self.u_dir.dot(self.v_dir).abs() > tolerance.validation { return Err(GeometryError::InvalidInput("planar surface directions must be orthogonal")); }
        if self.u_dir.cross(self.v_dir).norm() <= tolerance.modeling { return Err(GeometryError::InvalidInput("planar surface directions must define a plane")); }
        Ok(())
    }
    pub fn normal(self, tolerance: ToleranceContext) -> Result<Point3, GeometryError> { self.validate(tolerance)?; self.u_dir.cross(self.v_dir).normalized("planar surface requires a valid normal") }
    pub fn offset(self, distance: f64, tolerance: ToleranceContext) -> Result<Self, GeometryError> {
        self.validate(tolerance)?;
        if !distance.is_finite() { return Err(GeometryError::InvalidInput("surface offset distance must be finite")); }
        let normal = self.normal(tolerance)?;
        let origin = self.origin.add_point(normal.scale(distance));
        if !origin.finite() { return Err(GeometryError::InvalidInput("surface offset result is not finite")); }
        Ok(Self { origin, ..self })
    }
}

pub trait OffsetBackend: GeometryBackend {
    fn offset_planar_line(&self, definition: PlanarLineSegment3D, distance: f64, tolerance: ToleranceContext) -> Result<GeometryResult<Self::Shape>, GeometryError>;
    fn offset_planar_surface(&self, definition: PlanarSurfacePatch3D, distance: f64, tolerance: ToleranceContext) -> Result<GeometryResult<Self::Shape>, GeometryError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tolerance() -> ToleranceContext { ToleranceContext { modeling: 1e-9, validation: 1e-9 } }
    #[test] fn line_offset_is_an_exact_signed_left_normal_translation() { let line=PlanarLineSegment3D{start:Point3{x:0.,y:0.,z:0.},end:Point3{x:10.,y:0.,z:0.},plane_normal:Point3{x:0.,y:0.,z:1.}}; let result=line.offset(2.5,tolerance()).unwrap(); assert_eq!(result.start,Point3{x:0.,y:2.5,z:0.}); assert_eq!(result.end,Point3{x:10.,y:2.5,z:0.}); }
    #[test] fn negative_line_offset_reverses_the_declared_side() { let line=PlanarLineSegment3D{start:Point3{x:0.,y:0.,z:0.},end:Point3{x:0.,y:5.,z:0.},plane_normal:Point3{x:0.,y:0.,z:1.}}; let result=line.offset(-3.,tolerance()).unwrap(); assert_eq!(result.start,Point3{x:3.,y:0.,z:0.}); assert_eq!(result.end,Point3{x:3.,y:5.,z:0.}); }
    #[test] fn zero_distance_is_exact_identity() { let line=PlanarLineSegment3D{start:Point3{x:-2.,y:3.,z:4.},end:Point3{x:7.,y:-1.,z:4.},plane_normal:Point3{x:0.,y:0.,z:2.}}; assert_eq!(line.offset(0.,tolerance()).unwrap(),line); }
    #[test] fn line_offset_rejects_non_planar_tangent() { let line=PlanarLineSegment3D{start:Point3{x:0.,y:0.,z:0.},end:Point3{x:1.,y:0.,z:1.},plane_normal:Point3{x:0.,y:0.,z:1.}}; assert_eq!(line.validate(tolerance()),Err(GeometryError::InvalidInput("line tangent must lie in the declared offset plane"))); }
    #[test] fn line_offset_rejects_nonfinite_result() { let line=PlanarLineSegment3D{start:Point3{x:f64::MAX,y:0.,z:0.},end:Point3{x:f64::MAX,y:1.,z:0.},plane_normal:Point3{x:0.,y:0.,z:1.}}; assert_eq!(line.offset(-f64::MAX,tolerance()),Err(GeometryError::InvalidInput("line offset result is not finite"))); }
    #[test] fn surface_offset_translates_exactly_along_the_oriented_normal() { let patch=PlanarSurfacePatch3D{origin:Point3{x:1.,y:2.,z:3.},u_dir:Point3{x:1.,y:0.,z:0.},v_dir:Point3{x:0.,y:1.,z:0.},width:5.,height:8.}; let result=patch.offset(4.,tolerance()).unwrap(); assert_eq!(result.origin,Point3{x:1.,y:2.,z:7.}); }
    #[test] fn negative_surface_offset_uses_the_inverse_normal() { let patch=PlanarSurfacePatch3D{origin:Point3{x:0.,y:0.,z:10.},u_dir:Point3{x:0.,y:1.,z:0.},v_dir:Point3{x:1.,y:0.,z:0.},width:2.,height:3.}; let result=patch.offset(-2.,tolerance()).unwrap(); assert_eq!(result.origin,Point3{x:0.,y:0.,z:12.}); }
    #[test] fn surface_offset_rejects_non_orthogonal_parameter_directions() { let diagonal=std::f64::consts::FRAC_1_SQRT_2; let patch=PlanarSurfacePatch3D{origin:Point3{x:0.,y:0.,z:0.},u_dir:Point3{x:1.,y:0.,z:0.},v_dir:Point3{x:diagonal,y:diagonal,z:0.},width:1.,height:1.}; assert_eq!(patch.validate(tolerance()),Err(GeometryError::InvalidInput("planar surface directions must be orthogonal"))); }
    #[test] fn surface_offset_rejects_non_unit_parameter_directions() { let patch=PlanarSurfacePatch3D{origin:Point3{x:0.,y:0.,z:0.},u_dir:Point3{x:2.,y:0.,z:0.},v_dir:Point3{x:0.,y:1.,z:0.},width:1.,height:1.}; assert_eq!(patch.validate(tolerance()),Err(GeometryError::InvalidInput("planar surface directions must be unit vectors"))); }
    #[test] fn surface_offset_rejects_nonfinite_result() { let patch=PlanarSurfacePatch3D{origin:Point3{x:f64::MAX,y:0.,z:0.},u_dir:Point3{x:0.,y:1.,z:0.},v_dir:Point3{x:0.,y:0.,z:1.},width:1.,height:1.}; assert_eq!(patch.offset(f64::MAX,tolerance()),Err(GeometryError::InvalidInput("surface offset result is not finite"))); }
}
