# UMLCAD.V.7 — MAIN-BRANCH DEVELOPMENT PROMPT

Continue `UMLCAD.V.7` development from the **exact current `main` branch state**.

This document is the **global development law** for system-CAD work. It defines architecture, authority, correctness, testing, determinism, failure semantics, and engineering discipline.

It does **not** define a fixed feature sequence or a comprehensive implementation roadmap. The repository's active documents have distinct authority:

- `docs/SYSTEM_CAD_ARCHITECTURE.md` defines the System-CAD ownership model, C4 structure, canonical evaluation architecture, domain boundaries, and architectural gaps.
- `docs/MATH_AUTHORITY_ROADMAP.md` defines the mathematical authority and certified mathematical domains.
- `docs/CONTINUATION_HANDOFF.md` defines the current implementation/evidence state and immediate declared gap.
- `docs/doc.tex` is the published architecture handbook and must remain consistent with the active architecture document.

These documents are authoritative for **what the system means and what is currently implemented**. This prompt defines **how changes are developed and validated**. No agent should infer architecture from an old milestone sequence in this prompt.

---

## 1. FIRST: ESTABLISH THE EXACT REPOSITORY STATE

Before changing code:

1. Read the complete relevant `docs/` material at the exact `main` head, including architecture, prompts, milestone/gap, testing, handoff/status, system-boundary, capability-mapping, and mathematical-authority documents that govern the requested work.
2. Inspect the complete current project structure relevant to the requested capability.
3. Inspect the exact `main` commit, recent history, relevant branches/PRs, existing implementation, and existing tests.
4. Determine what is:
   - implemented;
   - contract-defined but not fully implemented;
   - partially implemented;
   - tested;
   - E2E-validated;
   - hardware-validated;
   - not validated;
   - unsupported;
   - blocked by infrastructure.
5. Do not infer completion from the existence of a class, interface, adapter, TODO removal, or test alone.
6. Reuse completed work. Do not recreate an existing contract, evaluator, adapter, solver, cache, representation path, or test architecture merely because another location makes it convenient.

The **repository at the exact `main` head is the shared source of truth**. Chat history is not an architectural authority.

---

# 2. DEVELOPMENT LAW — TDD FIRST

Every substantial change follows:

**INSPECT → AUTHORITATIVE RED → IMPLEMENT → GREEN → RED-TEAM → FULL E2E → DOCUMENT EVIDENCE**

For every capability:

### A. Inspect current behavior

Determine precisely what the system currently does, including successful and failure paths.

### B. Define the authoritative acceptance scenario

Define the smallest complete system scenario that proves the intended semantic behavior at the actual affected boundary.

### C. Run RED before completing the implementation

The authoritative test must fail because the required behavior is absent, incorrect, unsupported, or otherwise not yet established.

Record the meaningful failure.

### D. Identify the missing boundary

Classify the missing capability accurately:

- semantic model;
- contract;
- reference resolution;
- frame semantics;
- evaluation planning;
- dependency graph;
- kernel contract;
- kernel transport;
- kernel execution;
- result integration;
- topology provenance/evolution;
- representation derivation;
- persistence;
- cache/invalidation;
- application orchestration;
- engineering domain boundary;
- external provider boundary.

Fix the authoritative boundary rather than a downstream symptom.

### E. Implement the smallest complete architectural path

Implement only the smallest coherent path required by the governing contract and current milestone.

Do not create speculative frameworks for future features.

### F. Run GREEN

Run the targeted test and supporting component tests.

### G. RED-TEAM immediately

Attack the newly implemented contract and implementation.

At minimum, where applicable, test:

- malformed input;
- unsupported input;
- missing reference;
- stale reference;
- ambiguous reference;
- invalid reference provenance;
- wrong result identity;
- frame mismatch;
- invalid orientation;
- non-finite values;
- zero-length and degenerate geometry;
- conflicting constraints;
- tolerance boundaries;
- deterministic identity collision;
- cache contamination;
- stale cache;
- full versus incremental evaluation divergence;
- dependency-ordering errors;
- incorrect invalidation;
- incomplete kernel result;
- unexpected kernel status;
- transport corruption;
- provider failure;
- unsupported dispatch;
- partial/contradictory result state.

The system must **fail closed** wherever correctness cannot be established.

### H. Run complete authoritative E2E

An isolated unit test is not system completion.

Run the relevant complete Python E2E/red-team suites and the repository gates required by the governing milestone.

### I. Record exact evidence

Record:

- exact commit SHA;
- exact commands;
- RED result;
- implementation result;
- GREEN result;
- red-team result;
- E2E result;
- supporting test result;
- infrastructure limitations;
- whether each test actually executed;
- capability status.

Never turn an unexecuted check into PASS.

---

# 3. TEST AUTHORITY HIERARCHY

### Level 1 — Mathematical authority

The existing Rust mathematical layer is authoritative for the mathematical contracts it explicitly certifies.

Do not duplicate mathematical algorithms in .NET merely for convenience.

### Level 2 — Contract/component tests

.NET and component tests validate:

- semantic contracts;
- reference semantics;
- deterministic identity;
- dependency behavior;
- evaluation behavior;
- adapters;
- result mapping;
- failure semantics;
- cache/invalidation;
- provider boundaries.

### Level 3 — Python E2E / red-team

`tests/e2e/` is the system acceptance and adversarial integration layer.

It must exercise real system boundaries and prove complete semantic workflows rather than mock-only behavior.

### Level 4 — Full repository gates

Run every applicable declared repository gate.

A gate that could not execute is **Not validated**, not PASS and not a product failure.

---

# 4. ARCHITECTURAL AUTHORITY

The system is organized conceptually into authority strata:

```
Platform Foundation
Mathematical Authority
Science / Phenomena Services
CAD Core / Product Semantics
Engineering Resources
Specialized Engineering Domains
Application / Workflow / Presentation
```

These are **conceptual ownership strata, not a linear dependency chain**. A domain may consume several authoritative producers through explicit contracts; no rule requires every layer to depend only on the layer immediately above it.

The normative System-CAD structure, C4 Level 1–4 decomposition, canonical evaluation path, domain dependency rules, template/component/sheet model, simulation-adapter boundary, and mandatory early vertical slice are defined in `docs/SYSTEM_CAD_ARCHITECTURE.md`. This section establishes the governing principle; it does not replace that document.

The key responsibility boundary is:

- **Rust mathematics** provides certified mathematical authority.
- **.NET system-CAD** owns engineering meaning and system orchestration.
- **Providers/adapters** implement replaceable external or computational services.
- **Viewer/presentation** consumes derived results and never defines engineering truth.

The .NET system owns, where applicable:

- CAD meaning;
- specifications and design intent;
- Product / Part / Body / Sketch / Feature / Assembly semantics;
- references and publications;
- coordinate-frame semantics;
- configuration/context;
- evaluation planning/orchestration;
- deterministic identities;
- dependency graphs;
- cache/invalidation;
- persistence;
- authoritative result integration;
- topology provenance/evolution semantics;
- Drawing and PMI meaning;
- BOM/product-structure semantics;
- CAM meaning;
- Sheet Metal meaning;
- engineering-resource relationships;
- simulation service contracts;
- lifecycle/collaboration integration.

Do not move these responsibilities into the mathematical kernel merely because the kernel can technically carry more data.

# 5. MATHEMATICAL AUTHORITY RULE

The mathematical kernel and the complete CAD system are **not equivalent architectural layers**.

The Rust kernel is intentionally thin.

Use the existing mathematical authority whenever the required mathematical capability already exists.

Do not:

- implement a second solver in C#;
- duplicate certified geometry algorithms in .NET;
- make a GPU backend a semantic authority;
- silently downgrade precision;
- create approximate geometry to bypass an absent mathematical contract;
- introduce another numerical library without a demonstrated architectural reason and a verification plan.

CPU mathematical behavior remains the normative reference for the mathematical contracts declared by the repository.

Accelerators may improve execution speed, but they must not redefine semantic truth.

Unsupported, ambiguous, singular, degenerate, indeterminate, or non-finite mathematical states must fail closed.

---

# 6. CORE SEMANTIC SEPARATION

The system must preserve the separation:

```text
Specification / Design Intent
            ↓
         Evaluation
            ↓
  Authoritative Semantic Result
            ↓
       Derived Representation
```

### Specification

Stores semantic intent, definitions, references, parameters, constraints, configuration/context, and declared relationships.

### Evaluation

Determines dependencies and executes the semantic computation through the canonical evaluation architecture.

### Authoritative Result

Carries the system's trusted result, identity, status, evidence, topology/reference provenance, and applicable diagnostic information.

### Representation

Is derived from authoritative results for consumers such as viewers, drawings, manufacturing views, or exchange.

A representation is never allowed to become an alternative geometry or semantic authority.

---

# 7. SYSTEM RELATIONSHIP MODEL

The following relationships are architectural invariants.

```text
Science
├── Material / physical properties
└── Phenomena Simulation Service
↑
│
multiple providers
│
Engineering Resources
├── Machine
├── Tool
├── Fixture
├── Process
└── Capabilities
↑
┌───────┴────────┐
│                │
Sheet Metal          CAM
│                │
│                ├── Toolpath
│                ├── Postprocessor
│                └── deterministic G-code / NC
│
└── material/machine/process validation

CAD Product Structure
        ↓
       BOM
        ↓
Drawing / CAM / PLM
```

Interpret these relationships precisely:

### Science

Science owns reusable scientific concepts, including material/physical properties and the abstract **Phenomena Simulation Service**.

### Phenomena Simulation Service

The service is provider-neutral.

It defines the stable semantic service boundary used by engineering domains.

Multiple simulation providers may implement that boundary.

A caller must not depend on a specific simulation implementation when the contract does not require one.

### Engineering Resources

Engineering Resources describe manufacturing and engineering resources and their declared capabilities, including:

- machines;
- tools;
- fixtures;
- processes;
- capabilities.

These are reusable engineering facts and constraints, not CAM-owned private state.

### Sheet Metal

Sheet Metal consumes relevant engineering-resource information and participates in material/machine/process feasibility validation where its semantics require it.

Sheet Metal owns sheet-metal design semantics and must expose authoritative results to downstream consumers.

### CAM

CAM consumes authoritative CAD semantics and may consume:

- product structure/BOM;
- engineering resources;
- material/process information;
- Sheet Metal outputs where applicable;
- Phenomena Simulation Service through its provider-neutral contract.

CAM owns manufacturing-process semantics such as setups, manufacturing features/operations, strategies, toolpaths, and postprocessing.

CAM must ultimately produce **deterministic G-code / NC** through a defined postprocessor boundary.

A visual toolpath is not equivalent to postprocessed machine output.

### CAD Product Structure

CAD Product Structure owns Product/Part/Assembly/Occurrence and related structure semantics.

### BOM

BOM is derived from authoritative product structure.

BOM is not an independent competing source of product truth.

Downstream domains such as Drawing, CAM, and PLM consume the authoritative product/BOM semantics through explicit contracts.

### Drawing

Drawing consumes authoritative CAD semantics and product-structure information.

Drawing must not reconstruct geometry from presentation meshes merely because that is visually convenient.

### PLM

PLM consumes authoritative product/lifecycle information.

It must not become a replacement geometry authority.

---

# 8. DOMAIN OWNERSHIP AND DEPENDENCY RULES

A domain owns its semantic concepts but should consume upstream truth through explicit contracts.

Examples:

- CAM must not invent Product Structure.
- BOM must not become a second assembly authority.
- Drawing must not invent CAD topology.
- Sheet Metal must not implement a hidden independent kernel.
- Simulation providers must not redefine the scientific service contract.
- Engineering Resources must not depend on viewer state.
- Viewer must not determine manufacturing, topology, or physical truth.
- External systems must connect through adapters/providers rather than leaking concrete vendor types throughout the semantic model.

Prefer dependency direction toward **stable contracts and authoritative producers**.

Avoid concrete dependency chains that make domain semantics inseparable from infrastructure adapters.

---

# 9. REFERENCE SEMANTICS

References are semantic objects.

Never make accidental array position an engineering identity.

Do not use constructs equivalent to:

```text
Faces[12]
Edges[7]
```

as permanent semantic references.

A reference must resolve against the expected:

- producer;
- authoritative result identity;
- topology/provenance meaning;
- feature/publication meaning;
- coordinate frame;
- selector constraints.

Required resolution outcomes include, where applicable:

```text
Resolved
Missing
Ambiguous
Indeterminate
Unsupported
```

Do not silently select an arbitrary candidate.

When topology changes, preserve explicit correspondence/evolution evidence sufficient for downstream references.

Visual similarity is not proof of semantic correspondence.

---

# 10. COORDINATE AND FRAME SEMANTICS

Frames are first-class semantic inputs.

Where applicable, distinguish:

- World;
- Document;
- Part;
- Body;
- Sketch;
- Face;
- Occurrence;
- Drawing View;
- Simulation.

Do not derive engineering orientation from viewer camera state.

Direction, placement, transform, occurrence context, and handedness must be explicit.

Frame mismatch is a correctness failure, not a presentation issue.

---

# 11. DETERMINISM

Every semantic evaluation must be reproducible.

Determinism applies to:

- specification identity;
- evaluation identity;
- dependency ordering;
- reference resolution;
- result identity;
- topology/provenance output;
- kernel request construction;
- result mapping;
- cache keys;
- invalidation;
- observable diagnostic ordering;
- CAM toolpath and postprocessor output where declared deterministic;
- generated G-code/NC where deterministic output is a contract.

Every semantic input affecting an authoritative result must participate in the relevant identity.

Do not use:

- object addresses;
- process-local hashes;
- execution timestamps;
- viewer state;
- uncontrolled random values;

as semantic identity components.

Equivalent semantic inputs must produce equivalent authoritative outputs and identities.

---

# 12. CACHE AND INCREMENTAL EVALUATION

Cache behavior is a correctness concern.

The architecture must preserve:

```text
FreshEvaluation(EvaluationIdentity)
    ==
CacheHit(EvaluationIdentity)
```

Full and incremental evaluation must be semantically equivalent for the same resulting model state:

```text
AuthoritativeResult(FullRecompute)
    ==
AuthoritativeResult(IncrementalRecompute)
```

Incremental evaluation must use explicit dependency/change closure.

Invalidation must not be guessed from incidental execution order.

An unrelated change must not silently invalidate or regenerate unrelated authoritative results merely because the implementation is convenient.

Representation regeneration must be tied to authoritative semantic changes, not viewer refresh behavior.

---

# 13. FAILURE-CLOSED LAW

Whenever correctness cannot be proven, return explicit non-success state.

Examples include:

```text
Unsupported
Ambiguous
Indeterminate
Invalid
Failed
```

Never:

- guess;
- silently substitute;
- silently repair semantic state;
- select an arbitrary reference;
- convert a failed kernel result into plausible presentation geometry;
- approximate unsupported authoritative geometry;
- swallow contradictory provider results;
- downgrade a failure into success merely to keep a workflow moving.

A successful result means the system established the required authority.

---

# 14. PROVIDER AND ADAPTER LAW

External or replaceable implementations must sit behind stable contracts.

This applies to:

- mathematical execution adapters;
- simulation providers;
- manufacturing/resource integrations;
- machine/controller adapters;
- PLM providers;
- persistence providers;
- external calculation services.

Domain/application semantics should depend on the contract, not on vendor-specific implementation details.

A provider may be unavailable, unsupported, stale, or failed.

Those conditions must remain explicit.

Do not create a provider-specific semantic fork.

---

# 15. SIMULATION BOUNDARY

Phenomena simulation is a reusable scientific service.

The conceptual boundary is:

```text
Engineering Consumer
        ↓
IPhenomenaSimulationService
        ↓
IPhenomenaSimulationProvider
        ↓
Simulation Implementation
```

The caller asks for a scientific phenomenon/result contract.

The caller does not decide whether the provider is:

- internal;
- external;
- CPU;
- GPU;
- specialized;
- remote.

Provider selection is an implementation/service concern unless the semantic contract explicitly exposes provider identity as an input.

Simulation results must include sufficient identity, assumptions, inputs, and provenance for the consuming engineering domain to determine whether the result is valid for its purpose.

---

# 16. MANUFACTURING BOUNDARY

Manufacturing domains consume authoritative CAD/product semantics and engineering-resource facts.

CAM may transform engineering intent into:

```text
Manufacturing Definition
        ↓
Toolpath
        ↓
Postprocessor
        ↓
Deterministic G-code / NC
```

The postprocessor is a semantic boundary, not a string-formatting convenience.

Where deterministic output is required:

- machine configuration;
- tool;
- process;
- fixture;
- coordinate system;
- operation order;
- postprocessor version/configuration;
- relevant tolerances and manufacturing inputs

must participate in the required identity.

No untracked hidden state may influence deterministic output.

---

# 17. CAD PRODUCT STRUCTURE AND BOM

Product structure is authoritative for product composition.

The conceptual flow is:

```text
CAD Product Structure
        ↓
       BOM
        ↓
Drawing / CAM / PLM
```

BOM is a derived semantic view of product structure.

Do not create an independent BOM tree that can contradict the authoritative Product/Assembly structure without an explicit synchronization/authority contract.

Occurrences, quantities, variants/configurations, and product identities must remain semantically traceable.

Downstream consumers must be able to identify which authoritative product state produced a BOM view.

---

# 18. CATIA CAPABILITY COMPARISON

CATIA is a **functional capability reference**.

Compare at the level of engineering behavior and system capability, not internal implementation.

Do not reproduce CATIA's internal kernel architecture.

Trace a capability through:

```text
Capability
  ↓
UMLCAD semantic contract
  ↓
Service/domain implementation
  ↓
Mathematical contract when required
  ↓
Authoritative result
  ↓
Derived representation
  ↓
Acceptance evidence
```

Capability mapping documents in the repository determine which capability is relevant to the current work.

Do not use this prompt to invent a separate CATIA roadmap.

---

# 19. DO NOT REINTRODUCE REJECTED ARCHITECTURE

Before reviving an older implementation pattern, inspect the current repository history and governing documents.

Do not reintroduce:

- a legacy parallel evaluation architecture;
- duplicate solvers;
- hidden geometry approximations;
- viewer-owned engineering semantics;
- accidental topology-index identity;
- provider-specific semantics in the core;
- obsolete package paths;
- demo code as an unintended canonical host;
- concrete infrastructure dependencies where the architecture requires a contract boundary.

Reverted code is evidence that the previous architecture was unsuitable unless current documentation explicitly reauthorizes it.

---

# 20. CODING DISCIPLINE

Prefer:

- small coherent changes;
- typed contracts;
- explicit state;
- deterministic algorithms;
- immutable semantic inputs;
- explicit dependency edges;
- explicit provenance;
- explicit failure states;
- reuse of existing mathematical authority;
- narrow interfaces;
- focused tests.

Avoid:

- speculative abstractions;
- generic dictionaries as permanent semantic APIs;
- hidden global state;
- magic identifiers;
- accidental coupling;
- duplicated business meaning;
- broad refactors unrelated to the current task;
- test bypasses;
- silent fallbacks;
- approximation used as authoritative output.

Code should make illegal or ambiguous states difficult to represent.

---

# 21. EXACT EXECUTION PROCEDURE FOR EVERY NEXT CHANGE

For each capability:

```text
1. Read the governing repository documents.
2. Inspect exact current implementation.
3. Inspect existing tests.
4. Determine the exact missing boundary.
5. Define the authoritative system acceptance scenario.
6. Run RED before completing the implementation.
7. Implement the smallest complete architectural path.
8. Run targeted GREEN.
9. Run supporting unit/contract/integration tests.
10. Add adversarial red-team cases.
11. Run red-team GREEN.
12. Run the relevant complete Python E2E.
13. Run all applicable repository gates.
14. Check deterministic identity and output.
15. Check full versus incremental equivalence where applicable.
16. Check failure-closed behavior.
17. Update the governing repository documentation with exact evidence.
18. Commit one coherent increment.
```

Never skip the RED state for substantial new behavior.

Never declare a capability complete before the required E2E evidence exists.

Never declare a milestone GREEN before the applicable repository gates actually execute and pass.

---

# 22. INFRASTRUCTURE VALIDATION

Distinguish exactly between:

```text
test executed and failed
test executed and passed
test not executed
workflow infrastructure failure before test execution
hardware unavailable
```

When infrastructure prevents execution, use:

```text
Not validated — infrastructure execution unavailable
```

Do not describe such a result as either product PASS or product FAIL.

Hardware validation must only be claimed when the required hardware test actually ran.

---

# 23. DEFINITION OF DONE

A capability is complete only when every applicable gate is satisfied:

```text
Semantic contract
AND
Correct layer/domain ownership
AND
Implementation
AND
Required mathematical contract
AND
Reference semantics
AND
Frame/configuration semantics
AND
Deterministic identity
AND
Cache/invalidation
AND
Dependency/update behavior
AND
Authoritative result
AND
Provenance/topology evidence
AND
Derived representation where required
AND
Unit/component tests
AND
Red-team tests
AND
Metamorphic/property/differential tests where applicable
AND
Python E2E acceptance
AND
Full applicable repository validation
AND
Exact evidence documentation
```

Use explicit statuses:

```text
Implemented
Tested
E2E-validated
Hardware-validated
Not yet validated
Unsupported
Blocked by infrastructure
```

Do not use percentages as a substitute for acceptance gates.

---

# 24. DOCUMENTATION LAW

Documentation is part of the implementation.

Whenever architecture or behavior changes:

1. update the governing contract/document;
2. state ownership and dependency direction;
3. state authoritative inputs/outputs;
4. state failure semantics;
5. state determinism requirements;
6. record exact tests and commit evidence.

Do not maintain a second contradictory architecture in prose.

Historical documentation may remain as history, but active instructions must point to the current architecture.

---

# 25. MAIN-BRANCH CONTINUATION RULE

At the beginning of every new development iteration:

```text
Exact main HEAD
    ↓
Active System-CAD architecture
    ↓
Current implementation
    ↓
Current tests / evidence
    ↓
Current declared gap
    ↓
TDD RED
    ↓
Implementation
    ↓
Validation
```

Use the documents according to their authority:

- `SYSTEM_CAD_ARCHITECTURE.md` determines the intended architecture and permitted dependency direction.
- `MATH_AUTHORITY_ROADMAP.md` determines the mathematical authority available to that architecture.
- `CONTINUATION_HANDOFF.md` and any active gap/capability records determine the current implementation state and next declared work.
- This prompt determines the development/validation procedure.

The prompt deliberately does not embed a fixed feature roadmap.

When the repository changes its milestone structure, the prompt should remain valid unless an architectural invariant itself changes.

# 26. FINAL DEVELOPMENT PRINCIPLE

**Build the system around authoritative meaning, not around feature count.**

**Test the real system first. Implement to make the authoritative test green. Attack the boundary with red-team tests. Prove the complete E2E path.**

**The Rust kernel supplies mathematical authority where contracted. The .NET system supplies CAD and engineering meaning. Science supplies reusable scientific services. Engineering Resources supply reusable manufacturing/resource facts. Specialized domains consume those contracts. Product Structure remains authoritative for composition. BOM is derived. CAM produces deterministic G-code/NC. Simulation is provider-neutral. Presentation remains downstream.**

**The repository's current documentation and exact `main` state determine what comes next.**
