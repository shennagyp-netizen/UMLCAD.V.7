use umlcad_v6_freeform_api::{NurbsCurve3DDefinition, NurbsDefinitionError, Point3};

fn quarter_circle() -> NurbsCurve3DDefinition {
    NurbsCurve3DDefinition::new(
        2,
        vec![
            Point3 { x: 1.0, y: 0.0, z: 0.0 },
            Point3 { x: 1.0, y: 1.0, z: 0.0 },
            Point3 { x: 0.0, y: 1.0, z: 0.0 },
        ],
        vec![1.0, std::f64::consts::FRAC_1_SQRT_2, 1.0],
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
    )
}

#[test]
fn repeated_freeform_validation_is_deterministic() {
    let curve = quarter_circle();
    for _ in 0..10_000 {
        assert_eq!(curve.validate(), Ok(()));
        assert_eq!(curve.parameter_domain(), Ok((0.0, 1.0)));
    }
}

#[test]
fn repeated_clone_validation_cycles_preserve_source() {
    let source = quarter_circle();
    let before = source.clone();
    let mut current = source.clone();

    for _ in 0..5_000 {
        let clone = current.clone();
        assert_eq!(clone, source);
        assert_eq!(clone.validate(), Ok(()));
        assert_eq!(source, before);
        current = clone;
    }

    assert_eq!(source, before);
}

#[test]
fn repeated_invalid_freeform_cases_are_stable() {
    let mut cases = Vec::new();

    let mut non_finite = quarter_circle();
    non_finite.control_points[1].x = f64::NAN;
    cases.push(non_finite);

    let mut invalid_weight = quarter_circle();
    invalid_weight.weights[1] = 0.0;
    cases.push(invalid_weight);

    let mut invalid_knot = quarter_circle();
    invalid_knot.knots[3] = -0.1;
    cases.push(invalid_knot);

    for case in cases {
        let expected = case.validate();
        assert!(matches!(
            expected,
            Err(NurbsDefinitionError::NonFinite)
                | Err(NurbsDefinitionError::InvalidWeight)
                | Err(NurbsDefinitionError::KnotsMustBeNondecreasing)
        ));
        for _ in 0..5_000 {
            assert_eq!(case.validate(), expected);
        }
    }
}

#[test]
fn repeated_parameter_domain_queries_do_not_mutate_definition() {
    let curve = quarter_circle();
    let before = curve.clone();
    for _ in 0..10_000 {
        assert_eq!(curve.parameter_domain(), Ok((0.0, 1.0)));
        assert_eq!(curve, before);
    }
}
