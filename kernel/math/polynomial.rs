//! Univariate polynomial mathematics for the UMLCAD authority.
//!
//! Coefficients use ascending powers. Real-root isolation recursively partitions
//! a Cauchy-bounded domain by derivative roots, then uses safeguarded bisection.
//! Repeated roots are explicit candidates because they are roots of the derivative.

#[derive(Clone, Debug, PartialEq, Eq)]
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
pub struct Polynomial { pub coefficients: Vec<f64> }

impl Polynomial {
    pub fn new(mut coefficients: Vec<f64>) -> Result<Self, PolynomialError> {
        if coefficients.is_empty() { return Err(PolynomialError::Empty); }
        if coefficients.iter().any(|v| !v.is_finite()) { return Err(PolynomialError::NonFinite); }
        while coefficients.len() > 1 && coefficients.last().copied() == Some(0.0) { coefficients.pop(); }
        Ok(Self { coefficients })
    }
    pub fn zero() -> Self { Self { coefficients: vec![0.0] } }
    pub fn is_zero(&self) -> bool { self.coefficients.iter().all(|v| *v == 0.0) }
    pub fn degree(&self) -> Option<usize> { if self.is_zero(){None}else{Some(self.coefficients.len()-1)} }
    pub fn leading_coefficient(&self) -> f64 { self.coefficients.last().copied().unwrap_or(0.0) }

    pub fn evaluate(&self, x:f64)->Result<f64,PolynomialError>{
        if !x.is_finite(){return Err(PolynomialError::NonFinite);}
        let mut value=0.0;
        for &c in self.coefficients.iter().rev(){value=value*x+c;if !value.is_finite(){return Err(PolynomialError::NonFinite);}}
        Ok(value)
    }
    pub fn derivative(&self)->Self{if self.coefficients.len()<=1{Self::zero()}else{Self{coefficients:self.coefficients.iter().enumerate().skip(1).map(|(i,c)|*c*i as f64).collect()}}}
    pub fn derivative_n(&self,order:usize)->Self{let mut p=self.clone();for _ in 0..order{p=p.derivative();}p}
    pub fn normalize(&self)->Result<Self,PolynomialError>{if self.is_zero(){return Err(PolynomialError::ZeroPolynomial);}let lead=self.leading_coefficient();if lead==0.0||!lead.is_finite(){return Err(PolynomialError::NonFinite);}Self::new(self.coefficients.iter().map(|c|*c/lead).collect())}
    pub fn add(&self,other:&Self)->Result<Self,PolynomialError>{let n=self.coefficients.len().max(other.coefficients.len());Self::new((0..n).map(|i|self.coefficients.get(i).copied().unwrap_or(0.0)+other.coefficients.get(i).copied().unwrap_or(0.0)).collect())}
    pub fn sub(&self,other:&Self)->Result<Self,PolynomialError>{let n=self.coefficients.len().max(other.coefficients.len());Self::new((0..n).map(|i|self.coefficients.get(i).copied().unwrap_or(0.0)-other.coefficients.get(i).copied().unwrap_or(0.0)).collect())}
    pub fn mul(&self,other:&Self)->Result<Self,PolynomialError>{if self.is_zero()||other.is_zero(){return Ok(Self::zero());}let mut c=vec![0.0;self.coefficients.len()+other.coefficients.len()-1];for(i,a)in self.coefficients.iter().copied().enumerate(){for(j,b)in other.coefficients.iter().copied().enumerate(){c[i+j]+=a*b;}}Self::new(c)}

    pub fn div_rem(&self,divisor:&Self)->Result<(Self,Self),PolynomialError>{
        if divisor.is_zero(){return Err(PolynomialError::DivisionByZero);}
        if self.is_zero(){return Ok((Self::zero(),Self::zero()));}
        let n=self.degree().unwrap();let d=divisor.degree().unwrap();
        if n<d{return Ok((Self::zero(),self.clone()));}
        let mut r=self.coefficients.clone();let mut q=vec![0.0;n-d+1];let lead=divisor.leading_coefficient();
        while r.len()-1>=d && !(r.len()==1&&r[0]==0.0){let shift=r.len()-1-d;let factor=r.last().copied().unwrap()/lead;if !factor.is_finite(){return Err(PolynomialError::NonFinite);}q[shift]=factor;for j in 0..=d{r[shift+j]-=factor*divisor.coefficients[j];}while r.len()>1&&r.last().copied().unwrap()==0.0{r.pop();}}
        Ok((Self::new(q)?,Self::new(r)?))
    }

    pub fn cauchy_bound(&self)->Result<f64,PolynomialError>{
        if self.is_zero(){return Err(PolynomialError::ZeroPolynomial);}let lead=self.leading_coefficient().abs();if lead==0.0||!lead.is_finite(){return Err(PolynomialError::NonFinite);}let max_ratio=self.coefficients[..self.coefficients.len()-1].iter().map(|c|c.abs()/lead).fold(0.0,f64::max);let bound=1.0+max_ratio;if bound.is_finite(){Ok(bound)}else{Err(PolynomialError::NonFinite)}
    }

    fn root_at_or_near(&self,x:f64,tol:f64)->Result<bool,PolynomialError>{let f=self.evaluate(x)?;Ok(f.abs()<=tol*(1.0+x.abs()))}

    fn bisect(&self,mut a:f64,mut b:f64,tol:f64)->Result<(f64,(f64,f64)),PolynomialError>{
        let mut fa=self.evaluate(a)?;let mut fb=self.evaluate(b)?;
        if fa.abs()<=tol*(1.0+a.abs()){return Ok((a,(a,a)));}if fb.abs()<=tol*(1.0+b.abs()){return Ok((b,(b,b)));}
        for _ in 0..192{let mid=a+(b-a)*0.5;if mid==a||mid==b{break;}let fm=self.evaluate(mid)?;if fm.abs()<=tol*(1.0+mid.abs()){return Ok((mid,(a,b)));}if fa.signum()!=fm.signum(){b=mid;fb=fm;}else{a=mid;fa=fm;}if(b-a).abs()<=tol*(1.0+a.abs().max(b.abs())){let x=a+(b-a)*0.5;return Ok((x,(a,b)));}}
        let x=a+(b-a)*0.5;let fx=self.evaluate(x)?;if fx.abs()>tol*(1.0+x.abs()){return Err(PolynomialError::RootLimit);}Ok((x,(a,b)))
    }

    fn isolate_recursive(&self,tol:f64,max_roots:usize)->Result<Vec<RealRoot>,PolynomialError>{
        if self.is_zero(){return Err(PolynomialError::ZeroPolynomial);}
        match self.degree(){None=>Err(PolynomialError::ZeroPolynomial),Some(0)=>Ok(Vec::new()),Some(1)=>{let x=-self.coefficients[0]/self.coefficients[1];if x.is_finite(){Ok(vec![RealRoot{value:x,multiplicity:1,residual:self.evaluate(x)?.abs(),bracket:Some((x,x))}])}else{Err(PolynomialError::NonFinite)}} ,Some(_)=>{
            let critical=self.derivative().isolate_recursive(tol,max_roots)?;
            let bound=self.cauchy_bound()?;
            let mut points=Vec::with_capacity(critical.len()+2);points.push(-bound);for r in critical{if r.value>-bound&&r.value<bound{points.push(r.value);}}points.push(bound);points.sort_by(|a,b|a.total_cmp(b));
            let mut roots=Vec::new();
            for &x in &points{if self.root_at_or_near(x,tol)?{roots.push(RealRoot{value:x,multiplicity:1,residual:self.evaluate(x)?.abs(),bracket:Some((x,x))});}}
            for pair in points.windows(2){let a=pair[0];let b=pair[1];let fa=self.evaluate(a)?;let fb=self.evaluate(b)?;if fa==0.0||fb==0.0||fa.signum()==fb.signum(){continue;}let(x,bracket)=self.bisect(a,b,tol)?;roots.push(RealRoot{value:x,multiplicity:1,residual:self.evaluate(x)?.abs(),bracket:Some(bracket)});}
            roots.sort_by(|a,b|a.value.total_cmp(&b.value));let mut merged=Vec::new();for root in roots{if let Some(last)=merged.last_mut(){if(root.value-last.value).abs()<=tol*(1.0+root.value.abs().max(last.value.abs())){last.value=0.5*(last.value+root.value);last.residual=self.evaluate(last.value)?.abs();last.multiplicity+=1;continue;}}merged.push(root);if merged.len()>max_roots{return Err(PolynomialError::RootLimit);}}Ok(merged)
        }}
    }

    pub fn real_roots(&self,tol:f64,max_roots:usize)->Result<Vec<RealRoot>,PolynomialError>{if !tol.is_finite()||tol<=0.0||max_roots==0{return Err(PolynomialError::InvalidTolerance);}let mut roots=self.isolate_recursive(tol,max_roots)?;for root in &mut roots{root.multiplicity=self.multiplicity_at(root.value,tol)?.max(1);}Ok(roots)}

    pub fn multiplicity_at(&self,x:f64,tol:f64)->Result<usize,PolynomialError>{if !x.is_finite()||!tol.is_finite()||tol<=0.0{return Err(PolynomialError::InvalidTolerance);}let mut multiplicity=0;let scale=self.coefficients.iter().map(|c|c.abs()).fold(0.0,f64::max).max(1.0);let mut derivative=self.clone();for _ in 0..self.coefficients.len(){let value=derivative.evaluate(x)?.abs();let threshold=tol*scale*(1.0+x.abs()).powi(multiplicity as i32);if value>threshold{break;}multiplicity+=1;derivative=derivative.derivative();if derivative.is_zero(){break;}}Ok(multiplicity.max(1))}
}

#[cfg(test)]
mod tests{use super::*;#[test]fn basic_calculus(){let p=Polynomial::new(vec![1.,2.,1.]).unwrap();assert_eq!(p.evaluate(2.).unwrap(),9.);assert_eq!(p.derivative().coefficients,vec![2.,2.]);}#[test]fn division_reconstructs(){let p=Polynomial::new(vec![1.,-3.,2.]).unwrap();let d=Polynomial::new(vec![-1.,1.]).unwrap();let(q,r)=p.div_rem(&d).unwrap();assert_eq!(q.coefficients,vec![-1.,2.]);assert!(r.is_zero());}#[test]fn roots_are_refined(){let p=Polynomial::new(vec![6.,-5.,1.]).unwrap();let r=p.real_roots(1e-10,8).unwrap();assert_eq!(r.len(),2);assert!((r[0].value-2.).abs()<1e-8);assert!((r[1].value-3.).abs()<1e-8);}#[test]fn repeated_root_is_reported(){let p=Polynomial::new(vec![1.,-3.,3.,-1.]).unwrap();let r=p.real_roots(1e-8,8).unwrap();assert!(!r.is_empty());assert!((r[0].value-1.).abs()<1e-6);}#[test]fn nonfinite_is_rejected(){assert_eq!(Polynomial::new(vec![1.,f64::NAN]),Err(PolynomialError::NonFinite));}}
