//! Fixed-size matrix mathematics for the UMLCAD authority.
//!
//! Inversion uses scale normalization before singularity testing so a uniformly
//! rescaled matrix does not change classification merely because coordinates
//! are smaller than one. Tolerances are explicit inputs.

use super::vec::{Vec2, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatrixError { NonFinite, Singular, InvalidTolerance, DimensionMismatch, Overflow }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat2 { pub m: [[f64;2];2] }
impl Mat2 {
    pub const IDENTITY:Self=Self{m:[[1.0,0.0],[0.0,1.0]]};
    pub const fn new(m:[[f64;2];2])->Self{Self{m}}
    pub fn transpose(self)->Self{Self::new([[self.m[0][0],self.m[1][0]],[self.m[0][1],self.m[1][1]]])}
    pub fn determinant(self)->f64{self.m[0][0]*self.m[1][1]-self.m[0][1]*self.m[1][0]}
    pub fn inverse(self,tol:f64)->Result<Self,MatrixError>{validate2(self,tol)?;let scale=self.max_abs();if scale==0.0{return Err(MatrixError::Singular);}let a00=self.m[0][0]/scale;let a01=self.m[0][1]/scale;let a10=self.m[1][0]/scale;let a11=self.m[1][1]/scale;let det=a00*a11-a01*a10;if det.abs()<=tol{return Err(MatrixError::Singular);}let inv_scale=1.0/scale;let r=Self::new([[a11/det*inv_scale,-a01/det*inv_scale],[-a10/det*inv_scale,a00/det*inv_scale]]);if r.is_finite(){Ok(r)}else{Err(MatrixError::Overflow)}}
    pub fn solve(self,b:[f64;2],tol:f64)->Result<[f64;2],MatrixError>{if b.iter().any(|v|!v.is_finite()){return Err(MatrixError::NonFinite);}let i=self.inverse(tol)?;let r=[i.m[0][0]*b[0]+i.m[0][1]*b[1],i.m[1][0]*b[0]+i.m[1][1]*b[1]];if r.iter().all(|v|v.is_finite()){Ok(r)}else{Err(MatrixError::Overflow)}}
    pub fn mul(self,o:Self)->Self{let mut r=[[0.0;2];2];for i in 0..2{for j in 0..2{for k in 0..2{r[i][j]+=self.m[i][k]*o.m[k][j];}}}Self::new(r)}
    pub fn mul_vec(self,v:Vec2)->Result<Vec2,MatrixError>{if!self.is_finite()||!v.is_finite(){return Err(MatrixError::NonFinite);}let r=Vec2::new(self.m[0][0]*v.x+self.m[0][1]*v.y,self.m[1][0]*v.x+self.m[1][1]*v.y);if r.is_finite(){Ok(r)}else{Err(MatrixError::Overflow)}}
    pub fn max_abs(self)->f64{self.m.iter().flatten().map(|v|v.abs()).fold(0.0,f64::max)}
    pub fn norm_inf(self)->f64{(0..2).map(|i|self.m[i].iter().map(|v|v.abs()).sum()).fold(0.0,f64::max)}
    pub fn condition_estimate(self,tol:f64)->Result<f64,MatrixError>{let i=self.inverse(tol)?;let c=self.norm_inf()*i.norm_inf();if c.is_finite(){Ok(c)}else{Err(MatrixError::Overflow)}}
    fn is_finite(self)->bool{self.m.iter().flatten().all(|v|v.is_finite())}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat3 { pub m:[[f64;3];3] }
impl Mat3 {
    pub const IDENTITY:Self=Self{m:[[1.0,0.0,0.0],[0.0,1.0,0.0],[0.0,0.0,1.0]]};
    pub const fn new(m:[[f64;3];3])->Self{Self{m}}
    pub fn transpose(self)->Self{let mut r=[[0.0;3];3];for i in 0..3{for j in 0..3{r[i][j]=self.m[j][i];}}Self::new(r)}
    pub fn determinant(self)->f64{let a=self.m;a[0][0]*(a[1][1]*a[2][2]-a[1][2]*a[2][1])-a[0][1]*(a[1][0]*a[2][2]-a[1][2]*a[2][0])+a[0][2]*(a[1][0]*a[2][1]-a[1][1]*a[2][0])}
    pub fn inverse(self,tol:f64)->Result<Self,MatrixError>{validate3(self,tol)?;let scale=self.max_abs();if scale==0.0{return Err(MatrixError::Singular);}let a=std::array::from_fn(|i|std::array::from_fn(|j|self.m[i][j]/scale));let d=a[0][0]*(a[1][1]*a[2][2]-a[1][2]*a[2][1])-a[0][1]*(a[1][0]*a[2][2]-a[1][2]*a[2][0])+a[0][2]*(a[1][0]*a[2][1]-a[1][1]*a[2][0]);if d.abs()<=tol{return Err(MatrixError::Singular);}let c=[[a[1][1]*a[2][2]-a[1][2]*a[2][1],a[0][2]*a[2][1]-a[0][1]*a[2][2],a[0][1]*a[1][2]-a[0][2]*a[1][1]],[a[1][2]*a[2][0]-a[1][0]*a[2][2],a[0][0]*a[2][2]-a[0][2]*a[2][0],a[0][2]*a[1][0]-a[0][0]*a[1][2]],[a[1][0]*a[2][1]-a[1][1]*a[2][0],a[0][1]*a[2][0]-a[0][0]*a[2][1],a[0][0]*a[1][1]-a[0][1]*a[1][0]]];let inv_scale=1.0/scale;let r=Self::new(std::array::from_fn(|i|std::array::from_fn(|j|c[i][j]/d*inv_scale)));if r.is_finite(){Ok(r)}else{Err(MatrixError::Overflow)}}
    pub fn solve(self,b:[f64;3],tol:f64)->Result<[f64;3],MatrixError>{if b.iter().any(|v|!v.is_finite()){return Err(MatrixError::NonFinite);}let i=self.inverse(tol)?;let r=std::array::from_fn(|row|(0..3).map(|j|i.m[row][j]*b[j]).sum());if r.iter().all(|v|v.is_finite()){Ok(r)}else{Err(MatrixError::Overflow)}}
    pub fn mul(self,o:Self)->Self{let mut r=[[0.0;3];3];for i in 0..3{for j in 0..3{for k in 0..3{r[i][j]+=self.m[i][k]*o.m[k][j];}}}Self::new(r)}
    pub fn mul_vec(self,v:Vec3)->Result<Vec3,MatrixError>{if!self.is_finite()||!v.is_finite(){return Err(MatrixError::NonFinite);}let r=Vec3::new(self.m[0][0]*v.x+self.m[0][1]*v.y+self.m[0][2]*v.z,self.m[1][0]*v.x+self.m[1][1]*v.y+self.m[1][2]*v.z,self.m[2][0]*v.x+self.m[2][1]*v.y+self.m[2][2]*v.z);if r.is_finite(){Ok(r)}else{Err(MatrixError::Overflow)}}
    pub fn max_abs(self)->f64{self.m.iter().flatten().map(|v|v.abs()).fold(0.0,f64::max)}
    pub fn norm_inf(self)->f64{(0..3).map(|i|self.m[i].iter().map(|v|v.abs()).sum()).fold(0.0,f64::max)}
    pub fn condition_estimate(self,tol:f64)->Result<f64,MatrixError>{let i=self.inverse(tol)?;let c=self.norm_inf()*i.norm_inf();if c.is_finite(){Ok(c)}else{Err(MatrixError::Overflow)}}
    fn is_finite(self)->bool{self.m.iter().flatten().all(|v|v.is_finite())}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4 { pub m:[[f64;4];4] }
impl Mat4 {
    pub const IDENTITY:Self=Self{m:[[1.0,0.0,0.0,0.0],[0.0,1.0,0.0,0.0],[0.0,0.0,1.0,0.0],[0.0,0.0,0.0,1.0]]};
    pub const fn new(m:[[f64;4];4])->Self{Self{m}}
    pub fn transpose(self)->Self{let mut r=[[0.0;4];4];for i in 0..4{for j in 0..4{r[i][j]=self.m[j][i];}}Self::new(r)}
    pub fn determinant(self)->f64{if!self.is_finite(){return f64::NAN;}let s=self.max_abs();if s==0.0{return 0.0;}let mut a=std::array::from_fn(|i|std::array::from_fn(|j|self.m[i][j]/s));let mut d=1.0;for col in 0..4{let p=(col..4).max_by(|&i,&j|a[i][col].abs().total_cmp(&a[j][col].abs())).unwrap();if a[p][col]==0.0{return 0.0;}if p!=col{a.swap(p,col);d=-d;}let pivot=a[col][col];d*=pivot;for row in col+1..4{let f=a[row][col]/pivot;for k in col+1..4{a[row][k]-=f*a[col][k];}}}d*s.powi(4)}
    pub fn inverse(self,tol:f64)->Result<Self,MatrixError>{validate4(self,tol)?;let s=self.max_abs();if s==0.0{return Err(MatrixError::Singular);}let mut a=[[0.0;8];4];for i in 0..4{for j in 0..4{a[i][j]=self.m[i][j]/s;}a[i][4+i]=1.0;}for col in 0..4{let p=(col..4).max_by(|&i,&j|a[i][col].abs().total_cmp(&a[j][col].abs())).unwrap();if a[p][col].abs()<=tol{return Err(MatrixError::Singular);}if p!=col{a.swap(p,col);}let pivot=a[col][col];for j in 0..8{a[col][j]/=pivot;}for row in 0..4{if row==col{continue;}let f=a[row][col];for j in 0..8{a[row][j]-=f*a[col][j];}}}let inv_s=1.0/s;let r=Self::new(std::array::from_fn(|i|std::array::from_fn(|j|a[i][4+j]*inv_s)));if r.is_finite(){Ok(r)}else{Err(MatrixError::Overflow)}}
    pub fn solve(self,b:[f64;4],tol:f64)->Result<[f64;4],MatrixError>{if b.iter().any(|v|!v.is_finite()){return Err(MatrixError::NonFinite);}let i=self.inverse(tol)?;let r=std::array::from_fn(|row|(0..4).map(|j|i.m[row][j]*b[j]).sum());if r.iter().all(|v|v.is_finite()){Ok(r)}else{Err(MatrixError::Overflow)}}
    pub fn mul(self,o:Self)->Self{let mut r=[[0.0;4];4];for i in 0..4{for j in 0..4{for k in 0..4{r[i][j]+=self.m[i][k]*o.m[k][j];}}}Self::new(r)}
    pub fn max_abs(self)->f64{self.m.iter().flatten().map(|v|v.abs()).fold(0.0,f64::max)}
    pub fn norm_inf(self)->f64{(0..4).map(|i|self.m[i].iter().map(|v|v.abs()).sum()).fold(0.0,f64::max)}
    pub fn condition_estimate(self,tol:f64)->Result<f64,MatrixError>{let i=self.inverse(tol)?;let c=self.norm_inf()*i.norm_inf();if c.is_finite(){Ok(c)}else{Err(MatrixError::Overflow)}}
    fn is_finite(self)->bool{self.m.iter().flatten().all(|v|v.is_finite())}
}
fn validate2(m:Mat2,t:f64)->Result<(),MatrixError>{if!m.is_finite(){return Err(MatrixError::NonFinite);}validate_t(t)}
fn validate3(m:Mat3,t:f64)->Result<(),MatrixError>{if!m.is_finite(){return Err(MatrixError::NonFinite);}validate_t(t)}
fn validate4(m:Mat4,t:f64)->Result<(),MatrixError>{if!m.is_finite(){return Err(MatrixError::NonFinite);}validate_t(t)}
fn validate_t(t:f64)->Result<(),MatrixError>{if t.is_finite()&&t>=0.0{Ok(())}else{Err(MatrixError::InvalidTolerance)}}

#[cfg(test)]
mod tests{use super::*;const T:f64=1e-12;#[test]fn inverse_round_trips(){let a=Mat2::new([[2.0,1.0],[1.0,3.0]]);let p=a.mul(a.inverse(T).unwrap());assert!((p.m[0][0]-1.0).abs()<1e-12);assert!(p.m[0][1].abs()<1e-12);assert!(p.m[1][0].abs()<1e-12);assert!((p.m[1][1]-1.0).abs()<1e-12);}#[test]fn mat3_solve(){let a=Mat3::new([[1.,2.,3.],[0.,1.,4.],[5.,6.,0.]]);let x=a.solve([14.,14.,23.],T).unwrap();assert!((x[0]-1.).abs()<1e-12&&(x[1]-2.).abs()<1e-12&&(x[2]-3.).abs()<1e-12);}#[test]fn mat4_identity(){assert_eq!(Mat4::IDENTITY.inverse(T).unwrap(),Mat4::IDENTITY);assert!((Mat4::IDENTITY.condition_estimate(T).unwrap()-1.).abs()<1e-12);}#[test]fn singular_is_rejected(){assert_eq!(Mat2::new([[1.,2.],[2.,4.]]).inverse(T),Err(MatrixError::Singular));}#[test]fn tiny_full_rank_matrix_is_not_misclassified_by_unit_floor(){assert!(Mat2::new([[2e-12,1e-12],[1e-12,3e-12]]).inverse(T).is_ok());}}
