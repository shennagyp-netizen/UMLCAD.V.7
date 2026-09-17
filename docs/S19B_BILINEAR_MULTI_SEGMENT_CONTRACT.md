# V6 S19B — Bilinear Freeform/Planar Multi-Segment Intersection Contract

## Scope

S19B extends the S19A bounded unweighted bilinear NURBS / planar bilinear NURBS intersection family from one segment to the complete isolated two-root and four-root rectangle-boundary cases.

The semantic equation is the exact bilinear restriction

`a + b*s + c*t + d*s*t = 0`, with `0 <= s,t <= 1`.

A non-zero bilinear polynomial intersects the rectangular boundary in at most four isolated roots unless a boundary edge or the entire patch is underdetermined.

## Authority

The semantic layer owns the bilinear equation, root classification, parameter-domain validity, branch classification, deterministic root ordering, and segment pairing. OCCT is only a native realization/conformance oracle.

No native topology identity, renderer result, tolerance widening, sampling heuristic, or candidate-history inference may become semantic authority.

## Certified outcomes

- zero isolated boundary roots: `NoIntersection`;
- two isolated roots: one deterministic intersection segment;
- four isolated roots: two deterministic intersection segments, paired by the exact bilinear branch structure;
- one, three, boundary-coincident, or otherwise underdetermined cases: explicit `Ambiguous`/structured fail-closed outcome.

For four isolated roots, pairing is not based merely on lexicographic boundary ordering. The implementation uses the bilinear branch asymptote to partition roots into their two mathematical branches. A root sufficiently close to the asymptote, a degenerate asymptote, or an inconsistent partition is rejected as underdetermined rather than guessed.

## Gate requirements

The slice must pass independent semantic tests, branch-pairing adversarial tests, deterministic repeat tests, native conformance where applicable, workspace/kernel release tests, and both kernel Clippy gates before merge.

The contract does not claim unrestricted NURBS/NURBS tracing, arbitrary rational bilinear tracing, or general freeform intersection completeness.
