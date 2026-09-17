//! Four-dimensional vector values used by homogeneous/projective mathematics.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec4 { pub x: f64, pub y: f64, pub z: f64, pub w: f64 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vec4Error { NonFinite, Degenerate, Overflow }

impl Vec4 {
    pub const fn new(x:f64,y:f64,z:f64,w:f64)->Self{Self{x,y,z,w}}
    pub fn is_finite(self)->bool{self.x.is_finite()&&self.y.is_finite()&&self.z.is_finite()&&self.w.is_finite()}
    pub fn dot(self,o:Self)->f64{self.x*o.x+self.y*o.y+self.z*o.z+self.w*o.w}
    pub fn length(self)->f64{self.x.hypot(self.y).hypot(self.z).hypot(self.w)}
    pub fn normalized(self)->Result<Self,Vec4Error>{if!self.is_finite(){return Err(Vec4Error::NonFinite);}let l=self.length();if!l.is_finite(){return Err(Vec4Error::Overflow);}if l==0.0{return Err(Vec4Error::Degenerate);}let r=Self::new(self.x/l,self.y/l,self.z/l,self.w/l);if r.is_finite(){Ok(r)}else{Err(Vec4Error::Overflow)}}
    pub fn add(self,o:Self)->Self{Self::new(self.x+o.x,self.y+o.y,self.z+o.z,self.w+o.w)}
    pub fn sub(self,o:Self)->Self{Self::new(self.x-o.x,self.y-o.y,self.z-o.z,self.w-o.w)}
    pub fn scale(self,s:f64)->Self{Self::new(self.x*s,self.y*s,self.z*s,self.w*s)}
}

#[cfg(test)]
mod tests{use super::*;#[test]fn homogeneous_normalization(){let v=Vec4::new(1.,2.,3.,4.);let n=v.normalized().unwrap();assert!((n.length()-1.).abs()<1e-15);}#[test]fn extreme_length_is_finite(){let v=Vec4::new(1e308,-1e308,0.,0.);assert!(v.length().is_finite());}#[test]fn invalid_vector_is_rejected(){assert_eq!(Vec4::new(0.,0.,0.,0.).normalized(),Err(Vec4Error::Degenerate));}}
