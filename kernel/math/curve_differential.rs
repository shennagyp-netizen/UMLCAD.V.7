//! Analytic differential operations for the existing B-spline/NURBS curves.
//!
//! Derivatives are obtained from differentiated homogeneous/control nets rather
//! than finite differences. Rational derivatives use the quotient rule and
//! explicitly reject zero projective denominators and undefined derivative nets.

use super::{bspline::BSplineCurve2D, nurbs::NurbsCurve2D};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DifferentialError { NonFinite, InvalidDomain, Degenerate, Singular, Overflow }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CurveDerivative2D { pub first: (f64,f64), pub second: Option<(f64,f64)> }

fn bspline_eval(degree:usize, control:&[(f64,f64)], knots:&[f64], u:f64)->Result<(f64,f64),DifferentialError>{
 if control.len()<degree+1||knots.len()!=control.len()+degree+1{return Err(DifferentialError::InvalidDomain);}
 let n=control.len()-1;let start=knots[degree];let end=knots[n+1];if !u.is_finite()||u<start||u>end{return Err(DifferentialError::InvalidDomain);}
 let span=if u>=end{n}else{let mut lo=degree;let mut hi=n+1;let mut mid=(lo+hi)/2;while u<knots[mid]||u>=knots[mid+1]{if u<knots[mid]{hi=mid}else{lo=mid};mid=(lo+hi)/2;}mid};
 let mut work=control[span-degree..=span].to_vec();
 for level in 1..=degree{for j in (level..=degree).rev(){let i=span-degree+j;let den=knots[i+degree+1-level]-knots[i];let alpha=if den==0.0{0.0}else{(u-knots[i])/den};work[j]=((work[j-1].0*(1.0-alpha)+work[j].0*alpha),(work[j-1].1*(1.0-alpha)+work[j].1*alpha));}}
 let p=work[degree];if p.0.is_finite()&&p.1.is_finite(){Ok(p)}else{Err(DifferentialError::Overflow)}
}

fn derivative_control(degree:usize,control:&[(f64,f64)],knots:&[f64])->Result<(usize,Vec<(f64,f64)>,Vec<f64>),DifferentialError>{
 if degree==0||control.len()<2{return Err(DifferentialError::Degenerate);}
 let mut result=Vec::with_capacity(control.len()-1);for i in 0..control.len()-1{let den=knots[i+degree+1]-knots[i+1];if den==0.0{return Err(DifferentialError::Singular);}let f=degree as f64/den;result.push(((control[i+1].0-control[i].0)*f,(control[i+1].1-control[i].1)*f));}
 Ok((degree-1,result,knots[1..knots.len()-1].to_vec()))
}

pub fn bspline_derivative(curve:&BSplineCurve2D,u:f64)->Result<(f64,f64),DifferentialError>{curve.validate().map_err(|_|DifferentialError::InvalidDomain)?;let control:Vec<_>=curve.control_points.iter().map(|p|(p.x,p.y)).collect();let (d,c,k)=derivative_control(curve.degree,&control,&curve.knots)?;bspline_eval(d,&c,&k,u)}
pub fn bspline_second_derivative(curve:&BSplineCurve2D,u:f64)->Result<(f64,f64),DifferentialError>{curve.validate().map_err(|_|DifferentialError::InvalidDomain)?;let c0:Vec<_>=curve.control_points.iter().map(|p|(p.x,p.y)).collect();let(d1,c1,k1)=derivative_control(curve.degree,&c0,&curve.knots)?;let(d2,c2,k2)=derivative_control(d1,&c1,&k1)?;if d2==0{Ok(c2[0])}else{bspline_eval(d2,&c2,&k2,u)}}

#[derive(Clone,Copy)]
struct H{ x:f64,y:f64,w:f64 }
fn h_eval(degree:usize,control:&[H],knots:&[f64],u:f64)->Result<H,DifferentialError>{let v:Vec<_>=control.iter().map(|p|(p.x,p.y)).collect();let(a,b)=bspline_eval_h(degree,&v,&control.iter().map(|p|p.w).collect::<Vec<_>>(),knots,u)?;Ok(H{x:a.0,y:a.1,w:b})}
fn bspline_eval_h(degree:usize,xy:&[(f64,f64)],weights:&[f64],knots:&[f64],u:f64)->Result<((f64,f64),f64),DifferentialError>{if xy.len()!=weights.len(){return Err(DifferentialError::InvalidDomain);}if xy.len()<degree+1{return Err(DifferentialError::InvalidDomain);}let n=xy.len()-1;let start=knots[degree];let end=knots[n+1];if u<start||u>end{return Err(DifferentialError::InvalidDomain);}let span=if u>=end{n}else{let mut lo=degree;let mut hi=n+1;let mut mid=(lo+hi)/2;while u<knots[mid]||u>=knots[mid+1]{if u<knots[mid]{hi=mid}else{lo=mid};mid=(lo+hi)/2;}mid};let mut work:Vec<H>=(span-degree..=span).map(|i|H{x:xy[i].0*weights[i],y:xy[i].1*weights[i],w:weights[i]}).collect();for level in 1..=degree{for j in(level..=degree).rev(){let i=span-degree+j;let den=knots[i+degree+1-level]-knots[i];let a=if den==0.0{0.0}else{(u-knots[i])/den};let q=work[j-1];let r=work[j];work[j]=H{x:q.x*(1.-a)+r.x*a,y:q.y*(1.-a)+r.y*a,w:q.w*(1.-a)+r.w*a};}}let p=work[degree];if p.x.is_finite()&&p.y.is_finite()&&p.w.is_finite(){Ok(((p.x,p.y),p.w))}else{Err(DifferentialError::Overflow)}}

fn nurbs_homogeneous_derivative(curve:&NurbsCurve2D,order:usize,u:f64)->Result<H,DifferentialError>{curve.validate().map_err(|_|DifferentialError::InvalidDomain)?;if order>curve.degree{return Ok(H{x:0.,y:0.,w:0.});}let mut degree=curve.degree;let mut control:Vec<H>=curve.control_points.iter().zip(curve.weights.iter()).map(|(p,w)|H{x:p.x**w,y:p.y**w,w:*w}).collect();let mut knots=curve.knots.clone();for _ in 0..order{let mut next=Vec::with_capacity(control.len()-1);for i in 0..control.len()-1{let den=knots[i+degree+1]-knots[i+1];if den==0.0{return Err(DifferentialError::Singular);}let f=degree as f64/den;next.push(H{x:(control[i+1].x-control[i].x)*f,y:(control[i+1].y-control[i].y)*f,w:(control[i+1].w-control[i].w)*f});}control=next;knots=knots[1..knots.len()-1].to_vec();degree-=1;}let out=if degree==0{control[0]}else{let xy:Vec<_>=control.iter().map(|p|(p.x,p.y)).collect();let wt:Vec<_>=control.iter().map(|p|p.w).collect();let(v,w)=bspline_eval_h(degree,&xy,&wt,&knots,u)?;H{x:v.0,y:v.1,w:w}};if out.x.is_finite()&&out.y.is_finite()&&out.w.is_finite(){Ok(out)}else{Err(DifferentialError::Overflow)}}

pub fn nurbs_derivative(curve:&NurbsCurve2D,u:f64)->Result<(f64,f64),DifferentialError>{let p=nurbs_homogeneous_derivative(curve,0,u)?;let d=nurbs_homogeneous_derivative(curve,1,u)?;if p.w<=0.0{return Err(DifferentialError::Singular);}let x=(d.x*p.w-p.x*d.w)/(p.w*p.w);let y=(d.y*p.w-p.y*d.w)/(p.w*p.w);if x.is_finite()&&y.is_finite(){Ok((x,y))}else{Err(DifferentialError::Overflow)}}

pub fn nurbs_second_derivative(curve:&NurbsCurve2D,u:f64)->Result<(f64,f64),DifferentialError>{let p=nurbs_homogeneous_derivative(curve,0,u)?;let d=nurbs_homogeneous_derivative(curve,1,u)?;let dd=nurbs_homogeneous_derivative(curve,2,u)?;if p.w<=0.0{return Err(DifferentialError::Singular);}let w=p.w;let x=(dd.x*w*w-p.x*w*dd.w-2.0*d.x*w*d.w+2.0*p.x*d.w*d.w)/(w*w*w);let y=(dd.y*w*w-p.y*w*dd.w-2.0*d.y*w*d.w+2.0*p.y*d.w*d.w)/(w*w*w);if x.is_finite()&&y.is_finite(){Ok((x,y))}else{Err(DifferentialError::Overflow)}}

pub fn curvature_from_derivatives(first:(f64,f64),second:(f64,f64))->Result<f64,DifferentialError>{let speed=first.0.hypot(first.1);if !speed.is_finite(){return Err(DifferentialError::Overflow);}if speed==0.0{return Err(DifferentialError::Degenerate);}let numerator=(first.0*second.1-first.1*second.0).abs();let k=numerator/(speed*speed*speed);if k.is_finite(){Ok(k)}else{Err(DifferentialError::Overflow)}}

#[cfg(test)]
mod tests{use super::*;#[test]fn bspline_linear_derivative_is_constant(){let c=BSplineCurve2D::new(1,vec![crate::math::bspline::Point2{x:0.,y:0.},crate::math::bspline::Point2{x:2.,y:4.}],vec![0.,0.,1.,1.]);assert_eq!(bspline_derivative(&c,0.5).unwrap(),(2.,4.));}#[test]fn bspline_quadratic_second_derivative_is_constant(){let c=BSplineCurve2D::new(2,vec![crate::math::bspline::Point2{x:0.,y:0.},crate::math::bspline::Point2{x:1.,y:1.},crate::math::bspline::Point2{x:2.,y:0.}],vec![0.,0.,0.,1.,1.,1.]);let d=bspline_second_derivative(&c,.5).unwrap();assert!((d.0).abs()<1e-12&& (d.1+4.).abs()<1e-12);}#[test]fn rational_quarter_circle_derivative_is_finite(){let c=NurbsCurve2D::new(2,vec![crate::math::nurbs::Point2{x:1.,y:0.},crate::math::nurbs::Point2{x:1.,y:1.},crate::math::nurbs::Point2{x:0.,y:1.}],vec![1.,2f64.sqrt()/2.,1.],vec![0.,0.,0.,1.,1.,1.]);let d=nurbs_derivative(&c,.5).unwrap();assert!(d.0.is_finite()&&d.1.is_finite());}}
