//! Analytic intersection mathematics for supported primitive families.
//!
//! Each operation classifies geometric degeneracy explicitly. Tolerances are
//! dimensionless relative inputs applied to the natural scale of each equation.

use super::{
    analytic::{Ellipse2, Plane3},
    conics3d::{Circle3, Sphere3},
    geometry::{Circle, Line},
    predicates::Tri,
    vec::{Vec2, Vec3},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntersectionKind { None, Point, MultiplePoints, Circle, Line, Tangent, Parallel, Coincident, Skew, Degenerate, Indeterminate }

#[derive(Clone, Debug, PartialEq)]
pub struct Intersection2D { pub kind: IntersectionKind, pub points: Vec<Vec2>, pub parameters: Vec<f64> }
#[derive(Clone, Debug, PartialEq)]
pub struct Intersection3D { pub kind: IntersectionKind, pub points: Vec<Vec3>, pub parameters: Vec<f64> }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line2 { pub origin: Vec2, pub direction: Vec2 }
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line3 { pub origin: Vec3, pub direction: Vec3 }

impl Line2 { fn valid(&self)->bool{self.origin.is_finite()&&self.direction.is_finite()&&self.direction.length()>0.0} }
impl Line3 { fn valid(&self)->bool{self.origin.is_finite()&&self.direction.is_finite()&&self.direction.length()>0.0} }
fn valid_tol(t:f64)->bool{t.is_finite()&&t>=0.0}

pub fn line_line_2d(a:Line2,b:Line2,tol:f64)->Intersection2D{
 if !valid_tol(tol)||!a.valid()||!b.valid(){return Intersection2D{kind:IntersectionKind::Degenerate,points:vec![],parameters:vec![]};}
 let denom=a.direction.cross(b.direction);let scale=a.direction.length()*b.direction.length();
 if denom.abs()<=tol*scale{let offset=b.origin.sub(a.origin);let offset_scale=offset.length()*a.direction.length();if offset.cross(a.direction).abs()<=tol*offset_scale.max(f64::MIN_POSITIVE){Intersection2D{kind:IntersectionKind::Coincident,points:vec![],parameters:vec![]}}else{Intersection2D{kind:IntersectionKind::Parallel,points:vec![],parameters:vec![]}}}
 else{let offset=b.origin.sub(a.origin);let ta=offset.cross(b.direction)/denom;let tb=offset.cross(a.direction)/denom;let p=a.origin.add(a.direction.scale(ta));if p.is_finite()&&ta.is_finite()&&tb.is_finite(){Intersection2D{kind:IntersectionKind::Point,points:vec![p],parameters:vec![ta,tb]}}else{Intersection2D{kind:IntersectionKind::Indeterminate,points:vec![],parameters:vec![]}}}
}

pub fn line_circle_2d(line:Line2,circle:Circle,tol:f64)->Intersection2D{
 if !valid_tol(tol)||!line.valid()||circle.validate().is_err(){return Intersection2D{kind:IntersectionKind::Degenerate,points:vec![],parameters:vec![]};}
 let d=line.direction;let f=line.origin.sub(circle.center);let a=d.dot(d);let b=2.0*f.dot(d);let c=f.dot(f)-circle.radius*circle.radius;let disc=b*b-4.0*a*c;let scale=((b*b).abs()+(4.0*a*c).abs()).max(f64::MIN_POSITIVE);let eps=tol*scale;
 if !disc.is_finite(){return Intersection2D{kind:IntersectionKind::Indeterminate,points:vec![],parameters:vec![]};}
 if disc< -eps{return Intersection2D{kind:IntersectionKind::None,points:vec![],parameters:vec![]};}
 if disc.abs()<=eps{let t=-b/(2.0*a);let p=line.origin.add(d.scale(t));return if p.is_finite()&&t.is_finite(){Intersection2D{kind:IntersectionKind::Tangent,points:vec![p],parameters:vec![t]}}else{Intersection2D{kind:IntersectionKind::Indeterminate,points:vec![],parameters:vec![]}};}
 let root=disc.sqrt();let t1=(-b-root)/(2.0*a);let t2=(-b+root)/(2.0*a);let p1=line.origin.add(d.scale(t1));let p2=line.origin.add(d.scale(t2));if !p1.is_finite()||!p2.is_finite(){return Intersection2D{kind:IntersectionKind::Indeterminate,points:vec![],parameters:vec![]};}Intersection2D{kind:IntersectionKind::MultiplePoints,points:vec![p1,p2],parameters:vec![t1,t2]}
}

pub fn circle_circle_2d(a:Circle,b:Circle,tol:f64)->Intersection2D{
 if !valid_tol(tol)||a.validate().is_err()||b.validate().is_err(){return Intersection2D{kind:IntersectionKind::Degenerate,points:vec![],parameters:vec![]};}
 let delta=b.center.sub(a.center);let d=delta.length();let scale=a.radius.max(b.radius).max(d);let e=tol*scale;
 if d<=e{if(a.radius-b.radius).abs()<=e{return Intersection2D{kind:IntersectionKind::Coincident,points:vec![],parameters:vec![]};}return Intersection2D{kind:IntersectionKind::None,points:vec![],parameters:vec![]};}
 if d>a.radius+b.radius+e||d<=(a.radius-b.radius).abs()-e{return Intersection2D{kind:IntersectionKind::None,points:vec![],parameters:vec![]};}
 let x=(a.radius*a.radius-b.radius*b.radius+d*d)/(2.0*d);let h2=a.radius*a.radius-x*x;let h_tol=tol*scale*scale;
 if h2.abs()<=h_tol{let p=a.center.add(delta.scale(x/d));return Intersection2D{kind:IntersectionKind::Tangent,points:vec![p],parameters:vec![]};}
 if h2<0.0{return Intersection2D{kind:IntersectionKind::Indeterminate,points:vec![],parameters:vec![]};}
 let h=h2.sqrt();let base=a.center.add(delta.scale(x/d));let perp=Vec2::new(-delta.y/d,delta.x/d);Intersection2D{kind:IntersectionKind::MultiplePoints,points:vec![base.add(perp.scale(h)),base.sub(perp.scale(h))],parameters:vec![]}
}

pub fn ellipse_line_2d(ellipse:Ellipse2,line:Line2,tol:f64)->Intersection2D{
 if !valid_tol(tol)||!line.valid()||ellipse.validate().is_err(){return Intersection2D{kind:IntersectionKind::Degenerate,points:vec![],parameters:vec![]};}
 let c=ellipse.rotation.cos();let s=ellipse.rotation.sin();let local=|p:Vec2|{let d=p.sub(ellipse.center);Vec2::new(c*d.x+s*d.y,-s*d.x+c*d.y)};let o=local(line.origin);let q=Vec2::new(c*line.direction.x+s*line.direction.y,-s*line.direction.x+c*line.direction.y);
 let a=(q.x/ellipse.semi_axis_a).powi(2)+(q.y/ellipse.semi_axis_b).powi(2);let b=2.0*((o.x*q.x)/(ellipse.semi_axis_a*ellipse.semi_axis_a)+(o.y*q.y)/(ellipse.semi_axis_b*ellipse.semi_axis_b));let cc=(o.x/ellipse.semi_axis_a).powi(2)+(o.y/ellipse.semi_axis_b).powi(2)-1.0;let disc=b*b-4.0*a*cc;let eps=tol*((b*b).abs()+(4.0*a*cc).abs()).max(f64::MIN_POSITIVE);
 if disc< -eps{return Intersection2D{kind:IntersectionKind::None,points:vec![],parameters:vec![]};}if disc.abs()<=eps{let t=-b/(2.0*a);return Intersection2D{kind:IntersectionKind::Tangent,points:vec![line.origin.add(line.direction.scale(t))],parameters:vec![t]};}if disc<0.0{return Intersection2D{kind:IntersectionKind::Indeterminate,points:vec![],parameters:vec![]};}
 let h=disc.sqrt();let t1=(-b-h)/(2.0*a);let t2=(-b+h)/(2.0*a);Intersection2D{kind:IntersectionKind::MultiplePoints,points:vec![line.origin.add(line.direction.scale(t1)),line.origin.add(line.direction.scale(t2))],parameters:vec![t1,t2]}
}

pub fn line_plane_3d(line:Line3,plane:Plane3,tol:f64)->Intersection3D{
 if !valid_tol(tol)||!line.valid()||plane.validate().is_err(){return Intersection3D{kind:IntersectionKind::Degenerate,points:vec![],parameters:vec![]};}
 let n=plane.unit_normal().unwrap();let den=n.dot(line.direction);let num=n.dot(plane.origin.sub(line.origin));let eps=tol*line.direction.length();
 if den.abs()<=eps{let offset_scale=plane.origin.sub(line.origin).length().max(f64::MIN_POSITIVE);if num.abs()<=tol*offset_scale{return Intersection3D{kind:IntersectionKind::Coincident,points:vec![],parameters:vec![]};}return Intersection3D{kind:IntersectionKind::Parallel,points:vec![],parameters:vec![]};}
 let t=num/den;let p=line.origin.add(line.direction.scale(t));if p.is_finite()&&t.is_finite(){Intersection3D{kind:IntersectionKind::Point,points:vec![p],parameters:vec![t]}}else{Intersection3D{kind:IntersectionKind::Indeterminate,points:vec![],parameters:vec![]}}
}

pub fn plane_plane_3d(a:Plane3,b:Plane3,tol:f64)->Intersection3D{
 if !valid_tol(tol)||a.validate().is_err()||b.validate().is_err(){return Intersection3D{kind:IntersectionKind::Degenerate,points:vec![],parameters:vec![]};}
 let n1=a.unit_normal().unwrap();let n2=b.unit_normal().unwrap();let dir=n1.cross(n2);let dl=dir.length();if dl<=tol{let dist=a.signed_distance(b.origin).unwrap();if dist.abs()<=tol*b.origin.sub(a.origin).length().max(f64::MIN_POSITIVE){return Intersection3D{kind:IntersectionKind::Coincident,points:vec![],parameters:vec![]};}return Intersection3D{kind:IntersectionKind::Parallel,points:vec![],parameters:vec![]};}
 let d1=n1.dot(a.origin);let d2=n2.dot(b.origin);let numerator=n2.scale(d1).sub(n1.scale(d2));let point=numerator.cross(dir).scale(1.0/(dl*dl));if point.is_finite(){Intersection3D{kind:IntersectionKind::Line,points:vec![point],parameters:vec![0.0]}}else{Intersection3D{kind:IntersectionKind::Indeterminate,points:vec![],parameters:vec![]}}
}

pub fn line_sphere_3d(line:Line3,sphere:Sphere3,tol:f64)->Intersection3D{
 if !valid_tol(tol)||!line.valid()||sphere.validate().is_err(){return Intersection3D{kind:IntersectionKind::Degenerate,points:vec![],parameters:vec![]};}
 let f=line.origin.sub(sphere.center);let a=line.direction.dot(line.direction);let b=2.0*f.dot(line.direction);let c=f.dot(f)-sphere.radius*sphere.radius;let disc=b*b-4.0*a*c;let eps=tol*((b*b).abs()+(4.0*a*c).abs()).max(f64::MIN_POSITIVE);if disc< -eps{return Intersection3D{kind:IntersectionKind::None,points:vec![],parameters:vec![]};}if disc.abs()<=eps{let t=-b/(2.0*a);return Intersection3D{kind:IntersectionKind::Tangent,points:vec![line.origin.add(line.direction.scale(t))],parameters:vec![t]};}if disc<0.0{return Intersection3D{kind:IntersectionKind::Indeterminate,points:vec![],parameters:vec![]};}let h=disc.sqrt();let t1=(-b-h)/(2.0*a);let t2=(-b+h)/(2.0*a);Intersection3D{kind:IntersectionKind::MultiplePoints,points:vec![line.origin.add(line.direction.scale(t1)),line.origin.add(line.direction.scale(t2))],parameters:vec![t1,t2]}
}

pub fn plane_sphere_3d(plane:Plane3,sphere:Sphere3,tol:f64)->Intersection3D{
 if !valid_tol(tol)||plane.validate().is_err()||sphere.validate().is_err(){return Intersection3D{kind:IntersectionKind::Degenerate,points:vec![],parameters:vec![]};}let d=plane.signed_distance(sphere.center).unwrap();let e=tol*sphere.radius;if d.abs()>sphere.radius+e{return Intersection3D{kind:IntersectionKind::None,points:vec![],parameters:vec![]};}if(d.abs()-sphere.radius).abs()<=e{let p=plane.project_point(sphere.center).unwrap();return Intersection3D{kind:IntersectionKind::Tangent,points:vec![p],parameters:vec![]};}let r=(sphere.radius*sphere.radius-d*d).sqrt();Intersection3D{kind:IntersectionKind::Circle,points:vec![plane.project_point(sphere.center).unwrap()],parameters:vec![r]}
}

pub fn plane_circle_3d(plane:Plane3,circle:Circle3,tol:f64)->Intersection3D{
 if !valid_tol(tol)||plane.validate().is_err()||circle.validate().is_err(){return Intersection3D{kind:IntersectionKind::Degenerate,points:vec![],parameters:vec![]};}let cn=circle.unit_normal().unwrap();let pn=plane.unit_normal().unwrap();let parallel=cn.cross(pn).length();if parallel<=tol{let d=plane.signed_distance(circle.center).unwrap();if d.abs()<=tol*circle.radius{return Intersection3D{kind:IntersectionKind::Coincident,points:vec![],parameters:vec![]};}return Intersection3D{kind:IntersectionKind::None,points:vec![],parameters:vec![]};}
 let d=plane.signed_distance(circle.center).unwrap();let h2=circle.radius*circle.radius-d*d;let h_tol=tol*circle.radius*circle.radius;if h2< -h_tol{return Intersection3D{kind:IntersectionKind::None,points:vec![],parameters:vec![]};}let projected=plane.project_point(circle.center).unwrap();if h2.abs()<=h_tol{return Intersection3D{kind:IntersectionKind::Tangent,points:vec![projected],parameters:vec![]};}let dir=cn.cross(pn).normalized().unwrap();let h=h2.sqrt();Intersection3D{kind:IntersectionKind::MultiplePoints,points:vec![projected.add(dir.scale(h)),projected.sub(dir.scale(h))],parameters:vec![]}
}

pub fn sphere_sphere_3d(a:Sphere3,b:Sphere3,tol:f64)->Intersection3D{
 if !valid_tol(tol)||a.validate().is_err()||b.validate().is_err(){return Intersection3D{kind:IntersectionKind::Degenerate,points:vec![],parameters:vec![]};}let delta=b.center.sub(a.center);let d=delta.length();let scale=a.radius.max(b.radius).max(d);let e=tol*scale;if d<=e{if(a.radius-b.radius).abs()<=e{return Intersection3D{kind:IntersectionKind::Coincident,points:vec![],parameters:vec![]};}return Intersection3D{kind:IntersectionKind::None,points:vec![],parameters:vec![]};}if d>a.radius+b.radius+e||d<=(a.radius-b.radius).abs()-e{return Intersection3D{kind:IntersectionKind::None,points:vec![],parameters:vec![]};}let x=(a.radius*a.radius-b.radius*b.radius+d*d)/(2.0*d);let h2=a.radius*a.radius-x*x;let h_tol=tol*scale*scale;let center=a.center.add(delta.scale(x/d));if h2.abs()<=h_tol{return Intersection3D{kind:IntersectionKind::Tangent,points:vec![center],parameters:vec![]};}if h2<0.0{return Intersection3D{kind:IntersectionKind::Indeterminate,points:vec![],parameters:vec![]};}Intersection3D{kind:IntersectionKind::Circle,points:vec![center],parameters:vec![h2.sqrt()]}
}

pub fn predicate_classification(value:Tri)->IntersectionKind{match value{Tri::True=>IntersectionKind::Point,Tri::False=>IntersectionKind::None,Tri::Indeterminate=>IntersectionKind::Indeterminate}}

#[cfg(test)]
mod tests{use super::*;#[test]fn line_line_crossing_is_exact(){let a=Line2{origin:Vec2::new(0.,0.),direction:Vec2::new(1.,0.)};let b=Line2{origin:Vec2::new(.5,-1.),direction:Vec2::new(0.,1.)};let r=line_line_2d(a,b,1e-12);assert_eq!(r.kind,IntersectionKind::Point);assert_eq!(r.points[0],Vec2::new(.5,0.));}#[test]fn circle_circle_tangent_is_classified(){let a=Circle{center:super::super::geometry::Point{x:0.,y:0.},radius:1.};let b=Circle{center:super::super::geometry::Point{x:2.,y:0.},radius:1.};assert_eq!(circle_circle_2d(a,b,1e-12).kind,IntersectionKind::Tangent);}#[test]fn line_sphere_has_two_roots(){let l=Line3{origin:Vec3::new(-2.,0.,0.),direction:Vec3::new(1.,0.,0.)};let s=Sphere3{center:Vec3::new(0.,0.,0.),radius:1.};let r=line_sphere_3d(l,s,1e-12);assert_eq!(r.kind,IntersectionKind::MultiplePoints);assert_eq!(r.points.len(),2);}#[test]fn plane_plane_crossing_returns_line(){let a=Plane3{origin:Vec3::new(0.,0.,0.),normal:Vec3::new(0.,0.,1.)};let b=Plane3{origin:Vec3::new(0.,0.,0.),normal:Vec3::new(1.,0.,0.)};let r=plane_plane_3d(a,b,1e-12);assert_eq!(r.kind,IntersectionKind::Line);assert!(r.points[0].z.abs()<1e-12);assert!(r.points[0].x.abs()<1e-12);}}
