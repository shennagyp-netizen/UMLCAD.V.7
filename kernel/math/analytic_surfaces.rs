//! Parametric analytic surface authority for planes, spheres, cylinders, cones,
//! and tori.
//!
//! Surface parameterizations expose exact first and second derivatives. A
//! singular parameterization (for example a sphere pole or cone apex) is an
//! explicit failure state for normals/curvature rather than a fabricated value.

use super::{analytic::{Cone3,Cylinder3,Plane3,Torus3},conics3d::Sphere3,vec::Vec3};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum SurfaceEvalError{NonFinite,InvalidDomain,Singular,Overflow}
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct SurfaceJet3{pub point:Vec3,pub du:Vec3,pub dv:Vec3,pub duu:Vec3,pub duv:Vec3,pub dvv:Vec3}
fn finite_jet(j:SurfaceJet3)->Result<SurfaceJet3,SurfaceEvalError>{let a=[j.point,j.du,j.dv,j.duu,j.duv,j.dvv];if a.iter().all(|v|v.is_finite()){Ok(j)}else{Err(SurfaceEvalError::Overflow)}}
fn basis(axis:Vec3)->Result<(Vec3,Vec3,Vec3),SurfaceEvalError>{let n=axis.normalized().map_err(|_|SurfaceEvalError::Singular)?;let helper=if n.x.abs()<0.8{Vec3::new(1.,0.,0.)}else{Vec3::new(0.,1.,0.)};let u=n.cross(helper).normalized().map_err(|_|SurfaceEvalError::Singular)?;let v=n.cross(u).normalized().map_err(|_|SurfaceEvalError::Singular)?;Ok((u,v,n))}

impl Plane3{
 pub fn evaluate(&self,u:f64,v:f64)->Result<SurfaceJet3,SurfaceEvalError>{if!u.is_finite()||!v.is_finite(){return Err(SurfaceEvalError::NonFinite);}let(e1,e2,n)=basis(self.normal)?;finite_jet(SurfaceJet3{point:self.origin.add(e1.scale(u)).add(e2.scale(v)),du:e1,dv:e2,duu:Vec3::new(0.,0.,0.),duv:Vec3::new(0.,0.,0.),dvv:Vec3::new(0.,0.,0.)})}
 pub fn normal_at(&self,u:f64,v:f64)->Result<Vec3,SurfaceEvalError>{let j=self.evaluate(u,v)?;j.du.cross(j.dv).normalized().map_err(|_|SurfaceEvalError::Singular)}
}

impl Sphere3{
 /// `(theta,phi)` where theta is azimuth and phi is latitude.
 pub fn evaluate(&self,theta:f64,phi:f64)->Result<SurfaceJet3,SurfaceEvalError>{self.validate().map_err(|_|SurfaceEvalError::InvalidDomain)?;if!theta.is_finite()||!phi.is_finite(){return Err(SurfaceEvalError::NonFinite);}if phi< -std::f64::consts::FRAC_PI_2||phi>std::f64::consts::FRAC_PI_2{return Err(SurfaceEvalError::InvalidDomain);}let(e1,e2,n)=basis(Vec3::new(0.,0.,1.))?;let longitude=Vec3::new(e1.x*theta.cos()+e2.x*theta.sin(),e1.y*theta.cos()+e2.y*theta.sin(),e1.z*theta.cos()+e2.z*theta.sin());let tangent=Vec3::new(-e1.x*theta.sin()+e2.x*theta.cos(),-e1.y*theta.sin()+e2.y*theta.cos(),-e1.z*theta.sin()+e2.z*theta.cos());let c=phi.cos();let s=phi.sin();let radial=longitude.scale(c).add(n.scale(s));let point=self.center.add(radial.scale(self.radius));let du=tangent.scale(self.radius*c);let dv=longitude.scale(-self.radius*s).add(n.scale(self.radius*c));let duu=longitude.scale(-self.radius*c);let duv=tangent.scale(-self.radius*s);let dvv=radial.scale(-self.radius);finite_jet(SurfaceJet3{point,du,dv,duu,duv,dvv})}
 pub fn normal_at(&self,theta:f64,phi:f64)->Result<Vec3,SurfaceEvalError>{if phi.abs()>=std::f64::consts::FRAC_PI_2{return Err(SurfaceEvalError::Singular);}let j=self.evaluate(theta,phi)?;j.du.cross(j.dv).normalized().map_err(|_|SurfaceEvalError::Singular)}
}

impl Cylinder3{
 /// `(theta,z)` where z is signed distance along the cylinder axis from origin.
 pub fn evaluate(&self,theta:f64,z:f64)->Result<SurfaceJet3,SurfaceEvalError>{self.validate().map_err(|_|SurfaceEvalError::InvalidDomain)?;if!theta.is_finite()||!z.is_finite(){return Err(SurfaceEvalError::NonFinite);}let(e1,e2,n)=basis(self.axis)?;let c=theta.cos();let s=theta.sin();let radial=e1.scale(c).add(e2.scale(s));let tangent=e1.scale(-s).add(e2.scale(c));let point=self.origin.add(n.scale(z)).add(radial.scale(self.radius));let du=tangent.scale(self.radius);let dv=n;let duu=radial.scale(-self.radius);finite_jet(SurfaceJet3{point,du,dv,duu,duv:Vec3::new(0.,0.,0.),dvv:Vec3::new(0.,0.,0.)})}
 pub fn normal_at(&self,theta:f64,z:f64)->Result<Vec3,SurfaceEvalError>{let j=self.evaluate(theta,z)?;j.du.cross(j.dv).normalized().map_err(|_|SurfaceEvalError::Singular)}
}

impl Cone3{
 /// Positive nappe parameterization: `(theta,h)` with `h > 0`, measured from apex.
 pub fn evaluate_positive(&self,theta:f64,h:f64)->Result<SurfaceJet3,SurfaceEvalError>{self.validate().map_err(|_|SurfaceEvalError::InvalidDomain)?;if!theta.is_finite()||!h.is_finite(){return Err(SurfaceEvalError::NonFinite);}if h<=0.{return Err(SurfaceEvalError::Singular);}let(e1,e2,n)=basis(self.axis)?;let t=self.half_angle.tan();let c=theta.cos();let s=theta.sin();let radial=e1.scale(c).add(e2.scale(s));let tangent=e1.scale(-s).add(e2.scale(c));let point=self.apex.add(n.scale(h)).add(radial.scale(h*t));let du=tangent.scale(h*t);let dv=n.add(radial.scale(t));let duu=radial.scale(-h*t);let duv=tangent.scale(t);finite_jet(SurfaceJet3{point,du,dv,duu,duv,dvv:Vec3::new(0.,0.,0.)})}
 pub fn normal_positive(&self,theta:f64,h:f64)->Result<Vec3,SurfaceEvalError>{let j=self.evaluate_positive(theta,h)?;j.du.cross(j.dv).normalized().map_err(|_|SurfaceEvalError::Singular)}
}

impl Torus3{
 pub fn evaluate(&self,theta:f64,phi:f64)->Result<SurfaceJet3,SurfaceEvalError>{self.validate().map_err(|_|SurfaceEvalError::InvalidDomain)?;if!theta.is_finite()||!phi.is_finite(){return Err(SurfaceEvalError::NonFinite);}let(e1,e2,n)=basis(self.axis)?;let c=theta.cos();let s=theta.sin();let cp=phi.cos();let sp=phi.sin();let radial=e1.scale(c).add(e2.scale(s));let tangent=e1.scale(-s).add(e2.scale(c));let centerline_radial=self.major_radius+self.minor_radius*cp;let point=self.center.add(radial.scale(centerline_radial)).add(n.scale(self.minor_radius*sp));let du=tangent.scale(centerline_radial);let dv=radial.scale(-self.minor_radius*sp).add(n.scale(self.minor_radius*cp));let duu=radial.scale(-centerline_radial);let duv=tangent.scale(-self.minor_radius*sp);let dvv=radial.scale(-self.minor_radius*cp).add(n.scale(-self.minor_radius*sp));finite_jet(SurfaceJet3{point,du,dv,duu,duv,dvv})}
 pub fn normal_at(&self,theta:f64,phi:f64)->Result<Vec3,SurfaceEvalError>{let j=self.evaluate(theta,phi)?;j.du.cross(j.dv).normalized().map_err(|_|SurfaceEvalError::Singular)}
}

#[cfg(test)]
mod tests{use super::*;#[test]fn plane_is_affine_with_constant_derivatives(){let s=Plane3{origin:Vec3::new(1.,2.,3.),normal:Vec3::new(0.,0.,4.)};let j=s.evaluate(2.,5.).unwrap();assert!(j.du.cross(j.dv).length()>0.999);assert!(j.duu.length()<1e-15&&j.duv.length()<1e-15&&j.dvv.length()<1e-15);}#[test]fn sphere_has_radius_normal_and_pole_singularity(){let s=Sphere3{center:Vec3::new(0.,0.,0.),radius:2.};let j=s.evaluate(.3,.2).unwrap();assert!((j.point.length()-2.).abs()<1e-12);assert!((s.normal_at(.3,.2).unwrap().length()-1.).abs()<1e-14);assert_eq!(s.normal_at(.3,std::f64::consts::FRAC_PI_2),Err(SurfaceEvalError::Singular));}#[test]fn cylinder_and_torus_have_orthogonal_parameter_derivatives(){let c=Cylinder3{origin:Vec3::new(0.,0.,0.),axis:Vec3::new(0.,0.,1.),radius:2.};let j=c.evaluate(.7,3.).unwrap();assert!(j.du.dot(j.dv).abs()<1e-12);let t=Torus3{center:Vec3::new(0.,0.,0.),axis:Vec3::new(0.,0.,1.),major_radius:5.,minor_radius:1.};let j=t.evaluate(.7,.4).unwrap();assert!(j.point.is_finite()&&j.du.is_finite());}#[test]fn cone_apex_is_singular(){let c=Cone3{apex:Vec3::new(0.,0.,0.),axis:Vec3::new(0.,0.,1.),half_angle:.5};assert_eq!(c.evaluate_positive(.2,0.),Err(SurfaceEvalError::Singular));}}
