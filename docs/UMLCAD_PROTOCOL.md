# UMLCAD V5 — Transport-Neutral Semantic Client Protocol

## 1. Purpose

This protocol is the semantic contract between any UMLCAD client and the V5 host/session layer.

It is intentionally independent of:

- Swift;
- Metal;
- AppKit/UIKit;
- WinUI;
- browser APIs;
- SVG/Canvas/WebGL;
- WebSocket implementation details;
- any particular transport.

The architecture is:

```text
client
  ↓
semantic protocol binding
  ↓
V5 host/session
  ↓
V5 native kernel
```

WebSocket may be used as a transport, but it is not the semantic architecture.

## 2. Kernel authority

The V5 kernel owns the authoritative engineering state.

```text
semantic source / operation
        ↓
V5 kernel
        ↓
immutable build
        ↓
semantic projection
        ↓
client
```

The client never directly mutates kernel state.

## 3. Revision and build identity

Every authoritative operation must identify the build/revision on which it is based.

A mutation must include sufficient identity to establish:

```text
project
part/document
base revision
base build identity
operation id
```

Stale operations fail closed.

An accepted mutation produces a new authoritative result/build identity.

## 4. Message envelope

The semantic envelope is conceptually:

```ts
interface ProtocolMessage<T> {
  protocol: string;
  version: string;
  messageId: string;
  sessionId: string;
  kind: string;
  correlationId?: string;
  causationId?: string;
  revision?: string;
  buildIdentity?: string;
  payload: T;
}
```

Transport serialization is an implementation detail.

## 5. Semantic message categories

### Requests

Examples:

```text
session.open
session.resume
part.build
assembly.build
document.snapshot
kernel.query
kernel.analyze
kernel.solve
kernel.export
operation.propose
operation.commit
```

### Responses

Responses correlate to requests and must explicitly distinguish success, rejection, and insufficient evidence.

### Events

Events may include:

```text
build.created
build.reused
model.changed
diagnostics.changed
transaction.accepted
transaction.rejected
projection.changed
session.changed
```

### Errors

Errors are structured semantic data. Human-readable text is supplemental.

## 6. Client neutrality

All clients implement the same semantic contract.

```text
Web
macOS
Windows
Linux
iOS
Android
CLI
AI agent
future clients
```

Platform-specific differences may affect rendering, input devices, threading, transport, and local caches. They must not change CAD mathematics.

## 7. Semantic identity

Do not identify CAD objects by:

- array position;
- rendering order;
- screen position;
- SVG/DOM ID;
- GPU object ID;
- DXF handle;
- geometric similarity;
- random ID;
- timestamp.

Use semantic geometry/parameter/constraint/reference identities and immutable build identities.

## 8. Client capabilities

Capabilities describe supported semantic operations.

Examples:

```text
query-geometry
set-parameter
regenerate-part
build-assembly
analyze-constraints
solve
analyze-spatial
resolve-reference
export-dxf
```

Client capability information is not authorization.

The host recomputes authoritative validity.

## 9. Preview versus authority

A client may calculate local preview.

Preview is never accepted CAD state.

```text
client prediction
      ≠
authoritative build
```

When the base revision changes, stale preview state must be discarded or explicitly rebased.

## 10. Operation lifecycle

A semantic edit follows:

```text
PROPOSE
   ↓
VALIDATE BASE REVISION/BUILD
   ↓
BUILD CANDIDATE
   ↓
CONSTRAINT / NUMERICAL ANALYSIS
   ↓
TOPOLOGY / REFERENCE / ENGINEERING VALIDATION
   ↓
ACCEPT or REJECT
```

A rejected candidate must not mutate the accepted build.

## 11. AI operations

AI agents may propose semantic operations.

An AI operation should contain:

```text
operationId
baseBuild
baseRevision
semantic target
operation kind
operation parameters
```

The response should provide evidence:

```text
accepted
resultingBuild
newRevision
diagnostics
constraint result
conditioning result
validation result
```

Natural-language claims are not proof of correctness.

## 12. Constraint knowledge

A client may use the exposed constraint model for interaction prediction and user feedback.

The client cannot turn its prediction into accepted state.

```text
client = preview / interaction intelligence
kernel = mathematical authority
```

## 13. Source editing boundary

Source-aware applications may map semantic operations to source edits, but source-editing logic remains outside the mathematical kernel.

The source adapter must preserve semantic identity and must fail closed when a semantic target becomes ambiguous or stale.

Raw string replacement is not a substitute for a semantic source binding mechanism.

## 14. Transport rules

The transport layer may provide:

- connection management;
- ordering;
- retry mechanics;
- heartbeats;
- framing.

It must not redefine:

- geometry;
- constraints;
- revisions;
- build identity;
- validation;
- authorization;
- engineering semantics.

## 15. AI compatibility rule

Future AI clients must be able to operate without knowing the client rendering stack.

The stable integration surface is the semantic kernel contract, not the GUI implementation.

Any new GUI, agent, automation host, or future model should be able to discover capabilities, submit semantic operations, receive structured evidence, and consume immutable build results using this protocol.
