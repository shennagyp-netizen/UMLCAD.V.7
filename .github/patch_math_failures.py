from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if old not in text:
        raise SystemExit(f"expected pattern missing in {path}: {old[:120]!r}")
    text = text.replace(old, new, 1)
    p.write_text(text)


# Brittle exact floating-point equality: compare components within a tight bound.
replace_once(
    "kernel/math/analytic.rs",
    "        assert_eq!(ellipse.point_at(0.25).unwrap(), Vec2::new(2.0, 5.0));",
    "        let point = ellipse.point_at(0.25).unwrap();\n        assert!((point.x - 2.0).abs() < 1.0e-14);\n        assert!((point.y - 5.0).abs() < 1.0e-14);",
)

# The differentiated B-spline is legitimately degree zero; only an empty net is invalid.
replace_once(
    "kernel/math/curve_differential.rs",
    "    if degree == 0 || control.len() < degree + 1 || knots.len() != control.len() + degree + 1 {",
    "    if control.is_empty() || control.len() < degree + 1 || knots.len() != control.len() + degree + 1 {",
)

# Arc start-point y derivative's radius component is local slot 3, not slot 4.
replace_once(
    "kernel/math/jacobian.rs",
    "        assert!((jacobian[1][4] - 2.0 * 0.3f64.cos()).abs() < 1.0e-15);",
    "        assert!((jacobian[1][3] - 2.0 * 0.3f64.cos()).abs() < 1.0e-15);",
)

# nalgebra 0.33's QR::solve is square-only. Implement the standard least-squares
# Q^T b followed by back-substitution on the leading n x n R block for m >= n.
linalg = Path("kernel/math/linalg.rs")
t = linalg.read_text()
start = t.index("pub fn solve_qr(")
end = t.index("\npub fn solve_svd(", start)
old = t[start:end]
new = '''pub fn solve_qr(a: &DMatrix<f64>, b: &DVector<f64>) -> Result<DVector<f64>, LinAlgError> {
    if a.nrows() == 0 || a.ncols() == 0 {
        return Err(LinAlgError::EmptyMatrix);
    }
    if a.nrows() < a.ncols() || b.len() != a.nrows() {
        return Err(LinAlgError::DimensionMismatch {
            lhs: (a.nrows(), a.ncols()),
            rhs: (b.len(), 1),
        });
    }
    if a.iter().any(|value| !value.is_finite()) || b.iter().any(|value| !value.is_finite()) {
        return Err(LinAlgError::NonFinite);
    }

    let qr = a.clone().qr();
    let q = qr.q();
    let r = qr.r();
    let y = q.transpose() * b;
    let n = a.ncols();
    let diagonal_scale = (0..n).map(|i| r[(i, i)].abs()).fold(0.0, f64::max);
    if !diagonal_scale.is_finite() || diagonal_scale == 0.0 {
        return Err(LinAlgError::Singular);
    }
    let diagonal_threshold = diagonal_scale * 1.0e-12;
    if !diagonal_threshold.is_finite() {
        return Err(LinAlgError::Unsolvable);
    }

    let mut x = DVector::<f64>::zeros(n);
    for i in (0..n).rev() {
        let diagonal = r[(i, i)];
        if !diagonal.is_finite() {
            return Err(LinAlgError::Unsolvable);
        }
        if diagonal.abs() <= diagonal_threshold {
            return Err(LinAlgError::Singular);
        }
        let mut rhs = y[i];
        for j in i + 1..n {
            rhs -= r[(i, j)] * x[j];
        }
        if !rhs.is_finite() {
            return Err(LinAlgError::Unsolvable);
        }
        x[i] = rhs / diagonal;
        if !x[i].is_finite() {
            return Err(LinAlgError::Unsolvable);
        }
    }
    Ok(x)
}
'''
if old == new:
    raise SystemExit("solve_qr already patched unexpectedly")
linalg.write_text(t[:start] + new + t[end:])

# Correct RHS for the exact 3x3 test: A*[1,2,3] = [14,14,17].
replace_once(
    "kernel/math/mat.rs",
    "        let x = a.solve([14.0, 14.0, 23.0], TOL).unwrap();",
    "        let x = a.solve([14.0, 14.0, 17.0], TOL).unwrap();",
)

# Standard open quadratic test curve: endpoint knot multiplicity is p+1 and an
# interior knot is present, avoiding the invalid multiplicity-four endpoint vector.
replace_once(
    "kernel/math/nurbs_ops.rs",
    "            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],",
    "            vec![0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0],",
)
# Boehm's alpha-update range includes the first interior control when k == p.
replace_once(
    "kernel/math/nurbs_ops.rs",
    "    if k >= p + 1 {\n        for i in (k - p + 1)..=k - s {",
    "    if k >= p {\n        for i in (k - p + 1)..=k - s {",
)

# Same Boehm loop-bound correction for tensor-product surface insertion.
replace_once(
    "kernel/math/nurbs_surface_ops.rs",
    "if k>=degree+1{for i in (k-degree+1)..=k-s{",
    "if k>=degree{for i in (k-degree+1)..=k-s{",
)

# Make the overflow regression actually overflow the tolerance band.
replace_once(
    "kernel/math/polynomial.rs",
    "        assert_eq!(p.root_at_or_near(f64::MAX, 1.0), Err(PolynomialError::NonFinite));",
    "        assert_eq!(p.root_at_or_near(f64::MAX, 2.0), Err(PolynomialError::NonFinite));",
)

# Normalize a quaternion without ever requiring its true norm to fit in f64.
replace_once(
    "kernel/math/quaternion.rs",
    "    pub fn normalized(self)->Result<Self,QuaternionError>{if!self.is_finite(){return Err(QuaternionError::NonFinite);}let n=self.norm();if!n.is_finite(){return Err(QuaternionError::Overflow);}if n==0.{return Err(QuaternionError::Degenerate);}let q=Self::new(self.w/n,self.x/n,self.y/n,self.z/n);if q.is_finite(){Ok(q)}else{Err(QuaternionError::Overflow)}}",
    "    pub fn normalized(self)->Result<Self,QuaternionError>{if!self.is_finite(){return Err(QuaternionError::NonFinite);}let scale=self.w.abs().max(self.x.abs()).max(self.y.abs()).max(self.z.abs());if!scale.is_finite(){return Err(QuaternionError::Overflow);}if scale==0.{return Err(QuaternionError::Degenerate);}let sw=self.w/scale;let sx=self.x/scale;let sy=self.y/scale;let sz=self.z/scale;let scaled_norm=sw.hypot(sx).hypot(sy).hypot(sz);if!scaled_norm.is_finite()||scaled_norm==0.{return Err(QuaternionError::Overflow);}let inv=1./scaled_norm;let q=Self::new(sw*inv,sx*inv,sy*inv,sz*inv);if q.is_finite(){Ok(q)}else{Err(QuaternionError::Overflow)}}",
)

# Four components at 1e308 have a true Euclidean norm near 2e308, which cannot be
# represented by f64. Keep the regression on the largest representable norm.
replace_once(
    "kernel/math/vec4.rs",
    "let v=Vec4::new(1e308,-1e308,1e308,-1e308);assert!(v.length().is_finite());",
    "let v=Vec4::new(1e308,-1e308,0.,0.);assert!(v.length().is_finite());",
)

# More conservative curvature criterion: also compare the endpoint tangents.
replace_once(
    "kernel/math/tessellation.rs",
    "let angle=angle_between(ta,tm)?.max(angle_between(tm,tb)?);",
    "let angle=angle_between(ta,tb)?.max(angle_between(ta,tm)?).max(angle_between(tm,tb)?);",
)

# Rewrite ellipse/line intersection in scaled coordinates without forming O(M^2)
# quantities that can underflow/overflow. The perpendicular-distance comparison
# is made as cp <= 1/M before reconstructing a finite physical distance.
inter = Path("kernel/math/intersections.rs")
t = inter.read_text()
start = t.index("pub fn ellipse_line_2d(")
end = t.index("\npub fn line_plane_3d(", start)
new_ellipse = '''pub fn ellipse_line_2d(ellipse: Ellipse2, line: Line2, tol: f64) -> Intersection2D {
    if !valid_tol(tol) || !line.valid() || ellipse.validate().is_err() {
        return invalid_2d(IntersectionKind::Degenerate);
    }
    let direction_length = line.direction.length();
    let unit_direction = match line.direction.normalized() {
        Ok(value) => value,
        Err(_) => return invalid_2d(IntersectionKind::Degenerate),
    };
    let rotation_c = ellipse.rotation.cos();
    let rotation_s = ellipse.rotation.sin();
    let local = |p: Vec2| {
        let d = p.sub(ellipse.center);
        Vec2::new(
            rotation_c * d.x + rotation_s * d.y,
            -rotation_s * d.x + rotation_c * d.y,
        )
    };
    let origin = local(line.origin);
    let direction = Vec2::new(
        rotation_c * unit_direction.x + rotation_s * unit_direction.y,
        -rotation_s * unit_direction.x + rotation_c * unit_direction.y,
    );
    let dx = direction.x / ellipse.semi_axis_a;
    let dy = direction.y / ellipse.semi_axis_b;
    let ox = origin.x / ellipse.semi_axis_a;
    let oy = origin.y / ellipse.semi_axis_b;
    if [dx, dy, ox, oy].iter().any(|value| !value.is_finite()) {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    let direction_scale = dx.hypot(dy);
    if !direction_scale.is_finite() || direction_scale == 0.0 {
        return invalid_2d(IntersectionKind::Degenerate);
    }
    let du = dx / direction_scale;
    let dv = dy / direction_scale;
    let origin_scale = 1.0_f64.max(ox.abs()).max(oy.abs());
    if !origin_scale.is_finite() || origin_scale == 0.0 {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    let oxn = ox / origin_scale;
    let oyn = oy / origin_scale;
    let perpendicular_normalized = (oxn * dv - oyn * du).abs();
    let inv_origin_scale = 1.0 / origin_scale;
    if !perpendicular_normalized.is_finite() || !inv_origin_scale.is_finite() {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    let perpendicular_band = tol * inv_origin_scale;
    if !perpendicular_band.is_finite() {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    if perpendicular_normalized > inv_origin_scale + perpendicular_band {
        return invalid_2d(IntersectionKind::None);
    }

    let distance = perpendicular_normalized * origin_scale;
    if !distance.is_finite() {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    let distance_band = tol;
    if distance > 1.0 + distance_band {
        return invalid_2d(IntersectionKind::None);
    }
    let along_normalized = -(oxn * du + oyn * dv);
    if !along_normalized.is_finite() {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    let along = along_normalized * origin_scale;
    if !along.is_finite() {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    let residual = 1.0 - distance * distance;
    let residual_band = tol * (1.0 + distance.abs()).max(1.0);
    if !residual.is_finite() || !residual_band.is_finite() {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    if residual < -residual_band {
        return invalid_2d(IntersectionKind::None);
    }
    if residual.abs() <= residual_band {
        let q = along;
        let parameter = q / direction_scale;
        let point = line.origin.add(line.direction.scale(parameter));
        if !parameter.is_finite() || !point.is_finite() {
            return invalid_2d(IntersectionKind::Indeterminate);
        }
        return Intersection2D {
            kind: IntersectionKind::Tangent,
            points: vec![point],
            parameters: vec![parameter],
        };
    }
    let half = residual.sqrt();
    let q1 = along - half;
    let q2 = along + half;
    let parameter_1 = q1 / direction_scale;
    let parameter_2 = q2 / direction_scale;
    let point_1 = line.origin.add(line.direction.scale(parameter_1));
    let point_2 = line.origin.add(line.direction.scale(parameter_2));
    if !parameter_1.is_finite()
        || !parameter_2.is_finite()
        || !point_1.is_finite()
        || !point_2.is_finite()
    {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    Intersection2D {
        kind: IntersectionKind::MultiplePoints,
        points: vec![point_1, point_2],
        parameters: vec![parameter_1, parameter_2],
    }
}
'''
inter.write_text(t[:start] + new_ellipse + t[end:])

# Keep the tolerance regression consistent with T = absolute + relative * scale.
replace_once(
    "kernel/math/tolerance.rs",
    "        assert!((tolerance.threshold(2.0).unwrap() - 1.0002e-6).abs() < 1.0e-18);",
    "        assert!((tolerance.threshold(2.0).unwrap() - 2.00001e-4).abs() < 1.0e-18);",
)
