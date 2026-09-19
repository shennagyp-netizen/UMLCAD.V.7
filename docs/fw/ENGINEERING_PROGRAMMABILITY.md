# Programmable Engineering Knowledge, Build Rules, and Engineering Supervision

## 1. Purpose

UMLCAD shall provide a programmable engineering environment in which an engineer can express domain knowledge directly in the host programming language.

The platform supplies generic execution, knowledge-access, CAD-control, simulation, transaction, dependency, provenance, diagnostics, caching, and validation facilities. It does not attempt to enumerate every possible engineering rule in the core product.

An engineer-written program may therefore contain:

~~~csharp
if (...) { ... }
for (...) { ... }
try { ... } catch (...) { ... }
RunSimulation(...);
Cad.Features.Move(...);
Cad.Constraints.Add(...);
Recompute();
~~~

The programming language is the engineering knowledge and control medium. UMLCAD supplies stable semantic services and authority boundaries.

## 2. Three different responsibilities

The architecture keeps these responsibilities separate:

~~~
Physical / phenomena simulation
    produces physical facts, fields, influence regions and simulation evidence

Engineering rules / programs
    interpret facts, enforce engineering requirements, decide actions,
    and may invoke CAD and simulation services

CAD control services
    apply requested actions as controlled semantic CAD changes through the
    normal CAD evaluation architecture
~~~

Simulation does not decide that a hole is forbidden. An engineer-written rule can decide that from the available knowledge.

## 3. Normal build-time engineering enforcement

Engineering rules are part of the normal design-build lifecycle.

When an engineer builds a design:

~~~
Engineer CAD program / document
        |
        v
Semantic build
        |
        v
CAD evaluation
        |
        v
Engineering Rule Runtime
        |
        +--> inspect design
        +--> inspect manufacturing knowledge
        +--> call simulations if required
        +--> read cached simulation results when valid
        +--> perform engineering calculations
        +--> create controlled CAD changes/constraints if appropriate
        |
        v
Validation / recomputation
        |
        +--> all mandatory rules satisfied
        |          |
        |          v
        |       build succeeds
        |
        +--> rule rejects / vetoes / fails
                   |
                   v
               build refused
~~~

A build is not successful merely because CAD mathematics evaluated successfully. The engineering-rule layer may reject the build after valid geometry has been produced.

This is intentional. A geometrically valid design can be engineering-invalid.

## 4. Rules are executable services

The target contract is conceptually:

~~~csharp
public interface IEngineeringRule
{
    RuleIdentity Identity { get; }
    RuleApplicability CheckApplicability(EngineeringContext context);

    RuleExecutionResult Execute(
        EngineeringContext context,
        IEngineeringServices services);
}
~~~

The exact API is intentionally deferred until the semantic contracts are refined.

Normative properties:

- rule code is executable host-language code, not restricted to a fixed DSL;
- rule code can read typed engineering knowledge;
- rule code can invoke controlled CAD operations;
- rule code can request or invoke phenomena simulations;
- rule code can query cached simulation results;
- rule code can implement its own exception handling and recovery;
- rule code can accept, reject, transform, constrain, or otherwise control engineering state according to its authority and declared scope;
- the framework records transaction, provenance, dependency, identity, diagnostics, and evidence around execution.

A rule is therefore an engineering program/service, not merely a boolean database check.

## 5. Engineering context and knowledge access

EngineeringContext shall expose stable typed query services rather than a stringly typed dictionary.

Conceptually:

~~~
EngineeringContext
 ├── Design knowledge
 ├── CAD semantic model
 ├── Product / assembly structure
 ├── Geometry and topology references
 ├── Drawing / PMI / tolerances
 ├── Materials
 ├── Machines / tools / fixtures
 ├── Processes
 ├── Manufacturing simulation results
 ├── Spatial regions / influence regions
 ├── Physical fields
 ├── Measurements / inspection results
 └── Configuration / variants / policies
~~~

Queries may include:

~~~
Find lines / curves / faces / features
Find entities near another entity
Find regions surrounding an entity
Find HAZ / kerf / affected regions
Sample a physical/property field over a region
Measure distance / overlap / containment
Find material properties at a location or region
Find tolerance requirements
Find dependent features / references
Find manufacturing provenance
Find applicable rules
Find previous simulation executions
~~~

A rule can reason about an entity and its surrounding engineering state.

## 6. Simulation calls and simulation-result caching

Rules may request phenomena simulations as part of engineering evaluation.

Conceptually:

~~~csharp
var result = context.Simulation.GetOrRun(
    thermalRequest,
    cachePolicy: CachePolicy.ReuseValid);
~~~

The rule must not need to know whether the result came from:

~~~
an in-process solver
an external solver
a local process
a remote service
CPU
GPU
a previous valid simulation execution
~~~

### Cache identity

A cached simulation result is reusable only when its semantic inputs are equivalent.

The cache identity must include every input that can materially affect the simulation result, including as applicable:

~~~
phenomenon
model / solver contract
geometry/result identity
material state
machine/tool/process state
boundary conditions
loads
environment
mesh/discretization policy when semantically relevant
numerical settings when semantically relevant
tolerance / acceptance policy
configuration
provider/model version where relevant
~~~

A cache hit must be semantically equivalent to a fresh execution for the same identity.

A stale, incomplete, invalid, or provenance-incompatible simulation result is not a valid cache hit.

### Simulation during a build

A normal build may therefore be:

~~~
build
  -> rule
      -> simulation request
          -> valid cache hit
             OR
          -> simulation execution
      -> rule evaluates result
  -> build succeeds or is rejected
~~~

Simulation caching is an optimization and reproducibility mechanism, not a shortcut around physical validity.

## 7. Spatial knowledge is first-class

Manufacturing processes can produce spatially distributed effects. The architecture therefore needs generic SpatialRegion and Field semantics.

Conceptually:

~~~
SpatialRegion
    geometry / domain
    source / provenance
    context / frame
    validity

Field<T>
    spatial domain
    quantity/property definition
    values / representation
    validity
    uncertainty
    provenance
~~~

Possible derived knowledge includes:

~~~
KerfRegion
HeatAffectedRegion
ThermallyAlteredRegion
ResidualStressRegion
MechanicalDamageRegion
SurfaceQualityRegion
GeometricDeviationRegion
MaterialPropertyChangeRegion
ManufacturingExclusionRegion
~~~

These should normally be instances of generic region/field semantics, not mandatory hardcoded subclasses for every future phenomenon.

Important distinctions remain explicit:

~~~
removed material
    != thermally affected material
    != mechanically weakened material
    != geometrically deviated region
    != surface-quality affected region
~~~

A single operation may produce several overlapping regions and fields simultaneously.

## 8. CAD control from engineering code

The rule layer may control CAD through a controlled semantic CAD service.

Conceptually:

~~~csharp
public interface ICadControlService
{
    CadOperationResult MoveFeature(...);
    CadOperationResult CreateFeature(...);
    CadOperationResult DeleteFeature(...);
    CadOperationResult SuppressFeature(...);
    CadOperationResult AddConstraint(...);
    CadOperationResult RemoveConstraint(...);
    CadOperationResult UpdateParameter(...);
    CadOperationResult CreateReference(...);
    CadOperationResult Recompute(...);
}
~~~

The actual operation set will be derived from the stable CAD semantic model, not from kernel implementation details.

Every mutation is a semantic command/change. This preserves:

- evaluation dependencies;
- reference semantics;
- frame correctness;
- deterministic identity;
- history/provenance;
- cache invalidation;
- authoritative kernel evaluation;
- undo/redo or transaction semantics where applicable.

Human commands, automation, and engineering programs should ultimately use the same semantic CAD command machinery.

## 9. Rules may create engineering constraints

A manufacturing effect can become a design restriction without placing CAM-specific logic inside the CAD core.

Example:

~~~
laser operation
    -> phenomena simulation
    -> HAZ / material-strength field
    -> engineering rule
    -> engineering constraint
    -> CAD evaluation
~~~

Examples:

~~~
Hole H must not overlap region R.
Hole H must remain at least 4 mm from R.
Mounting surface may not intersect a strength-reduction field.
A critical edge shall not be exposed to specified thermal history.
A feature may proceed only when predicted deviation remains within PMI tolerance.
~~~

The rule owns the engineering interpretation. The generated constraint is part of the CAD/Knowledge semantic state and retains provenance to the rule and source evidence.

## 10. Rule execution is transactional

Rule-controlled CAD changes execute inside a transaction/change set.

~~~
Rule starts
    -> inspect context
    -> issue semantic CAD changes
    -> validate/recompute
    -> call additional rules/simulations as required
    -> commit

failure / uncaught exception / invalid result
    -> rollback rule change set
    -> preserve failure evidence
~~~

The rule may catch its own exception and intentionally recover.

If it does not catch an exception, the framework records an explicit rule-execution failure and rolls back the current rule transaction.

No rule is allowed to leave partially applied semantic mutations because of an exception.

## 11. Build refusal and expressive engineering diagnostics

Mandatory engineering rules run as part of build validation.

A rule can refuse the build with an expressive result such as:

~~~
ENGINEERING_RULE_FAILED

Rule:
    CriticalMountingRegion.HazExclusion v4

Affected entity:
    Hole H17

Affected region:
    ManufacturingRegion R42

Reason:
    Predicted minimum material strength in R42 = 280 MPa
    Required minimum strength = 350 MPa

Evidence:
    ThermalSimulationResult T938
    MaterialField M77

Action:
    Move the hole, modify the manufacturing process,
    or change the applicable engineering design.
~~~

The build framework should expose structured diagnostics to the engineer rather than reducing the outcome to a generic exception message.

Build refusal must be explicit and deterministic.

## 12. Rules can intentionally control workflow outcomes

A rule is not restricted to boolean output.

Possible controlled outcomes include:

~~~
Pass
Reject
Warn
RequestRecompute
RequestSimulation
ApplyChange
CreateConstraint
VetoOperation
Escalate / RequireHumanDecision
Indeterminate
Unsupported
Failed
~~~

A mandatory rejecting rule causes build refusal unless an explicitly authorized recovery/override path resolves it.

## 13. Rule composition and overrides

UMLCAD shall ship default rule implementations, but defaults are not semantic law.

Support:

~~~
Default implementation
       |
       +--> overridden by company implementation
       +--> overridden by project implementation
       +--> extended by another rule
       +--> disabled/replaced where policy allows
~~~

Selection and precedence are deterministic.

The selected implementation, version and policy participate in provenance and, when authoritative, in evaluation identity.

## 14. Rule identity and provenance

A rule result must identify at least:

~~~
rule identity
rule implementation/version
input context identity
relevant CAD/result identities
relevant simulation-result identities
configuration/policy identity
execution identity
affected semantic entities
affected spatial regions
outcome
diagnostics/evidence
~~~

If a rule implementation influences an authoritative build result, its identity/version must participate in the relevant evaluation identity.

## 15. Recursion, cycles and termination

Because rules can modify CAD and those modifications can trigger further evaluation, rule execution is a graph problem.

The platform must detect/control:

- direct recursion;
- A -> B -> A cycles;
- change/recompute storms;
- oscillating parameter corrections;
- repeated identical mutations;
- excessive execution depth/steps;
- constraint cycles;
- simulation/rule feedback loops.

The engine should use deterministic execution identities, dependency closure, transaction generations, simulation cache identity, and explicit execution budgets/guards.

A cycle or exhausted budget fails closed with evidence rather than continuing indefinitely.

## 16. Determinism and stateful engineering knowledge

A rule that contributes to an authoritative build must be deterministic for its declared inputs.

Hidden ambient state, wall-clock time, random numbers, untracked local files, or uncontrolled network responses must not influence authoritative decisions without being modeled as explicit inputs.

Stateful facts such as machine calibration, tool-life history, measurement databases, or simulation caches are legitimate, but their relevant snapshot/version/identity must become explicit evaluation inputs.

## 17. Engineering Supervision

Engineering Supervision is a distinct programmable layer above ordinary build-time rules.

Its purpose is not merely to ask whether the current design passes rules. It can write sophisticated, general CAD revisions and execute complete engineering scenarios that behave like engineering-level unit tests, integration tests, regression tests, and exploratory design procedures.

Conceptually:

~~~
Engineering Supervision Program
        |
        +--> create temporary/revision CAD state
        +--> modify CAD through the same semantic control API
        +--> invoke ordinary engineering rules
        +--> invoke simulations
        +--> inspect resulting knowledge/fields
        +--> assert engineering expectations
        +--> compare revisions/results
        +--> commit a deliberate revision
            OR
        +--> discard the experimental revision
~~~

A supervision program is therefore able to express workflows such as:

~~~
Create a sketch on a specified face
Add a rectangle and three circles
Create three holes
Build
Expect required rules to execute
Expect a selected rule to reject the build
Inspect the diagnostic
Modify the design
Rebuild
Run thermal simulation
Inspect the HAZ field
Create another revision
Verify tolerance result
Commit the acceptable revision
~~~

This is more powerful than a conventional test fixture because the supervision program itself can construct and modify a realistic engineering scenario using normal CAD semantics.

## 18. Engineering Supervision versus ordinary rules

The distinction is:

~~~
Engineering Rules
    protect the design during ordinary engineering builds.
    They are part of normal build validation and can veto the build.

Engineering Supervision
    deliberately drives the system through complex scenarios.
    It can create revisions, mutate designs, invoke rules/simulations,
    assert expected engineering behavior, and decide whether a revision
    should be retained.
~~~

A supervision program may call rules; ordinary rules do not implicitly become supervision programs.

Supervision should normally operate in isolated revision/transaction contexts so experimentation cannot silently corrupt the engineer's current authoritative design.

## 19. Engineering Supervision as executable engineering tests

Engineering Supervision can express tests at several levels:

~~~
Unit-like:
    test one engineering rule or CAD semantic operation.

Integration-like:
    create a realistic model and verify interactions between CAD,
    manufacturing, simulation, tolerancing and rules.

System-like:
    execute a complete design/manufacturing lifecycle.

Regression-like:
    replay a known engineering scenario against a new system revision.

Exploratory:
    programmatically generate and evaluate design alternatives.
~~~

The repository's traditional automated test infrastructure still remains necessary. Engineering Supervision is a product-level engineering verification capability, not a replacement for software unit tests, integration tests, or security testing.

## 20. Boundary to the UMLCAD Kernel

The rule system, Engineering Supervision, and all engineering .NET libraries use the UMLCAD Kernel API as the single concrete kernel boundary.

They must not depend on Rust language constructs, Rust crate names, OCCT classes, Metal types, CUDA types, or transport implementation details.

The current RustKernelService may remain transitional infrastructure until the kernel-facing implementation is refactored. The long-term semantic contract is named for the UMLCAD Kernel, not for its implementation language.

## 21. Security and trust boundary

Because rules and supervision are executable code with CAD-write authority, deployment and loading are trust decisions.

A production implementation should define:

- trusted assembly/package sources;
- version compatibility;
- capability declarations;
- reproducible build expectations;
- resource/time limits where needed;
- isolation policy for untrusted code;
- audit trail for rule/supervision installation and replacement.

Sandboxing protects the host; semantic contracts protect engineering truth.

## 22. Architectural invariant

~~~
Phenomena simulation
    -> produces engineering facts

Engineering rule/program
    -> reads facts
    -> reasons
    -> optionally simulates
    -> optionally changes CAD
    -> can reject the current build

Engineering Supervision
    -> deliberately constructs/revises CAD scenarios
    -> invokes rules/simulation
    -> verifies system behavior
    -> retains or discards revisions

CAD services
    -> apply semantic changes

UMLCAD Kernel
    -> proves mathematical CAD results

= Programmable, supervised System-CAD
~~~

