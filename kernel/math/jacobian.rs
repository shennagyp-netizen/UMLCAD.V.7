//! Analytic Jacobians for the current semantic constraint equations.
//!
//! The Jacobian is evaluated directly from the immutable snapshot geometry;
//! no finite-difference perturbation is used. Rows correspond exactly to the
//! residual equations emitted by `constraints::residual` for supported
//! constraints. Undefined derivatives, such as distance at coincident points,
//! are reported as `Indeterminate` rather than guessed.

use super::{constraints::endpoint, geometry::{Arc,Circle,Geometry,Line,Point}, snapshot::{Constraint,Endpoint,SemanticSnapshot}};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum JacobianError{UnknownGeometry,UnsupportedConstraint,InvalidDomain,NonFinite,Indeterminate}

pub fn geometry_vector_layout(snapshot:&SemanticSnapshot)->(Vec<String>,Vec<usize>){
 let mut ids=Vec::with_capacity(snapshot.geometry.len());let mut offsets=vec![0usize];
 for item in &snapshot.geometry{ids.push(item.id.clone());let width=match item.geometry{Geometry::Line(_)=>4,Geometry::Circle(_)=>3,Geometry::Arc(_)=>5};offsets.push(offsets.last().copied().unwrap()+width);}
 (ids,offsets)
}
fn index(ids:&[String],id:&str)->Option<usize>{ids.iter().position(|x|x==id)}
fn add_block(row:&mut[f64],offset:usize,values:&[f64]){for(i,v)in values.iter().copied().enumerate(){row[offset+i]=v;}}
fn finite_row(row:&[f64])->bool{row.iter().all(|v|v.is_finite())}

fn endpoint_value(g:&Geometry,e:Endpoint)->Result<Point,JacobianError>{endpoint(g,e).ok_or(JacobianError::InvalidDomain)}
fn endpoint_derivative(g:&Geometry,e:Endpoint)->Result<Vec<f64>,JacobianError>{match(g,e){(Geometry::Line(_),Endpoint::Start)=>Ok(vec![1.,0.,0.,0.]),(Geometry::Line(_),Endpoint::End)=>Ok(vec![0.,0.,1.,0.]),(Geometry::Arc(a),Endpoint::Start)=>Ok(vec![1.,0.,a.start_angle.cos(),0.,-a.radius*a.start_angle.sin()]),(Geometry::Arc(a),Endpoint::End)=>Ok(vec![1.,0.,a.end_angle.cos(),0.,-a.radius*a.end_angle.sin()]),(Geometry::Circle(_),_)=>Err(JacobianError::InvalidDomain)}}
fn endpoint_derivative_y(g:&Geometry,e:Endpoint)->Result<Vec<f64>,JacobianError>{match(g,e){(Geometry::Line(_),Endpoint::Start)=>Ok(vec![0.,1.,0.,0.]),(Geometry::Line(_),Endpoint::End)=>Ok(vec![0.,0.,0.,1.]),(Geometry::Arc(a),Endpoint::Start)=>Ok(vec![0.,1.,a.start_angle.sin(),0.,a.radius*a.start_angle.cos()]),(Geometry::Arc(a),Endpoint::End)=>Ok(vec![0.,1.,a.end_angle.sin(),0.,a.radius*a.end_angle.cos()]),(Geometry::Circle(_),_)=>Err(JacobianError::InvalidDomain)}}

fn local_identity(width:usize)->Vec<Vec<f64>>{(0..width).map(|i|{let mut r=vec![0.;width];r[i]=1.;r}).collect()}

pub fn analytic_constraint_jacobian(snapshot:&SemanticSnapshot)->Result<Vec<Vec<f64>>,JacobianError>{
 let(total_ids,offsets)=geometry_vector_layout(snapshot);let total=offsets.last().copied().unwrap_or(0);let mut rows=Vec::new();
 for(_,constraint)in &snapshot.constraints{
  match constraint{
   Constraint::Horizontal{entity_id}=>{let i=index(&total_ids,entity_id).ok_or(JacobianError::UnknownGeometry)?;let width=offsets[i+1]-offsets[i];if width!=4{return Err(JacobianError::InvalidDomain);}let mut r=vec![0.;total];r[offsets[i]+3]=1.;r[offsets[i]+1]=-1.;rows.push(r);}
   Constraint::Vertical{entity_id}=>{let i=index(&total_ids,entity_id).ok_or(JacobianError::UnknownGeometry)?;let width=offsets[i+1]-offsets[i];if width!=4{return Err(JacobianError::InvalidDomain);}let mut r=vec![0.;total];r[offsets[i]+2]=1.;r[offsets[i]]= -1.;rows.push(r);}
   Constraint::Coincident{first_geometry_id,first_point,second_geometry_id,second_point}=>{
    let ia=index(&total_ids,first_geometry_id).ok_or(JacobianError::UnknownGeometry)?;let ib=index(&total_ids,second_geometry_id).ok_or(JacobianError::UnknownGeometry)?;let ga=&snapshot.geometry[ia].geometry;let gb=&snapshot.geometry[ib].geometry;let dx=endpoint_derivative(ga,*first_point)?;let ex=endpoint_derivative_x(ga,*first_point)?;let dy=endpoint_derivative_y(ga,*first_point)?;let bx=endpoint_derivative_x(gb,*second_point)?;let by=endpoint_derivative_y(gb,*second_point)?;let mut rx=vec![0.;total];let mut ry=vec![0.;total];add_block(&mut rx,offsets[ia],&ex);add_block(&mut ry,offsets[ia],&dy);for(v,dv)in bx.iter().zip(by.iter()).enumerate(){let _=v;let _=dv;}add_block(&mut rx,offsets[ib],&bx.iter().map(|v|-*v).collect::<Vec<_>>());add_block(&mut ry,offsets[ib],&by.iter().map(|v|-*v).collect::<Vec<_>>());let _=dx;rows.push(rx);rows.push(ry);
   }
   Constraint::Fixed{entity_id}=>{let i=index(&total_ids,entity_id).ok_or(JacobianError::UnknownGeometry)?;let width=offsets[i+1]-offsets[i];for mut local in local_identity(width){let mut r=vec![0.;total];add_block(&mut r,offsets[i],&local);rows.push(r);}}
   Constraint::Distance{first_geometry_id,second_geometry_id,first_endpoint,second_endpoint,value:_}=>{
    let ia=index(&total_ids,first_geometry_id).ok_or(JacobianError::UnknownGeometry)?;let ga=&snapshot.geometry[ia].geometry;let ea=first_endpoint.unwrap_or(Endpoint::Start);
    if let Some(second_id)=second_geometry_id{
      let ib=index(&total_ids,second_id).ok_or(JacobianError::UnknownGeometry)?;let gb=&snapshot.geometry[ib].geometry;let eb=second_endpoint.unwrap_or(Endpoint::Start);let pa=endpoint_value(ga,ea)?;let pb=endpoint_value(gb,eb)?;let d=pa.distance(pb);if d==0.||!d.is_finite(){return Err(JacobianError::Indeterminate);}let ux=(pa.x-pb.x)/d;let uy=(pa.y-pb.y)/d;let dax=endpoint_derivative_x(ga,ea)?;let day=endpoint_derivative_y(ga,ea)?;let dbx=endpoint_derivative_x(gb,eb)?;let dby=endpoint_derivative_y(gb,eb)?;let mut r=vec![0.;total];let va=dax.iter().zip(day.iter()).map(|(x,y)|ux*x+uy*y).collect::<Vec<_>>();let vb=dbx.iter().zip(dby.iter()).map(|(x,y)|-(ux*x+uy*y)).collect::<Vec<_>>();add_block(&mut r,offsets[ia],&va);add_block(&mut r,offsets[ib],&vb);rows.push(r);
    }else{let mut r=vec![0.;total];match ga{Geometry::Line(Line{start,end})=>{let dx=end.x-start.x;let dy=end.y-start.y;let l=dx.hypot(dy);if l==0.{return Err(JacobianError::Indeterminate);}add_block(&mut r,offsets[ia],&[-dx/l,-dy/l,dx/l,dy/l]);},Geometry::Circle(Circle{..})=>add_block(&mut r,offsets[ia],&[0.,0.,1.]),Geometry::Arc(Arc{radius,start_angle,end_angle,..})=>{let s=(end_angle-start_angle).signum();add_block(&mut r,offsets[ia],&[0.,0.,(end_angle-start_angle).abs(),-radius*s,radius*s]);}}rows.push(r);}
   }
  }
 }
 if rows.iter().all(|r|finite_row(r)){Ok(rows)}else{Err(JacobianError::NonFinite)}
}

fn endpoint_derivative_x(g:&Geometry,e:Endpoint)->Result<Vec<f64>,JacobianError>{match(g,e){(Geometry::Line(_),Endpoint::Start)=>Ok(vec![1.,0.,0.,0.]),(Geometry::Line(_),Endpoint::End)=>Ok(vec![0.,0.,1.,0.]),(Geometry::Arc(a),Endpoint::Start)=>Ok(vec![1.,0.,a.start_angle.cos(),-a.radius*a.start_angle.sin(),0.]),(Geometry::Arc(a),Endpoint::End)=>Ok(vec![1.,0.,a.end_angle.cos(),0.,-a.radius*a.end_angle.sin()]),(Geometry::Circle(_),_)=>Err(JacobianError::InvalidDomain)}}

#[cfg(test)]
mod tests{use super::*;use crate::math::{geometry::{Geometry,Line,Point,Circle,Arc},snapshot::{Constraint,GeometryItem,SemanticSnapshot}};fn snap(g:Vec<GeometryItem>,c:Vec<Constraint>)->SemanticSnapshot{SemanticSnapshot{parameters:vec![],geometry:g,constraints:c.into_iter().enumerate().map(|(i,c)|(format!("c{i}"),c)).collect(),relations:vec![]}}
 #[test]fn horizontal_and_vertical_have_exact_sparse_rows(){let s=snap(vec![GeometryItem{id:"l".into(),geometry:Geometry::Line(Line{start:Point{x:0.,y:0.},end:Point{x:1.,y:1.}}),parameter_dependencies:vec![]}],vec![Constraint::Horizontal{entity_id:"l".into()},Constraint::Vertical{entity_id:"l".into()}]);let j=analytic_constraint_jacobian(&s).unwrap();assert_eq!(j.len(),2);assert_eq!(j[0],vec![0.,-1.,0.,1.]);assert_eq!(j[1],vec![-1.,0.,1.,0.]);}
 #[test]fn coincident_arc_endpoint_derivative_matches_formula(){let s=snap(vec![GeometryItem{id:"a".into(),geometry:Geometry::Arc(Arc{center:Point{x:0.,y:0.},radius:2.,start_angle:0.3,end_angle:1.2}),parameter_dependencies:vec![]},GeometryItem{id:"l".into(),geometry:Geometry::Line(Line{start:Point{x:3.,y:4.},end:Point{x:4.,y:4.}}),parameter_dependencies:vec![]}],vec![Constraint::Coincident{first_geometry_id:"a".into(),first_point:Endpoint::Start,second_geometry_id:"l".into(),second_point:Endpoint::Start}]);let j=analytic_constraint_jacobian(&s).unwrap();assert_eq!(j.len(),2);assert!((j[0][2]-0.3f64.cos()).abs()<1e-15);assert!((j[1][4]-2.*0.3f64.cos()).abs()<1e-15);}
 #[test]fn line_length_jacobian_is_unit_endpoint_direction(){let s=snap(vec![GeometryItem{id:"l".into(),geometry:Geometry::Line(Line{start:Point{x:0.,y:0.},end:Point{x:3.,y:4.}}),parameter_dependencies:vec![]}],vec![Constraint::Distance{first_geometry_id:"l".into(),second_geometry_id:None,first_endpoint:None,second_endpoint:None,value:5.}]);let j=analytic_constraint_jacobian(&s).unwrap();assert_eq!(j[0],vec![-0.6,-0.8,0.6,0.8]);}
 #[test]fn coincident_distance_has_no_finite_derivative(){let s=snap(vec![GeometryItem{id:"a".into(),geometry:Geometry::Line(Line{start:Point{x:0.,y:0.},end:Point{x:1.,y:0.}}),parameter_dependencies:vec![]},GeometryItem{id:"b".into(),geometry:Geometry::Line(Line{start:Point{x:0.,y:0.},end:Point{x:0.,y:1.}}),parameter_dependencies:vec![]}],vec![Constraint::Distance{first_geometry_id:"a".into(),second_geometry_id:Some("b".into()),first_endpoint:Some(Endpoint::Start),second_endpoint:Some(Endpoint::Start),value:0.}]);assert_eq!(analytic_constraint_jacobian(&s),Err(JacobianError::Indeterminate));}
}
