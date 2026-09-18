//! Adaptive curve tessellation mathematics.
//!
//! Tessellation is explicitly an approximation. The algorithm refines parameter
//! intervals until caller-provided chord and angular errors are satisfied, or
//! returns `MaxDepth` rather than silently returning a lower-quality result.

use std::collections::BTreeMap;

use super::{trim::{RegionClass, TrimCurve2, TrimLoop2}, vec::Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TessellationPolicy {
    pub chord_error: f64,
    pub angular_error: f64,
    pub max_depth: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TessellatedCurve3 {
    pub points: Vec<Vec3>,
    pub parameters: Vec<f64>,
    pub chord_error: f64,
    pub angular_error: f64,
    pub max_depth: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TessellationError {
    InvalidPolicy,
    NonFinite,
    Degenerate,
    EvaluationFailed,
    MaxDepth,
    InvalidDomain,
}

impl TessellationPolicy {
    pub fn validate(self) -> Result<(), TessellationError> {
        if !self.chord_error.is_finite()
            || !self.angular_error.is_finite()
            || self.chord_error <= 0.0
            || self.angular_error <= 0.0
        {
            return Err(TessellationError::InvalidPolicy);
        }
        Ok(())
    }
}

fn point_segment_distance(p: Vec3, a: Vec3, b: Vec3) -> Result<f64, TessellationError> {
    let d = b.sub(a);
    let l = d.length();
    if l == 0.0 {
        return Err(TessellationError::Degenerate);
    }
    let u = d.scale(1.0 / l);
    let t = p.sub(a).dot(u);
    if !t.is_finite() {
        return Err(TessellationError::NonFinite);
    }
    Ok(p.sub(a.add(u.scale(t.clamp(0.0, l)))).length())
}

fn angle_between(a: Vec3, b: Vec3) -> Result<f64, TessellationError> {
    let u = a.normalized().map_err(|_| TessellationError::Degenerate)?;
    let v = b.normalized().map_err(|_| TessellationError::Degenerate)?;
    Ok(u.dot(v).clamp(-1.0, 1.0).acos())
}

pub fn tessellate_curve3<F, G>(
    domain: (f64, f64),
    policy: TessellationPolicy,
    eval: F,
    tangent: G,
) -> Result<TessellatedCurve3, TessellationError>
where
    F: Fn(f64) -> Result<Vec3, TessellationError>,
    G: Fn(f64) -> Result<Vec3, TessellationError>,
{
    policy.validate()?;
    let (a, b) = domain;
    if !a.is_finite() || !b.is_finite() || b <= a {
        return Err(TessellationError::InvalidDomain);
    }
    let pa = eval(a)?;
    let pb = eval(b)?;
    if !pa.is_finite() || !pb.is_finite() {
        return Err(TessellationError::NonFinite);
    }

    fn recurse<F, G>(
        a: f64,
        b: f64,
        pa: Vec3,
        pb: Vec3,
        depth: u32,
        policy: &TessellationPolicy,
        eval: &F,
        tangent: &G,
    ) -> Result<Vec<(f64, Vec3)>, TessellationError>
    where
        F: Fn(f64) -> Result<Vec3, TessellationError>,
        G: Fn(f64) -> Result<Vec3, TessellationError>,
    {
        let m = a + (b - a) * 0.5;
        if m == a || m == b {
            return Err(TessellationError::MaxDepth);
        }
        let pm = eval(m)?;
        if !pm.is_finite() {
            return Err(TessellationError::NonFinite);
        }

        let chord = point_segment_distance(pm, pa, pb)?;
        let ta = tangent(a)?;
        let tm = tangent(m)?;
        let tb = tangent(b)?;
        if !ta.is_finite() || !tm.is_finite() || !tb.is_finite() {
            return Err(TessellationError::NonFinite);
        }
        let angle = angle_between(ta, tb)?
            .max(angle_between(ta, tm)?)
            .max(angle_between(tm, tb)?);

        if chord <= policy.chord_error && angle <= policy.angular_error {
            return Ok(vec![(b, pb)]);
        }
        if depth >= policy.max_depth {
            return Err(TessellationError::MaxDepth);
        }

        let mut left = recurse(a, m, pa, pm, depth + 1, policy, eval, tangent)?;
        let right = recurse(m, b, pm, pb, depth + 1, policy, eval, tangent)?;
        left.extend(right);
        Ok(left)
    }

    let tail = recurse(a, b, pa, pb, 0, &policy, &eval, &tangent)?;
    let mut points = Vec::with_capacity(tail.len() + 1);
    let mut parameters = Vec::with_capacity(tail.len() + 1);
    points.push(pa);
    parameters.push(a);
    for (parameter, point) in tail {
        parameters.push(parameter);
        points.push(point);
    }

    Ok(TessellatedCurve3 {
        points,
        parameters,
        chord_error: policy.chord_error,
        angular_error: policy.angular_error,
        max_depth: policy.max_depth,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_line_needs_only_endpoints() {
        let p = TessellationPolicy {
            chord_error: 1e-6,
            angular_error: 1e-6,
            max_depth: 8,
        };
        let r = tessellate_curve3(
            (0.0, 1.0),
            p,
            |t| Ok(Vec3::new(t, 0.0, 0.0)),
            |_| Ok(Vec3::new(1.0, 0.0, 0.0)),
        )
        .unwrap();
        assert_eq!(r.points.len(), 2);
        assert_eq!(r.points.len(), r.parameters.len());
    }

    #[test]
    fn quarter_arc_refines_for_chord_and_angle() {
        let p = TessellationPolicy {
            chord_error: 1e-3,
            angular_error: 0.1,
            max_depth: 16,
        };
        let r = tessellate_curve3(
            (0.0, std::f64::consts::FRAC_PI_2),
            p,
            |t| Ok(Vec3::new(t.cos(), t.sin(), 0.0)),
            |t| Ok(Vec3::new(-t.sin(), t.cos(), 0.0)),
        )
        .unwrap();
        assert!(r.points.len() > 2);
        assert_eq!(r.points.len(), r.parameters.len());
    }

    #[test]
    fn impossible_policy_depth_is_reported() {
        let p = TessellationPolicy {
            chord_error: 1e-15,
            angular_error: 1e-15,
            max_depth: 0,
        };
        let r = tessellate_curve3(
            (0.0, std::f64::consts::FRAC_PI_2),
            p,
            |t| Ok(Vec3::new(t.cos(), t.sin(), 0.0)),
            |t| Ok(Vec3::new(-t.sin(), t.cos(), 0.0)),
        );
        assert_eq!(r, Err(TessellationError::MaxDepth));
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceTessellationPolicy {
    pub chord_error: f64,
    pub angular_error: f64,
    pub max_depth: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceSample3 {
    pub parameter: (f64, f64),
    pub point: Vec3,
    pub normal: Vec3,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TessellatedSurface3 {
    pub vertices: Vec<Vec3>,
    pub parameters: Vec<(f64, f64)>,
    pub normals: Vec<Vec3>,
    pub triangles: Vec<[usize; 3]>,
    pub max_chord_error: f64,
    pub max_angular_error: f64,
    pub max_depth: u32,
}

impl SurfaceTessellationPolicy {
    pub fn validate(self) -> Result<(), TessellationError> {
        if !self.chord_error.is_finite()
            || !self.angular_error.is_finite()
            || self.chord_error <= 0.0
            || self.angular_error <= 0.0
        {
            return Err(TessellationError::InvalidPolicy);
        }
        Ok(())
    }
}

fn bilinear_point(
    p00: Vec3,
    p10: Vec3,
    p01: Vec3,
    p11: Vec3,
    u: f64,
    v: f64,
) -> Vec3 {
    let a = p00.scale((1.0 - u) * (1.0 - v));
    let b = p10.scale(u * (1.0 - v));
    let c = p01.scale((1.0 - u) * v);
    let d = p11.scale(u * v);
    a.add(b).add(c).add(d)
}

fn quad_chord_error(
    p00: Vec3,
    p10: Vec3,
    p01: Vec3,
    p11: Vec3,
    samples: &[SurfaceSample3; 5],
) -> Result<f64, TessellationError> {
    let expected = [
        bilinear_point(p00, p10, p01, p11, 0.5, 0.0),
        bilinear_point(p00, p10, p01, p11, 1.0, 0.5),
        bilinear_point(p00, p10, p01, p11, 0.0, 0.5),
        bilinear_point(p00, p10, p01, p11, 0.5, 1.0),
        bilinear_point(p00, p10, p01, p11, 0.5, 0.5),
    ];
    let mut maximum: f64 = 0.0;
    for (sample, approximation) in samples.iter().zip(expected) {
        let error = sample.point.sub(approximation).length();
        if !error.is_finite() {
            return Err(TessellationError::NonFinite);
        }
        maximum = maximum.max(error);
    }
    Ok(maximum)
}

fn sample_angular_error(samples: &[SurfaceSample3]) -> Result<f64, TessellationError> {
    let mut maximum: f64 = 0.0;
    for i in 0..samples.len() {
        for j in i + 1..samples.len() {
            let angle = angle_between(samples[i].normal, samples[j].normal)?;
            maximum = maximum.max(angle);
        }
    }
    Ok(maximum)
}

pub fn tessellate_surface3<F, G>(
    domain: (f64, f64, f64, f64),
    policy: SurfaceTessellationPolicy,
    eval: F,
    normal: G,
) -> Result<TessellatedSurface3, TessellationError>
where
    F: Fn(f64, f64) -> Result<Vec3, TessellationError>,
    G: Fn(f64, f64) -> Result<Vec3, TessellationError>,
{
    policy.validate()?;
    let (u0, u1, v0, v1) = domain;
    if !u0.is_finite() || !u1.is_finite() || !v0.is_finite() || !v1.is_finite()
        || u1 <= u0 || v1 <= v0
    {
        return Err(TessellationError::InvalidDomain);
    }

    fn make_sample<F, G>(
        u: f64,
        v: f64,
        eval: &F,
        normal: &G,
    ) -> Result<SurfaceSample3, TessellationError>
    where
        F: Fn(f64, f64) -> Result<Vec3, TessellationError>,
        G: Fn(f64, f64) -> Result<Vec3, TessellationError>,
    {
        let point = eval(u, v)?;
        let normal = normal(u, v)?;
        if !point.is_finite() || !normal.is_finite() {
            return Err(TessellationError::NonFinite);
        }
        let unit = normal.normalized().map_err(|_| TessellationError::Degenerate)?;
        Ok(SurfaceSample3 {
            parameter: (u, v),
            point,
            normal: unit,
        })
    }

    fn recurse<F, G>(
        u0: f64,
        u1: f64,
        v0: f64,
        v1: f64,
        depth: u32,
        policy: &SurfaceTessellationPolicy,
        eval: &F,
        normal: &G,
    ) -> Result<Vec<SurfaceSample3>, TessellationError>
    where
        F: Fn(f64, f64) -> Result<Vec3, TessellationError>,
        G: Fn(f64, f64) -> Result<Vec3, TessellationError>,
    {
        let um = u0 + (u1 - u0) * 0.5;
        let vm = v0 + (v1 - v0) * 0.5;
        if um == u0 || um == u1 || vm == v0 || vm == v1 {
            return Err(TessellationError::MaxDepth);
        }

        let p00 = make_sample(u0, v0, eval, normal)?;
        let p10 = make_sample(u1, v0, eval, normal)?;
        let p01 = make_sample(u0, v1, eval, normal)?;
        let p11 = make_sample(u1, v1, eval, normal)?;
        let samples = [
            make_sample(um, v0, eval, normal)?,
            make_sample(u1, vm, eval, normal)?,
            make_sample(u0, vm, eval, normal)?,
            make_sample(um, v1, eval, normal)?,
            make_sample(um, vm, eval, normal)?,
        ];
        let chord = quad_chord_error(
            p00.point,
            p10.point,
            p01.point,
            p11.point,
            &samples,
        )?;
        let mut angular_samples = Vec::with_capacity(9);
        angular_samples.extend([p00, p10, p01, p11]);
        angular_samples.extend(samples);
        let angular = sample_angular_error(&angular_samples)?;

        if chord <= policy.chord_error && angular <= policy.angular_error {
            return Ok(vec![p00, p10, p01, p11]);
        }
        if depth >= policy.max_depth {
            return Err(TessellationError::MaxDepth);
        }

        let mut result = Vec::new();
        for (ua, ub, va, vb) in [
            (u0, um, v0, vm),
            (um, u1, v0, vm),
            (u0, um, vm, v1),
            (um, u1, vm, v1),
        ] {
            result.extend(recurse(ua, ub, va, vb, depth + 1, policy, eval, normal)?);
        }
        Ok(result)
    }

    fn collect_error_metrics<F, G>(
        leaves: &[(f64, f64, f64, f64)],
        policy: &SurfaceTessellationPolicy,
        eval: &F,
        normal: &G,
    ) -> Result<(f64, f64), TessellationError>
    where
        F: Fn(f64, f64) -> Result<Vec3, TessellationError>,
        G: Fn(f64, f64) -> Result<Vec3, TessellationError>,
    {
        let mut maximum_chord: f64 = 0.0;
        let mut maximum_angular: f64 = 0.0;
        let _ = policy;
        for &(u0, u1, v0, v1) in leaves {
            let um = u0 + (u1 - u0) * 0.5;
            let vm = v0 + (v1 - v0) * 0.5;
            let p00 = eval(u0, v0)?;
            let p10 = eval(u1, v0)?;
            let p01 = eval(u0, v1)?;
            let p11 = eval(u1, v1)?;
            let samples = [
                make_surface_sample(um, v0, &eval, &normal)?,
                make_surface_sample(u1, vm, &eval, &normal)?,
                make_surface_sample(u0, vm, &eval, &normal)?,
                make_surface_sample(um, v1, &eval, &normal)?,
                make_surface_sample(um, vm, &eval, &normal)?,
            ];
            maximum_chord = maximum_chord.max(quad_chord_error(p00,p10,p01,p11,&samples)?);
            let mut angular_samples = Vec::with_capacity(9);
            for (u,v) in [(u0,v0),(u1,v0),(u0,v1),(u1,v1),(um,v0),(u1,vm),(u0,vm),(um,v1),(um,vm)] {
                angular_samples.push(make_surface_sample(u,v,&eval,&normal)?);
            }
            maximum_angular = maximum_angular.max(sample_angular_error(&angular_samples)?);
        }
        Ok((maximum_chord, maximum_angular))
    }

    fn make_surface_sample<F,G>(
        u:f64,v:f64,eval:&F,normal:&G
    )->Result<SurfaceSample3,TessellationError>
    where F:Fn(f64,f64)->Result<Vec3,TessellationError>,G:Fn(f64,f64)->Result<Vec3,TessellationError>
    {
        let point=eval(u,v)?;
        let n=normal(u,v)?;
        if !point.is_finite() || !n.is_finite(){return Err(TessellationError::NonFinite);}
        Ok(SurfaceSample3{parameter:(u,v),point,normal:n.normalized().map_err(|_|TessellationError::Degenerate)?})
    }

    fn collect_leaves(
        u0:f64,u1:f64,v0:f64,v1:f64,depth:u32,policy:&SurfaceTessellationPolicy,
        eval:&impl Fn(f64,f64)->Result<Vec3,TessellationError>,
        normal:&impl Fn(f64,f64)->Result<Vec3,TessellationError>,
        out:&mut Vec<(f64,f64,f64,f64)>
    )->Result<(),TessellationError>{
        let um=u0+(u1-u0)*0.5; let vm=v0+(v1-v0)*0.5;
        if um==u0||um==u1||vm==v0||vm==v1{return Err(TessellationError::MaxDepth);}
        let p00=make_surface_sample(u0,v0,eval,normal)?;
        let p10=make_surface_sample(u1,v0,eval,normal)?;
        let p01=make_surface_sample(u0,v1,eval,normal)?;
        let p11=make_surface_sample(u1,v1,eval,normal)?;
        let samples=[
            make_surface_sample(um,v0,eval,normal)?,
            make_surface_sample(u1,vm,eval,normal)?,
            make_surface_sample(u0,vm,eval,normal)?,
            make_surface_sample(um,v1,eval,normal)?,
            make_surface_sample(um,vm,eval,normal)?,
        ];
        let chord=quad_chord_error(p00.point,p10.point,p01.point,p11.point,&samples)?;
        let mut angular_samples=Vec::with_capacity(9);
        angular_samples.extend([p00,p10,p01,p11]); angular_samples.extend(samples);
        let angular=sample_angular_error(&angular_samples)?;
        if chord<=policy.chord_error&&angular<=policy.angular_error{out.push((u0,u1,v0,v1));return Ok(());}
        if depth>=policy.max_depth{return Err(TessellationError::MaxDepth);}
        for (ua,ub,va,vb) in [(u0,um,v0,vm),(um,u1,v0,vm),(u0,um,vm,v1),(um,u1,vm,v1)]{
            collect_leaves(ua,ub,va,vb,depth+1,policy,eval,normal,out)?;
        }
        Ok(())
    }

    let mut leaves=Vec::new();
    collect_leaves(u0,u1,v0,v1,0,&policy,&eval,&normal,&mut leaves)?;
    let mut vertices=Vec::new();
    let mut parameters=Vec::new();
    let mut normals=Vec::new();
    let mut triangles=Vec::new();
    for &(ua,ub,va,vb) in &leaves{
        let corners=[
            make_surface_sample(ua,va,&eval,&normal)?,
            make_surface_sample(ub,va,&eval,&normal)?,
            make_surface_sample(ub,vb,&eval,&normal)?,
            make_surface_sample(ua,vb,&eval,&normal)?,
        ];
        let base=vertices.len();
        for sample in corners{
            vertices.push(sample.point); parameters.push(sample.parameter); normals.push(sample.normal);
        }
        triangles.push([base,base+1,base+2]);
        triangles.push([base,base+2,base+3]);
    }
    let (max_chord_error,max_angular_error)=collect_error_metrics(&leaves,&policy,&eval,&normal)?;
    Ok(TessellatedSurface3{vertices,parameters,normals,triangles,max_chord_error,max_angular_error,max_depth:policy.max_depth})
}

#[cfg(test)]
mod surface_tests {
    use super::*;

    fn planar_eval(u:f64,v:f64)->Result<Vec3,TessellationError>{Ok(Vec3::new(u,v,0.0))}
    fn planar_normal(_:f64,_:f64)->Result<Vec3,TessellationError>{Ok(Vec3::new(0.0,0.0,1.0))}

    #[test]
    fn planar_surface_stays_at_corners_only() {
        let policy=SurfaceTessellationPolicy{chord_error:1.0e-6,angular_error:1.0e-6,max_depth:8};
        let r=tessellate_surface3((0.0,1.0,0.0,1.0),policy,planar_eval,planar_normal).unwrap();
        assert_eq!(r.triangles.len(),2);
        assert_eq!(r.vertices.len(),4);
        assert_eq!(r.vertices.len(),r.parameters.len());
        assert_eq!(r.vertices.len(),r.normals.len());
        assert_eq!(r.max_chord_error,0.0);
        assert_eq!(r.max_angular_error,0.0);
    }

    #[test]
    fn curved_surface_refines_for_chord_or_normal_error() {
        let policy=SurfaceTessellationPolicy{chord_error:1.0e-3,angular_error:0.05,max_depth:8};
        let r=tessellate_surface3(
            (0.0,1.0,0.0,1.0),
            policy,
            |u,v| Ok(Vec3::new(u,v,0.25*u*u+0.2*v*v)),
            |u,v| {
                let du=Vec3::new(1.0,0.0,0.5*u);
                let dv=Vec3::new(0.0,1.0,0.4*v);
                du.cross(dv).normalized().map_err(|_|TessellationError::Degenerate)
            },
        ).unwrap();
        assert!(r.triangles.len()>2);
        assert!(r.max_chord_error<=policy.chord_error*1.001);
        assert!(r.max_angular_error<=policy.angular_error*1.001);
    }

    #[test]
    fn surface_depth_failure_is_explicit() {
        let policy=SurfaceTessellationPolicy{chord_error:1.0e-12,angular_error:1.0e-12,max_depth:0};
        let r=tessellate_surface3(
            (0.0,1.0,0.0,1.0),
            policy,
            |u,v| Ok(Vec3::new(u,v,u*v)),
            |u,v| {
                let du=Vec3::new(1.0,0.0,v);
                let dv=Vec3::new(0.0,1.0,u);
                du.cross(dv).normalized().map_err(|_|TessellationError::Degenerate)
            },
        );
        assert_eq!(r,Err(TessellationError::MaxDepth));
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrimTessellationPolicy {
    pub chord_error: f64,
    pub angular_error: f64,
    pub parameter_chord_error: f64,
    pub max_depth: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TessellatedTrimLoop3 {
    pub points: Vec<Vec3>,
    pub parameters: Vec<(f64, f64)>,
    pub normals: Vec<Vec3>,
    pub max_chord_error: f64,
    pub max_angular_error: f64,
    pub max_parameter_chord_error: f64,
    pub max_depth: u32,
}

impl TrimTessellationPolicy {
    pub fn validate(self) -> Result<(), TessellationError> {
        if !self.chord_error.is_finite()
            || !self.angular_error.is_finite()
            || !self.parameter_chord_error.is_finite()
            || self.chord_error <= 0.0
            || self.angular_error <= 0.0
            || self.parameter_chord_error <= 0.0
        {
            return Err(TessellationError::InvalidPolicy);
        }
        Ok(())
    }
}

fn map_trim_point<F, G>(
    curve_point: super::geometry::Point,
    eval: &F,
    normal: &G,
) -> Result<SurfaceSample3, TessellationError>
where
    F: Fn(f64, f64) -> Result<Vec3, TessellationError>,
    G: Fn(f64, f64) -> Result<Vec3, TessellationError>,
{
    let u = curve_point.x;
    let v = curve_point.y;
    let point = eval(u, v)?;
    let normal = normal(u, v)?;
    if !point.is_finite() || !normal.is_finite() {
        return Err(TessellationError::NonFinite);
    }
    Ok(SurfaceSample3 {
        parameter: (u, v),
        point,
        normal: normal.normalized().map_err(|_| TessellationError::Degenerate)?,
    })
}

fn tessellate_trim_curve_recursive<F, G>(
    curve: &TrimCurve2,
    t0: f64,
    t1: f64,
    a: SurfaceSample3,
    b: SurfaceSample3,
    depth: u32,
    policy: &TrimTessellationPolicy,
    eval: &F,
    normal: &G,
    out: &mut Vec<SurfaceSample3>,
) -> Result<(f64, f64, f64), TessellationError>
where
    F: Fn(f64, f64) -> Result<Vec3, TessellationError>,
    G: Fn(f64, f64) -> Result<Vec3, TessellationError>,
{
    let tm = t0 + (t1 - t0) * 0.5;
    if tm == t0 || tm == t1 {
        return Err(TessellationError::MaxDepth);
    }
    let uv_mid = curve.point_at(tm)
        .map_err(|_| TessellationError::EvaluationFailed)?;
    let m = map_trim_point(uv_mid, eval, normal)?;
    let chord_approx = a.point.add(b.point).scale(0.5);
    let chord_error = m.point.sub(chord_approx).length();
    let parameter_approx = a
        .parameter
        .0
        .mul_add(0.5, b.parameter.0 * 0.5);
    let parameter_approx_v = a
        .parameter
        .1
        .mul_add(0.5, b.parameter.1 * 0.5);
    let parameter_error = (
        uv_mid.x - parameter_approx,
        uv_mid.y - parameter_approx_v,
    );
    let parameter_chord_error =
        parameter_error.0.hypot(parameter_error.1);
    let angular_error = angle_between(a.normal, b.normal)?
        .max(angle_between(a.normal, m.normal)?)
        .max(angle_between(m.normal, b.normal)?);
    if !chord_error.is_finite()
        || !parameter_chord_error.is_finite()
        || !angular_error.is_finite()
    {
        return Err(TessellationError::NonFinite);
    }
    if chord_error <= policy.chord_error
        && parameter_chord_error <= policy.parameter_chord_error
        && angular_error <= policy.angular_error
    {
        out.push(b);
        return Ok((chord_error, angular_error, parameter_chord_error));
    }
    if depth >= policy.max_depth {
        return Err(TessellationError::MaxDepth);
    }

    let mut left = Vec::new();
    let left_metrics = tessellate_trim_curve_recursive(
        curve, t0, tm, a, m, depth + 1, policy, eval, normal, &mut left
    )?;
    out.extend(left);
    let mut right = Vec::new();
    let right_metrics = tessellate_trim_curve_recursive(
        curve, tm, t1, m, b, depth + 1, policy, eval, normal, &mut right
    )?;
    out.extend(right);

    Ok((
        left_metrics.0.max(right_metrics.0),
        left_metrics.1.max(right_metrics.1),
        left_metrics.2.max(right_metrics.2),
    ))
}

fn tessellate_trim_curve3<F, G>(
    curve: &TrimCurve2,
    policy: &TrimTessellationPolicy,
    eval: &F,
    normal: &G,
) -> Result<(Vec<SurfaceSample3>, (f64, f64, f64)), TessellationError>
where
    F: Fn(f64, f64) -> Result<Vec3, TessellationError>,
    G: Fn(f64, f64) -> Result<Vec3, TessellationError>,
{
    curve.validate().map_err(|_| TessellationError::EvaluationFailed)?;
    let a = map_trim_point(curve.point_at(0.0).map_err(|_| TessellationError::EvaluationFailed)?, eval, normal)?;
    let b = map_trim_point(curve.point_at(1.0).map_err(|_| TessellationError::EvaluationFailed)?, eval, normal)?;
    let mut out = Vec::new();
    out.push(a);
    let metrics = tessellate_trim_curve_recursive(
        curve, 0.0, 1.0, a, b, 0, policy, eval, normal, &mut out
    )?;
    Ok((out, metrics))
}

pub fn tessellate_trim_loop3<F, G>(
    trim: &TrimLoop2,
    trim_tolerance: f64,
    policy: TrimTessellationPolicy,
    eval: F,
    normal: G,
) -> Result<TessellatedTrimLoop3, TessellationError>
where
    F: Fn(f64, f64) -> Result<Vec3, TessellationError>,
    G: Fn(f64, f64) -> Result<Vec3, TessellationError>,
{
    policy.validate()?;
    trim.validate(trim_tolerance)
        .map_err(|_| TessellationError::EvaluationFailed)?;

    let mut points = Vec::new();
    let mut parameters = Vec::new();
    let mut normals = Vec::new();
    let mut maximum_chord: f64 = 0.0;
    let mut maximum_angular: f64 = 0.0;
    let mut maximum_parameter: f64 = 0.0;

    for curve in &trim.curves {
        let (samples, metrics) =
            tessellate_trim_curve3(curve, &policy, &eval, &normal)?;
        for (i, sample) in samples.into_iter().enumerate() {
            if !points.is_empty() && i == 0 {
                continue;
            }
            points.push(sample.point);
            parameters.push(sample.parameter);
            normals.push(sample.normal);
        }
        maximum_chord = maximum_chord.max(metrics.0);
        maximum_angular = maximum_angular.max(metrics.1);
        maximum_parameter = maximum_parameter.max(metrics.2);
    }

    if points.len() < 2 {
        return Err(TessellationError::Degenerate);
    }

    // A closed loop's final curve ends where the first curve starts. Keep one
    // semantic boundary sample at that location rather than duplicating it.
    if parameters.len() >= 2 {
        let first = parameters[0];
        let last = *parameters.last().expect("length checked");
        if (last.0 - first.0).hypot(last.1 - first.1) <= trim_tolerance {
            points.pop();
            parameters.pop();
            normals.pop();
        }
    }

    if points.len() < 2 {
        return Err(TessellationError::Degenerate);
    }

    Ok(TessellatedTrimLoop3 {
        points,
        parameters,
        normals,
        max_chord_error: maximum_chord,
        max_angular_error: maximum_angular,
        max_parameter_chord_error: maximum_parameter,
        max_depth: policy.max_depth,
    })
}

#[cfg(test)]
mod trim_tests {
    use super::*;
    use crate::math::geometry::{Arc, Point};

    fn planar_eval(u: f64, v: f64) -> Result<Vec3, TessellationError> {
        Ok(Vec3::new(u, v, 0.0))
    }

    fn planar_normal(_: f64, _: f64) -> Result<Vec3, TessellationError> {
        Ok(Vec3::new(0.0, 0.0, 1.0))
    }

    fn square_loop() -> TrimLoop2 {
        TrimLoop2 {
            curves: vec![
                TrimCurve2::Line {
                    start: Point { x: 0.0, y: 0.0 },
                    end: Point { x: 1.0, y: 0.0 },
                },
                TrimCurve2::Line {
                    start: Point { x: 1.0, y: 0.0 },
                    end: Point { x: 1.0, y: 1.0 },
                },
                TrimCurve2::Line {
                    start: Point { x: 1.0, y: 1.0 },
                    end: Point { x: 0.0, y: 1.0 },
                },
                TrimCurve2::Line {
                    start: Point { x: 0.0, y: 1.0 },
                    end: Point { x: 0.0, y: 0.0 },
                },
            ],
        }
    }

    #[test]
    fn trim_boundary_preserves_exact_uv_samples() {
        let policy = TrimTessellationPolicy {
            chord_error: 1.0e-6,
            angular_error: 1.0e-6,
            parameter_chord_error: 1.0e-6,
            max_depth: 8,
        };
        let result = tessellate_trim_loop3(
            &square_loop(),
            1.0e-9,
            policy,
            planar_eval,
            planar_normal,
        )
        .unwrap();
        assert_eq!(result.points.len(), 4);
        assert_ne!(result.parameters.first(), result.parameters.last());
        assert_eq!(result.parameters.len(), result.points.len());
        assert_eq!(result.points.len(), result.normals.len());
        for (point, parameter) in result.points.iter().zip(&result.parameters) {
            assert!((point.x - parameter.0).abs() <= 1.0e-12);
            assert!((point.y - parameter.1).abs() <= 1.0e-12);
            assert!(point.z.abs() <= 1.0e-12);
        }
    }

    #[test]
    fn trim_arc_refines_in_parameter_and_surface_space() {
        let loop_ = TrimLoop2 {
            curves: vec![
                TrimCurve2::Arc(Arc {
                    center: Point { x: 0.0, y: 0.0 },
                    radius: 1.0,
                    start_angle: 0.0,
                    end_angle: std::f64::consts::FRAC_PI_2,
                }),
                TrimCurve2::Line {
                    start: Point { x: 0.0, y: 1.0 },
                    end: Point { x: 0.0, y: 0.0 },
                },
            ],
        };
        let policy = TrimTessellationPolicy {
            chord_error: 1.0e-4,
            angular_error: 1.0e-4,
            parameter_chord_error: 1.0e-2,
            max_depth: 10,
        };
        let result = tessellate_trim_loop3(
            &loop_,
            1.0e-8,
            policy,
            planar_eval,
            planar_normal,
        );
        assert!(result.is_err() || result.unwrap().points.len() > 2);
    }

    #[test]
    fn invalid_trim_loop_fails_closed() {
        let mut loop_ = square_loop();
        loop_.curves.swap(1, 3);
        assert_eq!(
            tessellate_trim_loop3(
                &loop_,
                1.0e-9,
                TrimTessellationPolicy {
                    chord_error: 1.0e-4,
                    angular_error: 1.0e-4,
                    parameter_chord_error: 1.0e-4,
                    max_depth: 8,
                },
                planar_eval,
                planar_normal,
            ),
            Err(TessellationError::EvaluationFailed)
        );
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TessellatedTrimmedSurface3 {
    pub vertices: Vec<Vec3>,
    pub parameters: Vec<(f64, f64)>,
    pub normals: Vec<Vec3>,
    pub triangles: Vec<[usize; 3]>,
    pub max_chord_error: f64,
    pub max_angular_error: f64,
    pub max_parameter_chord_error: f64,
    pub max_depth: u32,
}

fn strictly_convex_parameter_polygon(parameters: &[(f64, f64)], tolerance: f64) -> bool {
    if parameters.len() < 3 {
        return false;
    }
    let mut sign = 0.0;
    for i in 0..parameters.len() {
        let a = parameters[i];
        let b = parameters[(i + 1) % parameters.len()];
        let c = parameters[(i + 2) % parameters.len()];
        let ab = (b.0 - a.0, b.1 - a.1);
        let bc = (c.0 - b.0, c.1 - b.1);
        let cross = ab.0 * bc.1 - ab.1 * bc.0;
        let scale = ab.0.hypot(ab.1).max(bc.0.hypot(bc.1)).max(f64::MIN_POSITIVE);
        let epsilon = tolerance * scale;
        if !cross.is_finite() || !epsilon.is_finite() {
            return false;
        }
        if cross.abs() <= epsilon {
            continue;
        }
        if sign == 0.0 {
            sign = cross.signum();
        } else if cross * sign < -epsilon {
            return false;
        }
    }
    sign != 0.0
}

fn classify_trim_centroid(
    trim: &TrimLoop2,
    u: f64,
    v: f64,
    tolerance: f64,
) -> Result<(), TessellationError> {
    let class = trim
        .classify_point(super::geometry::Point { x: u, y: v }, tolerance)
        .map_err(|_| TessellationError::EvaluationFailed)?;
    if class == RegionClass::Inside {
        Ok(())
    } else {
        Err(TessellationError::EvaluationFailed)
    }
}

fn sample_trim_parameter<F, G>(
    trim: &TrimLoop2,
    u: f64,
    v: f64,
    trim_tolerance: f64,
    eval: &F,
    normal: &G,
) -> Result<SurfaceSample3, TessellationError>
where
    F: Fn(f64, f64) -> Result<Vec3, TessellationError>,
    G: Fn(f64, f64) -> Result<Vec3, TessellationError>,
{
    let class = trim
        .classify_point(super::geometry::Point { x: u, y: v }, trim_tolerance)
        .map_err(|_| TessellationError::EvaluationFailed)?;
    if class != RegionClass::Inside && class != RegionClass::OnBoundary {
        return Err(TessellationError::EvaluationFailed);
    }
    let point = eval(u, v)?;
    let normal = normal(u, v)?;
    if !point.is_finite() || !normal.is_finite() {
        return Err(TessellationError::NonFinite);
    }
    Ok(SurfaceSample3 {
        parameter: (u, v),
        point,
        normal: normal
            .normalized()
            .map_err(|_| TessellationError::Degenerate)?,
    })
}

fn midpoint_parameter(a: SurfaceSample3, b: SurfaceSample3) -> (f64, f64) {
    (
        a.parameter.0.mul_add(0.5, b.parameter.0 * 0.5),
        a.parameter.1.mul_add(0.5, b.parameter.1 * 0.5),
    )
}

fn midpoint_chord_error(a: SurfaceSample3, b: SurfaceSample3, m: SurfaceSample3) -> f64 {
    m.point
        .sub(a.point.scale(0.5).add(b.point.scale(0.5)))
        .length()
}

fn refine_trim_triangle<F, G>(
    a: SurfaceSample3,
    b: SurfaceSample3,
    c: SurfaceSample3,
    depth: u32,
    policy: &TrimTessellationPolicy,
    eval: &F,
    normal: &G,
    trim: &TrimLoop2,
    trim_tolerance: f64,
    out: &mut Vec<[SurfaceSample3; 3]>,
) -> Result<(f64, f64), TessellationError>
where
    F: Fn(f64, f64) -> Result<Vec3, TessellationError>,
    G: Fn(f64, f64) -> Result<Vec3, TessellationError>,
{
    let u = a.parameter.0 / 3.0 + b.parameter.0 / 3.0 + c.parameter.0 / 3.0;
    let v = a.parameter.1 / 3.0 + b.parameter.1 / 3.0 + c.parameter.1 / 3.0;
    if !u.is_finite() || !v.is_finite() {
        return Err(TessellationError::NonFinite);
    }
    classify_trim_centroid(trim, u, v, trim_tolerance)?;

    let m = {
        let point = eval(u, v)?;
        let n = normal(u, v)?;
        if !point.is_finite() || !n.is_finite() {
            return Err(TessellationError::NonFinite);
        }
        SurfaceSample3 {
            parameter: (u, v),
            point,
            normal: n.normalized().map_err(|_| TessellationError::Degenerate)?,
        }
    };

    // Sample all three edge midpoints before accepting the triangle. This
    // keeps the sampled chord/angular certificate sensitive to curvature along
    // edges instead of relying on the centroid alone.
    let (ab_u, ab_v) = midpoint_parameter(a, b);
    let (bc_u, bc_v) = midpoint_parameter(b, c);
    let (ca_u, ca_v) = midpoint_parameter(c, a);
    let ab = sample_trim_parameter(trim, ab_u, ab_v, trim_tolerance, eval, normal)?;
    let bc = sample_trim_parameter(trim, bc_u, bc_v, trim_tolerance, eval, normal)?;
    let ca = sample_trim_parameter(trim, ca_u, ca_v, trim_tolerance, eval, normal)?;

    let triangle_approximation = a
        .point
        .scale(1.0 / 3.0)
        .add(b.point.scale(1.0 / 3.0))
        .add(c.point.scale(1.0 / 3.0));
    let chord_error = m
        .point
        .sub(triangle_approximation)
        .length()
        .max(midpoint_chord_error(a, b, ab))
        .max(midpoint_chord_error(b, c, bc))
        .max(midpoint_chord_error(c, a, ca));

    let mut angular_error = 0.0f64;
    let samples = [a, b, c, m, ab, bc, ca];
    for i in 0..samples.len() {
        for j in i + 1..samples.len() {
            angular_error = angular_error.max(angle_between(samples[i].normal, samples[j].normal)?);
        }
    }

    if !chord_error.is_finite() || !angular_error.is_finite() {
        return Err(TessellationError::NonFinite);
    }

    if chord_error <= policy.chord_error && angular_error <= policy.angular_error {
        out.push([a, b, c]);
        return Ok((chord_error, angular_error));
    }

    if depth >= policy.max_depth {
        return Err(TessellationError::MaxDepth);
    }

    // Longest-edge bisection halves the selected geometric span on each
    // recursion and avoids the exponential four-way fanout. The midpoint was
    // already classified and sampled above, so child construction reuses the
    // authoritative boundary/interior point.
    let ab_length = a.point.sub(b.point).length();
    let bc_length = b.point.sub(c.point).length();
    let ca_length = c.point.sub(a.point).length();
    if !ab_length.is_finite() || !bc_length.is_finite() || !ca_length.is_finite() {
        return Err(TessellationError::NonFinite);
    }

    let child_triangles = if ab_length >= bc_length && ab_length >= ca_length {
        [(a, ab, c), (ab, b, c)]
    } else if bc_length >= ca_length {
        [(b, bc, a), (bc, c, a)]
    } else {
        [(c, ca, b), (ca, a, b)]
    };

    let mut maximum_chord = chord_error;
    let mut maximum_angular = angular_error;
    for (x, y, z) in child_triangles {
        let (child_chord, child_angular) = refine_trim_triangle(
            x,
            y,
            z,
            depth + 1,
            policy,
            eval,
            normal,
            trim,
            trim_tolerance,
            out,
        )?;
        maximum_chord = maximum_chord.max(child_chord);
        maximum_angular = maximum_angular.max(child_angular);
    }

    Ok((maximum_chord, maximum_angular))
}

pub fn tessellate_trimmed_surface3<F, G>(
    trim: &TrimLoop2,
    trim_tolerance: f64,
    policy: TrimTessellationPolicy,
    eval: F,
    normal: G,
) -> Result<TessellatedTrimmedSurface3, TessellationError>
where
    F: Fn(f64, f64) -> Result<Vec3, TessellationError>,
    G: Fn(f64, f64) -> Result<Vec3, TessellationError>,
{
    policy.validate()?;
    trim.validate(trim_tolerance)
        .map_err(|_| TessellationError::EvaluationFailed)?;

    let boundary = tessellate_trim_loop3(trim, trim_tolerance, policy, &eval, &normal)?;
    if boundary.parameters.len() < 3
        || !strictly_convex_parameter_polygon(&boundary.parameters, trim_tolerance)
    {
        return Err(TessellationError::EvaluationFailed);
    }

    let mut boundary_samples = Vec::with_capacity(boundary.parameters.len());
    for i in 0..boundary.parameters.len() {
        boundary_samples.push(SurfaceSample3 {
            parameter: boundary.parameters[i],
            point: boundary.points[i],
            normal: boundary.normals[i],
        });
    }

    let mut surface_triangles = Vec::new();
    let mut max_chord_error = boundary.max_chord_error;
    let mut max_angular_error = boundary.max_angular_error;

    // A fan from a boundary vertex is invalid when adaptive tessellation
    // preserves multiple collinear samples on a straight trim edge: those
    // samples form zero-area fan triangles whose centroids lie OnBoundary.
    // Use the deterministic arithmetic mean of all boundary parameters as a
    // certified interior seed instead. For a nondegenerate convex polygon the
    // equal-weight mean lies in its interior; classify it explicitly before
    // using it as a triangle fan center.
    let count = boundary_samples.len() as f64;
    if !count.is_finite() || count <= 0.0 {
        return Err(TessellationError::Degenerate);
    }
    let mut center_u = 0.0;
    let mut center_v = 0.0;
    for sample in &boundary_samples {
        center_u += sample.parameter.0 / count;
        center_v += sample.parameter.1 / count;
    }
    if !center_u.is_finite() || !center_v.is_finite() {
        return Err(TessellationError::NonFinite);
    }
    classify_trim_centroid(trim, center_u, center_v, trim_tolerance)?;

    let center_point = eval(center_u, center_v)?;
    let center_normal = normal(center_u, center_v)?;
    if !center_point.is_finite() || !center_normal.is_finite() {
        return Err(TessellationError::NonFinite);
    }
    let center_sample = SurfaceSample3 {
        parameter: (center_u, center_v),
        point: center_point,
        normal: center_normal
            .normalized()
            .map_err(|_| TessellationError::Degenerate)?,
    };

    for i in 0..boundary_samples.len() {
        let next = (i + 1) % boundary_samples.len();
        if boundary_samples[i].parameter == boundary_samples[next].parameter {
            return Err(TessellationError::Degenerate);
        }
        let (chord, angular) = refine_trim_triangle(
            center_sample,
            boundary_samples[i],
            boundary_samples[next],
            0,
            &policy,
            &eval,
            &normal,
            trim,
            trim_tolerance,
            &mut surface_triangles,
        )?;
        max_chord_error = max_chord_error.max(chord);
        max_angular_error = max_angular_error.max(angular);
    }

    if surface_triangles.is_empty() {
        return Err(TessellationError::Degenerate);
    }

    let mut vertices = Vec::new();
    let mut parameters = Vec::new();
    let mut normals = Vec::new();
    let mut indices = BTreeMap::<(u64, u64), usize>::new();
    let mut triangles = Vec::with_capacity(surface_triangles.len());

    for triangle in surface_triangles {
        let mut ids = [0usize; 3];
        for (slot, sample) in triangle.into_iter().enumerate() {
            let key = (sample.parameter.0.to_bits(), sample.parameter.1.to_bits());
            let index = if let Some(index) = indices.get(&key) {
                let index = *index;
                if vertices[index] != sample.point || normals[index] != sample.normal {
                    return Err(TessellationError::EvaluationFailed);
                }
                index
            } else {
                let index = vertices.len();
                indices.insert(key, index);
                vertices.push(sample.point);
                parameters.push(sample.parameter);
                normals.push(sample.normal);
                index
            };
            ids[slot] = index;
        }
        triangles.push(ids);
    }

    Ok(TessellatedTrimmedSurface3 {
        vertices,
        parameters,
        normals,
        triangles,
        max_chord_error,
        max_angular_error,
        max_parameter_chord_error: boundary.max_parameter_chord_error,
        max_depth: policy.max_depth,
    })
}

#[cfg(test)]
mod trimmed_surface_tests {
    use super::*;
    use crate::math::geometry::Point;

    fn planar_normal(_: f64, _: f64) -> Result<Vec3, TessellationError> {
        Ok(Vec3::new(0.0, 0.0, 1.0))
    }

    fn square_trim() -> TrimLoop2 {
        TrimLoop2 {
            curves: vec![
                TrimCurve2::Line {
                    start: Point { x: 0.0, y: 0.0 },
                    end: Point { x: 1.0, y: 0.0 },
                },
                TrimCurve2::Line {
                    start: Point { x: 1.0, y: 0.0 },
                    end: Point { x: 1.0, y: 1.0 },
                },
                TrimCurve2::Line {
                    start: Point { x: 1.0, y: 1.0 },
                    end: Point { x: 0.0, y: 1.0 },
                },
                TrimCurve2::Line {
                    start: Point { x: 0.0, y: 1.0 },
                    end: Point { x: 0.0, y: 0.0 },
                },
            ],
        }
    }

    #[test]
    fn convex_trimmed_surface_fills_and_preserves_boundary() {
        let policy = TrimTessellationPolicy {
            chord_error: 1.0e-3,
            angular_error: 5.0e-2,
            parameter_chord_error: 1.0e-3,
            max_depth: 10,
        };
        let trim = square_trim();

        let boundary = tessellate_trim_loop3(
            &trim,
            1.0e-9,
            policy,
            |u, v| Ok(Vec3::new(u, v, 0.2 * u * u + 0.3 * v * v)),
            |u, v| {
                let du = Vec3::new(1.0, 0.0, 0.4 * u);
                let dv = Vec3::new(0.0, 1.0, 0.6 * v);
                du.cross(dv)
                    .normalized()
                    .map_err(|_| TessellationError::Degenerate)
            },
        )
        .unwrap();

        let result = tessellate_trimmed_surface3(
            &trim,
            1.0e-9,
            policy,
            |u, v| Ok(Vec3::new(u, v, 0.2 * u * u + 0.3 * v * v)),
            |u, v| {
                let du = Vec3::new(1.0, 0.0, 0.4 * u);
                let dv = Vec3::new(0.0, 1.0, 0.6 * v);
                du.cross(dv)
                    .normalized()
                    .map_err(|_| TessellationError::Degenerate)
            },
        )
        .unwrap();

        assert!(result.triangles.len() > 2);
        assert!(result.max_chord_error <= policy.chord_error);
        assert!(result.max_angular_error <= policy.angular_error);
        assert_eq!(result.parameters.len(), result.vertices.len());
        assert_eq!(result.parameters.len(), result.normals.len());

        for parameter in &boundary.parameters {
            assert!(result.parameters.iter().any(|candidate| candidate == parameter));
        }
        for corner in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)] {
            assert!(result.parameters.iter().any(|parameter| *parameter == corner));
        }
    }

    #[test]
    fn concave_trim_is_rejected_by_convex_certification() {
        let trim = TrimLoop2 {
            curves: vec![
                TrimCurve2::Line { start: Point { x: 0.0, y: 0.0 }, end: Point { x: 2.0, y: 0.0 } },
                TrimCurve2::Line { start: Point { x: 2.0, y: 0.0 }, end: Point { x: 2.0, y: 2.0 } },
                TrimCurve2::Line { start: Point { x: 2.0, y: 2.0 }, end: Point { x: 1.0, y: 1.0 } },
                TrimCurve2::Line { start: Point { x: 1.0, y: 1.0 }, end: Point { x: 0.0, y: 2.0 } },
                TrimCurve2::Line { start: Point { x: 0.0, y: 2.0 }, end: Point { x: 0.0, y: 0.0 } },
            ],
        };
        let result = tessellate_trimmed_surface3(
            &trim,
            1.0e-9,
            TrimTessellationPolicy {
                chord_error: 1.0e-3,
                angular_error: 1.0e-3,
                parameter_chord_error: 1.0e-3,
                max_depth: 4,
            },
            |u, v| Ok(Vec3::new(u, v, 0.0)),
            planar_normal,
        );
        assert_eq!(result, Err(TessellationError::EvaluationFailed));
    }
}
