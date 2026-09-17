mod trimmed_surface;

pub use trimmed_surface::*;

use umlcad_v6_geometry_api::{GeometryBackend, GeometryError, GeometryResult, ToleranceContext};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3 {
    fn scale(self, s: f64) -> Self { Self { x: self.x * s, y: self.y * s, z: self.z * s } }
    pub fn dot(self, other: Self) -> f64 { self.x * other.x + self.y * other.y + self.z * other.z }
    pub fn cross(self, other: Self) -> Self { Self { x: self.y * other.z - self.z * other.y, y: self.z * other.x - self.x * other.z, z: self.x * other.y - self.y * other.x } }
    pub fn norm(self) -> f64 { self.x.hypot(self.y.hypot(self.z)) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NurbsSurfaceDefinitionError { NonFinite, InvalidDegree, InvalidControlGrid, InvalidWeightCount, InvalidWeight, InvalidKnotCount, KnotsMustBeNondecreasing, NotClamped, InvalidDomain }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NurbsSurfaceEvaluationError { NonFinite, OutOfDomain, InsufficientContinuity, ZeroProjectiveWeight, Overflow, DegenerateNormal }
pub type ParameterDomain = ((f64, f64), (f64, f64));
pub type Degrees = (usize, usize);
pub type GridCounts = (usize, usize);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NurbsSurfaceDifferential { pub point: Point3, pub du: Point3, pub dv: Point3, pub duu: Point3, pub duv: Point3, pub dvv: Point3 }
impl NurbsSurfaceDifferential {
    pub fn normal(&self) -> Result<Point3, NurbsSurfaceEvaluationError> {
        let cross = self.du.cross(self.dv); let magnitude = cross.norm();
        if !magnitude.is_finite() { return Err(NurbsSurfaceEvaluationError::Overflow); }
        if magnitude == 0.0 { return Err(NurbsSurfaceEvaluationError::DegenerateNormal); }
        Ok(cross.scale(1.0 / magnitude))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NurbsSurface3DDefinition { pub degree_u: usize, pub degree_v: usize, pub control_points: Vec<Point3>, pub weights: Vec<f64>, pub count_u: usize, pub count_v: usize, pub knots_u: Vec<f64>, pub knots_v: Vec<f64> }
impl NurbsSurface3DDefinition {
    pub fn new(degrees: Degrees, control_points: Vec<Point3>, weights: Vec<f64>, counts: GridCounts, knots_u: Vec<f64>, knots_v: Vec<f64>) -> Self { Self { degree_u: degrees.0, degree_v: degrees.1, control_points, weights, count_u: counts.0, count_v: counts.1, knots_u, knots_v } }
    pub fn validate(&self) -> Result<(), NurbsSurfaceDefinitionError> {
        if self.degree_u == 0 || self.degree_v == 0 { return Err(NurbsSurfaceDefinitionError::InvalidDegree); }
        if self.count_u < self.degree_u + 1 || self.count_v < self.degree_v + 1 { return Err(NurbsSurfaceDefinitionError::InvalidControlGrid); }
        let expected = self.count_u.checked_mul(self.count_v).ok_or(NurbsSurfaceDefinitionError::InvalidControlGrid)?;
        if self.control_points.len() != expected { return Err(NurbsSurfaceDefinitionError::InvalidControlGrid); }
        if self.weights.len() != expected { return Err(NurbsSurfaceDefinitionError::InvalidWeightCount); }
        if self.knots_u.len() != self.count_u + self.degree_u + 1 || self.knots_v.len() != self.count_v + self.degree_v + 1 { return Err(NurbsSurfaceDefinitionError::InvalidKnotCount); }
        if self.control_points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite()) || self.weights.iter().any(|w| !w.is_finite()) || self.knots_u.iter().any(|k| !k.is_finite()) || self.knots_v.iter().any(|k| !k.is_finite()) { return Err(NurbsSurfaceDefinitionError::NonFinite); }
        if self.weights.iter().any(|w| *w <= 0.0) { return Err(NurbsSurfaceDefinitionError::InvalidWeight); }
        if self.knots_u.windows(2).any(|p| p[1] < p[0]) || self.knots_v.windows(2).any(|p| p[1] < p[0]) { return Err(NurbsSurfaceDefinitionError::KnotsMustBeNondecreasing); }
        let u0 = self.knots_u[self.degree_u]; let u1 = self.knots_u[self.count_u]; let v0 = self.knots_v[self.degree_v]; let v1 = self.knots_v[self.count_v];
        if u1 <= u0 || v1 <= v0 { return Err(NurbsSurfaceDefinitionError::InvalidDomain); }
        if !self.knots_u[..=self.degree_u].iter().all(|k| *k == u0) || !self.knots_u[self.count_u..].iter().all(|k| *k == u1) || !self.knots_v[..=self.degree_v].iter().all(|k| *k == v0) || !self.knots_v[self.count_v..].iter().all(|k| *k == v1) { return Err(NurbsSurfaceDefinitionError::NotClamped); }
        Ok(())
    }
    pub fn parameter_domain(&self) -> Result<ParameterDomain, NurbsSurfaceDefinitionError> { self.validate()?; Ok(((self.knots_u[self.degree_u], self.knots_u[self.count_u]), (self.knots_v[self.degree_v], self.knots_v[self.count_v]))) }
    pub fn differential_at(&self, u: f64, v: f64) -> Result<NurbsSurfaceDifferential, NurbsSurfaceEvaluationError> {
        self.validate().map_err(|_| NurbsSurfaceEvaluationError::Overflow)?;
        if !u.is_finite() || !v.is_finite() { return Err(NurbsSurfaceEvaluationError::NonFinite); }
        let ((u0, u1), (v0, v1)) = self.parameter_domain().map_err(|_| NurbsSurfaceEvaluationError::Overflow)?;
        if u < u0 || u > u1 || v < v0 || v > v1 { return Err(NurbsSurfaceEvaluationError::OutOfDomain); }
        require_second_order_continuity(u, self.degree_u, &self.knots_u, self.count_u)?; require_second_order_continuity(v, self.degree_v, &self.knots_v, self.count_v)?;
        let bu = basis_derivatives(u, self.degree_u, &self.knots_u, self.count_u); let bv = basis_derivatives(v, self.degree_v, &self.knots_v, self.count_v);
        let mut h = [[Homogeneous::zero(); 3]; 3];
        for (i, bu_i) in bu.iter().enumerate().take(self.count_u) {
            for (j, bv_j) in bv.iter().enumerate().take(self.count_v) {
                let idx = i * self.count_v + j; let p = self.control_points[idx]; let w = self.weights[idx];
                let q = Homogeneous { xw: p.x*w, yw: p.y*w, zw: p.z*w, w };
                for (ku, h_ku) in h.iter_mut().enumerate() { for (kv, h_kuv) in h_ku.iter_mut().enumerate() { *h_kuv = h_kuv.add(q.scale(bu_i[ku]*bv_j[kv])); } }
            }
        }
        rationalize_differential(h)
    }
}

pub trait NurbsSurfaceBackend: GeometryBackend { fn nurbs_surface3d(&self, definition: &NurbsSurface3DDefinition, tolerance: ToleranceContext) -> Result<GeometryResult<Self::Shape>, GeometryError>; }
pub trait NurbsSurfaceDifferentialBackend: NurbsSurfaceBackend { fn nurbs_surface3d_differential_at(&self, shape: &Self::Shape, u: f64, v: f64, tolerance: ToleranceContext) -> Result<NurbsSurfaceDifferential, GeometryError>; }

#[derive(Clone, Copy)] struct Homogeneous { xw: f64, yw: f64, zw: f64, w: f64 }
impl Homogeneous { const fn zero() -> Self { Self { xw: 0.0, yw: 0.0, zw: 0.0, w: 0.0 } } fn add(self, o: Self) -> Self { Self { xw: self.xw+o.xw, yw: self.yw+o.yw, zw: self.zw+o.zw, w: self.w+o.w } } fn scale(self, s: f64) -> Self { Self { xw: self.xw*s, yw: self.yw*s, zw: self.zw*s, w: self.w*s } } }
fn knot_continuity_order(t: f64, degree: usize, knots: &[f64], count: usize) -> usize { let m = knots.iter().filter(|k| **k == t).count(); if t == knots[degree] || t == knots[count] { degree } else { degree.saturating_sub(m) } }
fn require_second_order_continuity(t: f64, degree: usize, knots: &[f64], count: usize) -> Result<(), NurbsSurfaceEvaluationError> { if degree < 2 { return Ok(()); } if t != knots[degree] && t != knots[count] && knot_continuity_order(t, degree, knots, count) < 2 { return Err(NurbsSurfaceEvaluationError::InsufficientContinuity); } Ok(()) }
fn basis_derivatives(t: f64, degree: usize, knots: &[f64], count: usize) -> Vec<[f64;3]> { (0..count).map(|i| [basis_derivative_recursive(i,degree,t,knots,0),basis_derivative_recursive(i,degree,t,knots,1),basis_derivative_recursive(i,degree,t,knots,2)]).collect() }
fn basis_derivative_recursive(i: usize, p: usize, t: f64, k: &[f64], order: usize) -> f64 {
    if order > p { return 0.0; }
    if p == 0 { if order != 0 { return 0.0; } let last=k.len()-1; return if (k[i] <= t && t < k[i+1]) || (t == k[last] && i+1 == last) { 1.0 } else { 0.0 }; }
    let dl=k[i+p]-k[i]; let dr=k[i+p+1]-k[i+1];
    if order == 0 { let a=if dl==0.0 {0.0} else {(t-k[i])/dl*basis_derivative_recursive(i,p-1,t,k,0)}; let b=if dr==0.0 {0.0} else {(k[i+p+1]-t)/dr*basis_derivative_recursive(i+1,p-1,t,k,0)}; a+b }
    else { let a=if dl==0.0 {0.0} else {p as f64/dl*basis_derivative_recursive(i,p-1,t,k,order-1)}; let b=if dr==0.0 {0.0} else {p as f64/dr*basis_derivative_recursive(i+1,p-1,t,k,order-1)}; a-b }
}
fn rationalize_differential(h:[[Homogeneous;3];3])->Result<NurbsSurfaceDifferential,NurbsSurfaceEvaluationError>{
    if h.iter().flatten().any(|q| !q.xw.is_finite()||!q.yw.is_finite()||!q.zw.is_finite()||!q.w.is_finite()){return Err(NurbsSurfaceEvaluationError::Overflow)}
    let w=h[0][0].w; if w<=0.0||!w.is_finite(){return Err(NurbsSurfaceEvaluationError::ZeroProjectiveWeight)}
    let p=Point3{x:h[0][0].xw/w,y:h[0][0].yw/w,z:h[0][0].zw/w}; let du=quotient_first(h[1][0],w,p)?; let dv=quotient_first(h[0][1],w,p)?; let duu=quotient_second(h[2][0],h[1][0].w,w,du,p)?; let duv=quotient_mixed(h[1][1],h[1][0].w,h[0][1].w,w,du,dv,p)?; let dvv=quotient_second(h[0][2],h[0][1].w,w,dv,p)?; let d=NurbsSurfaceDifferential{point:p,du,dv,duu,duv,dvv}; if [d.point,d.du,d.dv,d.duu,d.duv,d.dvv].iter().flat_map(|q|[q.x,q.y,q.z]).any(|x|!x.is_finite()){return Err(NurbsSurfaceEvaluationError::Overflow)} Ok(d)
}
fn quotient_first(d:Homogeneous,w:f64,p:Point3)->Result<Point3,NurbsSurfaceEvaluationError>{Ok(Point3{x:(d.xw-p.x*d.w)/w,y:(d.yw-p.y*d.w)/w,z:(d.zw-p.z*d.w)/w})}
fn quotient_second(d2:Homogeneous,w1:f64,w:f64,p1:Point3,p:Point3)->Result<Point3,NurbsSurfaceEvaluationError>{Ok(Point3{x:(d2.xw-2.0*w1*p1.x-d2.w*p.x)/w,y:(d2.yw-2.0*w1*p1.y-d2.w*p.y)/w,z:(d2.zw-2.0*w1*p1.z-d2.w*p.z)/w})}
fn quotient_mixed(d2:Homogeneous,wu:f64,wv:f64,w:f64,du:Point3,dv:Point3,p:Point3)->Result<Point3,NurbsSurfaceEvaluationError>{Ok(Point3{x:(d2.xw-wu*dv.x-wv*du.x-d2.w*p.x)/w,y:(d2.yw-wu*dv.y-wv*du.y-d2.w*p.y)/w,z:(d2.zw-wu*dv.z-wv*du.z-d2.w*p.z)/w})}

#[cfg(test)]
mod tests { use super::*;
    fn bilinear()->NurbsSurface3DDefinition{NurbsSurface3DDefinition::new((1,1),vec![Point3{x:0.0,y:0.0,z:0.0},Point3{x:0.0,y:1.0,z:1.0},Point3{x:1.0,y:0.0,z:1.0},Point3{x:1.0,y:1.0,z:2.0}],vec![1.0;4],(2,2),vec![0.0,0.0,1.0,1.0],vec![0.0,0.0,1.0,1.0])}
    #[test]fn bilinear_surface_is_valid(){assert_eq!(bilinear().parameter_domain().unwrap(),((0.0,1.0),(0.0,1.0)))}
    #[test]fn bilinear_differential_is_exact(){let d=bilinear().differential_at(0.25,0.75).unwrap();assert_eq!(d.point,Point3{x:0.25,y:0.75,z:1.0});assert_eq!(d.du,Point3{x:1.0,y:0.0,z:1.0});assert_eq!(d.dv,Point3{x:0.0,y:1.0,z:1.0});assert_eq!(d.duu,Point3{x:0.0,y:0.0,z:0.0});assert_eq!(d.duv,Point3{x:0.0,y:0.0,z:0.0});assert_eq!(d.dvv,Point3{x:0.0,y:0.0,z:0.0});assert!((d.normal().unwrap().norm()-1.0).abs()<1e-14)}
    #[test]fn out_of_domain_is_rejected(){assert_eq!(bilinear().differential_at(-0.1,0.5),Err(NurbsSurfaceEvaluationError::OutOfDomain))}
    #[test]fn invalid_weight_is_rejected(){let mut s=bilinear();s.weights[2]=0.0;assert!(s.differential_at(0.5,0.5).is_err())}
    #[test]fn nondecreasing_knot_contract_is_enforced(){let mut s=bilinear();s.knots_v[2]=-1.0;assert_eq!(s.validate(),Err(NurbsSurfaceDefinitionError::KnotsMustBeNondecreasing))}
}
