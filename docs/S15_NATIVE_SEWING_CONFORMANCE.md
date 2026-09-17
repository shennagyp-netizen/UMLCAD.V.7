# UMLCAD V6 S15 — Native Sewing Conformance Gate

## Purpose

This gate verifies that the OCCT reference backend can realize a bounded closed-shell sewing case while keeping UMLCAD topology semantics authoritative.

## Bounded fixture

The native test constructs one axis-aligned box, extracts its six faces, and submits those faces to `BRepBuilderAPI_Sewing` with an explicit tolerance.

Expected native result:

```text
1 solid
1 shell
6 faces
12 edges
8 vertices
0 free edges
0 multiple edges
0 degenerated shapes
valid = true
manifold = true
```

The exact topology counts are used here because the fixture is a fixed primitive and therefore representation-independent for this conformance case.

## Native policy

The bridge uses `BRepBuilderAPI_Sewing` in its default manifold mode and does not enable non-manifold sewing. The operation is rejected for non-finite or negative tolerance and null/invalid native results.

Native OCCT validity is checked with `BRepCheck_Analyzer`, while the existing UMLCAD topology validation remains the semantic authority.

## Boundary

This is a bounded native realization test, not a claim of arbitrary sewing support. It does not yet expose automatic geometric edge matching, free-edge closure, gap healing, degenerate repair, cavity classification, or imported-pathology repair.

In particular, the native sewing tolerance is an algorithmic backend input. It must not be substituted for UMLCAD engineering or validation tolerances.

## Required evidence

The test must establish all of the following:

1. native sewing completes successfully;
2. no free, multiple, or degenerated edges are reported;
3. topology counts match the fixed box-face fixture;
4. native validity is true;
5. existing manifold evidence is true;
6. invalid tolerance is rejected before native sewing execution.

A renderer-visible result without this evidence is not a green gate.
