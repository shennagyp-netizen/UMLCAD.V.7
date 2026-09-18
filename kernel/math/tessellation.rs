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

fn strictly_convex_parameter_polygon(
    parameters: &[(f64, f64)],
    tolerance: f64,
) -> bool {
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
            // Repeated subdivision along a straight trim edge legitimately
            // produces collinear consecutive samples. They do not invalidate
            // convexity unless a nonzero turn reverses orientation.
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
        .classify_point(
            super::geometry::Point { x: u, y: v },
            tolerance,
        )
        .map_err(|_| TessellationError::EvaluationFailed)?;
    if class == RegionClass::Inside {
        Ok(())
    } else {
        Err(TessellationError::EvaluationFailed)
    }
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
    let u = (a.parameter.0 / 3.0)
        + (b.parameter.0 / 3.0)
        + (c.parameter.0 / 3.0);
    let v = (a.parameter.1 / 3.0)
        + (b.parameter.1 / 3.0)
        + (c.parameter.1 / 3.0);
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

    let approximation = a.point.scale(1.0 / 3.0)
        .add(b.point.scale(1.0 / 3.0))
        .add(c.point.scale(1.0 / 3.0));
    let chord_error = m.point.sub(approximation).length();
    let angular_error = angle_between(a.normal, b.normal)?
        .max(angle_between(a.normal, c.normal)?)
        .max(angle_between(b.normal, c.normal)?)
        .max(angle_between(a.normal, m.normal)?)
        .max(angle_between(b.normal, m.normal)?)
        .max(angle_between(c.normal, m.normal)?);
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

    let (ab_chord, ab_angle) = refine_trim_triangle(
        a, b, m, depth + 1, policy, eval, normal, trim, trim_tolerance, out
    )?;
    let (bc_chord, bc_angle) = refine_trim_triangle(
        b, c, m, depth + 1, policy, eval, normal, trim, trim_tolerance, out
    )?;
    let (ca_chord, ca_angle) = refine_trim_triangle(
        c, a, m, depth + 1, policy, eval, normal, trim, trim_tolerance, out
    )?;

    Ok((
        chord_error.max(ab_chord).max(bc_chord).max(ca_chord),
        angular_error.max(ab_angle).max(bc_angle).max(ca_angle),
    ))
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

    let boundary = tessellate_trim_loop3(
        trim,
        trim_tolerance,
        policy,
        &eval,
        &normal,
    )?;
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

    let mut triangles = Vec::new();
    let mut max_chord_error = boundary.max_chord_error;
    let mut max_angular_error = boundary.max_angular_error;
    for i in 1..boundary_samples.len() - 1 {
        let a = boundary_samples[0];
        let b = boundary_samples[i];
        let c = boundary_samples[i + 1];
        let (chord, angular) = refine_trim_triangle(
            a,
            b,
            c,
            0,
            &policy,
            &eval,
            &normal,
            trim,
            trim_tolerance,
            &mut triangles,
        )?;
        max_chord_error = max_chord_error.max(chord);
        max_angular_error = max_angular_error.max(angular);
    }

    if triangles.is_empty() {
        return Err(TessellationError::Degenerate);
    }

    let mut vertices = Vec::new();
    let mut parameters = Vec::new();
    let mut normals = Vec::new();
    let mut indices = BTreeMap::<(u64, u64), usize>::new();
    let mut mesh_triangles = Vec::with_capacity(triangles.len());

    for triangle in triangles {
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
        mesh_triangles.push(ids);
    }

    Ok(TessellatedTrimmedSurface3 {
        vertices,
        parameters,
        normals,
        triangles: mesh_triangles,
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
            angular_error: 1.0e-3,
            parameter_chord_error: 1.0e-3,
            max_depth: 10,
        };
        let result = tessellate_trimmed_surface3(
            &square_trim(),
            1.0e-9,
            policy,
            |u, v| Ok(Vec3::new(u, v, 0.2 * u * u + 0.3 * v * v)),
            |u, v| {
                let du = Vec3::new(1.0, 0.0, 0.4 * u);
                let dv = Vec3::new(0.0, 1.0, 0.6 * v);
                du.cross(dv).normalized().map_err(|_| TessellationError::Degenerate)
            },
        )
        .unwrap();
        assert!(result.triangles.len() > 2);
        assert!(result.max_chord_error <= policy.chord_error);
        assert!(result.max_angular_error <= policy.angular_error);
        assert!(result.parameters.iter().any(|p| *p == (0.0, 0.0)));
        assert!(result.parameters.iter().any(|p| *p == (1.0, 0.0)));
        assert!(result.parameters.iter().any(|p| *p == (1.0, 1.0)));
        assert!(result.parameters.iter().any(|p| *p == (0.0, 1.0)));
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