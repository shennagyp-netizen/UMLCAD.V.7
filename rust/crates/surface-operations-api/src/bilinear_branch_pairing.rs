use crate::SurfaceSurfaceIntersectionError;

pub(crate) type BilinearRoot = (f64, f64);
pub(crate) type BilinearRootPair = (BilinearRoot, BilinearRoot);

pub(crate) fn pair_bilinear_roots(
    roots: &[BilinearRoot],
    coefficients: [f64; 4],
    tolerance: f64,
) -> Result<Vec<BilinearRootPair>, SurfaceSurfaceIntersectionError> {
    match roots.len() {
        0 => Ok(Vec::new()),
        2 => Ok(vec![(roots[0], roots[1])]),
        4 => {
            let [a, b, c, d] = coefficients;
            let coefficient_scale = [a, b, c, d].into_iter().map(f64::abs).fold(1.0, f64::max);
            let coefficient_tol = tolerance.max(1e-12 * coefficient_scale);
            if d.abs() <= coefficient_tol {
                return Err(SurfaceSurfaceIntersectionError::CoincidentOrUnderdetermined);
            }
            let asymptote_s = -c / d;
            if !asymptote_s.is_finite() || asymptote_s <= 0.0 || asymptote_s >= 1.0 {
                return Err(SurfaceSurfaceIntersectionError::CoincidentOrUnderdetermined);
            }
            let numerator_at_asymptote = a + b * asymptote_s;
            if numerator_at_asymptote.abs() <= coefficient_tol {
                return Err(SurfaceSurfaceIntersectionError::CoincidentOrUnderdetermined);
            }
            let side_tol = tolerance.max(1e-12);
            let mut left = Vec::new();
            let mut right = Vec::new();
            for root in roots.iter().copied() {
                if root.0 < asymptote_s - side_tol {
                    left.push(root);
                } else if root.0 > asymptote_s + side_tol {
                    right.push(root);
                } else {
                    return Err(SurfaceSurfaceIntersectionError::CoincidentOrUnderdetermined);
                }
            }
            if left.len() != 2 || right.len() != 2 {
                return Err(SurfaceSurfaceIntersectionError::CoincidentOrUnderdetermined);
            }
            left.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.total_cmp(&y.1)));
            right.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.total_cmp(&y.1)));
            Ok(vec![(left[0], left[1]), (right[0], right[1])])
        }
        _ => Err(SurfaceSurfaceIntersectionError::CoincidentOrUnderdetermined),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOTS: [BilinearRoot; 4] = [
        (0.0, 0.05),
        (0.0125, 0.0),
        (0.95, 1.0),
        (1.0, 0.9875),
    ];
    const COEFFICIENTS: [f64; 4] = [0.01, -0.8, -0.2, 1.0];

    #[test]
    fn four_roots_are_paired_by_the_bilinear_branch_asymptote() {
        let pairs = pair_bilinear_roots(&ROOTS, COEFFICIENTS, 1e-12).unwrap();
        assert_eq!(pairs.len(), 2);
        assert!(pairs[0].0.0 < 0.2 && pairs[0].1.0 < 0.2);
        assert!(pairs[1].0.0 > 0.2 && pairs[1].1.0 > 0.2);
    }

    #[test]
    fn four_root_pairing_is_deterministic() {
        let expected = pair_bilinear_roots(&ROOTS, COEFFICIENTS, 1e-12).unwrap();
        for _ in 0..1000 {
            assert_eq!(
                expected,
                pair_bilinear_roots(&ROOTS, COEFFICIENTS, 1e-12).unwrap()
            );
        }
    }

    #[test]
    fn factorized_asymptote_is_rejected_as_underdetermined() {
        let roots = [(0.5, 0.0), (0.5, 1.0), (0.0, 0.0), (1.0, 0.0)];
        assert_eq!(
            pair_bilinear_roots(&roots, [0.0, 0.5, 0.0, -1.0], 1e-12),
            Err(SurfaceSurfaceIntersectionError::CoincidentOrUnderdetermined)
        );
    }
}
