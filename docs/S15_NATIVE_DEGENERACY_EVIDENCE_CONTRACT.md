# UMLCAD V6 S15 — Native Degeneracy Evidence Contract

## Purpose

This gate adds explicit native evidence for OCCT-declared degenerated edges without promoting that evidence into an automatic UMLCAD pathology or repair decision.

OCCT exposes `BRep_Tool::Degenerated` as a direct edge property. A degenerated edge is a legitimate B-Rep representation for some analytic surfaces; therefore native degeneracy and native invalidity are separate observations.

## Evidence

The reference backend exposes one read-only measurement:

```text
shape -> count of edges for which BRep_Tool::Degenerated(edge) is true
```

The operation is deterministic for an immutable native shape and performs no topology or geometry mutation.

## Conformance boundary

The gate proves two representative cases:

1. a regular box has zero native degenerated edges;
2. a sphere exposes at least one native degenerated edge while remaining valid and edge-manifold under the existing native validator.

The exact sphere count is not treated as a semantic invariant; only the native property itself is asserted.

## Authority separation

The evidence flow is:

```text
OCCT representation
      ↓
explicit native degeneracy evidence
      ↓
UMLCAD interpretation / policy
```

Native evidence does not by itself authorize deletion, merging, healing, tolerance changes, or conversion into a topological failure.

## Boundary

This gate does not detect or repair:

- zero-area faces;
- geometric self-intersections;
- coincident but separately identified entities;
- invalid parameterizations;
- gaps or near-coincident boundaries;
- imported-shape defects beyond the explicit native evidence collected;
- automatic degenerate-edge removal.

Any repair of a native-degenerate edge requires a separate contract that specifies the geometric invariant, topology invariant, admissibility conditions, changed-state evidence, and failure behavior.

## Required gate

```text
native degenerated-edge evidence
-> valid/invalid separation
-> deterministic read-only measurement
-> null-input rejection
-> full debug/release/format/Clippy CI
```
