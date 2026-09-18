# CAD Core S1 — Specification → Evaluation → Authoritative Result

## Scope

This milestone introduces the typed CAD reference/result boundary needed by the next implementation stage. It separates design specification from evaluation, authoritative geometry/topology results, and derived representation.

## Contract flow

CadDocumentSpecification → SpecificationGraph → EvaluationPlanner → CadEvaluationEngine → ICadKernelEvaluator → AuthoritativeCadResult → CadRepresentation.

The engine contains no exact geometry implementation. It cannot manufacture a B-Rep result and it cannot infer topology from display data.

## Reference law

A reference contains:
- target specification identity;
- explicit reference kind;
- topology selector;
- frame/configuration context;
- optional expected result identity and occurrence path.

Resolution is explicit: Resolved, Missing, Ambiguous, Indeterminate, or Unsupported. The engine fails closed whenever a required reference is not uniquely resolved.

The vertical slice uses a planar-face selector defined by normal and point evidence. The production box adapter now supplies that evidence from the certified box topology contract, so resolution does not depend on face-array order.

## Vertical slice

The contract tests cover:
1. box/base specification;
2. semantic support-face reference;
3. sketch with two circular profile elements and constraints;
4. additive extrusion;
5. subtractive extrusion;
6. authoritative topology/result handoff;
7. representation identity derived from the authoritative result;
8. ambiguous-reference fail-closed behavior;
9. deterministic evaluation identities;
10. full/incremental recomputation equivalence;
11. cycle rejection before kernel evaluation.

## Deterministic evaluation

Evaluation identity is content-addressed and includes the normalized feature definition, semantic reference resolution evidence, upstream evaluation/result identities, kernel contract version, document revision/configuration context, and representation policy.

Incremental invalidation is the explicit transitive dependent closure of the change set. No mutable Dirty graph is authoritative.

## Kernel boundary

The production Rust kernel already contains authoritative B-Rep/topology mathematics. The .NET contract deliberately does not reach into those implementation types. The concrete .NET Rust adapter now exists for the first certified geometry subset (axis-aligned box). Semantic reference resolution is owned by the CAD Engine/Core and occurs before kernel evaluation. It translates the typed kernel-evaluation contract through the existing geometry transport without routing CAD semantics through the legacy V4 build model.

## Downstream

Drawing, CAM, Sheet Metal, BOM, and other engineering domains consume AuthoritativeCadResult/representation contracts. They do not redefine geometric truth.
