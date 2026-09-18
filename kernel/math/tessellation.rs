//! Adaptive curve tessellation mathematics.
//!
//! Tessellation is explicitly an approximation. The algorithm refines parameter
//! intervals until caller-provided chord and angular errors are satisfied, or
//! returns `MaxDepth` rather than silently returning a lower-quality result.

use super::vec::Vec3;

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
