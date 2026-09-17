# UMLCAD V6 — S16 Cancellation and Operation Error Semantics Contract

## Purpose

This contract defines the backend-neutral cancellation and structured error semantics for kernel operations. It exists so operation execution can stop deterministically without making the kernel stateful and so callers can distinguish cancellation, semantic failures, unsupported cases, and native-backend failures without parsing free-form messages.

## Authority

The semantic contract is authoritative. A native backend may provide implementation-specific diagnostics, but those diagnostics do not define operation meaning or success.

## Cancellation

Cancellation is represented by a caller-owned query object implementing the immutable `OperationCancellation` trait.

The kernel may query cancellation at operation-defined safe points. The query must have no required mutation, global state, thread-local state, backend handle, or kernel-owned lifetime.

Required semantics:

1. If cancellation is already observed before execution starts, the operation returns `OperationErrorKind::Cancelled` and produces no committed output.
2. If cancellation is observed at any declared safe point during execution, the operation returns `OperationErrorKind::Cancelled` and produces no committed output.
3. A cancellation request is not a semantic geometry/topology failure.
4. Cancellation does not imply rollback semantics. Temporary implementation state may be discarded by the executing layer; transaction and rollback guarantees remain outside this contract.
5. The kernel must not silently convert cancellation into success, partial success, or an approximate result.
6. A cancellation query may change between calls because it is caller-owned; the kernel does not cache or mutate it.

`NeverCancel` provides a deterministic always-false implementation for callers and tests that do not need cancellation.

## Structured operation errors

`OperationError` contains:

- the positive `OperationId` associated with the attempted operation;
- an explicit `OperationPhase` identifying where failure was observed;
- an `OperationErrorKind` category.

The semantic categories are:

- `Cancelled`: execution was intentionally stopped by the cancellation query;
- `InvalidInput`: caller-provided operation inputs or preconditions are invalid;
- `Unsupported`: the requested operation is outside the implemented semantic family;
- `SemanticInvariantViolation`: an operation would violate a kernel invariant or produce an uncertifiable semantic result;
- `BackendFailure`: the selected native backend failed to realize an otherwise valid semantic request;
- `Internal`: an implementation failure not attributable to caller input or backend realization.

The category is machine-readable and is not inferred from diagnostic strings.

`BackendFailure` identifies only that backend realization failed. Backend-specific diagnostics, native handles, exception text, and native error codes remain backend-layer data and are not part of semantic identity or kernel state.

## Propagation

`OperationOutcome<T>` is the backend-neutral result form: `Result<T, OperationError>`.

Operation layers must propagate an existing `OperationError` without changing its category merely to add context. Additional context belongs to the surrounding layer's own diagnostics.

Cancellation must remain distinguishable from all other error categories after propagation.

## Immutability and state boundaries

The cancellation interface and error values are ordinary values/queries. They do not own mutable kernel state. An operation implementation may allocate temporary backend state internally, but no such state may become semantic identity.

This contract does not define:

- transactions or rollback;
- persistence or recovery;
- authorization;
- service transport;
- retry policy;
- scheduler/threading policy;
- native backend error schemas;
- partial-result semantics.

Unsupported cases remain explicit rather than being approximated.

## Required conformance tests

An implementation conforming to this contract must prove at minimum:

1. pre-start cancellation returns `Cancelled` and does not invoke/commit semantic execution;
2. mid-operation cancellation at a declared safe point returns `Cancelled`;
3. `NeverCancel` never requests cancellation;
4. invalid-input, unsupported, semantic-invariant, backend-failure, internal, and cancelled categories remain distinguishable;
5. operation id and phase survive error propagation unchanged;
6. cancellation and errors do not mutate input semantic snapshots or provenance;
7. no native backend type is required by the backend-neutral API.
