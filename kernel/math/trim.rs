//! Parameter-space trimming mathematics for analytic line/arc trim curves.
//!
//! A trim loop is an ordered closed boundary in the surface parameter domain.
//! Classification is performed in parameter space; no display mesh is involved.
//! Boundary cases are explicit, and unsupported pairwise degeneracies return
//! `Indeterminate` rather than an arbitrary inside/outside decision.

use super::{geometry::{Arc,Point},predicates::Tri};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum TrimError{NonFinite,InvalidDomain,Degenerate,NotClosed,SelfIntersection,Indeterminate}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum RegionClass{Inside,Outside,OnBoundary,Indeterminate}
#[derive(Clone,Copy,Debug,PartialEq)]
pub enum TrimCurve2{Line{start:Point,end:Point},Arc(Arc)}
#[derive(Clone,Debug,PartialEq)]
pub struct TrimLoop2{pub curves:Vec<TrimCurve2>}

impl TrimCurve2{
 pub fn validate(&self)->Result<(),TrimError>{match self{Self::Line{start,end}=>{if !start.x.is_finite()||!start.y.is_finite()||!end.x.is_finite()||!end.y.is_finite(){Err(TrimError::NonFinite)}else if start.distance(*end)==0.{Err(TrimError::Degenerate)}else{Ok(())}},Self::Arc(a)=>a.validate().map_err(|_|TrimError::Degenerate)}}
 pub fn start(&self)->Point{match self{Self::Line{start,..}|Self::Arc(Arc{center:start,..})=>match self{Self::Line{start,..}=>*start,Self::Arc(a)=>a.start_point()} }}
 pub fn end(&self)->Point{match self{Self::Line{end,..}=>*end,Self::Arc(a)=>a.end_point()}}
 pub fn point_at(&self,t:f64)->Result<Point,TrimError>{if!t.is_finite()||!(0.0..=1.0).contains(&t){return Err(TrimError::InvalidDomain);}match self{Self::Line{start,end}=>Ok(Point{x:start.x+(end.x-start.x)*t,y:start.y+(end.y-start.y)*t}),Self::Arc(a)=>Ok(a.point_at(t))}}
 pub fn signed_area_contribution(&self)->Result<f64,TrimError>{self.validate()?;match self{Self::Line{start,end}=>Ok(.5*(start.x*end.y-start.y*end.x)),Self::Arc(a)=>{let t0=a.start_angle;let t1=a.end_angle;let p0=a.start_point();let p1=a.end_point();Ok(.5*(a.radius*a.radius*(t1-t0)+a.center.x*(p1.y-p0.y)-a.center.y*(p1.x-p0.x)))}}}
}

impl TrimLoop2{
 pub fn validate(&self,tolerance:f64)->Result<(),TrimError>{if!tolerance.is_finite()||tolerance<0.{return Err(TrimError::InvalidDomain);}if self.curves.is_empty(){return Err(TrimError::Degenerate);}for c in &self.curves{c.validate()?;}for pair in self.curves.windows(2){if pair[0].end().distance(pair[1].start())>tolerance{return Err(TrimError::NotClosed);}}if self.curves.last().unwrap().end().distance(self.curves[0].start())>tolerance{return Err(TrimError::NotClosed);}if self.has_self_intersection(tolerance)==Tri::True{return Err(TrimError::SelfIntersection);}Ok(())}
 pub fn signed_area(&self,tolerance:f64)->Result<f64,TrimError>{self.validate(tolerance)?;let mut area=0.;for c in &self.curves{area+=c.signed_area_contribution()?;}if area.is_finite(){Ok(area)}else{Err(TrimError::NonFinite)}}
 pub fn orientation(&self,tolerance:f64)->Result<Tri,TrimError>{let area=self.signed_area(tolerance)?;let scale=self.curves.iter().map(|c|{let p=c.start();p.x.abs().max(p.y.abs())}).fold(0.,f64::max).max(1.);let eps=tolerance*scale*scale;if area>eps{Ok(Tri::True)}else if area< -eps{Ok(Tri::False)}else{Ok(Tri::Indeterminate)}}
 pub fn classify_point(&self,p:Point,tolerance:f64)->Result<RegionClass,TrimError>{self.validate(tolerance)?;if !p.x.is_finite()||!p.y.is_finite(){return Err(TrimError::NonFinite);}let mut crossings=0usize;for c in &self.curves{match ray_crossing(c,p,tolerance){RayHit::Boundary=>return Ok(RegionClass::OnBoundary),RayHit::Cross=>crossings+=1,RayHit::None=>{},RayHit::Indeterminate=>return Ok(RegionClass::Indeterminate)}}Ok(if crossings%2==1{RegionClass::Inside}else{RegionClass::Outside})}
 fn has_self_intersection(&self,tolerance:f64)->Tri{for i in 0..self.curves.len(){for j in i+1..self.curves.len(){if j==i+1||(i==0&&j+1==self.curves.len()){continue;}match curve_pair_intersects(&self.curves[i],&self.curves[j],tolerance){Tri::True=>return Tri::True,Tri::Indeterminate=>return Tri::Indeterminate,Tri::False=>{}}}}Tri::False}
}

enum RayHit{None,Cross,Boundary,Indeterminate}
fn ray_crossing(c:&TrimCurve2,p:Point,tol:f64)->RayHit{match c{TrimCurve2::Line{start,end}=>{let minx=start.x.min(end.x);let maxx=start.x.max(end.x);let miny=start.y.min(end.y);let maxy=start.y.max(end.y);let d=end.sub(*start);let cross=d.cross(p.sub(*start));if cross.abs()<=tol*(d.length()+p.sub(*start).length()).max(f64::MIN_POSITIVE)&&p.x>=minx-tol&&p.x<=maxx+tol&&p.y>=miny-tol&&p.y<=maxy+tol{return RayHit::Boundary;}if (start.y>p.y)!=(end.y>p.y){let x=start.x+(p.y-start.y)*d.x/d.y;if !x.is_finite(){return RayHit::Indeterminate;}if x>p.x{return RayHit::Cross;}}RayHit::None},TrimCurve2::Arc(a)=>{let rel=(p.y-a.center.y)/a.radius;if rel.abs()>1.+tol{return RayHit::None;}if rel.abs()>=1.-tol{return RayHit::Indeterminate;}let theta=rel.clamp(-1.,1.).asin();let candidates=[theta,std::f64::consts::PI-theta];let mut hit=0;for angle in candidates{if a.contains_angle(angle){let x=a.center.x+a.radius*angle.cos();if (x-p.x).abs()<=tol{return RayHit::Boundary;}if x>p.x{hit+=1;}}}if hit%2==1{RayHit::Cross}else{RayHit::None}}}}

fn curve_pair_intersects(a:&TrimCurve2,b:&TrimCurve2,tol:f64)->Tri{match(a,b){(TrimCurve2::Line{start:a0,end:a1},TrimCurve2::Line{start:b0,end:b1})=>segment_intersection(*a0,*a1,*b0,*b1,tol),_=>Tri::Indeterminate}}
fn segment_intersection(a:Point,b:Point,c:Point,d:Point,tol:f64)->Tri{let ab=b.sub(a);let cd=d.sub(c);let c1=ab.cross(c.sub(a));let c2=ab.cross(d.sub(a));let c3=cd.cross(a.sub(c));let c4=cd.cross(b.sub(c));let scale=(ab.length()*cd.length()).max(f64::MIN_POSITIVE);let e=tol*scale;let s1=if c1>e{1}else if c1< -e{-1}else{0};let s2=if c2>e{1}else if c2< -e{-1}else{0};let s3=if c3>e{1}else if c3< -e{-1}else{0};let s4=if c4>e{1}else if c4< -e{-1}else{0};if s1==0&&is_on_segment(a,b,c,tol)||s2==0&&is_on_segment(a,b,d,tol)||s3==0&&is_on_segment(c,d,a,tol)||s4==0&&is_on_segment(c,d,b,tol){Tri::True}else if s1*s2<0&&s3*s4<0{Tri::True}else{Tri::False}}
fn is_on_segment(a:Point,b:Point,p:Point,tol:f64)->bool{let minx=a.x.min(b.x)-tol;let maxx=a.x.max(b.x)+tol;let miny=a.y.min(b.y)-tol;let maxy=a.y.max(b.y)+tol;p.x>=minx&&p.x<=maxx&&p.y>=miny&&p.y<=maxy}

#[cfg(test)]
mod tests{use super::*;#[test]fn closed_square_has_positive_orientation_and_inside_test(){let q=TrimLoop2{curves:vec![TrimCurve2::Line{start:Point{x:0.,y:0.},end:Point{x:1.,y:0.}},TrimCurve2::Line{start:Point{x:1.,y:0.},end:Point{x:1.,y:1.}},TrimCurve2::Line{start:Point{x:1.,y:1.},end:Point{x:0.,y:1.}},TrimCurve2::Line{start:Point{x:0.,y:1.},end:Point{x:0.,y:0.}}]};assert_eq!(q.validate(1e-12),Ok(()));assert_eq!(q.orientation(1e-12).unwrap(),Tri::True);assert_eq!(q.classify_point(Point{x:.5,y:.5},1e-12).unwrap(),RegionClass::Inside);assert_eq!(q.classify_point(Point{x:2.,y:.5},1e-12).unwrap(),RegionClass::Outside);}#[test]fn boundary_is_not_inside_or_outside(){let q=TrimLoop2{curves:vec![TrimCurve2::Line{start:Point{x:0.,y:0.},end:Point{x:1.,y:0.}},TrimCurve2::Line{start:Point{x:1.,y:0.},end:Point{x:1.,y:1.}},TrimCurve2::Line{start:Point{x:1.,y:1.},end:Point{x:0.,y:1.}},TrimCurve2::Line{start:Point{x:0.,y:1.},end:Point{x:0.,y:0.}}]};assert_eq!(q.classify_point(Point{x:0.,y:.5},1e-12).unwrap(),RegionClass::OnBoundary);}#[test]fn nonclosed_loop_is_rejected(){let q=TrimLoop2{curves:vec![TrimCurve2::Line{start:Point{x:0.,y:0.},end:Point{x:1.,y:0.}},TrimCurve2::Line{start:Point{x:1.,y:0.},end:Point{x:1.,y:1.}}]};assert_eq!(q.validate(1e-12),Err(TrimError::NotClosed));}}
