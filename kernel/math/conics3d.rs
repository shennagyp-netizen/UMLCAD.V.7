//! Three-dimensional analytic conic primitives used by intersection mathematics.

use super::vec::Vec3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Conic3DError { NonFinite, Degenerate, InvalidRadius, InvalidParameter }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Circle3 {
    pub center: Vec3,
    pub normal: Vec3,
    pub radius: f64,
}

impl Circle3 {
    pub fn validate(&self)->Result<(),Conic3DError>{if!self.center.is_finite()||!self.normal.is_finite()||!self.radius.is_finite(){return Err(Conic3DError::NonFinite);}if self.radius<=0.0{return Err(Conic3DError::InvalidRadius);}if self.normal.length()==0.0{return Err(Conic3DError::Degenerate);}Ok(())}
    pub fn unit_normal(&self)->Result<Vec3,Conic3DError>{self.validate()?;self.normal.normalized().map_err(|_|Conic3DError::Degenerate)}
    pub fn point_at(&self,t:f64)->Result<Vec3,Conic3DError>{if!t.is_finite()||!(0.0..=1.0).contains(&t){return Err(Conic3DError::InvalidParameter);}let n=self.unit_normal()?;let helper=if n.x.abs()<0.9{Vec3::new(1.,0.,0.)}else{Vec3::new(0.,1.,0.)};let u=n.cross(helper).normalized().map_err(|_|Conic3DError::Degenerate)?;let v=n.cross(u);let theta=t*std::f64::consts::TAU;let p=self.center.add(u.scale(self.radius*theta.cos())).add(v.scale(self.radius*theta.sin()));if p.is_finite(){Ok(p)}else{Err(Conic3DError::NonFinite)}}
    pub fn plane_distance(&self,p:Vec3)->Result<f64,Conic3DError>{if!p.is_finite(){return Err(Conic3DError::NonFinite);}let n=self.unit_normal()?;let d=p.sub(self.center).dot(n);if d.is_finite(){Ok(d)}else{Err(Conic3DError::NonFinite)}}
    pub fn implicit_radial_value(&self,p:Vec3)->Result<f64,Conic3DError>{let n=self.unit_normal()?;let q=p.sub(self.center);let axial=q.dot(n);let radial=q.sub(n.scale(axial)).length();let v=radial-self.radius;if v.is_finite(){Ok(v)}else{Err(Conic3DError::NonFinite)}}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sphere3 { pub center: Vec3, pub radius: f64 }
impl Sphere3 {
    pub fn validate(&self)->Result<(),Conic3DError>{if!self.center.is_finite()||!self.radius.is_finite(){return Err(Conic3DError::NonFinite);}if self.radius<=0.0{return Err(Conic3DError::InvalidRadius);}Ok(())}
    pub fn implicit_value(&self,p:Vec3)->Result<f64,Conic3DError>{if!p.is_finite(){return Err(Conic3DError::NonFinite);}self.validate()?;let d=p.distance(self.center);let v=d-self.radius;if v.is_finite(){Ok(v)}else{Err(Conic3DError::NonFinite)}}
    pub fn squared_implicit(&self,p:Vec3)->Result<f64,Conic3DError>{if!p.is_finite(){return Err(Conic3DError::NonFinite);}self.validate()?;let q=p.sub(self.center);let v=q.dot(q)-self.radius*self.radius;if v.is_finite(){Ok(v)}else{Err(Conic3DError::NonFinite)}}
}

#[cfg(test)]
mod tests{use super::*;#[test]fn circle_parameterization_is_on_plane_and_radius(){let c=Circle3{center:Vec3::new(1.,2.,3.),normal:Vec3::new(0.,0.,2.),radius:4.};let p=c.point_at(.125).unwrap();assert!((p.distance(c.center)-4.).abs()<1e-12);assert!((p.z-3.).abs()<1e-12);}#[test]fn sphere_equation_is_zero_on_known_point(){let s=Sphere3{center:Vec3::new(1.,2.,3.),radius:4.};assert!(s.squared_implicit(Vec3::new(5.,2.,3.)).unwrap().abs()<1e-12);}#[test]fn invalid_radius_is_rejected(){assert_eq!(Sphere3{center:Vec3::new(0.,0.,0.),radius:0.}.validate(),Err(Conic3DError::InvalidRadius));}}
