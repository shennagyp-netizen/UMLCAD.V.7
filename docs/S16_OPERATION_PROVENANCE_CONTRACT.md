# V6 S16 — Operation Provenance Contract

## Purpose

This contract defines a small backend-neutral provenance value for an ordered sequence of kernel operations.

Each operation records an explicit semantic operation identifier, operation kind, ordered input topology snapshot identities, and one output topology snapshot identity.

The provenance value is immutable: appending an operation creates a new value and does not mutate the prior provenance.

## Semantic authority

The operation descriptor is semantic data. It does not contain OCCT handles, native pointers, allocation addresses, renderer state, process state, filesystem paths, or transport-specific identifiers.

`TopologySnapshotId` values refer only to the backend-neutral topology snapshot contract defined in `docs/S16_TOPOLOGY_SNAPSHOT_IDENTITY_CONTRACT.md`.

## Operation identity

The current bounded API uses a positive monotonic `OperationId` supplied by the caller.

Within one provenance value:

- operation identifiers must be strictly increasing;
- a duplicate operation identifier is rejected;
- zero is rejected.

The operation identifier provides deterministic provenance order. It does not claim to be a globally unique database identifier or a distributed transaction identifier.

## Operation descriptor

An `OperationDescriptor` contains:

- `id: OperationId`;
- `kind: OperationKind`;
- `inputs: Vec<TopologySnapshotId>`;
- `output: TopologySnapshotId`.

The bounded operation kinds are `Create`, `Transform`, `Modify`, `Boolean`, `Repair`, `Import`, and `Export`.

An operation must contain at least one input. Every input topology snapshot identity must be non-zero, and the output topology snapshot identity must also be non-zero. This prevents incomplete or uninitialized identity values from entering immutable provenance history.

Input ordering is semantic and is retained exactly. For example, the ordered inputs of a future Boolean operation may distinguish the two operands without relying on a native operation object.

## Provenance identity

`OperationProvenance::identity()` computes a deterministic SHA-256 digest from:

1. a fixed domain tag;
2. the operation count;
3. each operation identifier;
4. the operation kind discriminator;
5. the input count and ordered input snapshot identities;
6. the output snapshot identity.

The digest is value-derived and contains no runtime state.

## Immutability

`append()` takes `&self`, clones the small provenance value, appends the validated descriptor, and returns the new value. Existing provenance remains unchanged.

The referenced snapshot identities are values; the provenance object does not own or mutate the underlying topology graphs.

## Fail-closed behavior

The current bounded contract rejects:

- zero operation identifiers;
- operations without inputs;
- zero input snapshot identities;
- zero output snapshot identities;
- duplicate operation identifiers;
- non-increasing operation identifiers.

The zero-input diagnostic identifies the zero-based input position so callers can correct the descriptor deterministically.

The API does not silently renumber operations, reorder caller inputs, infer missing outputs, or synthesize identifiers.

## Scope boundary

This slice does not yet define:

- distributed/global operation identity;
- concurrent branch reconciliation;
- undo/redo semantics;
- geometry snapshot identity outside topology;
- persistence formats;
- cancellation state;
- transaction/commit semantics;
- service authorization or transport protocols.

Those concerns require separate contracts and must not be inferred from this provenance primitive.
