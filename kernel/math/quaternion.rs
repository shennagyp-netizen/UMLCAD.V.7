//! Quaternion mathematics for 3D rotations.
//!
//! A quaternion used as a rotation is required to be finite and non-zero.
//! Normalization uses chained hypot operations to avoid overflow in the norm.

use super::vec::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quaternion { pub w:f64, pub x:f64, pub y:f64, pub z:f64 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuaternionError { NonFinite, Degenerate, Overflow }

impl Quaternion {
    pub const IDENTITY:Self=Self{w:1.,x:0.,y:0.,z:0.};
    pub const fn new(w:f64,x:f64,y:f64,z:f64)->Self{Self{w,x,y,z}}
    pub fn is_finite(self)->bool{self.w.is_finite()&&self.x.is_finite()&&self.y.is_finite()&&self.z.is_finite()}
    pub fn norm(self)->f64{self.w.hypot(self.x).hypot(self.y).hypot(self.z)}
    pub fn norm_squared(self)->f64{self.norm()*self.norm()}
    pub fn normalized(self)->Result<Self,QuaternionError>{if!self.is_finite(){return Err(QuaternionError::NonFinite);}let n=self.norm();if!n.is_finite(){return Err(QuaternionError::Overflow);}if n==0.{return Err(QuaternionError::Degenerate);}let q=Self::new(self.w/n,self.x/n,self.y/n,self.z/n);if q.is_finite(){Ok(q)}else{Err(QuaternionError::Overflow)}}
    pub fn conjugate(self)->Self{Self::new(self.w,-self.x,-self.y,-self.z)}
    pub fn inverse(self)->Result<Self,QuaternionError>{if!self.is_finite(){return Err(QuaternionError::NonFinite);}let n=self.norm();if!n.is_finite(){return Err(QuaternionError::Overflow);}if n==0.{return Err(QuaternionError::Degenerate);}let q=self.normalized()?;let s=1./n;if!s.is_finite(){return Err(QuaternionError::Overflow);}Ok(q.conjugate().scale(s))}
    pub fn scale(self,s:f64)->Self{Self::new(self.w*s,self.x*s,self.y*s,self.z*s)}
    pub fn mul(self,o:Self)->Self{Self::new(self.w*o.w-self.x*o.x-self.y*o.y-self.z*o.z,self.w*o.x+self.x*o.w+self.y*o.z-self.z*o.y,self.w*o.y-self.x*o.z+self.y*o.w+self.z*o.x,self.w*o.z+self.x*o.y-self.y*o.x+self.z*o.w)}
    pub fn from_axis_angle(axis:Vec3,angle:f64)->Result<Self,QuaternionError>{if!axis.is_finite()||!angle.is_finite(){return Err(QuaternionError::NonFinite);}let unit=axis.normalized().map_err(|_|QuaternionError::Degenerate)?;let half=0.5*angle;let s=half.sin();Ok(Self::new(half.cos(),unit.x*s,unit.y*s,unit.z*s))}
    pub fn rotate_vector(self,vector:Vec3)->Result<Vec3,QuaternionError>{if!vector.is_finite(){return Err(QuaternionError::NonFinite);}let q=self.normalized()?;let p=Self::new(0.,vector.x,vector.y,vector.z);let r=q.mul(p).mul(q.conjugate());let result=Vec3::new(r.x,r.y,r.z);if result.is_finite(){Ok(result)}else{Err(QuaternionError::Overflow)}}
    pub fn to_mat3(self)->Result<super::mat::Mat3,QuaternionError>{let q=self.normalized()?;let(w,x,y,z)=(q.w,q.x,q.y,q.z);let m=super::mat::Mat3::new([[1.-2.*(y*y+z*z),2.*(x*y-z*w),2.*(x*z+y*w)],[2.*(x*y+z*w),1.-2.*(x*x+z*z),2.*(y*z-x*w)],[2.*(x*z-y*w),2.*(y*z+x*w),1.-2.*(x*x+y*y)]]);if m.m.iter().flatten().all(|v|v.is_finite()){Ok(m)}else{Err(QuaternionError::Overflow)}}
}

#[cfg(test)]
mod tests{use super::*;use crate::math::constants::HALF_PI;#[test]fn axis_angle_quarter_turn(){let q=Quaternion::from_axis_angle(Vec3::new(0.,0.,1.),HALF_PI).unwrap();let r=q.rotate_vector(Vec3::new(1.,0.,0.)).unwrap();assert!(r.x.abs()<1e-14&&(r.y-1.).abs()<1e-14&&r.z.abs()<1e-14);}#[test]fn inverse_round_trip_is_identity_with_tolerance(){let q=Quaternion::new(2.,3.,-1.,4.);let p=q.mul(q.inverse().unwrap()).normalized().unwrap();assert!((p.w-1.).abs()<1e-14&&p.x.abs()<1e-14&&p.y.abs()<1e-14&&p.z.abs()<1e-14);}#[test]fn extreme_finite_components_normalize_without_overflow(){let q=Quaternion::new(1e308,-1e308,1e308,-1e308);assert!(q.normalized().unwrap().is_finite());}#[test]fn zero_axis_is_rejected(){assert_eq!(Quaternion::from_axis_angle(Vec3::new(0.,0.,0.),1.),Err(QuaternionError::Degenerate));}}
