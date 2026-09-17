//! Exact polyhedral solid mathematics.
//!
//! This module handles an explicitly triangular, closed, oriented shell. It is
//! not a tessellation-based approximation of curved geometry: triangles are
//! the authoritative planar faces of this mathematical subset.

use super::vec::Vec3;
use std::collections::BTreeMap;

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Triangle3{pub a:Vec3,pub b:Vec3,pub c:Vec3}
#[derive(Clone,Debug,PartialEq)]
pub struct PolyhedralSolid{pub triangles:Vec<Triangle3>}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum SolidError{Empty,NonFinite,Degenerate,OpenShell,NonManifold,InconsistentOrientation,ZeroVolume,Indeterminate,Overflow}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum PointSolidClass{Inside,Outside,OnBoundary,Indeterminate}

impl Triangle3{
 pub fn validate(&self)->Result<(),SolidError>{if!self.a.is_finite()||!self.b.is_finite()||!self.c.is_finite(){return Err(SolidError::NonFinite);}let cross=self.b.sub(self.a).cross(self.c.sub(self.a));if cross.length()==0.{return Err(SolidError::Degenerate);}Ok(())}
 pub fn area(&self)->Result<f64,SolidError>{self.validate()?;let a=0.5*self.b.sub(self.a).cross(self.c.sub(self.a)).length();if a.is_finite(){Ok(a)}else{Err(SolidError::Overflow)}}
 pub fn signed_tetra_volume(&self)->Result<f64,SolidError>{self.validate()?;let v=self.a.dot(self.b.cross(self.c))/6.;if v.is_finite(){Ok(v)}else{Err(SolidError::Overflow)}}
 pub fn centroid_with_origin(&self)->Vec3{self.a.add(self.b).add(self.c).scale(0.25)}
}

impl PolyhedralSolid{
 pub fn validate_closed_oriented(&self,tolerance:f64)->Result<(),SolidError>{if self.triangles.is_empty(){return Err(SolidError::Empty);}if!tolerance.is_finite()||tolerance<0.{return Err(SolidError::Indeterminate);}for t in &self.triangles{t.validate()?;}let mut edges:BTreeMap<(VertexKey,VertexKey),Vec<DirectedEdge>>=BTreeMap::new();for (face_id,t) in self.triangles.iter().enumerate(){for (a,b) in[(t.a,t.b),(t.b,t.c),(t.c,t.a)]{let ka=VertexKey::from(a);let kb=VertexKey::from(b);if ka==kb{return Err(SolidError::Degenerate);}let key=if ka<=kb{(ka,kb)}else{(kb,ka)};edges.entry(key).or_default().push(DirectedEdge{face_id,from:ka,to:kb});}}
 for incidence in edges.values(){if incidence.len()!=2{return Err(SolidError::OpenShell);}if incidence[0].from==incidence[1].from||incidence[0].to==incidence[1].to{return Err(SolidError::InconsistentOrientation);}}
 let volume=self.volume_unchecked()?;let characteristic=self.bounding_extent().max(f64::MIN_POSITIVE);if volume.abs()<=tolerance*characteristic.powi(3){return Err(SolidError::ZeroVolume);}Ok(())}
 pub fn surface_area(&self)->Result<f64,SolidError>{if self.triangles.is_empty(){return Err(SolidError::Empty);}let mut area=0.;for t in &self.triangles{area+=t.area()?;}if area.is_finite(){Ok(area)}else{Err(SolidError::Overflow)}}
 pub fn volume(&self,tolerance:f64)->Result<f64,SolidError>{self.validate_closed_oriented(tolerance)?;self.volume_unchecked().map(f64::abs)}
 fn volume_unchecked(&self)->Result<f64,SolidError>{let mut v=0.;for t in &self.triangles{v+=t.signed_tetra_volume()?;}if v.is_finite(){Ok(v)}else{Err(SolidError::Overflow)}}
 pub fn signed_volume(&self)->Result<f64,SolidError>{self.volume_unchecked()}
 pub fn centroid(&self,tolerance:f64)->Result<Vec3,SolidError>{self.validate_closed_oriented(tolerance)?;let mut weighted=Vec3::new(0.,0.,0.);let mut total=0.;for t in &self.triangles{let v=t.signed_tetra_volume()?;weighted=weighted.add(t.centroid_with_origin().scale(v));total+=v;}if total==0.{return Err(SolidError::ZeroVolume);}let c=weighted.scale(1./total);if c.is_finite(){Ok(c)}else{Err(SolidError::Overflow)}}
 pub fn bounding_extent(&self)->f64{let mut min=Vec3::new(f64::INFINITY,f64::INFINITY,f64::INFINITY);let mut max=Vec3::new(f64::NEG_INFINITY,f64::NEG_INFINITY,f64::NEG_INFINITY);for t in &self.triangles{for p in[t.a,t.b,t.c]{min.x=min.x.min(p.x);min.y=min.y.min(p.y);min.z=min.z.min(p.z);max.x=max.x.max(p.x);max.y=max.y.max(p.y);max.z=max.z.max(p.z);}}max.sub(min).length()}
 pub fn classify_point(&self,p:Vec3,direction:Vec3,tolerance:f64)->Result<PointSolidClass,SolidError>{self.validate_closed_oriented(tolerance)?;if!p.is_finite()||!direction.is_finite(){return Err(SolidError::NonFinite);}let dir=direction.normalized().map_err(|_|SolidError::Degenerate)?;let boundary_tol=tolerance*self.bounding_extent().max(f64::MIN_POSITIVE);let mut hits=0usize;for t in &self.triangles{match ray_triangle(p,dir,*t,boundary_tol){RayHit::Boundary=>return Ok(PointSolidClass::OnBoundary),RayHit::Hit=>hits+=1,RayHit::Miss=>{},RayHit::Indeterminate=>return Ok(PointSolidClass::Indeterminate)}}Ok(if hits%2==1{PointSolidClass::Inside}else{PointSolidClass::Outside})}
}

#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]struct VertexKey(u64,u64,u64);impl VertexKey{fn from(p:Vec3)->Self{Self(p.x.to_bits(),p.y.to_bits(),p.z.to_bits())}}
#[derive(Clone,Copy)]struct DirectedEdge{face_id:usize,from:VertexKey,to:VertexKey}
enum RayHit{Miss,Hit,Boundary,Indeterminate}

fn ray_triangle(origin:Vec3,dir:Vec3,t:Triangle3,tol:f64)->RayHit{let e1=t.b.sub(t.a);let e2=t.c.sub(t.a);let h=dir.cross(e2);let det=e1.dot(h);let scale=e1.length()*e2.length();if!det.is_finite()||!scale.is_finite(){return RayHit::Indeterminate;}if det.abs()<=tol*scale.max(f64::MIN_POSITIVE){return RayHit::Indeterminate;}let inv=1./det;let s=origin.sub(t.a);let u=inv*s.dot(h);let q=s.cross(e1);let v=inv*dir.dot(q);let tau=inv*e2.dot(q);if!u.is_finite()||!v.is_finite()||!tau.is_finite(){return RayHit::Indeterminate;}if tau< -tol{return RayHit::Miss;}let bary_tol=tol.max(1e-14);if u< -bary_tol||v< -bary_tol||u+v>1.+bary_tol{return RayHit::Miss;}if u<=bary_tol||v<=bary_tol||(1.-u-v)<=bary_tol{return RayHit::Boundary;}RayHit::Hit}

#[cfg(test)]
mod tests{use super::*;fn tetra()->PolyhedralSolid{let a=Vec3::new(0.,0.,0.);let b=Vec3::new(1.,0.,0.);let c=Vec3::new(0.,1.,0.);let d=Vec3::new(0.,0.,1.);PolyhedralSolid{triangles:vec![Triangle3{a:b,b:c,c:d},Triangle3{a:a,b:d,c:c},Triangle3{a:a,b:b,c:d},Triangle3{a:a,b:c,c:b}]}}#[test]fn tetrahedron_has_expected_volume_area_and_centroid(){let s=tetra();assert!((s.volume(1e-12).unwrap()-1./6.).abs()<1e-12);assert!((s.centroid(1e-12).unwrap().x-0.25).abs()<1e-12);assert!((s.centroid(1e-12).unwrap().y-0.25).abs()<1e-12);assert!((s.centroid(1e-12).unwrap().z-0.25).abs()<1e-12);assert!((s.surface_area().unwrap()-(3.+3f64.sqrt())/2.).abs()<1e-12);}#[test]fn tetra_point_classification(){let s=tetra();assert_eq!(s.classify_point(Vec3::new(0.1,0.1,0.1),Vec3::new(1.,0.2,0.3),1e-12).unwrap(),PointSolidClass::Inside);assert_eq!(s.classify_point(Vec3::new(2.,2.,2.),Vec3::new(1.,0.2,0.3),1e-12).unwrap(),PointSolidClass::Outside);}#[test]fn open_shell_is_rejected(){let mut s=tetra();s.triangles.pop();assert_eq!(s.validate_closed_oriented(1e-12),Err(SolidError::OpenShell));}}
