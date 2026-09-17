# UMLCAD V6 — S12 Continuation Handoff

## Station status

**S12 is COMPLETE and merged to `main`.**

PR #44 (`v6-s12-curve-surface-operations`) was merged by squash after the final authoritative CI gate passed. Merge commit: `4277cf40e1923484b2faa87638b348ff8b94389a`.

The final authoritative CI run for the pre-merge head was fully green across workspace debug/release tests, format, Clippy, kernel format/tests, and both kernel Clippy gates.

## Completed S12 semantic capabilities

- exact affine-planar line/NURBS-surface intersection with explicit transverse, disjoint, and coplanar/underdetermined outcomes;
- deterministic multi-seed NURBS curve/surface isolated-root solver with explicit ambiguity handling;
- conservative isolated tangent detection and coincident/underdetermined handling for supported curve/surface cases;
- bounded affine-planar surface/surface intersection with exact plane-plane construction and UV clipping;
- bounded planar point/surface closest-point and distance oracle;
- intersection-driven deterministic line splitting that fails closed on ambiguity;
- positive-weight NURBS control-net convex-hull broad phase for certified surface-pair disjointness;
- general surface-pair dispatcher that returns deterministic `NoIntersection` when disjointness is certified and otherwise delegates only to a proven solver family;
- opaque OCCT realizations with semantic-first conformance against independently computed expectations.

## S12 scope boundary

S12 does not claim an exhaustive arbitrary NURBS/NURBS intersection-curve solver. General potential contact outside the proven solver families remains explicitly unsupported. General freeform intersection tracing and generalized trimming remain later kernel work; no renderer approximation or ambiguous numerical trace is promoted into topology.

## Verified completion gate

1. Representative curve/surface and surface/surface operations have independent semantic definitions.
2. Invalid, degenerate, tangent, coincident, ambiguous, and unsupported cases have explicit outcomes in the supported families.
3. Tests cover non-unit parameter domains and deterministic result ordering.
4. OCCT realization is validated against the semantic authority rather than defining it.
5. Intersection-driven operations do not promote unverified numerical traces into topology.
6. The authoritative full CI matrix passed.
7. PR #44 is closed and merged to `main`.

## Post-S12 continuation

The next station starts from the merged `main` branch:

- **S13:** offsets and healing with explicit repair authority and failure semantics;
- **S14:** freeform feature generation extensions;
- **S15:** robust B-Rep topology completion;
- **S16:** deterministic kernel integration and provenance/reference stability;
- **S17:** performance and stress validation;
- **S18:** V6 part-geometry-kernel completion gate.

Preserve the invariant:

```text
semantic mathematics -> backend-neutral contract -> opaque native realization -> independent conformance -> topology
```

No later station may turn an unsupported or ambiguous S12 result into silently accepted geometry.