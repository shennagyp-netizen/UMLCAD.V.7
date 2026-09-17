# UMLCAD V5 — Native Kernel Architecture

## Authority

V5 is the active engineering kernel. V4 is reference material only; V5 core semantics must not depend on V4.

## Semantic authority

The kernel owns the authoritative semantic model:

- parameters and values;
- line, circle, and circular-arc geometry;
- constraints;
- deterministic dependency graph;
- immutable evaluated snapshots;
- validation and diagnostics;
- build identities and immutable part cache entries;
- assembly composition from immutable part builds;
- geometric queries;
- constraint analysis and bounded solving;
- topology;
- dimensions;
- spatial analysis;
- semantic references;
- deterministic DXF projection;
- capability metadata for machine clients.

Everything else is a projection or an adapter.

## Evaluation model

```text
semantic program
    ↓
validated immutable snapshot
    ↓
analysis / solve / topology / dimensions / spatial queries
    ↓
artifacts and manifests
```

A rendering, SVG path, DXF file, screen coordinate, cache record, or LLM proposal is never authoritative.

## Build model

The fundamental reusable unit is an immutable part build:

```text
source revision + part identity + build profile
              ↓
       immutable part build
              ↓
 assembly consumes selected part builds
```

Changing a part creates another build identity. Existing builds remain valid and are not mutated.

## Constraint model

Constraint analysis reports residuals, rank, equation/variable counts, degrees of freedom, and satisfaction. Solving is deterministic and bounded; the solver returns a new geometric result rather than mutating the input snapshot.

## Client and AI boundary

A future AI client communicates through semantic identifiers and typed capabilities. It should inspect the authoritative revision, formulate an explicit operation, receive validation/solver evidence, and only then request an accepted state transition through an application layer.

The kernel does not know whether a request originated from a human, an LLM, a browser, a CLI, or an automated design agent.

The kernel must never infer user intent from pixels or silently repair an invalid request.

## Transport independence

Kernel contracts are transport-neutral. HTTP, WebSocket, Git, authentication, browser state, rendering, source editing, and LLM orchestration remain outside the core.

## Determinism

Equivalent authoritative inputs produce equivalent semantic ordering and results. Cache loss affects performance only.

## Extension rule

A new feature belongs in the kernel only when it adds semantic engineering authority. Client-specific behavior belongs above the kernel.
