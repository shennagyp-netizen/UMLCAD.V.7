# UMLCAD V5 Kernel Specification

## Authority

The V5 native kernel is the engineering authority. Clients, agents, routing services, renderers, Git integrations, and LLMs may propose intent or source changes, but they never decide geometric validity, constraint satisfaction, topology, or engineering acceptance.

## Kernel boundary

The kernel is a separate, transport-neutral engineering API.

The kernel core is **stateless with respect to project/session/live application state**. A kernel evaluation receives all authoritative inputs required for the requested operation and returns an immutable result. Any deployment cache is an implementation optimization only and is never semantic state or an authority.

The kernel does not own:

- browser sessions;
- WebSocket sessions;
- authentication or authorization;
- Git state;
- source-editor state;
- collaboration/live-session state;
- renderer state;
- UI state.

A kernel service may cache immutable builds for performance, but cache loss must not change the result.

## Semantic model

A project is evaluated into an immutable semantic snapshot. A snapshot contains parameters, parameter values, geometric primitives, constraints/relations, and deterministic dependency edges. Source, UI coordinates, SVG, rendered pixels, and cached artifacts are derived representations.

A mutating-looking operation is evaluated functionally:

```text
base immutable input + explicit operation
              ↓
        candidate evaluation
              ↓
 validation / analysis / solve
              ↓
       new immutable result
```

The prior result is never mutated.

Relations are first-class semantic equations. `appendRelations()` creates a new immutable relation-bearing snapshot with deterministic relation IDs. Relation references are preserved separately from the base `DependencyNode` graph; geometry-to-relation dependencies are explicit in the relation snapshot and are not represented by casting a relation into a base constraint.

## Geometry

The native V5 2D kernel currently defines line segments, circles, and circular arcs. Geometry has stable semantic IDs and finite-value/degeneracy validation. Geometry queries use exact analytic representations rather than sampled display paths.

## Constraints, relations, and solving

Native constraints and geometric relations are semantic equations with explicit domains. Domain-invalid operations fail closed.

Geometric relations include line orientation/collinearity, equal length/radius, angle, concentricity, radius/diameter, tangency, midpoint, point-on-curve, explicit point distances, and symmetry within the existing line/circle/arc 2D surface. Tangency between circular curves explicitly supports external, internal, or any tangency mode.

Constraint/relation analysis reports residuals, scaling, equation/variable counts, numerical rank, degrees of freedom, conditioning information, and satisfaction where applicable.

The hardened solver uses scaled damped QR least-squares, bounded iteration, explicit convergence criteria, and geometric validation of candidate states before acceptance. Relation residuals participate in the same equation system and are re-evaluated against every candidate state before that state is accepted. The solver does not use normal equations as its authoritative linear solve.

## Dependencies and builds

Part builds are content/revision identified and immutable. A changed part creates a new build identity. Assemblies consume immutable part builds and apply instance transforms; they do not mutate their source parts.

## Engineering projections

Topology is derived from semantic geometry and connectivity. Dimensions evaluate semantic measurements. Spatial analysis uses an AABB broad phase followed by exact 2D curve narrow-phase predicates. Semantic references remain ID-based and migration is explicit. DXF is deterministic and generated from the semantic snapshot.

## AI contract

An AI client should reason in semantic IDs and typed operations. It should request capabilities, inspect the authoritative immutable revision/build, propose an explicit operation or bounded source change, and wait for kernel acceptance. It must never infer validity from rendering, invent missing geometry, silently mutate accepted state, or treat cached/artifact output as authoritative.

## Determinism

Equivalent authoritative input must produce equivalent semantic ordering, build identity, diagnostics, and export text. Caches may change performance only.

## Failure model

Malformed requests are rejected before engineering evaluation. Invalid geometry, broken references, invalid constraint/relation domains, unsatisfied constraints, singular or ill-conditioned solves, invalid topology, and unsupported operations produce structured diagnostics. Rejection does not mutate the prior immutable build.

## Extension rule

V5 kernel work may recover V4 engineering semantics within the already-defined V5 2D surface. It must not expand V5 into V6-class geometry/modeling functionality.

New kernel features must add semantic authority rather than client-specific behavior. Public contracts must remain transport-neutral. HTTP, WebSocket, Git, browser APIs, LLM APIs, and rendering frameworks remain outside the kernel core.
