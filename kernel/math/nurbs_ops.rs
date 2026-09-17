//! Exact value-preserving operations on the existing rational NURBS curve.
//!
//! Knot insertion uses homogeneous control points (Boehm/Piegl-Tiller form),
//! then dehomogenizes only after the new representation is complete. Reversal
//! transforms the parameter domain exactly and preserves geometry.

use super::nurbs::{NurbsCurve2D,NurbsError,Point2};

#[derive(Clone,Debug,PartialEq)]
pub struct KnotMultiplicity{pub knot:f64,pub multiplicity:usize,pub continuity:Option<usize>}

#[derive(Clone,Copy)]struct H{ x:f64,y:f64,w:f64 }
fn add(a:H,b:H,alpha:f64)->H{H{x:a.x*(1.-alpha)+b.x*alpha,y:a.y*(1.-alpha)+b.y*alpha,w:a.w*(1.-alpha)+b.w*alpha}}
fn find_span(curve:&NurbsCurve2D,u:f64)->usize{let n=curve.control_points.len()-1;if u>=curve.knots[n+1]{return n;}if u<=curve.knots[curve.degree]{return curve.degree;}let mut lo=curve.degree;let mut hi=n+1;let mut mid=(lo+hi)/2;while u<curve.knots[mid]||u>=curve.knots[mid+1]{if u<curve.knots[mid]{hi=mid}else{lo=mid};mid=(lo+hi)/2;}mid}
fn multiplicity(knots:&[f64],u:f64)->usize{knots.iter().filter(|k|**k==u).count()}
fn to_h(curve:&NurbsCurve2D)->Vec<H>{curve.control_points.iter().zip(curve.weights.iter()).map(|(p,w)|H{x:p.x*w,y:p.y*w,w:*w}).collect()}

pub fn insert_knot(curve:&NurbsCurve2D,u:f64)->Result<NurbsCurve2D,NurbsError>{
 curve.validate()?;if !u.is_finite(){return Err(NurbsError::NonFinite);}let(a,b)=curve.parameter_domain()?;if u<=a||u>=b{return Err(NurbsError::OutOfDomain);}let p=curve.degree;let s=multiplicity(&curve.knots,u);if s>=p{return Err(NurbsError::InvalidKnotCount);}let k=find_span(curve,u);let pw=to_h(curve);let n=pw.len()-1;let mut qw=vec![H{x:0.,y:0.,w:0.};pw.len()+1];
 for i in 0..=k-p{qw[i]=pw[i];}
 for i in k-s..=n{qw[i+1]=pw[i];}
 if k>=p+1{for i in (k-p+1)..=k-s{let den=curve.knots[i+p]-curve.knots[i];if den==0.{return Err(NurbsError::InvalidDomain);}let alpha=(u-curve.knots[i])/den;qw[i]=add(pw[i-1],pw[i],alpha);}}
 let mut uq=Vec::with_capacity(curve.knots.len()+1);for i in 0..=k{uq.push(curve.knots[i]);}uq.push(u);for i in k+1..curve.knots.len(){uq.push(curve.knots[i]);}
 let mut cp=Vec::with_capacity(qw.len());let mut wt=Vec::with_capacity(qw.len());for q in qw{if !q.w.is_finite()||q.w<=0.{return Err(NurbsError::InvalidWeight);}let p2=Point2{x:q.x/q.w,y:q.y/q.w};if !p2.x.is_finite()||!p2.y.is_finite(){return Err(NurbsError::Overflow);}cp.push(p2);wt.push(q.w);}let result=NurbsCurve2D::new(p,cp,wt,uq);result.validate()?;Ok(result)
}

pub fn insert_knot_repeated(curve:&NurbsCurve2D,u:f64,count:usize)->Result<NurbsCurve2D,NurbsError>{let mut result=curve.clone();for _ in 0..count{result=insert_knot(&result,u)?;}Ok(result)}

pub fn reverse(curve:&NurbsCurve2D)->Result<NurbsCurve2D,NurbsError>{curve.validate()?;let(a,b)=curve.parameter_domain()?;let cp=curve.control_points.iter().rev().copied().collect::<Vec<_>>();let wt=curve.weights.iter().rev().copied().collect::<Vec<_>>();let knots=curve.knots.iter().rev().map(|u|a+b-*u).collect::<Vec<_>>();let result=NurbsCurve2D::new(curve.degree,cp,wt,knots);result.validate()?;Ok(result)}

pub fn interior_knot_multiplicities(curve:&NurbsCurve2D)->Result<Vec<KnotMultiplicity>,NurbsError>{curve.validate()?;let(a,b)=curve.parameter_domain()?;let mut result=Vec::new();let mut i=curve.degree+1;while i<curve.control_points.len(){let u=curve.knots[i];if u>a&&u<b{let m=multiplicity(&curve.knots,u);if result.last().map(|r:&KnotMultiplicity|r.knot==u).unwrap_or(false){i+=1;continue;}let continuity=if m<=curve.degree{Some(curve.degree-m)}else{None};result.push(KnotMultiplicity{knot:u,multiplicity:m,continuity});}i+=1;}Ok(result)}

/// Returns the parametric continuity order at an interior knot.
/// `None` means positional continuity is lost (`C^-1`) under the knot-multiplicity model.
pub fn continuity_at(curve:&NurbsCurve2D,u:f64)->Result<Option<usize>,NurbsError>{curve.validate()?;let(a,b)=curve.parameter_domain()?;if !u.is_finite(){return Err(NurbsError::NonFinite);}if u<=a||u>=b{return Err(NurbsError::OutOfDomain);}let m=multiplicity(&curve.knots,u);if m==0{return Ok(Some(curve.degree));}if m>curve.degree{Ok(None)}else{Ok(Some(curve.degree-m))}}

pub fn reverse_parameter(curve:&NurbsCurve2D,u:f64)->Result<f64,NurbsError>{curve.validate()?;let(a,b)=curve.parameter_domain()?;if !u.is_finite(){return Err(NurbsError::NonFinite);}if u<a||u>b{return Err(NurbsError::OutOfDomain);}Ok(a+b-u)}

#[cfg(test)]
mod tests{use super::*;#[test]fn single_knot_insertion_preserves_points(){let c=NurbsCurve2D::new(2,vec![Point2{x:0.,y:0.},Point2{x:1.,y:1.},Point2{x:2.,y:0.}],vec![1.,1.,1.],vec![0.,0.,0.,1.,1.,1.]);let q=insert_knot(&c,.5).unwrap();for u in [0.,.25,.5,.75,1.]{let a=c.point_at(u).unwrap();let b=q.point_at(u).unwrap();assert!((a.x-b.x).abs()<1e-12&&(a.y-b.y).abs()<1e-12);}}#[test]fn reversal_reverses_parameterization(){let c=NurbsCurve2D::new(2,vec![Point2{x:0.,y:0.},Point2{x:1.,y:1.},Point2{x:2.,y:0.}],vec![1.,1.,1.],vec![0.,0.,0.,1.,1.,1.]);let r=reverse(&c).unwrap();for u in [0.,.25,.5,.75,1.]{let a=c.point_at(u).unwrap();let b=r.point_at(1.-u).unwrap();assert!((a.x-b.x).abs()<1e-12&&(a.y-b.y).abs()<1e-12);}}#[test]fn simple_internal_knot_has_c1_continuity(){let c=NurbsCurve2D::new(2,vec![Point2{x:0.,y:0.},Point2{x:1.,y:1.},Point2{x:2.,y:0.},Point2{x:3.,y:1.}],vec![1.,1.,1.,1.],vec![0.,0.,0.,.5,1.,1.,1.]);assert_eq!(continuity_at(&c,.5).unwrap(),Some(1));}#[test]fn excessive_multiplicity_is_not_claimed_smooth(){let c=NurbsCurve2D::new(2,vec![Point2{x:0.,y:0.},Point2{x:1.,y:1.},Point2{x:2.,y:0.},Point2{x:3.,y:1.},Point2{x:4.,y:0.}],vec![1.;5],vec![0.,0.,0.,.5,.5,1.,1.,1.]);assert_eq!(continuity_at(&c,.5).unwrap(),Some(0));}}
