//! Adaptive curve tessellation mathematics.
//!
//! Tessellation is explicitly an approximation. The algorithm refines parameter
//! intervals until caller-provided chord and angular errors are satisfied, or
//! returns `MaxDepth` rather than silently returning a lower-quality result.

use super::vec::Vec3;

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct TessellationPolicy{pub chord_error:f64,pub angular_error:f64,pub max_depth:u32}
#[derive(Clone,Debug,PartialEq)]
pub struct TessellatedCurve3{pub points:Vec<Vec3>,pub parameters:Vec<f64>,pub chord_error:f64,pub angular_error:f64,pub max_depth:u32}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum TessellationError{InvalidPolicy,NonFinite,Degenerate,EvaluationFailed,MaxDepth,InvalidDomain}

impl TessellationPolicy{pub fn validate(self)->Result<(),TessellationError>{if!self.chord_error.is_finite()||!self.angular_error.is_finite()||self.chord_error<=0.||self.angular_error<=0.{return Err(TessellationError::InvalidPolicy);}Ok(())}}
fn point_segment_distance(p:Vec3,a:Vec3,b:Vec3)->Result<f64,TessellationError>{let d=b.sub(a);let l=d.length();if l==0.{return Err(TessellationError::Degenerate);}let u=d.scale(1./l);let t=p.sub(a).dot(u);if!t.is_finite(){return Err(TessellationError::NonFinite);}Ok(p.sub(a.add(u.scale(t.clamp(0.,l)))).length())}
fn angle_between(a:Vec3,b:Vec3)->Result<f64,TessellationError>{let u=a.normalized().map_err(|_|TessellationError::Degenerate)?;let v=b.normalized().map_err(|_|TessellationError::Degenerate)?;Ok(u.dot(v).clamp(-1.,1.).acos())}

pub fn tessellate_curve3<F,G>(domain:(f64,f64),policy:TessellationPolicy,eval:F,tangent:G)->Result<TessellatedCurve3,TessellationError>where F:Fn(f64)->Result<Vec3,TessellationError>,G:Fn(f64)->Result<Vec3,TessellationError>{policy.validate()?;let(a,b)=domain;if!a.is_finite()||!b.is_finite()||b<=a{return Err(TessellationError::InvalidDomain);}let pa=eval(a)?;let pb=eval(b)?;if!pa.is_finite()||!pb.is_finite(){return Err(TessellationError::NonFinite);}let mut points=vec![pa];let mut parameters=vec![a];fn recurse<F,G>(a:f64,b:f64,pa:Vec3,pb:Vec3,depth:u32,p:&TessellationPolicy,eval:&F,tangent:&G,points:&mut Vec<Vec3>,params:&mut Vec<f64>)->Result<(),TessellationError>where F:Fn(f64)->Result<Vec3,TessellationError>,G:Fn(f64)->Result<Vec3,TessellationError>{let m=a+(b-a)*0.5;let pm=eval(m)?;let chord=point_segment_distance(pm,pa,pb)?;let ta=tangent(a)?;let tm=tangent(m)?;let tb=tangent(b)?;let angle=angle_between(ta,tb)?.max(angle_between(ta,tm)?).max(angle_between(tm,tb)?);if chord<=p.chord_error&&angle<=p.angular_error{points.push(pb);params.push(b);return Ok(());}if depth>=p.max_depth{return Err(TessellationError::MaxDepth);}recurse(a,m,pa,pm,depth+1,p,eval,tangent,points,params)?;points.pop();params.pop();recurse(m,b,pm,pb,depth+1,p,eval,tangent,points,params)}recurse(a,b,pa,pb,0,&policy,&eval,&tangent,&mut points,&mut parameters)?;Ok(TessellatedCurve3{points,parameters,chord_error:policy.chord_error,angular_error:policy.angular_error,max_depth:policy.max_depth})}

#[cfg(test)]
mod tests{use super::*;#[test]fn straight_line_needs_only_endpoints(){let p=TessellationPolicy{chord_error:1e-6,angular_error:1e-6,max_depth:8};let r=tessellate_curve3((0.,1.),p,|t|Ok(Vec3::new(t,0.,0.)),|_|Ok(Vec3::new(1.,0.,0.))).unwrap();assert_eq!(r.points.len(),2);assert_eq!(r.points.len(),r.parameters.len());}#[test]fn quarter_arc_refines_for_chord_and_angle(){let p=TessellationPolicy{chord_error:1e-3,angular_error:0.1,max_depth:16};let r=tessellate_curve3((0.,std::f64::consts::FRAC_PI_2),p,|t|Ok(Vec3::new(t.cos(),t.sin(),0.)),|t|Ok(Vec3::new(-t.sin(),t.cos(),0.))).unwrap();assert!(r.points.len()>2);assert_eq!(r.points.len(),r.parameters.len());}#[test]fn impossible_policy_depth_is_reported(){let p=TessellationPolicy{chord_error:1e-15,angular_error:1e-15,max_depth:0};let r=tessellate_curve3((0.,std::f64::consts::FRAC_PI_2),p,|t|Ok(Vec3::new(t.cos(),t.sin(),0.)),|t|Ok(Vec3::new(-t.sin(),t.cos(),0.)));assert_eq!(r,Err(TessellationError::MaxDepth));}}
