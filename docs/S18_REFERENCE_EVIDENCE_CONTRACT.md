# V6 S18 — Reference Evolution Evidence Contract

## Purpose

The cardinality classifier produces explicit evidence that a persistent reference consumer can inspect without treating ambiguity as success.

## Evidence fields

`ReferenceEvolutionEvidence` records:

- `before_matches`: semantic matches in the source model;
- `after_matches`: semantic matches in the regenerated model;
- `evolution`: the declared cardinality evolution;
- `unique_identity_preserved`: true only for `OneToOne`.

The evidence object records the classifier decision; it does not select a target.

## Required interpretation

```text
OneToOne   → one unique candidate remains
OneToZero  → the prior unique candidate disappeared
OneToMany  → one candidate became multiple candidates
ManyToOne  → multiple candidates collapsed to one
ManyToMany → multiple candidates remain multiple
```

Only `OneToOne` is a unique-preservation result. The other classes must remain visible to downstream consumers rather than being silently converted to a guessed identity.

## Rejected input

`0 → n` is invalid because there is no source reference population. `many → 0` remains rejected until a dedicated contract defines that evolution class.

## Boundary

This evidence layer does not claim geometric correspondence, topology-history reconstruction, or automatic reference repair. Those require separate evidence-producing algorithms and contracts.
