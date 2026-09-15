# UMLCAD V5 — AI Framework and Kernel Usage Manual

## Purpose

This document is the operating contract for an AI agent that creates, modifies, reviews, or refactors UMLCAD V5.

V5 is a native CAD kernel and semantic CAD system. It is not a GUI-first drawing program. V4 is historical/reference material only; V5 code and V5 kernel contracts are the authority.

## 1. Architectural authority

The authoritative pipeline is:

```text
authored semantic source / operation
        ↓
V5 kernel
        ↓
immutable semantic build
        ↓
validation / analysis / solver
        ↓
projection / export / client
```

The renderer, SVG scene, Metal scene, browser state, client cache, or transport is never the engineering authority.

The kernel owns:

- semantic geometry;
- parameters;
- constraints and relations;
- dependency graph;
- deterministic evaluation;
- immutable snapshots;
- part build identities;
- part-level cache;
- assembly composition;
- diagnostics;
- engineering analysis;
- export semantics.

## 2. Before changing code

An AI agent must first inspect:

```text
KERNEL_SPEC.md
V4_V5_KERNEL_AUDIT.md
ARCHITECTURE.md
kernel/src/native-index.ts
kernel/src/core.ts
kernel/src/kernel.ts
```

Then inspect the exact capability being modified.

Do not infer semantics from a GUI or from rendered output.

## 3. V4 usage rule

V4 may be consulted for proven semantics, numerical algorithms, tests, edge cases, and missing engineering behavior.

Do not introduce a V4 runtime dependency merely to avoid implementing V5-native semantics.

The migration rule is:

```text
V4 reference
    ↓
understand semantic contract
    ↓
V5-native implementation
    ↓
V5 tests and evidence
```

## 4. Immutable build model

The primary incremental unit is a part.

```text
source revision
    ↓
part build
    ↓
immutable build identity
    ↓
cache
```

An assembly consumes selected immutable part builds and is built separately.

A changed part creates a new build identity. Existing builds are not mutated.

## 5. Identity

Never derive engineering identity from:

- array index;
- execution order;
- screen position;
- SVG/DOM ID;
- renderer object identity;
- floating-point similarity;
- random values;
- timestamps.

Use authored semantic IDs and explicit build identities.

## 6. Constraints and solver

Constraints are engineering semantics, not UI hints.

When adding a constraint, define all of:

```text
semantic meaning
residual components
units
scale
valid domain
Jacobian
degeneracy behavior
contradiction behavior
acceptance rule
```

Never add a constraint because it makes a demo converge.

A solver result is not automatically an accepted engineering result. Acceptance must include geometry validity and all relevant engineering validation.

## 7. Numerical safety

AI-generated numerical code must explicitly handle:

- scale differences;
- near-zero values;
- singular systems;
- ill-conditioned systems;
- finite/non-finite values;
- degeneracy;
- tolerance definitions;
- stable parameterization domains.

Do not use an axis-aligned bounding box as a final intersection predicate.

Do not use rank alone as a conditioning test.

Do not silently clamp invalid geometry into validity.

## 8. Geometry semantics

Line, circle, and arc semantics must remain mathematically explicit.

A circle has no ordinary start/end endpoints. If a closed-edge representation requires a parameterized point, that parameterization must be documented and must never substitute the center for an endpoint.

Arc length and circle circumference are not endpoint chord lengths.

## 9. Topology

Topology is semantic connectivity, not drawing order.

A topology builder must detect ambiguous and non-manifold connectivity instead of selecting an arbitrary continuation.

## 10. Spatial analysis

Use a broad-phase/narrow-phase architecture:

```text
AABB candidate filter
        ↓
exact geometric predicate
        ↓
authoritative result
```

Bounding-box overlap is not itself proof of intersection.

## 11. Dimensions

A dimension is an engineering quantity, not formatted text.

Keep separate:

```text
mathematical quantity
units
nominal/constraint meaning
presentation formatting
```

Display precision must never change the engineering value.

## 12. Transactional editing

Mutations follow:

```text
accepted build
    ↓
candidate operation
    ↓
evaluation
    ↓
constraint/validation evidence
    ↓
accept OR reject
```

A failed operation must leave the accepted state unchanged.

Every mutation must declare the base revision/build it targets. Stale operations fail closed.

## 13. AI operation discipline

An AI may propose operations such as:

```text
set-parameter
edit semantic geometry
add/remove constraint
request regeneration
build part
build assembly
query geometry
request diagnostics
export
```

Operations must be semantic and reproducible.

An AI must not directly manipulate rendering coordinates as a substitute for a semantic CAD edit.

## 14. Evidence-first responses

A future AI integration should prefer structured evidence:

```text
accepted
build identity
revision
changed semantic targets
diagnostics
constraint analysis
conditioning
validation result
```

over natural-language claims such as "the model looks correct".

## 15. Tests

Tests must prove engineering behavior, not only code execution.

Required classes include:

- degenerate geometry;
- near-degenerate geometry;
- constraint contradiction;
- ill-conditioned systems;
- exact geometric intersections;
- topology ambiguity;
- stale revisions;
- deterministic build identity;
- cache reuse;
- assembly immutability;
- export determinism.

## 16. Completion rule

An AI agent must not declare a kernel capability scientifically complete merely because TypeScript compiles or CI is green.

The implementation, tests, numerical semantics, and documentation must agree.
