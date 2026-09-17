//! Univariate polynomial mathematics for the UMLCAD authority.
//!
//! Coefficients are stored in ascending power order. Root isolation uses a
//! derivative-partition strategy with safeguarded bisection/Newton refinement.
//! The implementation is deterministic and never treats non-finite numerical
//! results as valid roots.

#[derive(Clone, Debug, PartialEq)]
pub enum PolynomialError {
    Empty,
    NonFinite,
    ZeroPolynomial,
    DivisionByZero,
    InvalidTolerance,
    RootLimit,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RealRoot {
    pub value: f64,
    pub multiplicity: usize,
    pub residual: f64,
    pub bracket: Option<(f64, f64)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Polynomial {
    /// Ascending powers: `coefficients[i]` multiplies `x^i`.
    pub coefficients: Vec<f64>,
}

impl Polynomial {
    pub fn new(mut coefficients: Vec<f64>) -> Result<Self, PolynomialError> {
        if coefficients.is_empty() { return Err(PolynomialError::Empty); }
        if coefficients.iter().any(|v| !v.is_finite()) { return Err(PolynomialError::NonFinite); }
        while coefficients.len() > 1 && *coefficients.last().unwrap() == 0.0 { coefficients.pop(); }
        Ok(Self { coefficients })
    }

    pub fn zero() -> Self { Self { coefficients: vec![0.0] } }

    pub fn is_zero(&self) -> bool { self.coefficients.iter().all(|v| *v == 0.0) }

    pub fn degree(&self) -> Option<usize> {
        if self.is_zero() { None } else { Some(self.coefficients.len() - 1) }
    }

    pub fn leading_coefficient(&self) -> f64 { *self.coefficients.last().unwrap() }

    pub fn evaluate(&self, x: f64) -> Result<f64, PolynomialError> {
        if !x.is_finite() { return Err(PolynomialError::NonFinite); }
        let mut value = 0.0;
        for &coefficient in self.coefficients.iter().rev() {
            value = value * x + coefficient;
            if !value.is_finite() { return Err(PolynomialError::NonFinite); }
        }
        Ok(value)
    }

    pub fn derivative(&self) -> Self {
        if self.coefficients.len() <= 1 { return Self::zero(); }
        Self { coefficients: self.coefficients.iter().enumerate().skip(1).map(|(i,c)| *c * i as f64).collect() }
    }

    pub fn derivative_n(&self, order: usize) -> Self {
        let mut result = self.clone();
        for _ in 0..order { result = result.derivative(); }
        result
    }

    pub fn normalize(&self) -> Result<Self, PolynomialError> {
        if self.is_zero() { return Err(PolynomialError::ZeroPolynomial); }
        let leading = self.leading_coefficient();
        if leading == 0.0 || !leading.is_finite() { return Err(PolynomialError::NonFinite); }
        Self::new(self.coefficients.iter().map(|c| *c / leading).collect())
    }

    pub fn add(&self, other: &Self) -> Result<Self, PolynomialError> {
        let n=self.coefficients.len().max(other.coefficients.len());
        let coefficients=(0..n).map(|i|self.coefficients.get(i).copied().unwrap_or(0.0)+other.coefficients.get(i).copied().unwrap_or(0.0)).collect();
        Self::new(coefficients)
    }

    pub fn sub(&self, other: &Self) -> Result<Self, PolynomialError> {
        let n=self.coefficients.len().max(other.coefficients.len());
        let coefficients=(0..n).map(|i|self.coefficients.get(i).copied().unwrap_or(0.0)-other.coefficients.get(i).copied().unwrap_or(0.0)).collect();
        Self::new(coefficients)
    }

    pub fn mul(&self, other:&Self)->Result<Self,PolynomialError>{
        if self.is_zero() || other.is_zero() { return Ok(Self::zero()); }
        let mut coefficients=vec![0.0;self.degree().unwrap()+other.degree().unwrap()+1];
        for (i,a) in self.coefficients.iter().copied().enumerate(){for (j,b) in other.coefficients.iter().copied().enumerate(){coefficients[i+j]+=a*b;}}
        if coefficients.iter().any(|v|!v.is_finite()){Err(PolynomialError::NonFinite)}else{Self::new(coefficients)}
    }

    pub fn div_rem(&self, divisor:&Self)->Result<(Self,Self),PolynomialError>{
        if divisor.is_zero(){return Err(PolynomialError::DivisionByZero);}
        if self.is_zero(){return Ok((Self::zero(),Self::zero()));}
        if self.degree().unwrap()<divisor.degree().unwrap(){return Ok((Self::zero(),self.clone()));}
        let divisor_degree=divisor.degree().unwrap();
        let mut remainder=self.coefficients.clone();
        let mut quotient=vec![0.0;self.degree().unwrap()-divisor_degree+1];
        let lead=divisor.leading_coefficient();
        while remainder.len()-1>=divisor_degree && !(remainder.len()==1 && remainder[0]==0.0){
            let d=remainder.len()-1-divisor_degree;
            let factor=remainder.last().copied().unwrap()/lead;
            if !factor.is_finite(){return Err(PolynomialError::NonFinite);}
            quotient[d]=factor;
            for j in 0..=divisor_degree{remainder[d+j]-=factor*divisor.coefficients[j];}
            while remainder.len()>1 && remainder.last().copied().unwrap().abs()<=f64::EPSILON*remainder.iter().map(|v|v.abs()).fold(0.0,f64::max).max(1.0){remainder.pop();}
        }
        Ok((Self::new(quotient)?,Self::new(remainder)?))
    }

    pub fn cauchy_bound(&self)->Result<f64,PolynomialError>{
        if self.is_zero(){return Err(PolynomialError::ZeroPolynomial);}
        let lead=self.leading_coefficient().abs();
        if !lead.is_finite()||lead==0.0{return Err(PolynomialError::NonFinite);}
        let ratio=self.coefficients[..self.coefficients.len()-1].iter().map(|c|c.abs()/lead).fold(0.0,f64::max);
        let bound=1.0+ratio;
        if bound.is_finite(){Ok(bound)}else{Err(PolynomialError::NonFinite)}
    }

    fn sign_with_scale(&self,x:f64)->Result<i8,PolynomialError>{let value=self.evaluate(x)?;let scale=self.coefficients.iter().map(|c|c.abs()*x.abs().powi(self.coefficients.iter().position(|v|std::ptr::eq(v,c)).unwrap_or(0) as i32)).filter(|v|v.is_finite()).fold(0.0,f64::max).max(1.0);let threshold=32.0*f64::EPSILON*scale;if value.abs()<=threshold{Ok(0)}else if value>0.0{Ok(1)}else{Ok(-1)}}

    fn refine_bracket(&self,mut a:f64,mut b:f64,tol:f64,max_iter:usize)->Result<(f64,f64,f64),PolynomialError>{
        let mut fa=self.evaluate(a)?;let mut fb=self.evaluate(b)?;
        if fa.abs()<=tol{return Ok((a,a,fa));} if fb.abs()<=tol{return Ok((b,b,fb));}
        for _ in 0..max_iter{
            let mid=0.5*(a+b); if !mid.is_finite()||mid==a||mid==b{break;}
            let fm=self.evaluate(mid)?;
            if fm.abs()<=tol{return Ok((mid,mid,fm));}
            let use_left=fa.signum()!=fm.signum();
            if use_left{b=mid;fb=fm;}else{a=mid;fa=fm;}
            if (b-a).abs()<=tol*(1.0+a.abs().max(b.abs())){let x=0.5*(a+b);return Ok((x,x,self.evaluate(x)?));}
            let der=self.derivative();
            if !der.is_zero(){let fd=der.evaluate(mid)?;if fd.is_finite()&&fd.abs()>tol{let candidate=mid-fm/fd;if candidate>a&&candidate<b&&candidate.is_finite(){let fc=self.evaluate(candidate)?;if fc.abs()<fm.abs(){if fa.signum()!=fc.signum(){b=candidate;fb=fc;}else{a=candidate;fa=fc;}}}}}
        }
        let x=0.5*(a+b);Ok((a,b,self.evaluate(x)?))
    }

    pub fn real_roots(&self,tol:f64,max_roots:usize)->Result<Vec<RealRoot>,PolynomialError>{
        if !tol.is_finite()||tol<=0.0{return Err(PolynomialError::InvalidTolerance);}
        if self.is_zero(){return Err(PolynomialError::ZeroPolynomial);}
        if self.degree()==Some(0){return Ok(Vec::new());}
        let derivative=self.derivative();
        let critical=derivative.real_roots(tol, max_roots.saturating_mul(2).max(16)).unwrap_or_default();
        let bound=self.cauchy_bound()?;
        let mut points=Vec::with_capacity(critical.len()+2);points.push(-bound);
        for r in critical{if r.value>-bound&&r.value<bound{points.push(r.value);}}
        points.push(bound);points.sort_by(|a,b|a.total_cmp(b));points.dedup_by(|a,b|(*a-*b).abs()<=tol*(1.0+a.abs().max(b.abs())));
        let mut roots=Vec::new();
        for &x in &points{let fx=self.evaluate(x)?;if fx.abs()<=tol*(1.0+x.abs()){roots.push(RealRoot{value:x,multiplicity:1,residual:fx.abs(),bracket:Some((x,x))});}}
        for window in points.windows(2){
            let a=window[0];let b=window[1];let fa=self.evaluate(a)?;let fb=self.evaluate(b)?;
            if fa.signum()==fb.signum(){continue;}
            let (x,lo_or_x,res)=self.refine_bracket(a,b,tol,128)?;
            let bracket=if (x-lo_or_x).abs()>0.0{Some((x,lo_or_x))}else{Some((x,x))};
            roots.push(RealRoot{value:x,multiplicity:1,residual:res.abs(),bracket});
        }
        roots.sort_by(|a,b|a.value.total_cmp(&b.value));
        let mut merged:Vec<RealRoot>=Vec::new();
        for root in roots{if let Some(last)=merged.last_mut(){if (root.value-last.value).abs()<=tol*(1.0+root.value.abs().max(last.value.abs())){last.value=0.5*(last.value+root.value);last.residual=self.evaluate(last.value)?.abs();last.multiplicity+=1;continue;}}merged.push(root);if merged.len()>max_roots{return Err(PolynomialError::RootLimit);}}
        Ok(merged)
    }

    /// Multiplicity estimate obtained by testing successive derivatives at the root.
    pub fn multiplicity_at(&self, x:f64, tol:f64)->Result<usize,PolynomialError>{
        if !x.is_finite()||!tol.is_finite()||tol<=0.0{return Err(PolynomialError::InvalidTolerance);}
        let scale=self.coefficients.iter().map(|c|c.abs()).fold(0.0,f64::max).max(1.0);
        for order in 1..=self.coefficients.len(){let value=self.derivative_n(order-1).evaluate(x)?;if value.abs()>(tol*scale*(1.0+x.abs()).powi(order as i32)){return Ok(order-1);}}
        Ok(self.coefficients.len()-1)
    }
}

#[cfg(test)]
mod tests{
 use super::*;
 #[test]fn evaluate_and_derivative(){let p=Polynomial::new(vec![1.0,2.0,1.0]).unwrap();assert_eq!(p.evaluate(2.0).unwrap(),9.0);assert_eq!(p.derivative().coefficients,vec![2.0,2.0]);}
 #[test]fn division_reconstructs(){let p=Polynomial::new(vec![1.0,-3.0,2.0]).unwrap();let d=Polynomial::new(vec![-1.0,1.0]).unwrap();let(q,r)=p.div_rem(&d).unwrap();assert_eq!(q.coefficients,vec![-1.0,2.0]);assert!(r.is_zero());}
 #[test]fn real_roots_finds_two_distinct_roots(){let p=Polynomial::new(vec![6.0,-5.0,1.0]).unwrap();let roots=p.real_roots(1.0e-10,8).unwrap();assert_eq!(roots.len(),2);assert!((roots[0].value-2.0).abs()<1e-8);assert!((roots[1].value-3.0).abs()<1e-8);}
 #[test]fn repeated_root_is_detected(){let p=Polynomial::new(vec![1.0,-3.0,3.0,-1.0]).unwrap();let roots=p.real_roots(1.0e-8,8).unwrap();assert!(!roots.is_empty());assert!((roots[0].value-1.0).abs()<1e-6);}
 #[test]fn cauchy_bound_contains_known_root(){let p=Polynomial::new(vec![-6.0,11.0,-6.0,1.0]).unwrap();assert!(p.cauchy_bound().unwrap()>=3.0);}
}
