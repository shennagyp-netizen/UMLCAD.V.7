//! Backend-neutral GPU execution contracts.
//!
//! This module contains no fake GPU implementation. CPU is the normative
//! reference; Metal and CUDA implementations must satisfy these contracts when
//! added. Device/resource state is outside mathematical semantics.

use super::vec::Vec3;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum BackendKind{Auto,Cpu,Metal,Cuda}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Precision{F64}
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct BackendCapability{pub backend:BackendKind,pub available:bool,pub f64:bool,pub deterministic:bool,pub max_batch:Option<usize>}
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct DispatchPolicy{pub preferred:BackendKind,pub minimum_parallel_batch:usize,pub require_determinism:bool,pub required_precision:Precision}
#[derive(Clone,Debug,PartialEq)]
pub struct VectorBatch3{pub values:Vec<Vec3>}
#[derive(Clone,Debug,PartialEq)]
pub enum GpuError{Unavailable,UnsupportedPrecision,UnsupportedOperation,InvalidBatch,NonFinite,BackendFailure}

impl Default for DispatchPolicy{fn default()->Self{Self{preferred:BackendKind::Auto,minimum_parallel_batch:1024,require_determinism:true,required_precision:Precision::F64}}}
impl VectorBatch3{pub fn new(values:Vec<Vec3>)->Result<Self,GpuError>{if values.iter().any(|v|!v.is_finite()){Err(GpuError::NonFinite)}else{Ok(Self{values})}}pub fn len(&self)->usize{self.values.len()}pub fn is_empty(&self)->bool{self.values.is_empty()}}

pub fn select_backend(policy:DispatchPolicy,cpu:BackendCapability,metal:BackendCapability,cuda:BackendCapability,batch_size:usize)->BackendKind{
 let candidates=[metal,cuda];
 let acceptable=|c:BackendCapability|c.available&&c.f64&&(!policy.require_determinism||c.deterministic)&&c.max_batch.map_or(true,|m|batch_size<=m);
 match policy.preferred{
  BackendKind::Cpu=>BackendKind::Cpu,
  BackendKind::Metal=>if acceptable(metal){BackendKind::Metal}else{BackendKind::Cpu},
  BackendKind::Cuda=>if acceptable(cuda){BackendKind::Cuda}else{BackendKind::Cpu},
  BackendKind::Auto=>{if batch_size<policy.minimum_parallel_batch{return BackendKind::Cpu;}if acceptable(metal){BackendKind::Metal}else if acceptable(cuda){BackendKind::Cuda}else{BackendKind::Cpu}}
 }
}

pub trait BatchExecutor {
 fn backend(&self)->BackendKind;
 fn capability(&self)->BackendCapability;
 fn evaluate_points(&self,input:&VectorBatch3)->Result<VectorBatch3,GpuError>;
}

#[derive(Clone,Copy,Debug,Default)]
pub struct CpuReferenceExecutor;
impl BatchExecutor for CpuReferenceExecutor{
 fn backend(&self)->BackendKind{BackendKind::Cpu}
 fn capability(&self)->BackendCapability{BackendCapability{backend:BackendKind::Cpu,available:true,f64:true,deterministic:true,max_batch:None}}
 fn evaluate_points(&self,input:&VectorBatch3)->Result<VectorBatch3,GpuError>{VectorBatch3::new(input.values.clone())}
}

#[cfg(test)]
mod tests{use super::*;#[test]fn cpu_is_selected_for_small_auto_batch(){let p=DispatchPolicy::default();let cpu=BackendCapability{backend:BackendKind::Cpu,available:true,f64:true,deterministic:true,max_batch:None};let gpu=BackendCapability{backend:BackendKind::Metal,available:true,f64:true,deterministic:true,max_batch:None};assert_eq!(select_backend(p,cpu,gpu,gpu,16),BackendKind::Cpu);}#[test]fn auto_selects_metal_when_capable_and_large_enough(){let p=DispatchPolicy{minimum_parallel_batch:8,..Default::default()};let cpu=BackendCapability{backend:BackendKind::Cpu,available:true,f64:true,deterministic:true,max_batch:None};let metal=BackendCapability{backend:BackendKind::Metal,available:true,f64:true,deterministic:true,max_batch:None};let cuda=BackendCapability{backend:BackendKind::Cuda,available:false,f64:false,deterministic:false,max_batch:None};assert_eq!(select_backend(p,cpu,metal,cuda,8),BackendKind::Metal);}#[test]fn deterministic_requirement_rejects_nondeterministic_gpu(){let p=DispatchPolicy{minimum_parallel_batch:1,require_determinism:true,..Default::default()};let cpu=BackendCapability{backend:BackendKind::Cpu,available:true,f64:true,deterministic:true,max_batch:None};let metal=BackendCapability{backend:BackendKind::Metal,available:true,f64:true,deterministic:false,max_batch:None};assert_eq!(select_backend(p,cpu,metal,metal,4),BackendKind::Cpu);}#[test]fn cpu_executor_preserves_immutable_batch(){let input=VectorBatch3::new(vec![Vec3::new(1.,2.,3.)]).unwrap();let output=CpuReferenceExecutor.evaluate_points(&input).unwrap();assert_eq!(input,output);}}
