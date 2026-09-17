//! Deterministic point/line/segment/plane/analytic-surface distance operations.
//!
//! Results carry closest points and parameters where meaningful. Parallel and
//! non-unique cases remain explicitly classified; no arbitrary representative
//! is promoted to mathematical truth.

use super::{
    analytic::{Cylinder3, Plane3, Ray3},
    conics3d::{Circle3, Sphere3},
    segments::Segment3,
    vec::Vec3,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DistanceStatus { Unique, Intersecting, Parallel, NonUnique, Indeterminate }
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClosestPoint3 { pub point: Vec3, pub parameter: f64 }
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Distance3 { pub distance: f64, pub first: ClosestPoint3, pub second: Option<ClosestPoint3>, pub status: DistanceStatus }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DistanceError { NonFinite, Degenerate, InvalidParameter, Overflow }

fn checked_point(origin:Vec3,direction:Vec3,t:f64)->Result<Vec3,DistanceError>{if!t.is_finite(){return Err(DistanceError::Overflow);}let p=origin.add(direction.scale(t));if p.is_finite(){Ok(p)}else{Err(DistanceError::Overflow)}}

pub fn point_point(a:Vec3,b:Vec3)->Result<f64,DistanceError>{if!a.is_finite()||!b.is_finite(){return Err(DistanceError::NonFinite);}let d=a.sub(b).length();if d.is_finite(){Ok(d)}else{Err(DistanceError::Overflow)}}

pub fn point_line(point:Vec3,origin:Vec3,direction:Vec3)->Result<Distance3,DistanceError>{if!point.is_finite()||!origin.is_finite()||!direction.is_finite(){return Err(DistanceError::NonFinite);}let unit=direction.normalized().map_err(|_|DistanceError::Degenerate)?;let t=point.sub(origin).dot(unit);if!t.is_finite(){return Err(DistanceError::Overflow);}let q=checked_point(origin,unit,t)?;let d=point.sub(q).length();if!d.is_finite(){return Err(DistanceError::Overflow);}Ok(Distance3{distance:d,first:ClosestPoint3{point,parameter:0.},second:Some(ClosestPoint3{point:q,parameter:t}),status:if d==0.{DistanceStatus::Intersecting}else{DistanceStatus::Unique}})}

pub fn point_ray(point:Vec3,ray:&Ray3)->Result<Distance3,DistanceError>{ray.validate().map_err(|_|DistanceError::Degenerate)?;let unit=ray.unit_direction().map_err(|_|DistanceError::Degenerate)?;let t=point.sub(ray.origin).dot(unit).max(0.);let q=checked_point(ray.origin,unit,t)?;let d=point.sub(q).length();if!d.is_finite(){return Err(DistanceError::Overflow);}Ok(Distance3{distance:d,first:ClosestPoint3{point,parameter:0.},second:Some(ClosestPoint3{point:q,parameter:t}),status:if d==0.{DistanceStatus::Intersecting}else{DistanceStatus::Unique}})}

pub fn point_segment(point:Vec3,segment:&Segment3)->Result<Distance3,DistanceError>{let q=segment.closest_point(point).map_err(|_|DistanceError::Degenerate)?;let t=segment.supporting_parameter(q).map_err(|_|DistanceError::Overflow)?;let d=point.sub(q).length();if!d.is_finite(){return Err(DistanceError::Overflow);}Ok(Distance3{distance:d,first:ClosestPoint3{point,parameter:0.},second:Some(ClosestPoint3{point:q,parameter:t}),status:if d==0.{DistanceStatus::Intersecting}else{DistanceStatus::Unique}})}

pub fn point_plane(point:Vec3,plane:&Plane3)->Result<Distance3,DistanceError>{let q=plane.project_point(point).map_err(|_|DistanceError::Degenerate)?;let d=point.sub(q).length();if!d.is_finite(){return Err(DistanceError::Overflow);}Ok(Distance3{distance:d,first:ClosestPoint3{point,parameter:0.},second:Some(ClosestPoint3{point:q,parameter:0.}),status:if d==0.{DistanceStatus::Intersecting}else{DistanceStatus::Unique}})}

pub fn point_circle(point:Vec3,circle:&Circle3)->Result<Distance3,DistanceError>{if!point.is_finite(){return Err(DistanceError::NonFinite);}let n=circle.unit_normal().map_err(|_|DistanceError::Degenerate)?;let q=point.sub(circle.center);let axial=q.dot(n);let radial=q.sub(n.scale(axial));let rl=radial.length();if rl==0.{let d=circle.radius.hypot(axial);return Ok(Distance3{distance:d,first:ClosestPoint3{point,parameter:0.},second:None,status:DistanceStatus::NonUnique});}let surface=circle.center.add(n.scale(axial)).add(radial.scale(circle.radius/rl));let d=point.sub(surface).length();if!d.is_finite(){return Err(DistanceError::Overflow);}Ok(Distance3{distance:d,first:ClosestPoint3{point,parameter:0.},second:Some(ClosestPoint3{point:surface,parameter:0.}),status:if d==0.{DistanceStatus::Intersecting}else{DistanceStatus::Unique}})}

pub fn point_sphere(point:Vec3,sphere:&Sphere3)->Result<Distance3,DistanceError>{if!point.is_finite(){return Err(DistanceError::NonFinite);}sphere.validate().map_err(|_|DistanceError::Degenerate)?;let q=point.sub(sphere.center);let l=q.length();if l==0.{return Ok(Distance3{distance:sphere.radius,first:ClosestPoint3{point,parameter:0.},second:None,status:DistanceStatus::NonUnique});}let surface=sphere.center.add(q.scale(sphere.radius/l));let d=(l-sphere.radius).abs();if!d.is_finite(){return Err(DistanceError::Overflow);}Ok(Distance3{distance:d,first:ClosestPoint3{point,parameter:0.},second:Some(ClosestPoint3{point:surface,parameter:0.}),status:if d==0.{DistanceStatus::Intersecting}else{DistanceStatus::Unique}})}

pub fn point_cylinder(point:Vec3,cylinder:&Cylinder3)->Result<Distance3,DistanceError>{if!point.is_finite(){return Err(DistanceError::NonFinite);}let n=cylinder.unit_axis().map_err(|_|DistanceError::Degenerate)?;let q=point.sub(cylinder.origin);let axial=q.dot(n);let radial=q.sub(n.scale(axial));let rl=radial.length();if rl==0.{return Ok(Distance3{distance:cylinder.radius,first:ClosestPoint3{point,parameter:0.},second:None,status:DistanceStatus::NonUnique});}let surface=cylinder.origin.add(n.scale(axial)).add(radial.scale(cylinder.radius/rl));let d=(rl-cylinder.radius).abs();if!d.is_finite(){return Err(DistanceError::Overflow);}Ok(Distance3{distance:d,first:ClosestPoint3{point,parameter:0.},second:Some(ClosestPoint3{point:surface,parameter:axial}),status:if d==0.{DistanceStatus::Intersecting}else{DistanceStatus::Unique}})}

pub fn line_line_3d(a_origin:Vec3,a_dir:Vec3,b_origin:Vec3,b_dir:Vec3,tol:f64)->Result<Distance3,DistanceError>{if!a_origin.is_finite()||!a_dir.is_finite()||!b_origin.is_finite()||!b_dir.is_finite(){return Err(DistanceError::NonFinite);}if!tol.is_finite()||tol<0.{return Err(DistanceError::InvalidParameter);}let u=a_dir.normalized().map_err(|_|DistanceError::Degenerate)?;let v=b_dir.normalized().map_err(|_|DistanceError::Degenerate)?;let w=b_origin.sub(a_origin);let c=u.dot(v);let den=1.-c*c;if den.abs()<=tol{let t=w.dot(u);let pa=checked_point(a_origin,u,t)?;let d=pa.sub(b_origin).length();if!d.is_finite(){return Err(DistanceError::Overflow);}return Ok(Distance3{distance:d,first:ClosestPoint3{point:pa,parameter:t},second:Some(ClosestPoint3{point:b_origin,parameter:0.}),status:if d==0.{DistanceStatus::Intersecting}else{DistanceStatus::Parallel}})}let wu=w.dot(u);let wv=w.dot(v);let t=(wu-c*wv)/den;let s=(c*wu-wv)/den;let pa=checked_point(a_origin,u,t)?;let pb=checked_point(b_origin,v,s)?;let d=pa.sub(pb).length();if!d.is_finite(){return Err(DistanceError::Overflow);}Ok(Distance3{distance:d,first:ClosestPoint3{point:pa,parameter:t},second:Some(ClosestPoint3{point:pb,parameter:s}),status:if d==0.{DistanceStatus::Intersecting}else{DistanceStatus::Unique}})}

#[cfg(test)]
mod tests{use super::*;#[test]fn point_line_projection_is_exact(){let r=point_line(Vec3::new(2.,3.,0.),Vec3::new(0.,0.,0.),Vec3::new(1.,0.,0.)).unwrap();assert!((r.distance-3.).abs()<1e-14);assert!((r.second.unwrap().parameter-2.).abs()<1e-14);}#[test]fn point_sphere_center_is_nonunique(){let s=Sphere3{center:Vec3::new(0.,0.,0.),radius:2.};assert_eq!(point_sphere(Vec3::new(0.,0.,0.),&s).unwrap().status,DistanceStatus::NonUnique);}#[test]fn skew_lines_return_closest_pair(){let r=line_line_3d(Vec3::new(0.,0.,0.),Vec3::new(1.,0.,0.),Vec3::new(0.,1.,1.),Vec3::new(0.,1.,0.),1e-12).unwrap();assert!((r.distance-1.).abs()<1e-12);assert_eq!(r.status,DistanceStatus::Unique);}#[test]fn direction_scale_does_not_change_distance(){let a=line_line_3d(Vec3::new(0.,0.,0.),Vec3::new(1.,0.,0.),Vec3::new(0.,1.,1.),Vec3::new(0.,5.,0.),1e-12).unwrap();let b=line_line_3d(Vec3::new(0.,0.,0.),Vec3::new(1000.,0.,0.),Vec3::new(0.,1.,1.),Vec3::new(0.,0.001,0.),1e-12).unwrap();assert!((a.distance-b.distance).abs()<1e-12);}}
