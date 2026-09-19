# UMLCAD.V.7 — Application Layer Development Prompt

## 0. Purpose and authority

This document is the governing development prompt for all production .NET Application Layer implementation work in UMLCAD.V.7.

It applies to every new feature, refactor, library, service, semantic contract, engineering domain, kernel-facing integration, application workflow, and supporting implementation under:

~~~
dotnet/src/
~~~

It is subordinate to:

1. docs/SYSTEM_CAD_ARCHITECTURE.md for architectural authority and ownership;
2. docs/MATH_AUTHORITY_ROADMAP.md for mathematical authority and certified mathematical scope;
3. the repository's current main state for actual implementation/evidence;
4. this document for the mandatory Application Layer development and validation procedure.

This document does not replace the repository's general development prompt. It specializes that procedure for the .NET Application Layer.

The objective is not merely to produce working code. The objective is to preserve architectural integrity, semantic authority, deterministic behavior, maintainability, extensibility, testability, and long-term correctness while the Application Layer grows into a complete System-CAD platform.

---

# 1. Absolute architectural model

All production .NET libraries under:

~~~
dotnet/src/
~~~

are part of the UMLCAD Application Layer.

They may be separated into multiple assemblies for ownership, cohesion, compilation, and lifecycle reasons, but they remain one architectural layer.

The mathematical kernel is outside this layer:

~~~
kernel/
~~~

The Application Layer owns engineering meaning.

The mathematical kernel owns only the mathematical authority explicitly contracted to it.

The target boundary is:

~~~
.NET Application Layer
        |
        v
Single concrete UMLCAD Kernel API
        |
        v
Kernel implementation
(Rust/native/accelerators/internal transport)
~~~

The Application Layer must never become coupled to the kernel's implementation language, process model, transport protocol, native library, or accelerator technology.

---

# 2. Kernel access law

There shall be exactly one dedicated production .NET kernel-access library.

The current transitional implementation is:

~~~
UMLCAD.Kernel.Client
~~~

The long-term architectural concept is:

~~~
UMLCAD.Kernel
~~~

The implementation name may change, but the architectural invariant does not.

Only the dedicated gateway may know implementation-level facts such as:

~~~
Rust
HTTP transport
kernel_host
native process launch
native library loading
transport endpoint/path
Rust-specific DTOs
Rust crate concepts
Metal/CUDA execution details
kernel process lifecycle
~~~

No other Application Layer library may contain those details.

All other Application Layer libraries must communicate with the kernel only through the stable UMLCAD Kernel API.

Forbidden:

~~~
CAM -> kernel_host
Drawing -> HTTP kernel endpoint
SheetMetal -> Rust type
Science -> native kernel process
Engineering Rule -> Rust implementation
Framework -> kernel transport internals
~~~

Required:

~~~
Application domain
    ->
stable UMLCAD Kernel API
    ->
kernel gateway
    ->
kernel implementation
~~~

A developer must never copy the current kernel transport implementation into another library merely because it is convenient.

---

# 3. Dependency architecture is an executable law

The Application Layer is allowed to contain multiple assemblies, but its dependency graph must remain a directed acyclic graph.

For every production .csproj, ProjectReference entries must resolve to allowed Application Layer dependencies.

The following are mandatory:

~~~
production ProjectReference graph = acyclic
production assembly dependency graph = acyclic
kernel-access gateway count = exactly one
kernel implementation leakage outside gateway = zero
~~~

An architecture diagram or documentation statement is not sufficient evidence.

The repository must mechanically test the actual project graph and, when implemented, the compiled assembly graph.

The authoritative architecture gate is:

~~~
python3 app_e2e/run.py
~~~

This command is mandatory.

A feature is not accepted when the architecture gate has not executed successfully against the exact implementation head being accepted.

---

# 4. Mandatory app_e2e gate

app_e2e is a first-class architecture and application-system validation project.

Every Application Layer task must execute:

~~~
python3 app_e2e/run.py
~~~

at minimum:

1. before substantial implementation begins, to establish a clean architectural baseline;
2. after the TDD RED state is created;
3. after implementation reaches targeted GREEN;
4. after red-team tests are added;
5. before the task is declared complete;
6. on the exact commit intended for merge.

The comprehensive E2E harness must also execute the Application Layer architecture gate.

CI must execute the same gate.

The local result and CI result must not be treated as interchangeable claims unless both actually executed against the same intended commit.

A test that did not execute is:

~~~
NOT VALIDATED
~~~

not PASS.

---

# 5. app_e2e responsibilities

The app_e2e architecture suite shall grow with the Application Layer and must verify the repository rather than trusting declarations.

At minimum it shall enforce:

## 5.1 Project placement

Every production .NET project is under:

~~~
dotnet/src/
~~~

## 5.2 Dependency resolution

Every production ProjectReference:

- resolves to an existing production project;
- points only to an allowed Application Layer project;
- does not escape the declared Application Layer;
- does not reference test projects from production code;
- does not reference generated/local accidental projects.

## 5.3 Project graph

The complete graph is acyclic.

The test must report the complete detected cycle path, not merely "cycle detected".

## 5.4 Assembly graph

Where practical, inspect compiled assemblies and verify the actual assembly-reference graph against the allowed dependency graph.

A clean .csproj graph is not sufficient if the compiled product violates the architecture.

## 5.5 Single kernel gateway

Exactly one production assembly is designated as the kernel gateway.

## 5.6 Kernel implementation leakage

The suite rejects implementation-specific kernel knowledge outside the gateway.

The list of forbidden markers must evolve as the kernel implementation evolves.

## 5.7 Native/process/transport access

No non-gateway Application Layer library may:

- start the kernel process;
- load kernel native binaries;
- directly issue the kernel transport;
- embed the kernel host endpoint;
- reproduce kernel serialization/transport logic;
- import Rust/native types;
- bypass the kernel gateway.

## 5.8 Test dependency law

Production code must not reference test code or test-only helpers.

## 5.9 Architecture configuration law

Architectural allow-lists, gateway identity, and boundary rules must be versioned with the repository.

Do not hide architecture policy in a developer's local machine.

## 5.10 Deterministic diagnostics

Architecture failures must report:

- violating file/project/assembly;
- dependency edge or forbidden access;
- rule identifier;
- expected architecture;
- actual architecture.

---

# 6. TDD is mandatory

Every substantial Application Layer capability uses strict Red-Green-Refactor discipline.

Required sequence:

~~~
Understand
    ->
Define contract
    ->
Write failing authoritative test
    ->
RED
    ->
Minimal implementation
    ->
GREEN
    ->
Refactor
    ->
Adversarial tests
    ->
Integration/E2E
    ->
Exact evidence
~~~

Never write the full implementation first and then invent tests to match it.

Never weaken an authoritative test merely to get GREEN.

Never delete an inconvenient test because the implementation is difficult.

When a test is incorrect, change the contract/test only with explicit evidence and update the governing documentation.

---

# 7. TDD RED must test real behavior

The first meaningful test must be at the smallest authoritative boundary that proves the intended behavior.

Do not use a mock-only test as the sole RED test for an architectural capability.

Where the feature crosses semantic boundaries, create tests that prove the real interaction.

Examples:

~~~
CAD semantic change
    -> evaluation
    -> kernel API
    -> authoritative result

Rule
    -> engineering context
    -> simulation
    -> CAD control
    -> recompute
    -> build outcome

CAM
    -> CAD/product semantics
    -> manufacturing resources
    -> simulation
    -> toolpath
    -> NC/G-code
~~~

Tests must establish that the actual production path works.

---

# 8. Required testing pyramid

Every Application Layer capability must select and execute the applicable levels.

## 8.1 Unit tests

Test:

- pure functions;
- value semantics;
- state transitions;
- identity/canonicalization;
- deterministic algorithms;
- validation;
- failure states;
- edge conditions.

Unit tests must remain fast and focused.

## 8.2 Contract tests

Test stable boundaries between assemblies/services.

Examples:

- semantic contract;
- kernel API contract;
- simulation contract;
- resource contract;
- serialization/version contract;
- CAD command contract.

The tests must detect contract drift.

## 8.3 Component tests

Exercise a real library/component with its required collaborators.

Avoid replacing the entire component under test with mocks.

## 8.4 Integration tests

Use real production components at architectural boundaries.

Examples:

~~~
CAD Engine + Kernel Gateway
CAM + CAD Semantics + Resources
Rule Runtime + CAD Control + Simulation
Drawing + Product Structure + CAD result
~~~

Integration tests must prove real dependency direction and data/provenance propagation.

## 8.5 System / E2E tests

Exercise complete application workflows through the actual executable path.

Use the repository's Python E2E infrastructure.

No substantial System-CAD capability is complete without the applicable E2E proof.

---

# 9. Red-team testing is mandatory

After GREEN, actively attack the implementation.

The purpose of red-team tests is to discover how the architecture can be bypassed or how semantic correctness can fail.

Red-team coverage must include, as applicable:

~~~
invalid inputs
null/missing values
empty collections
duplicate identities
ambiguous references
stale references
wrong frame
wrong configuration
wrong revision
stale cache
invalid cache identity
cache contamination
contradictory state
non-finite numeric input
overflow/underflow
extreme scale
degenerate geometry
unsupported operation
provider unavailable
provider contradiction
transport failure
timeout
cancellation
partial result
malformed result
wrong result identity
wrong build identity
rule exception
rollback failure
partial semantic mutation
constraint cycle
dependency cycle
recursive rule
oscillating rule
recompute storm
concurrent mutation
race condition
duplicate command
replay
out-of-order message
version mismatch
serialization mismatch
unauthorized kernel access
direct native access
transport bypass
test-only dependency leakage
~~~

Red-team tests are not optional cleanup.

They are part of the implementation.

---

# 10. Property, metamorphic, differential and invariant testing

Use stronger forms of verification where the capability permits them.

### Property testing

Verify general laws instead of only examples.

Examples:

~~~
canonicalization is idempotent
same semantic input -> same identity
apply + inverse where contractually defined -> original state
cache hit == fresh evaluation
full evaluation == equivalent incremental evaluation
~~~

### Metamorphic testing

Verify behavior under controlled transformations.

Examples:

~~~
uniform model translation
uniform scale where mathematically invariant
independent feature reordering
serialization/deserialization round trip
equivalent expression representations
~~~

### Differential testing

Where an independent implementation/reference exists, compare results.

The reference must not merely share the same implementation defect.

### Invariant testing

Continuously assert architectural invariants:

~~~
references remain resolvable or explicitly invalid
identities remain stable
provenance is preserved
authoritative result status is truthful
failed transactions do not leave semantic mutations
~~~

---

# 11. Integration test law for every dependency edge

Every new dependency edge must have evidence for why it exists.

For:

~~~
A -> B
~~~

the implementation must establish:

- what contract A consumes from B;
- why A owns no duplicate concept;
- why the dependency direction is correct;
- why the dependency cannot be inverted or removed;
- how the edge is tested.

When an edge introduces a cycle, the implementation is incomplete regardless of feature correctness.

Do not solve cycles by adding arbitrary abstractions merely to satisfy the compiler.

First re-evaluate ownership.

---

# 12. Highest coding standards

Production Application Layer code must meet the following standards.

## 12.1 Compiler strictness

Projects must use:

~~~
Nullable = enable
TreatWarningsAsErrors = true
~~~

No warning is ignored merely to obtain a green build.

Compiler suppressions require:

- exact scope;
- explicit justification;
- proof that the warning is not hiding a correctness defect.

Global warning suppression is prohibited unless explicitly justified and documented.

## 12.2 Strong typing

Prefer explicit domain types over primitive strings/numbers when semantics matter.

Examples:

~~~
DocumentId
FeatureId
ResultIdentity
RevisionId
FrameId
SimulationIdentity
RuleIdentity
ToleranceId
~~~

Do not use stringly-typed dictionaries as permanent semantic APIs.

## 12.3 Nullability

Nullability must describe real semantics.

Do not use null as an unspecified error state when an explicit result status is required.

## 12.4 Immutability

Semantic specifications, identities, authoritative inputs, and evidence should be immutable unless mutation is explicitly part of the contract.

Mutable internal state must have an explicit ownership boundary.

## 12.5 Explicit result states

Use explicit states such as:

~~~
Success
Invalid
Unsupported
Ambiguous
Indeterminate
Failed
Cancelled
Unavailable
~~~

Do not convert failure into an apparently valid default value.

## 12.6 Exceptions

Exceptions are appropriate for exceptional control flow and programmer-contract violations, but they must not silently replace semantic result states.

Cross-boundary failures must be translated into explicit contract-level outcomes.

Engineer-written rules may implement their own exception handling.

The framework must still provide transactional rollback for uncaught failures.

## 12.7 Async discipline

Use asynchronous APIs where the operation is genuinely asynchronous.

Do not introduce:

- async void outside event-handler boundaries;
- blocking waits on asynchronous operations;
- hidden thread creation;
- uncontrolled fire-and-forget tasks.

Cancellation must propagate through all long-running boundaries.

## 12.8 Determinism

Do not use as semantic identity:

~~~
object address
process-local hash
wall-clock timestamp
random GUID without semantic purpose
thread scheduling
iteration order of unordered collections
viewer state
machine-local environment state
~~~

Use deterministic ordering and canonicalization wherever results are authoritative.

## 12.9 Resource ownership

Every externally owned resource must have explicit ownership/lifetime semantics.

Examples:

~~~
HttpClient
stream
file
database connection
native handle
temporary file
simulation process
kernel connection
~~~

Avoid hidden global ownership.

## 12.10 Dependency injection

Use dependency injection for replaceable application services.

Do not turn every class into a service merely to use dependency injection.

Prefer explicit constructor dependencies over service-locator lookups.

Avoid hidden dependency acquisition.

## 12.11 API surface

Public APIs must be narrow and intentional.

Do not expose mutable internal collections.

Do not expose implementation classes when the consumer needs only a contract.

Do not create speculative abstraction layers without a demonstrated architectural purpose.

## 12.12 Documentation

Public engineering contracts must document:

- purpose;
- ownership;
- inputs;
- outputs;
- invariants;
- failure states;
- identity;
- determinism;
- lifecycle;
- thread/concurrency assumptions;
- provenance where relevant.

Code comments explain why, not merely what the syntax does.

## 12.13 Naming

Names must communicate semantic ownership.

Avoid generic names such as:

~~~
Manager
Helper
Util
Data
Service
Processor
Handler
Context
~~~

unless the complete name and contract give a precise meaning.

## 12.14 No dead code

Do not retain unreachable or abandoned implementation paths.

Compatibility paths must have explicit lifecycle status.

## 12.15 No duplicate authority

Never introduce a second implementation of an authoritative engineering concept merely because using the existing one is inconvenient.

---

# 13. Layering and ownership law

Each library must have one primary responsibility.

A library may consume several upstream contracts, but it must not silently become the owner of another domain.

Examples:

~~~
CAM
    owns manufacturing-process semantics
    does not own Product Structure

Drawing
    owns drawing/PMI semantics
    does not own CAD topology

Science
    owns scientific/material semantics
    does not own CAD feature meaning

Engineering Resources
    owns machines/tools/process facts
    does not own CAM operations

Kernel gateway
    owns kernel access mechanics
    does not own engineering meaning
~~~

When a required concept does not have a clear owner:

1. stop implementation;
2. define ownership;
3. define dependency direction;
4. write the RED test;
5. only then implement.

---

# 14. No direct semantic mutation

Engineering code must not mutate raw/private CAD state.

The required path is:

~~~
Engineer / rule / supervision / automation
        ->
semantic CAD command
        ->
transaction/change set
        ->
dependency closure
        ->
evaluation
        ->
UMLCAD Kernel API
        ->
authoritative result
~~~

Forbidden:

~~~
editing private feature dictionaries
changing cached topology directly
editing evaluator internals
writing kernel memory
mutating result objects behind the transaction system
bypassing dependency invalidation
bypassing provenance
~~~

The same semantic mutation path should ultimately serve:

~~~
human UI
automation
engineering rules
engineering supervision
~~~

This prevents multiple competing CAD implementations.

---

# 15. Transaction and rollback law

Any Application Layer operation that can modify semantic state must have explicit transaction semantics.

Required properties:

~~~
atomicity
rollback
provenance
dependency tracking
deterministic identity
exception safety
cancellation safety
nested-operation semantics
~~~

An uncaught exception must not leave partial semantic state.

Red-team tests must deliberately fail after:

~~~
1 mutation
3 mutations
nested mutation
simulation call
recompute
constraint creation
reference creation
~~~

and verify that the semantic state is exactly restored where rollback is the declared behavior.

---

# 16. Cache law

A cache is correct only when the result corresponds to the exact semantic identity required by its consumer.

For any authoritative computation:

~~~
Fresh(EvaluationIdentity)
==
CacheHit(EvaluationIdentity)
~~~

A cache key must include all semantically relevant inputs.

A stale, incomplete, invalid, provenance-incompatible or version-incompatible result is not a cache hit.

Every new cache must have tests for:

~~~
hit
miss
stale
invalidation
collision resistance at the semantic level
cross-revision isolation
cross-configuration isolation
wrong-provider/model version
corrupted result
concurrent access
deterministic retrieval
~~~

---

# 17. Kernel API integration law

Every kernel call must pass through the dedicated gateway.

The gateway is responsible for:

- request validation;
- contract/version checks;
- cancellation;
- transport;
- process/native boundary;
- result deserialization;
- result identity verification;
- result provenance;
- failure translation;
- response-size/resource limits where required.

Application domains must not repeat these concerns.

The Application Layer must treat the kernel as one stable concrete API.

Do not create a "Rust provider abstraction" in every domain library.

Kernel implementation replacement must be possible without changing CAD/CAM/Science domain code.

---

# 18. Simulation integration law

Engineering Application Layer code accesses simulation through the stable phenomena-simulation contract.

It must not depend directly on a specific simulation application.

Rules and engineering services may:

- request a simulation;
- inspect cached results;
- require a fresh run where policy permits;
- consume spatial regions/fields/evidence;
- reject an invalid or insufficient result.

Simulation-provider implementation remains behind the simulation boundary.

---

# 19. Engineering programmability law

Engineer-written rules are executable services.

The framework must permit ordinary programming-language logic:

~~~
if
for
while
try/catch
calculations
helper methods
service calls
simulation calls
CAD semantic commands
~~~

Do not reduce the engineering system to a fixed catalog of hardcoded conditions.

At the same time, engineering programs must never bypass semantic authority.

The framework supplies:

~~~
typed EngineeringContext
CAD Control API
simulation service
transactions
dependency tracking
identity
provenance
diagnostics
caching
termination controls
~~~

The engineer supplies the domain logic.

---

# 20. Rule execution law

Normal engineering rules may:

- inspect engineering knowledge;
- calculate;
- invoke simulation;
- reuse simulation cache;
- issue semantic CAD changes;
- create constraints;
- request recomputation;
- reject builds;
- produce structured diagnostics.

Mandatory rule failure must be a real build failure.

A geometrically valid design is not automatically an engineering-valid design.

Rule identity and implementation/version must be tracked.

---

# 21. Engineering Supervision law

Engineering Supervision is distinct from ordinary build validation.

It may:

- create isolated revisions;
- construct CAD programmatically;
- modify CAD through the same semantic command API;
- invoke ordinary engineering rules;
- invoke/reuse simulations;
- inspect fields/regions/tolerances/results;
- assert expected engineering behavior;
- compare revisions;
- retain or discard a revision.

Engineering Supervision does not create a second CAD implementation.

Its actions must converge on the same semantic CAD control path as human UI, automation and normal engineering rules.

---

# 22. Concurrency and parallelism

Concurrency must be explicit.

For each concurrent service define:

~~~
thread-safety model
ownership model
ordering requirements
cancellation semantics
failure propagation
reentrancy
idempotency
~~~

Never make a class thread-safe merely by adding locks around an unclear design.

Prefer immutable semantic state and controlled state transitions.

Red-team concurrent tests must cover:

~~~
parallel reads
parallel builds
concurrent cache access
simultaneous rule execution
duplicate requests
cancellation during commit
cancellation during kernel call
~~~

---

# 23. Security and trust

The Application Layer may execute engineer-provided code and may grant CAD mutation authority.

Therefore implementation must account for:

- trusted/untrusted code;
- package origin;
- version;
- capabilities;
- resource limits;
- auditability;
- isolated revision execution where needed;
- protection from unauthorized kernel access;
- protection from filesystem/network side effects where policy requires it.

A test that proves "the code works" is not a security proof.

Security red-team tests are required for every executable-code extension mechanism.

---

# 24. Performance law

Do not optimize before correctness.

After correctness is established, measure.

Any performance-sensitive capability must identify:

~~~
workload
baseline
measurement method
hardware/environment
warm/cold state
allocation behavior where relevant
latency
throughput
scaling
~~~

Do not use a microbenchmark result as proof of architectural performance.

Performance optimizations must preserve semantic equivalence.

---

# 25. Compatibility law

Any compatibility change must explicitly identify:

~~~
old contract
new contract
affected consumers
migration path
identity/version impact
test coverage
~~~

Do not silently change serialized semantic meaning.

Do not keep obsolete paths forever without an explicit compatibility policy.

---

# 26. Test fixture law

Fixtures must be:

- deterministic;
- minimal;
- semantically meaningful;
- independently understandable;
- reusable when they represent a canonical scenario.

Do not let a fixture hide the behavior under test.

For cross-library scenarios, prefer one canonical fixture consumed independently by:

~~~
unit/contract reference
integration test
E2E
red-team
~~~

where this improves comparability.

---

# 27. Forbidden shortcuts

Never:

- skip TDD because the change is "small" when it changes behavior;
- skip app_e2e;
- claim tests passed when they did not execute;
- convert infrastructure failures into product PASS;
- disable tests without explicit documented reason;
- mark ignored tests as success in authoritative gates;
- catch and swallow exceptions without semantic justification;
- weaken validation to accommodate an implementation;
- copy kernel transport into another domain;
- introduce direct native/Rust references outside the gateway;
- create circular dependencies;
- use a second mathematical authority in .NET;
- use viewer geometry as semantic truth;
- bypass transactions;
- bypass dependency invalidation;
- silently accept ambiguous references;
- silently substitute unsupported behavior;
- use random or environment-dependent semantic identity;
- commit generated test artifacts as source;
- hardcode engineering logic into the wrong domain merely to make one scenario pass.

---

# 28. Required development procedure for every task

For every Application Layer implementation task execute exactly this discipline.

## Phase A — Establish the baseline

1. Read:
   - README.md;
   - docs/SYSTEM_CAD_ARCHITECTURE.md;
   - docs/UMLCAD_V7_MAIN_BRANCH_DEVELOPMENT_PROMPT.md;
   - relevant docs/fw/ documents;
   - current tests;
   - exact current main state.
2. Run:
   ~~~
   python3 app_e2e/run.py
   ~~~
3. Record whether the baseline is PASS, FAIL, or blocked by infrastructure.
4. Do not misclassify a baseline failure as caused by the new work without evidence.

## Phase B — Define the boundary

5. Identify the owning Application Layer library.
6. Identify every consumer and provider.
7. Draw the dependency edges affected by the task.
8. Verify no circular dependency is required.
9. Verify kernel access remains through the single gateway.
10. Define the authoritative semantic contract.
11. Define failure and unsupported states.
12. Define deterministic identity.
13. Define cache/invalidation requirements.
14. Define transaction/concurrency requirements.
15. Define acceptance evidence.

## Phase C — Write authoritative RED

16. Write the smallest test that proves the missing capability.
17. Run it.
18. Confirm it fails for the intended reason.
19. Record the RED state.

If the test passes before implementation, either:

- the capability already exists; or
- the test does not actually prove the intended missing behavior.

Investigate instead of pretending TDD occurred.

## Phase D — Implement

20. Make the smallest complete production change.
21. Preserve ownership boundaries.
22. Do not introduce speculative abstractions.
23. Do not duplicate mathematical/semantic authority.
24. Do not bypass existing contracts merely to move faster.
25. Keep APIs narrow.
26. Keep semantic state explicit and deterministic.

## Phase E — Targeted GREEN

27. Run targeted unit/contract/component tests.
28. Run the relevant integration tests.
29. Confirm the RED becomes GREEN.
30. Inspect diagnostics/output, not only exit code.

## Phase F — Architecture gate

31. Run:

~~~
python3 app_e2e/run.py
~~~

32. A failure is a development failure until explained and corrected.
33. Do not postpone architecture failures until final integration.

## Phase G — Red team

34. Add adversarial tests against the new boundary.
35. Include misuse and bypass attempts.
36. Test exceptions and rollback.
37. Test invalid identities and stale state.
38. Test concurrency where applicable.
39. Test boundary leakage.
40. Re-run the complete relevant red-team set.

## Phase H — Complete system validation

41. Run the complete applicable Python E2E.
42. Run the relevant .NET unit/component/integration suites.
43. Run Rust/kernel tests when the path crosses kernel authority.
44. Run black-box tests when transport/process behavior is involved.
45. Run demo/application scenarios when affected.
46. Run release-path tests where the repository contract requires them.
47. Ensure ignored/skipped tests are handled according to repository policy.

## Phase I — Determinism and equivalence

48. Repeat deterministic scenarios where meaningful.
49. Compare fresh versus cached evaluation where applicable.
50. Compare full versus incremental evaluation where applicable.
51. Verify stable result/diagnostic ordering.
52. Verify provenance and identity stability.
53. Verify no hidden environment input changes the result.

## Phase J — Documentation and evidence

54. Update the governing document when architecture/behavior changes.
55. Record exact tests executed.
56. Record exact commit/head.
57. Record infrastructure limitations.
58. Record known unsupported cases.
59. Never claim validation that did not execute.

## Phase K — Final acceptance

60. The capability is complete only when all applicable gates are GREEN.

The minimum acceptance equation is:

~~~
correct contract
AND
correct ownership
AND
acyclic architecture
AND
kernel boundary compliance
AND
implementation
AND
TDD GREEN
AND
integration tests
AND
red-team tests
AND
app_e2e PASS
AND
applicable Python E2E PASS
AND
deterministic behavior
AND
failure-closed behavior
AND
documentation/evidence
~~~

---

# 29. Definition of test status

Use these terms precisely.

### PASS

The test actually executed and passed.

### FAIL

The test actually executed and failed.

### NOT VALIDATED

The test did not execute.

### BLOCKED BY INFRASTRUCTURE

Execution could not occur because required infrastructure was unavailable.

### UNSUPPORTED

The implementation explicitly reports that the requested capability is outside the certified contract.

Do not use PASS as a synonym for "the code looks correct".

---

# 30. Change isolation and commit discipline

Prefer one coherent architectural increment per commit/PR.

A commit should be understandable as:

~~~
one boundary
one capability
one testable reason
~~~

Do not mix unrelated cleanup into a correctness-critical change.

A commit that changes architecture must include the corresponding architecture tests and documentation.

---

# 31. Review checklist

Before accepting any Application Layer change, review all of the following:

~~~
[ ] correct owning library
[ ] correct dependency direction
[ ] no new circular dependency
[ ] single kernel gateway preserved
[ ] no Rust/native/transport leakage
[ ] no direct kernel bypass
[ ] semantic authority preserved
[ ] TDD RED demonstrated
[ ] targeted GREEN demonstrated
[ ] integration tests executed
[ ] red-team tests executed
[ ] app_e2e executed
[ ] complete applicable E2E executed
[ ] deterministic identity verified
[ ] failure-closed behavior verified
[ ] cache/invalidation verified
[ ] transaction/rollback verified
[ ] concurrency verified where applicable
[ ] documentation updated
[ ] exact evidence recorded
~~~

---

# 32. Final rule

The Application Layer must be able to grow to a very large System-CAD platform without becoming a collection of mutually coupled features.

Every new capability must strengthen the architecture rather than merely increasing feature count.

The governing principle is:

~~~
Architecture first
    ->
authoritative RED
    ->
minimal correct implementation
    ->
GREEN
    ->
attack the implementation
    ->
real integration
    ->
complete application E2E
    ->
exact evidence
~~~

And the most important invariant is:

~~~
Many .NET libraries
      +
One Application Layer
      +
One concrete UMLCAD Kernel API
      +
One executable architecture gate
      +
Acyclic dependencies
      +
TDD + integration + red-team + E2E
      =
Maintainable System-CAD Application Layer
~~~

No implementation task is allowed to weaken this invariant in exchange for local convenience.
