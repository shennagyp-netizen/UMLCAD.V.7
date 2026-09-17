# V6 S18 — Reference Evolution Contract

## Purpose

V6 persistent semantic references must classify topology evolution explicitly rather than guessing that a regenerated target is the previous target.

This contract defines the cardinality layer used after an upstream descriptor/reference resolver has produced semantic match counts.

## Required classes

For semantic match counts:

```text
1 → 1 = OneToOne
1 → 0 = OneToZero
1 → many = OneToMany
many → 1 = ManyToOne
many → many = ManyToMany
```

A previous target set with zero members is not a valid evolution origin. A multi-target origin whose entire target set disappears is rejected because the declared S18 vocabulary does not yet define a `many → 0` class.

## Authority

The classifier is backend-neutral. It does not inspect:

- native OCCT handles;
- pointer addresses;
- renderer/display ordering;
- array indices as semantic identity;
- process-local state;
- coordinate proximity as an implicit identity decision.

It consumes only semantic cardinalities established by the resolver layer.

## Fail-closed rule

`0 → n` is rejected with `EmptyBefore`. `1 → 0` is explicitly classified as `OneToZero`. `many → 0` is rejected with `MultipleTargetsLost` until a separate contract defines that case.

The classifier never converts disappearance into `NotFound`, `Unique`, or an inferred identity.

## Ambiguity rule

`OneToMany`, `ManyToOne`, and `ManyToMany` are explicitly non-unique for a persistent-reference consumer. The classifier does not choose a descendant or predecessor.

`OneToZero` is also not a successful identity-preservation result: the prior unique target has disappeared.

A higher semantic layer may apply an independently defined provenance/history rule later, but that rule is outside this contract.

## Determinism

For identical cardinality inputs, the result is identical on every evaluation. No tolerance, backend, allocation order, or execution state participates in classification.

## S18 boundary

This slice closes the cardinality vocabulary only. It does not claim a complete topology-history matcher, geometric correspondence algorithm, or automatic reference repair. Those remain separate contracts requiring independent evidence.
