# CAD Core S1 — TDD and Validation Status

## Baseline

- Main mathematical baseline: `f1ab581140453c087beb50aac2216684196d4bdf`
- Mathematical roadmap: M0–M16 closed for declared domains.
- System-CAD work is based on the existing S0/S1 implementation branch; S0 is not being restarted.

## Current development head

`0dccf7406e8b58240385eb6727586b422e28debd`

PR: #39 — production S1 vertical-slice TDD gate.

## TDD evidence

The mandatory production-path RED test was introduced first:

```
box
 → support face
 → sketch with circle A + circle B
 → constraints
 → + feature
 → - feature
 → recompute
 → authoritative result
 → derived representation
```

RED test commit:

`f452bf5c903d72fcffdaca8042df1977f1c64803`

The expected first missing boundary is production support for Sketch evaluation and, after that, exact circle-based solid composition.

The test has not been represented as GREEN merely because implementation was added.

## Testing paradigms now applied

### Unit / contract
Typed sketch request/result contracts validate:
- finite positive circle geometry;
- deterministic identity inputs;
- unique identities;
- non-stale fixed-constraint references;
- solver-option validity.

### Adapter integration
The .NET `RustSketchConstraintService` maps the typed HTTP contract and fails closed on:
- HTTP failure;
- malformed JSON;
- unexpected schema;
- inconsistent status/result;
- cancellation/timeout.

### Production evaluator integration
`RustCadKernelEvaluator` now dispatches certified Sketch input through `ISketchConstraintService` and preserves the typed solver result in the evaluation result.

### Metamorphic
Sketch request collections are canonicalized by identity. Reordering circles and fixed constraints is required to preserve the semantic request.

The Python E2E probe also compares the result of the canonical ordering with the result of the reversed non-semantic ordering.

### Property / generated bounded testing
Python E2E generates twelve deterministic valid fixed-circle sketches across a bounded coordinate/radius range.

### System E2E
`tests/e2e/run.py` now probes the real `/v1/geometry/solve-sketch` boundary.

### Red-team
The Python gate attacks malformed JSON and unsupported/wrong schemas for the sketch endpoint.

The broader E2E suite continues to attack:
- unknown endpoints;
- invalid build schemas;
- missing semantic payloads;
- unsupported geometry;
- invalid box geometry;
- invalid extrusion frames.

### Execution-population hardening
Authoritative Rust gates reject ignored tests.

Authoritative .NET gates now reject:
- missing test-result summaries;
- zero executed tests;
- failed tests;
- skipped tests.

Engineering-test discovery also requires the S1 production RED suite to be visible to the test runner.

## Current validation limitation

GitHub Actions jobs for the current branch continue to finish as failed zero-step jobs with no executable job steps. Therefore those runs are:

**Not validated — infrastructure execution unavailable**

They must not be interpreted as source-level test failures or passes.

The local environment available to this development session also has no reachable .NET SDK/test runner and no external network access for repository cloning, so local execution evidence cannot honestly be claimed.

## Current implementation status

Implemented:
- typed Sketch solver contract;
- Rust sketch solver HTTP endpoint;
- .NET sketch solver adapter;
- dependency-injected production sketch dispatch;
- typed solver-result preservation;
- TDD RED gate;
- E2E sketch acceptance/red-team cases;
- deterministic/metamorphic sketch request behavior;
- test execution-population enforcement.

Not yet E2E-validated:
- real .NET test execution on the branch;
- real Rust HTTP sketch execution from the authoritative E2E runner;
- complete mandatory S1 production vertical slice.

Still blocked in S1:
- exact circle profile → solid contract;
- exact cylindrical/curved B-Rep authority;
- additive/subtractive solid composition;
- topology evolution across those operations;
- final mandatory end-to-end B-Rep/referenced representation.

## Mathematical authority rule

The current certified extrusion operation accepts polygonal convex planar profiles.

The mandatory S1 scenario uses exact circles.

No polygonal approximation is being introduced merely to satisfy the test.

The next mathematical increment therefore requires a new explicit contract and red test for exact circle/cylindrical solid authority, followed by the smallest certified kernel implementation and its adversarial/E2E gates.

## Milestone rule

S1 remains **IN PROGRESS**.

It must not be called GREEN until its applicable gates execute and pass:

```
TDD RED
→ implementation
→ targeted GREEN
→ red-team GREEN
→ metamorphic/property/conformance
→ full Python E2E
→ full repository gates
→ exact evidence
```