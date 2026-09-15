# UMLCAD V6 — OCCT/TDD Kernel Integration Kickoff

## Starting point

The V6 repository already contains the .NET framework, semantic model, kernel client, and real HTTP end-to-end tests. The current E2E fixture is intentionally 2D and exercises lines, constraints, invalid geometry, stale references, concurrency, and deterministic build identity.

V6 must now introduce a separate 3D geometry backend without making OCCT the application or semantic authority.

```text
.NET semantic source
        ↓
immutable BuildPackage
        ↓
UMLCAD kernel API
        ↓
Rust backend-neutral geometry API
        ↓
OCCT adapter
        ↓
B-Rep / NURBS / 3D result
        ↓
UMLCAD validation + semantic topology
        ↓
Compiled result
```

## Branch

Development begins on:

```text
v6-occt-tdd
```

`main` remains unchanged until a TDD slice reaches its gate.

## TDD rule

Every new geometry capability follows this exact loop:

```text
RED
  ↓
backend-neutral contract test
  ↓
minimal implementation
  ↓
GREEN
  ↓
unit red-team
  ↓
integration red-team
  ↓
performance measurement
  ↓
optimization
  ↓
full regression
```

A feature is not accepted because OCCT returned a shape. The UMLCAD acceptance contract must pass.

## Test layers

### 1. API unit tests

Test pure Rust types and semantics without OCCT:

- invalid finite values;
- dimensional/tolerance rules;
- operation identity;
- immutable result behavior;
- diagnostic classification;
- canonical serialization;
- deterministic ordering.

### 2. Backend contract tests

The same tests must run against every geometry backend.

Initial contract classes:

- primitive construction;
- translation/rotation;
- geometry validity;
- topology validity;
- Boolean semantics;
- fillet/chamfer semantics;
- intersection semantics;
- tessellation invariants.

The contract must never mention OCCT classes.

### 3. OCCT adapter tests

Test only the Rust/OCCT boundary:

- object creation and ownership;
- exception/status translation;
- tolerance propagation;
- topology extraction;
- topology mapping;
- resource cleanup;
- supported/unsupported operation mapping;
- OCCT version/build configuration.

### 4. .NET → Rust → OCCT integration tests

The existing `RustKernelEndToEndTests` pattern becomes the integration model for 3D:

```text
C# sample model
   ↓
BuildPackage
   ↓
real kernel request
   ↓
Rust semantic evaluation
   ↓
OCCT geometry
   ↓
UMLCAD validation
   ↓
compiled model / diagnostics
```

These tests must use the actual HTTP/API boundary, not an in-process shortcut.

### 5. Full-system red-team tests

Attack the complete stack with:

- malformed semantic requests;
- stale references;
- topology-changing rebuilds;
- invalid dimensions;
- zero/negative extents;
- near-zero features;
- near-coincident faces;
- tangent/near-tangent intersections;
- large-coordinate/small-feature combinations;
- repeated Boolean chains;
- fillets approaching geometric limits;
- invalid imported geometry;
- pathological assembly graphs;
- concurrent requests for the same immutable build;
- concurrent independent builds.

## .NET geometry corpus

The .NET projects are the source of truth for engineering fixtures. Do not hand-create a separate geometry corpus that silently diverges from the framework.

The current repository contains a real .NET test suite under:

```text
dotnet/tests/UMLCAD.Framework.Tests/
```

The existing `RustKernelEndToEndTests.cs` provides the first established pattern for real kernel integration and deterministic concurrency.

As V6 3D authoring APIs are introduced, each .NET sample project must be promoted into a versioned geometry fixture where practical.

Each fixture records:

- fixture ID;
- source project/file;
- intended engineering meaning;
- parameters;
- expected topology class;
- expected valid/invalid classification;
- acceptable numerical error;
- expected persistent references;
- expected failure mode where applicable.

## Required initial geometry corpus

The first 3D corpus must contain progressively harder manufactured/mechanical examples:

```text
01 primitive box
02 cylinders and bores
03 mounting plate
04 bracket
05 revolved shaft
06 bearing housing
07 lead-screw support
08 MGN12C-style rail/carriage geometry
09 pulley/gear-like rotational body
10 multi-feature housing
11 filleted housing
12 patterned holes
13 shell/thin-wall part
14 sweep
15 loft/freeform surface
16 Boolean-heavy part
17 imported imperfect geometry
18 topology-changing rebuild sequence
19 multi-part assembly
20 adversarial/invalid models
```

The specific geometry should come from actual .NET sample projects once their V6 authoring API exists; the test corpus must not invent an API that does not exist.

## Reference oracle policy

No single backend is treated as absolute truth for every property.

For each test family classify the oracle:

```text
analytic mathematical invariant
        OR
independent geometric calculation
        OR
backend-independent topology invariant
        OR
cross-backend agreement
        OR
explicitly documented backend behavior
```

OCCT output is not automatically proof of correctness.

Likewise, a future native UMLCAD geometry implementation is not automatically proof merely because it disagrees with OCCT.

## Accuracy policy

Keep three tolerance domains separate:

```text
modeling tolerance
validation/acceptance tolerance
measurement/comparison tolerance
```

Never silently reuse one tolerance for all purposes.

Exact analytic constructions should remain exact where the representation permits it. Numerical operations must expose the numerical context used for acceptance.

A result that cannot be classified confidently must return structured `INDETERMINATE`/`UNSUPPORTED` evidence rather than a fabricated success.

## Performance policy

Optimize only after correctness gates pass.

Every performance optimization requires:

1. a benchmark;
2. a baseline;
3. a measured improvement;
4. no semantic regression;
5. no new nondeterminism;
6. no loss of diagnostic evidence.

Measure separately:

- FFI overhead;
- OCCT modeling time;
- topology extraction;
- semantic reference mapping;
- validation;
- tessellation;
- serialization;
- end-to-end request latency;
- memory allocation/peak memory;
- concurrency scaling.

Prefer immutable caching keyed by content/build identity. Cache loss must never alter a result.

## First implementation gates

### Gate A — contract foundation

- backend-neutral API exists;
- tolerance model exists;
- structured result/evidence exists;
- contract tests exist;
- no OCCT types leak above the adapter.

### Gate B — first OCCT primitive

Implement one primitive end-to-end and make the corresponding contract tests GREEN.

Required first primitive:

```text
box solid
```

The first primitive must already prove:

- Rust ↔ OCCT ownership boundary;
- deterministic result metadata;
- topology extraction;
- validation;
- .NET integration path;
- failure classification.

### Gate C — first parametric change

A .NET parameter change must rebuild the same semantic model with a new build identity and must not mutate the previous immutable result.

### Gate D — topology reference

A face/edge/vertex reference must survive a safe rebuild and fail explicitly when it becomes ambiguous or stale.

### Gate E — first adversarial operation

Boolean cut on a controlled box/cylinder fixture, followed by red-team variants for coincident/tangent/near-degenerate configurations.

## Immediate rule

Do not implement the full CAD feature list now.

The first objective is to prove that one 3D operation can travel through the complete V6 architecture:

```text
.NET sample
  → immutable semantic package
  → transport-neutral kernel request
  → Rust geometry contract
  → OCCT
  → topology extraction
  → UMLCAD validation
  → immutable result
  → deterministic evidence
```

Once that vertical slice is proven, repeat the same TDD discipline for the next operation.
