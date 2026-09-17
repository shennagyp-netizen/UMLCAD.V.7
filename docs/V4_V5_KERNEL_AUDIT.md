# UMLCAD V4 vs V5 Kernel — Engineering and Scientific Audit

## Executive conclusion

V5 is architecturally cleaner, more deterministic, more explicitly immutable, and better shaped for future AI-driven CAD operations. It is not yet a scientific replacement for V4.

V4 currently has materially stronger constraint/relation semantics, solver scaling, conditioning diagnostics, contradiction analysis, topology, spatial validation, reference integrity, engineering validation, authoring contracts, and production-grade export semantics.

V5 currently has important architectural advantages: native ownership of its kernel, immutable part build identities, part-level caching, explicit assembly composition from immutable parts, a transport-neutral direction, capability discovery, and an explicit AI protocol surface.

The most important finding is that the current V5 native 2D implementation contains correctness vulnerabilities that can produce mathematically false engineering results even while compilation and unit tests are green. These must be treated as kernel defects, not merely feature gaps.

## 1. Capability comparison

| Domain | V4 | V5 native | Assessment |
|---|---|---|---|
| Canonical design model | Mature | Native foundation | V5 architecture is cleaner |
| Immutable build identity | Not primary | First-class | V5 advantage |
| Part cache | Not primary | First-class | V5 advantage |
| Assembly from immutable parts | Composition support | Explicit | V5 advantage |
| Dependency graph | Parameters, geometry, constraints, relations | Parameters, geometry, constraints | V4 broader |
| Constraint vocabulary | Broad | Small initial subset | V4 stronger |
| Geometric relations | Broad | Not equivalent | V4 much stronger |
| Constraint diagnostics | Residuals, contradictions, redundancy, classification | Residuals/basic rank | V4 stronger |
| Conditioning | Singular values/condition number | No equivalent | V4 much stronger |
| Solver scaling | Explicit policy | Raw normal equations | V4 much stronger |
| Jacobians | Finite difference + analytic relation Jacobians | Forward finite difference | V4 stronger |
| Candidate/accepted state | Explicit | Snapshot oriented | Both conceptually sound |
| Topology | Vertices, edges, wires, faces | Vertices, edges, wires | V4 stronger |
| Spatial validation | Exact validation pipeline | Bounding-box separation | V4 much stronger |
| Reference migration | Explicit | Minimal | V4 stronger |
| Engineering rules | Explicit | Not equivalent | V4 stronger |
| Dimensions | Broad semantic system | Basic evaluator | V4 stronger |
| DXF | Semantic/manufacturing adapters | Basic deterministic exporter | V4 stronger |
| AI-facing protocol | Guidance/documentation | Explicit capability + operation contracts | V5 advantage |

## 2. Critical scientific vulnerabilities

### CRITICAL — Circle endpoint semantics

V5 `geometryEndpoint()` currently returns the circle center for both `start` and `end`. A circle has no center-as-endpoint semantics.

Consequences include incorrect topology and incorrect operations that consume generic endpoint abstractions.

Required correction: circles must either be explicitly endpoint-less or use a documented circumference parameterization point when a closed-edge representation requires one. The center must never be returned as a circle endpoint.

### CRITICAL — Circle and arc length

V5 dimension `length` is calculated from query start/end Euclidean distance. For a full circle this yields zero. For an arc this yields the chord, not the arc length.

Correct engineering semantics:

- circle circumference = `2 * PI * radius`
- arc length = `radius * abs(endAngle - startAngle)` with the kernel's defined angular convention.

### CRITICAL — Spatial intersection is not exact

V5 spatial analysis treats axis-aligned bounding-box distance as geometry distance and derives `intersects` from it.

Overlapping AABBs do not imply geometric intersection. This can create false positives.

AABB must be a broad-phase filter only:

```text
AABB broad phase
    -> exact curve/curve narrow phase
    -> authoritative distance/intersection result
```

### HIGH — Greedy topology construction

V5 builds wires by choosing the first unused incident edge. This is not sufficient for manifold topology.

It can silently choose an arbitrary continuation at branching or ambiguous vertices and does not diagnose non-manifold or ambiguous topology.

Topology construction must model incidence explicitly and fail/diagnose when continuation is not uniquely defined.

### HIGH — Invalid constraint domains silently satisfy

Horizontal/vertical constraints return zero residual for non-line geometry. That makes an invalid constraint appear satisfied.

Domain-invalid constraints must produce a structured error/diagnostic instead.

### HIGH — Geometry-to-geometry distance is ambiguous

V5 distance between two geometry IDs is implemented as distance between their `start` points. That is not a generic geometric distance.

The kernel must distinguish explicit operations such as:

- point-to-point;
- endpoint-to-endpoint;
- center-to-center;
- minimum curve distance;
- signed distance where applicable.

### HIGH — Solver conditioning and scaling

V5 solves normal equations `J^T J`, which can worsen conditioning, and lacks the physical/scaled variable treatment present in V4.

V5 needs:

- characteristic variable scales;
- residual scales/units;
- robust least-squares linear algebra;
- conditioning diagnostics;
- explicit singular/ill-conditioned states.

### HIGH — Convergence is not complete validity

A numerically small residual does not prove that the materialized geometry is valid.

Acceptance must require all relevant predicates:

```text
numerical convergence
AND geometric validity
AND constraint satisfaction
AND relation satisfaction
AND topology validity
AND reference validity
AND engineering validation
```

### MEDIUM — Rank alone is not conditioning

V5 reports numerical rank but not the singular spectrum/condition number needed to distinguish well-conditioned from nearly singular solutions.

### MEDIUM — Build identity depends on supplied source revision

The build hash includes `sourceRevision`, but does not independently hash the executable design function itself. Reproducibility therefore depends on the source revision being authoritative and content-derived at the caller boundary.

### MEDIUM — Units are not yet first-class

Raw model numbers are used without a comprehensive dimensional system for constraints, scales, and tolerances. V4 contains stronger engineering semantics here.

## 3. V4 capabilities that V5 should recover natively

V4 remains the scientific reference for:

- broad parametric constraint vocabulary;
- geometric relations;
- named residual components;
- solver scaling policy;
- characteristic variable scales;
- conditioning and singular-value analysis;
- contradiction diagnostics;
- candidate/accepted state validation;
- robust topology and faces;
- exact spatial validation;
- reference migration and stale-reference diagnostics;
- engineering-rule validation;
- semantic dimensions;
- deterministic semantic/manufacturing export;
- authoring settings, layers, standards, and configuration contracts.

V5 should reimplement these natively rather than wrapping V4 as its runtime authority.

## 4. AI trust contract

A future AI must never equate:

```text
solver converged
geometry valid
engineering design valid
```

They are separate facts. The kernel should expose evidence for each.

The AI-facing result should eventually contain at least:

```text
structuralValidity
constraintValidity
relationValidity
numericalConditioning
referenceValidity
topologyValidity
spatialValidity
engineeringRuleValidity
exportValidity
```

AI-generated operations should fail closed when required evidence is absent.

## 5. Scientific completion gate

V5 should not be called a scientifically complete successor to V4 until:

1. circle/arc endpoint semantics are explicit;
2. circle/arc lengths are exact;
3. spatial analysis has an exact narrow phase;
4. topology detects ambiguity/non-manifold cases;
5. invalid constraint domains are rejected;
6. solver scaling is explicit;
7. solver acceptance validates the materialized geometry;
8. conditioning diagnostics distinguish safe and ill-conditioned results;
9. reference validity is explicit;
10. engineering validation gates acceptance;
11. property/differential tests cover degenerate and near-degenerate geometry;
12. AI-facing evidence exposes confidence/conditioning rather than only `converged`.
