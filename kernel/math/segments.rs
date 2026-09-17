//! Finite segment mathematics in 2D and 3D.
//!
//! Segments are bounded parameter domains `[0,1]`. Projection and containment
//! use explicit tolerances and never clamp invalid endpoints silently.

use super::{predicates::{is_between2d,is_between3d,Tri},vec::{Vec2,Vec3}};

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Segment2{pub start:Vec2,pub end:Vec2}
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Segment3{pub start:Vec3,pub end:Vec3}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum SegmentError{NonFinite,Degenerate,InvalidTolerance,InvalidParameter,Overflow}

impl Segment2{
 pub fn validate(&self)->Result<(),SegmentError>{if!self.start.is_finite()||!self.end.is_finite(){return Err(SegmentError::NonFinite);}if self.end.sub(self.start).length()==0.{return Err(SegmentError::Degenerate);}Ok(())}
 pub fn length(&self)->f64{self.end.sub(self.start).length()}
 pub fn point_at(&self,t:f64)->Result<Vec2,SegmentError>{self.validate()?;if!t.is_finite(){return Err(SegmentError::NonFinite);}if!(0.0..=1.0).contains(&t){let p=self.start.add(self.end.sub(self.start).scale(t));if p.is_finite(){Ok(p)}else{Err(SegmentError::Overflow)}}else{Err(SegmentError::InvalidParameter)}}
 pub fn supporting_parameter(&self,p:Vec2)->Result<f64,SegmentError>{self.validate()?;if!p.is_finite(){return Err(SegmentError::NonFinite);}let d=self.end.sub(self.start);let t=p.sub(self.start).dot(d)/d.dot(d);if t.is_finite(){Ok(t)}else{Err(SegmentError::Overflow)}}
 pub fn closest_parameter(&self,p:Vec2)->Result<f64,SegmentError>{let t=self.supporting_parameter(p)?;Ok(t.clamp(0.,1.))}
 pub fn closest_point(&self,p:Vec2)->Result<Vec2,SegmentError>{self.point_at(self.closest_parameter(p)?)}
 pub fn distance_to_point(&self,p:Vec2)->Result<f64,SegmentError>{Ok(p.sub(self.closest_point(p)?).length())}
 pub fn contains(&self,p:Vec2,tol:f64)->Tri{is_between2d(self.start,self.end,p,tol)}
}

impl Segment3{
 pub fn validate(&self)->Result<(),SegmentError>{if!self.start.is_finite()||!self.end.is_finite(){return Err(SegmentError::NonFinite);}if self.end.sub(self.start).length()==0.{return Err(SegmentError::Degenerate);}Ok(())}
 pub fn length(&self)->f64{self.end.sub(self.start).length()}
 pub fn point_at(&self,t:f64)->Result<Vec3,SegmentError>{self.validate()?;if!t.is_finite(){return Err(SegmentError::NonFinite);}if!(0.0..=1.0).contains(&t){let p=self.start.add(self.end.sub(self.start).scale(t));if p.is_finite(){Ok(p)}else{Err(SegmentError::Overflow)}}else{Err(SegmentError::InvalidParameter)}}
 pub fn supporting_parameter(&self,p:Vec3)->Result<f64,SegmentError>{self.validate()?;if!p.is_finite(){return Err(SegmentError::NonFinite);}let d=self.end.sub(self.start);let t=p.sub(self.start).dot(d)/d.dot(d);if t.is_finite(){Ok(t)}else{Err(SegmentError::Overflow)}}
 pub fn closest_parameter(&self,p:Vec3)->Result<f64,SegmentError>{Ok(self.supporting_parameter(p)?.clamp(0.,1.))}
 pub fn closest_point(&self,p:Vec3)->Result<Vec3,SegmentError>{self.point_at(self.closest_parameter(p)?)}
 pub fn distance_to_point(&self,p:Vec3)->Result<f64,SegmentError>{Ok(p.sub(self.closest_point(p)?).length())}
 pub fn contains(&self,p:Vec3,tol:f64)->Tri{is_between3d(self.start,self.end,p,tol)}
}

#[cfg(test)]
mod tests{use super::*;#[test]fn segment_projection_and_distance(){let s=Segment2{start:Vec2::new(0.,0.),end:Vec2::new(2.,0.)};assert!((s.closest_parameter(Vec2::new(1.,3.)).unwrap()-.5).abs()<1e-15);assert!((s.distance_to_point(Vec2::new(1.,3.)).unwrap()-3.).abs()<1e-15);}#[test]fn segment_domain_is_closed(){let s=Segment3{start:Vec3::new(0.,0.,0.),end:Vec3::new(1.,0.,0.)};assert_eq!(s.point_at(0.).unwrap(),s.start);assert_eq!(s.point_at(1.).unwrap(),s.end);assert_eq!(s.point_at(1.1),Err(SegmentError::InvalidParameter));}#[test]fn degenerate_segment_fails_closed(){let s=Segment2{start:Vec2::new(0.,0.),end:Vec2::new(0.,0.)};assert_eq!(s.validate(),Err(SegmentError::Degenerate));}}
