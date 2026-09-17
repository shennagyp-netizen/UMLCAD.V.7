# V5 Native Kernel — Completion Status

## Scope

This status applies only to the V5 native 2D kernel surface: lines, circles, circular arcs, semantic constraints/relations, topology, spatial analysis, dimensions, references, deterministic DXF, and engineering acceptance. V6 remains the generation for 3D/freeform/full mechanical CAD kernel breadth.

## Implemented scientific gates

1. Circle and arc endpoint semantics are explicit; circles are closed edges with no fabricated endpoints.
2. Circle circumference and arc length are evaluated analytically.
3. Spatial analysis uses AABB broad phase followed by exact analytic 2D narrow phase.
4. Topology explicitly detects non-manifold/ambiguous incidence instead of selecting arbitrary continuations.
5. Invalid constraint/relation domains fail closed with diagnostics.
6. Solver uses scaled damped QR least squares rather than normal equations.
7. Candidate geometry is validated before solver acceptance; the original snapshot remains immutable.
8. Solver analysis reports rank, degrees of freedom, and conditioning evidence using the scaled Jacobian rank diagnostic.
9. Semantic references have explicit stale-reference and migration diagnostics.
10. `solveAndValidateEngineering()` makes engineering acceptance an explicit transaction: solve → materialize → validate → accept/reject.
11. Scientific regression coverage includes exact geometry, degeneracy, near-tangency, topology ambiguity, relation solving, immutability, contradiction/redundancy diagnostics, tangent modes, relation dependencies, acceptance behavior, scale invariance, tolerance boundaries, and deterministic export.
12. AI evidence explicitly separates structural, constraint, relation, conditioning, reference, topology, spatial, engineering-rule, and export validity.

## Test coverage

The native CI gate executes every `test/native-*.test.ts` file through `npm run test:native`, rather than a single kernel smoke suite. The hardening set adds dedicated relation-completeness, kernel-invariant/spatial, solver, and end-to-end engineering integration suites.

## Relation architecture

Relations are represented by a structurally separate immutable `RelationSnapshot` containing the authoritative base snapshot, typed relations, and explicit geometry-to-relation dependencies. `solveRelations()` passes the base snapshot plus typed relations into the same scaled QR equation system. No relation is cast into the legacy `Constraint` union.

## Architecture consistency

`ARCHITECTURE.md` distinguishes the stateless mathematical kernel from the stateful host/session layer. Host-side live revisions, process history, and caches are outside the mathematical kernel authority.

## Verification

The expanded native gate passed on the exact hardening head in GitHub Actions run #148 (`native-kernel`, `npm run check:native`). That verified tree was promoted to `main`. The promoted `main` commit then passed the same full native gate in GitHub Actions run #85 (`native-kernel`). The final `main` branch therefore has direct CI verification of the complete native test gate.

The hardening run exposed and corrected a genuine rectangular-Jacobian rank/conditioning diagnostic defect. The rank diagnostic now operates on the transpose of the scaled Jacobian; satisfied underdetermined systems are not rejected solely because they contain unconstrained degrees of freedom, while rank/DOF/conditioning evidence remains explicit.

## Non-goals

This completion does not introduce V6-class geometry/modeling such as NURBS, full 3D B-Rep, booleans, shells, fillets, drafts, sweeps, lofts, or full mechanical assembly solving.
