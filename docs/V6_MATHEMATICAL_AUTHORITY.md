# UMLCAD V6 — Mathematical Authority

## Purpose

This document records the mathematical authority of the V6 kernel so that backend implementations cannot silently become the definition of UMLCAD geometry.

## Authority split

UMLCAD owns the semantic mathematical model:

- point/vector and transform semantics;
- B-spline and NURBS parameterization;
- de Boor evaluation in homogeneous coordinates;
- rational dehomogenization;
- exact analytic derivatives implemented by differentiated control nets and the quotient rule;
- parameter domains and validation contracts;
- control-hull bounds used as backend-independent conservative bounds;
- immutable transformation semantics;
- mathematical regression oracles and adversarial fixtures.

OCCT is a reference realization and conformance oracle. It does not define the UMLCAD semantic equations.

## Independent verification rule

For every mathematical operation that has an analytic UMLCAD implementation, at least one contract test must have an expected result derived independently from the implementation under test.

A backend comparison alone is not a mathematical proof because both implementations could share the same defect or convention.

The preferred oracle hierarchy is:

1. closed-form analytic solution;
2. independently derived polynomial/rational identity;
3. geometric invariant;
4. independent backend conformance;
5. numerical approximation only where no exact practical oracle exists.

Numerical finite differences must not replace an available analytic derivative.

## NURBS representation

The semantic NURBS curve is represented by control points, positive weights, degree, and a validated nondecreasing full knot vector. Evaluation is performed in projective homogeneous coordinates, followed by division by the accumulated weight.

For a homogeneous control point `H_i = (w_i P_i, w_i)`, the curve is evaluated with the same B-spline basis as the ordinary curve. The Euclidean point is recovered only after homogeneous evaluation.

For first derivatives, the differentiated homogeneous control polygon is evaluated analytically and the Euclidean derivative is obtained with the quotient rule. This avoids finite-difference step-size dependence.

The tensor-product surface implementation follows the same rule independently in U and V directions.

## Backend independence

An OCCT adapter may reject inputs because of backend-specific realizability limits, native tolerance requirements, or API restrictions. Such limits are backend facts, not changes to UMLCAD's mathematical definition.

A backend failure must therefore remain distinguishable from:

- invalid UMLCAD mathematical input;
- a mathematically degenerate result;
- an unsupported UMLCAD operation;
- an OCCT construction failure.

## Review rule

When a backend and the independent semantic evaluator disagree:

```text
Do not widen tolerance first.
Do not copy the backend result into the semantic layer.
Reproduce → classify → derive an independent oracle → fix the faulty side → add regression coverage.
```
