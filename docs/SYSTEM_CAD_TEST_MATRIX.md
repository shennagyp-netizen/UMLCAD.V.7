# UMLCAD.V.7 — System-CAD Test Matrix

This document defines the validation matrix for the current .NET System-CAD increment.

It is subordinate to:

- `docs/SYSTEM_CAD_ARCHITECTURE.md` for semantic ownership and runtime architecture;
- `docs/UMLCAD_V7_MAIN_BRANCH_DEVELOPMENT_PROMPT.md` for TDD/red-team/E2E procedure;
- `docs/CONTINUATION_HANDOFF.md` for repository evidence and implementation status.

A test being present does not imply that the capability under test is implemented. The mandatory vertical-slice tests include explicit RED gates for architecture that is not yet implemented.

## 1. Test authority layers

| Layer | Test suite | Purpose |
|---|---|---|
| Unit / contract | `SemanticReferenceServiceTests.cs` | Initial reference contract behavior and failure states |
| Component / integration | `SemanticReferenceIntegrationTests.cs` | Cross-service resolution across application, part, assembly and drawing scopes |
| System-CAD acceptance | `SystemCadAcceptanceTests.cs` | Complete currently implemented semantic build path and authority boundaries |
| Red-team | `SystemCadRedTeamTests.cs` | Malformed, ambiguous, forged, cross-scope, identity, determinism and fail-closed attacks |
| Vertical-slice gate | `SystemCadVerticalSliceGateTests.cs` | Mandatory cube -> face -> sketch -> feature -> result/topology -> representation proof |
| Repository E2E | `tests/e2e/run.py` | Executes the .NET suite together with Rust, native, black-box, demo and raw HTTP red-team gates |

The comprehensive E2E harness explicitly requires discovery of all System-CAD suites above. A missing suite is a discovery failure.

## 2. Component / integration coverage

The integration suite covers:

- application-to-part/assembly/drawing resolution;
- part geometry, constraint and component resolution;
- assembly occurrence resolution;
- drawing sheet resolution;
- missing producer versus missing target;
- producer-scope isolation;
- unsupported target kinds;
- exact/case-sensitive identity;
- null and malformed service inputs;
- result-identity checks;
- candidate identity preservation;
- repeated-call determinism.

The suite must never silently widen a producer's semantic scope to find a target elsewhere.

## 3. System-CAD acceptance coverage

The acceptance suite covers the currently implemented path:

```
Part semantic definition
    -> geometry / constraint
    -> Assembly occurrence
    -> occurrence transform/context
    -> Drawing definition / Sheet
    -> semantic reference resolution
    -> compiled semantic projection
```

It also proves:

- an occurrence carries instance state without mutating the source definition;
- source-definition identity and occurrence identity remain distinct;
- equivalent semantic inputs produce identical compiled manifests;
- dangling drawing references are rejected before publication of an application snapshot;
- invalid occurrence definitions fail before compilation;
- ambiguous sheets are never silently selected;
- drawing semantics are not reconstructed from render/topology data.

## 4. Red-team matrix

The red-team suite attacks the current implementation at the exact semantic boundary.

| Attack | Expected behavior |
|---|---|
| Duplicate geometry identity | `Ambiguous`, never first-match |
| Duplicate constraint identity | `Ambiguous` |
| Duplicate component identity | `Ambiguous` |
| Wrong target kind | Never resolve by reinterpretation |
| Producer whitespace/case mutation | Missing, not normalized into a match |
| Target encoding mutation | Missing unless the exact semantic identity exists |
| Unicode identities | Preserve exact semantic identity |
| Stale result identity | `Indeterminate` |
| Unsupported target | `Unsupported`, no usable candidate |
| Malformed identity fields | `Indeterminate` / invalid diagnostic |
| Null public inputs | Fail at service boundary |
| Application/definition identity collision | Build rejected |
| Structural ID delimiter injection | Compiled IDs escape path delimiters |
| Forged reserved metadata | Authoritative occurrence metadata wins |
| Builder mutation after snapshot | Existing application remains isolated |
| Repeated compilation | Stable output |

These tests intentionally do not treat a rendered or serialized artifact as a semantic authority.

## 5. Mandatory vertical-slice gates

The architectural proof is:

```
Create Cube
 -> semantic Face reference
 -> Sketch on Face
 -> two circles / constraints
 -> Feature +FaceNormal
 -> Feature -FaceNormal
 -> recompute
 -> authoritative result
 -> topology / provenance
 -> derived representation
 -> viewer semantic selection
```

The current branch contains explicit RED gates for the missing parts of this proof.

### RED gate A — Face semantics

A face reference must resolve as an authoritative semantic target. The current generic reference implementation intentionally returns `Unsupported` for `face`, so this gate is currently RED.

### RED gate B — Authoritative topology and representation

The cube path must produce authoritative topology/provenance plus a derived representation. The current compiled projection produces neither for the cube fixture, so this gate is currently RED.

### RED gate C — Sketch/feature semantic dependency

The vertical slice must contain a semantic sketch-on-face path and feature dependency path. This gate is intentionally represented as an architectural RED gate, not as a claim that specific future type names already exist.

### Passing control

A separate control test verifies that the present foundation stops at the unsupported face boundary and does not fabricate topology.

## 6. Deliberately untested until their contracts exist

The following architecture requirements are not converted into speculative production APIs merely to manufacture green tests:

- frame mismatch and frame transformation contracts;
- configuration-aware reference resolution;
- explicit publication contracts;
- topology provenance records;
- topology evolution/remapping;
- authoritative evaluation-result identity distinct from semantic build identity;
- evaluation cache identity/invalidation;
- kernel result integration;
- representation-to-semantic selection;
- Sketch/Feature semantics beyond the current builder surface.

These require their own explicit contracts and TDD RED tests before implementation. They must not be inferred from the generic reference resolver.

## 7. External terminology boundary

The reference architecture is consistent with established STEP product-data concepts. ISO/TS 10303-1032:2024 covers identification of shape portions, relationships between shape portions, occurrence-context shape and representation association. ISO/TS 10303-1033:2014 covers externally supplied 3D geometric representations.

These standards inform terminology and interoperability semantics; they do not replace UMLCAD's explicit authority model.

## 8. Acceptance rule

A System-CAD increment is not complete merely because:

- the new unit tests compile;
- a reference service exists;
- a compiled node can be produced;
- a viewer artifact exists.

Completion requires the affected architectural boundary to be demonstrated through the relevant integration, acceptance, red-team and complete vertical-slice gates, with exact execution evidence.

For the current branch, the semantic reference foundation is implemented and heavily tested, while the mandatory cube/face/sketch/feature/result/topology/representation slice remains explicitly RED.


## 9. Current incremental status — publication/provenance foundation

The branch now contains a concrete **semantic publication/provenance layer** for published planar/shape targets:

- `ShapePublicationSemantic` records publication identity, semantic target, declared result identity and topology-binding identity.
- `TopologyBindingSemantic` records the semantic target, result identity and authoritative topology identity supplied by the binding contract.
- Builder validation rejects duplicate publication IDs, duplicate topology-binding IDs, dangling publications, orphan bindings, and publication/binding mismatches.
- Face references resolve only through a matching publication and matching topology binding; a bare `solid` geometry record is never interpreted as a Face.
- Multiple publications of the same face target are ambiguous without a result selector and can be resolved by an exact result identity.
- Compiled semantic projections retain publication identity separately from face target identity.

This remains a **semantic contract boundary**, not yet authoritative mathematical result integration. The result identity and topology identity are not yet produced/verified by the Rust mathematical evaluation path. Therefore the mandatory cube vertical-slice RED gates remain open.
