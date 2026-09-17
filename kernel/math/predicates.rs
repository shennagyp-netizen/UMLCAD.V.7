//! Tri-state geometric predicates for the UMLCAD mathematical authority.
//!
//! `Indeterminate` is deliberately observable. It represents non-finite input,
//! degenerate input, or a determinant inside the caller-provided tolerance band.

use super::vec::{Vec2, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tri { True, False, Indeterminate }
impl Tri {
    pub fn is_true(self)->bool{matches!(self,Self::True)}
    pub fn is_false(self)->bool{matches!(self,Self::False)}
    pub fn is_indeterminate(self)->bool{matches!(self,Self::Indeterminate)}
    pub fn or_false(self)->bool{self.is_true()}
}
fn valid_tolerance(t:f64)->bool{t.is_finite()&&t>=0.0}

pub fn orient2d(a:Vec2,b:Vec2,c:Vec2,tol:f64)->Tri{if!a.is_finite()||!b.is_finite()||!c.is_finite()||!valid_tolerance(tol){return Tri::Indeterminate;}let ab=match b.sub(a).normalized(){Ok(v)=>v,Err(_)=>return Tri::Indeterminate};let ac=match c.sub(a).normalized(){Ok(v)=>v,Err(_)=>return Tri::Indeterminate};let d=ab.cross(ac);if!d.is_finite(){Tri::Indeterminate}else if d>tol{Tri::True}else if d< -tol{Tri::False}else{Tri::Indeterminate}}

pub fn is_collinear2d(a:Vec2,b:Vec2,c:Vec2,tol:f64)->Tri{match orient2d(a,b,c,tol){Tri::True|Tri::False=>Tri::False,Tri::Indeterminate=>Tri::Indeterminate}}
pub fn is_parallel2d(a:Vec2,b:Vec2,tol:f64)->Tri{if!a.is_finite()||!b.is_finite()||!valid_tolerance(tol){return Tri::Indeterminate;}let a=match a.normalized(){Ok(v)=>v,Err(_)=>return Tri::Indeterminate};let b=match b.normalized(){Ok(v)=>v,Err(_)=>return Tri::Indeterminate};let c=a.cross(b).abs();if!c.is_finite(){Tri::Indeterminate}else if c<=tol{Tri::True}else{Tri::False}}
pub fn is_perpendicular2d(a:Vec2,b:Vec2,tol:f64)->Tri{if!a.is_finite()||!b.is_finite()||!valid_tolerance(tol){return Tri::Indeterminate;}let a=match a.normalized(){Ok(v)=>v,Err(_)=>return Tri::Indeterminate};let b=match b.normalized(){Ok(v)=>v,Err(_)=>return Tri::Indeterminate};let d=a.dot(b).abs();if!d.is_finite(){Tri::Indeterminate}else if d<=tol{Tri::True}else{Tri::False}}

pub fn is_between2d(a:Vec2,b:Vec2,p:Vec2,tol:f64)->Tri{if!a.is_finite()||!b.is_finite()||!p.is_finite()||!valid_tolerance(tol){return Tri::Indeterminate;}let raw=b.sub(a);let ab=match raw.normalized(){Ok(v)=>v,Err(_)=>return Tri::Indeterminate};let ap=p.sub(a);let cross=ab.cross(ap).abs();let ap_len=ap.length();if!cross.is_finite()||!ap_len.is_finite(){return Tri::Indeterminate;}if cross>tol*ap_len{return Tri::False;}let proj=ap.dot(ab);let length=raw.length();if!proj.is_finite()||!length.is_finite(){return Tri::Indeterminate;}let ptol=tol*length.max(ap_len).max(f64::MIN_POSITIVE);if proj< -ptol||proj>length+ptol{Tri::False}else{Tri::True}}

pub fn is_same_side_2d(line_a:Vec2,line_b:Vec2,p:Vec2,q:Vec2,tol:f64)->Tri{if!line_a.is_finite()||!line_b.is_finite()||!p.is_finite()||!q.is_finite()||!valid_tolerance(tol){return Tri::Indeterminate;}if line_b.sub(line_a).length()==0.{return Tri::Indeterminate;}match(orient2d(line_a,line_b,p,tol),orient2d(line_a,line_b,q,tol)){(Tri::True,Tri::True)|(Tri::False,Tri::False)=>Tri::True,(Tri::True,Tri::False)|(Tri::False,Tri::True)=>Tri::False,_=>Tri::Indeterminate}}

pub fn orient3d(a:Vec3,b:Vec3,c:Vec3,d:Vec3,tol:f64)->Tri{if!a.is_finite()||!b.is_finite()||!c.is_finite()||!d.is_finite()||!valid_tolerance(tol){return Tri::Indeterminate;}let ab=match b.sub(a).normalized(){Ok(v)=>v,Err(_)=>return Tri::Indeterminate};let ac=match c.sub(a).normalized(){Ok(v)=>v,Err(_)=>return Tri::Indeterminate};let ad=match d.sub(a).normalized(){Ok(v)=>v,Err(_)=>return Tri::Indeterminate};let det=ab.cross(ac).dot(ad);if!det.is_finite(){Tri::Indeterminate}else if det>tol{Tri::True}else if det< -tol{Tri::False}else{Tri::Indeterminate}}
pub fn is_coplanar(a:Vec3,b:Vec3,c:Vec3,d:Vec3,tol:f64)->Tri{match orient3d(a,b,c,d,tol){Tri::True|Tri::False=>Tri::False,Tri::Indeterminate=>Tri::Indeterminate}}
pub fn is_parallel3d(a:Vec3,b:Vec3,tol:f64)->Tri{if!a.is_finite()||!b.is_finite()||!valid_tolerance(tol){return Tri::Indeterminate;}let a=match a.normalized(){Ok(v)=>v,Err(_)=>return Tri::Indeterminate};let b=match b.normalized(){Ok(v)=>v,Err(_)=>return Tri::Indeterminate};let c=a.cross(b).length();if!c.is_finite(){Tri::Indeterminate}else if c<=tol{Tri::True}else{Tri::False}}
pub fn is_perpendicular3d(a:Vec3,b:Vec3,tol:f64)->Tri{if!a.is_finite()||!b.is_finite()||!valid_tolerance(tol){return Tri::Indeterminate;}let a=match a.normalized(){Ok(v)=>v,Err(_)=>return Tri::Indeterminate};let b=match b.normalized(){Ok(v)=>v,Err(_)=>return Tri::Indeterminate};let d=a.dot(b).abs();if!d.is_finite(){Tri::Indeterminate}else if d<=tol{Tri::True}else{Tri::False}}
pub fn is_between3d(a:Vec3,b:Vec3,p:Vec3,tol:f64)->Tri{if!a.is_finite()||!b.is_finite()||!p.is_finite()||!valid_tolerance(tol){return Tri::Indeterminate;}let raw=b.sub(a);let ab=match raw.normalized(){Ok(v)=>v,Err(_)=>return Tri::Indeterminate};let ap=p.sub(a);let cross=ab.cross(ap).length();let apl=ap.length();if!cross.is_finite()||!apl.is_finite(){return Tri::Indeterminate;}if cross>tol*apl{return Tri::False;}let proj=ap.dot(ab);let l=raw.length();if!proj.is_finite()||!l.is_finite(){return Tri::Indeterminate;}let pt=tol*l.max(apl).max(f64::MIN_POSITIVE);if proj< -pt||proj>l+pt{Tri::False}else{Tri::True}}

pub fn is_degenerate2d(v:Vec2,tol:f64)->Tri{if!v.is_finite()||!valid_tolerance(tol){Tri::Indeterminate}else if v.length()<=tol{Tri::True}else{Tri::False}}
pub fn is_degenerate3d(v:Vec3,tol:f64)->Tri{if!v.is_finite()||!valid_tolerance(tol){Tri::Indeterminate}else if v.length()<=tol{Tri::True}else{Tri::False}}

#[cfg(test)]
mod tests{use super::*;const T:f64=1e-12;#[test]fn orientation_signs(){assert_eq!(orient2d(Vec2::new(0.,0.),Vec2::new(1.,0.),Vec2::new(0.,1.),T),Tri::True);assert_eq!(orient3d(Vec3::new(0.,0.,0.),Vec3::new(1.,0.,0.),Vec3::new(0.,1.,0.),Vec3::new(0.,0.,1.),T),Tri::True);}#[test]fn degeneracy_is_not_reinterpreted_as_collinearity(){assert_eq!(is_collinear2d(Vec2::new(0.,0.),Vec2::new(0.,0.),Vec2::new(1.,0.),T),Tri::Indeterminate);assert_eq!(is_coplanar(Vec3::new(0.,0.,0.),Vec3::new(0.,0.,0.),Vec3::new(1.,0.,0.),Vec3::new(0.,1.,0.),T),Tri::Indeterminate);}#[test]fn parallel_and_perpendicular_are_scale_invariant(){assert_eq!(is_parallel2d(Vec2::new(1e-12,0.),Vec2::new(2e12,0.),T),Tri::True);assert_eq!(is_perpendicular3d(Vec3::new(1e-12,0.,0.),Vec3::new(0.,5e12,0.),T),Tri::True);}#[test]fn between_checks_projection(){assert_eq!(is_between2d(Vec2::new(0.,0.),Vec2::new(2.,0.),Vec2::new(1.,0.),T),Tri::True);assert_eq!(is_between3d(Vec3::new(0.,0.,0.),Vec3::new(2.,0.,0.),Vec3::new(3.,0.,0.),T),Tri::False);}}
