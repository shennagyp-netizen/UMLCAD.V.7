# V6 S18 — Reference Migration Contract

## Purpose

S18 must distinguish topology/reference evolution from identity selection. A migration result describes what the semantic match cardinality means; it does not select a candidate or reconstruct modeling history.

## Mapping

The migration layer consumes the already-authoritative cardinality classification:

```text
1 → 1     = Preserved
1 → 0     = Invalidated
1 → many  = Split
many → 1  = Merged
many → many = Ambiguous
```

`many → 0` remains rejected because the current S18 cardinality contract does not define a multi-source disappearance class.

## Identity safety

Only `Preserved` is safe to expose as unique identity continuation.

`Invalidated` means the previous unique candidate disappeared.

`Split` means one previous candidate has multiple semantic successors; no successor is selected.

`Merged` means multiple previous candidates have one semantic successor; no predecessor is selected.

`Ambiguous` means multiple candidates remain on both sides; no identity continuation is inferred.

## Authority restrictions

This API is backend-neutral and value-based. It must not inspect:

- native OCCT handles or pointer addresses;
- renderer/display order;
- container/storage order;
- array indices as identity;
- process-local state;
- coordinate proximity as an identity rule;
- undocumented operation history.

The upstream resolver supplies semantic match counts. This layer classifies their meaning only.

## Fail-closed behavior

Invalid source cardinality remains an explicit error. Multi-target disappearance remains an explicit error. No error is converted to `Preserved` or any other successful migration class.

The absence of a migration match is therefore observable rather than silently interpreted as identity preservation.

## Immutability and determinism

Migration evidence is an immutable value derived only from the two semantic match counts and the classifier result. Equal inputs yield equal results on every evaluation.

## Scope boundary

This contract does not provide:

- geometric correspondence algorithms;
- topology-history reconstruction;
- automatic reference repair;
- candidate ranking;
- provenance inference;
- persistent storage;
- distributed synchronization.

Those require independent contracts and evidence.
