# UMLCAD.V.7 — Future Work Milestones (fw)

These milestones are a future architectural refinement plan. They do not reopen M0-M16 mathematical-authority closure. New mathematical capabilities require separate explicit kernel contracts and gates.

## FW-00 — Architecture freeze of the programmable-engineering model

### Objective
Establish the exact conceptual boundary before implementation.

### Required outcomes
- One UMLCAD Kernel API is the only kernel-facing concept visible to .NET semantic/engineering code.
- Rust/HTTP/OCCT/Metal/CUDA names are confined to infrastructure/kernel implementation.
- Engineering rules are executable services, not a fixed catalog of built-in conditions.
- Rules can read typed engineering knowledge.
- Rules can call phenomena simulations.
- Rules can reuse valid cached simulation results.
- Rules can invoke controlled CAD semantic operations.
- Rules can implement their own exception handling and recovery.
- Build-time mandatory rules can refuse a build with expressive diagnostics.
- Rule-controlled CAD changes are transactional and provenance-preserving.
- Engineering Supervision is a distinct programmable layer that can create/revise CAD scenarios and verify system behavior.
- Simulation produces facts; rules interpret facts; CAD services apply semantic decisions.

### Acceptance
Architecture review of dependency direction, API ownership, build/refusal behavior, simulation-cache semantics, and at least three conceptual scenarios: laser HAZ, milling tool wear, and tolerance-driven design restriction.

## FW-01 — Kernel-facing .NET boundary cleanup

### Objective
Refine the new UMLCAD.Kernel gateway under app/framework/libraries/ so engineering libraries depend only on the UMLCAD Kernel API rather than Rust-specific names.

### Work
- define stable kernel operation/result contracts at the correct boundary;
- isolate transport/process details;
- rename or move Rust-specific adapters to infrastructure;
- ensure engineering projects contain zero Rust/HTTP implementation knowledge;
- preserve the current mathematical authority semantics;
- add boundary tests proving implementation replacement does not alter domain contracts.

### Acceptance
A test kernel implementation can satisfy the same UMLCAD Kernel API as the real kernel without changes to CAD/CAM/Science/rule code.

## FW-02 — Generic Engineering Context

### Objective
Create the typed information surface available to engineering rules and supervision programs.

### Work
Define query services for:

~~~
CAD semantics
geometry/topology references
product structure
drawing/PMI/tolerances
materials
machines/tools/fixtures
processes
simulation results
spatial regions
fields
configuration
inspection/measurement
manufacturing history
~~~

### Acceptance
A rule can retrieve a line/curve/face, its surrounding influence regions, relevant PMI/tolerance information, associated manufacturing/simulation provenance, and prior valid simulation results without stringly typed dictionaries.

## FW-03A — Explicit programmatic CAD pathway

### Objective
Make the rule/supervision-to-CAD path a first-class architectural contract rather than an implied use of internal services.

### Work
Define and test the complete sequence:

~~~text
engineering program
 -> knowledge query
 -> semantic CAD command
 -> transaction/change set
 -> dependency closure
 -> recompute
 -> UMLCAD Kernel API
 -> authoritative result
 -> updated engineering context
~~~

The same semantic command path must be usable by normal user actions, automation, engineering rules, and Engineering Supervision.

### Acceptance
A supervision program can create and modify CAD entirely through the public semantic command pathway, while direct access to private semantic collections or kernel implementation objects is impossible from the engineering-programming surface.

## FW-03 — CAD Control API

### Objective
Allow engineer-written rules and supervision programs to control CAD through semantic operations.

### Work
Define command/change services for feature creation, modification, suppression, deletion, parameter changes, constraints, references, recomputation, and revision/change-set creation.

Human commands, automation, rules, and supervision should converge on the same semantic command machinery.

### Acceptance
A custom rule changes a CAD parameter and creates a design constraint, then the normal evaluation engine recomputes the affected semantic closure and produces the authoritative result. A supervision program can construct the same scenario programmatically.

## FW-04 — Transactional rule execution and exception semantics

### Objective
Make custom engineering code safe to execute against semantic CAD state.

### Work
- transaction/change-set boundary;
- commit/rollback;
- rule-local exception handling;
- framework handling of uncaught exceptions;
- deterministic failure diagnostics;
- nested service-call semantics;
- partial mutation prevention;
- cancellation and execution budgets.

### Acceptance
A rule that mutates three CAD objects and then throws leaves no partial mutation after rollback. A rule that catches its exception may intentionally recover and commit its chosen result.

## FW-04A — Build-time engineering-rule enforcement

### Objective
Make mandatory engineering rules part of the normal build contract.

### Work
- rule applicability and mandatory/optional classification;
- rule scheduling within the build lifecycle;
- structured rejection diagnostics;
- exception-to-diagnostic mapping;
- deterministic refusal semantics;
- rule-triggered recomputation;
- build identity inclusion for authoritative rule decisions.

### Acceptance
A geometrically valid design is refused because an engineering rule detects an unacceptable condition, and the engineer receives an expressive diagnostic identifying the rule, affected entity/region, evidence, and corrective direction.

## FW-05 — Rule registration, replacement and precedence

### Objective
Make default engineering knowledge replaceable.

### Work
Support default, company, project, part, and process-specific rule registrations, with explicit precedence and versioning.

### Acceptance
The same rule contract can execute the built-in implementation or a custom implementation without changing callers. Selection is deterministic and provenance records the selected implementation.

## FW-05A — Simulation requests and cache semantics

### Objective
Allow build-time rules to call phenomena simulations and safely reuse valid results.

### Work
- typed simulation request contracts;
- simulation execution identity;
- cache-key construction from all semantic inputs;
- validity/provenance checking;
- stale/incomplete-result rejection;
- cache invalidation;
- deterministic cache hit versus fresh execution equivalence;
- policy for forced refresh.

### Acceptance
A rule requests a simulation twice with the same semantic inputs and obtains a valid cache hit on the second request. Changing a simulation input forces a new identity and prevents invalid reuse.

## FW-06 — Spatial regions and engineering fields

### Objective
Represent manufacturing and physical effects as queryable engineering knowledge.

### Work
Define generic SpatialRegion, Field<T>, provenance, frame, validity, uncertainty, sampling, overlap, containment, distance, and spatial query semantics.

Initial manufacturing examples:

~~~
kerf
HAZ
thermal alteration
surface-quality change
material-property change
geometric deviation
residual stress
mechanical damage
~~~

Do not create a hardcoded class for every future physical effect unless semantics genuinely require one.

### Acceptance
A laser simulation result exposes a HAZ region and at least one spatial material-property field. A rule can query whether a candidate feature intersects the region and evaluate a threshold over the field.

## FW-07 — Phenomena-simulation foundation

### Objective
Expand the current generic phenomena service into a serious provider-neutral simulation boundary.

### Work
Model phenomena, models, solver executions, inputs, assumptions, validity, convergence/evidence, fields, and results. Support thermal, structural, fluid, and coupled/multiphysics concepts without making any one solver application semantic authority.

### Acceptance
At least one complete phenomena path returns spatial/physical evidence consumable by a manufacturing simulation, a rule, and a supervision program. External-solver adapters remain outside the semantic model.

## FW-08 — Manufacturing digital workpiece state

### Objective
Model the evolving manufactured object rather than only a toolpath.

### Work
Represent nominal workpiece, stock, material state, tool state, machine state, fixture state, process state, physical fields, geometric deviation, and uncertainty. Support state evolution over operations.

### Acceptance
A multi-operation sequence produces distinct intermediate workpiece states and preserves deterministic provenance linking each state to operations, tools, machine state and simulation evidence.

## FW-09 — Manufacturing Effect Graph

### Objective
Allow many simultaneous manufacturing effects without process-specific hardcoding.

### Work
Define effect models that declare inputs, outputs, applicability, state dependencies, uncertainty, and coupling. Build an effect dependency graph with deterministic scheduling and explicit handling of coupled/iterative effects.

Example effects:

~~~
tool wear
machine thermal drift
cutting force
temperature
surface generation
kerf
material transformation
residual stress
fixture deformation
geometric error
~~~

### Acceptance
Two or more effects can consume and modify shared state in one simulation without a laser-specific or milling-specific orchestration switch. Cycles are explicitly solved or fail closed.

## FW-10 — Tool, machine and fixture state evolution

### Objective
Make manufacturing resources physically stateful when their behavior depends on history.

### Work
Model tool wear/life, calibration, machine thermal/dynamic/geometric state, fixture state, and process history as versioned simulation inputs/state.

### Acceptance
Repeated operations with the same nominal tool produce different predicted outcomes when the tool state has changed, and the state is reproducibly tied to the manufacturing history.

## FW-11 — Tolerance, deviation and uncertainty reasoning

### Objective
Make manufacturing tolerance a computational engineering domain rather than a drawing-only annotation.

### Work
Model required tolerances, process capability, predicted deviation, measurement uncertainty, statistical/worst-case representations, correlations, and spatial deviation fields.

### Acceptance
The engine can compare a required PMI tolerance to a simulated as-manufactured deviation and distinguish Pass, Fail, and Indeterminate with explicit evidence.

## FW-12 — Manufacturing-to-design rule feedback

### Objective
Turn manufacturing knowledge into design constraints and controlled design changes.

### Work
Rules consume manufacturing simulation fields/regions and can create, modify, or veto CAD semantic operations. Support generated constraints with explicit provenance back to the rule and simulation evidence.

### Acceptance
A custom rule detects a prohibited HAZ overlap around a torque-bearing feature, creates the appropriate design restriction, and forces deterministic recomputation without embedding the engineering rule into CAM code.

## FW-13 — CAM simulation-driven planning and compensation

### Objective
Move CAM from nominal toolpath generation toward simulation-validated manufacturing planning.

### Work
- simulate candidate operations before NC generation;
- predict geometric and physical effects;
- evaluate tolerance and quality requirements;
- account for tool/machine state;
- generate compensation or alternative process plans;
- postprocess only a validated plan.

### Acceptance
A CAM candidate is simulated, evaluated against engineering requirements, compensated or rejected, and only then converted into deterministic NC/G-code.

## FW-14 — Engineering Supervision runtime

### Objective
Provide a programmable engineering-supervision layer capable of constructing sophisticated CAD revisions and testing the integrated behavior of CAD, rules, simulation and manufacturing semantics.

### Work
- create isolated temporary/revision CAD states;
- use the same semantic CAD control API as normal engineering code;
- invoke ordinary rules;
- invoke simulations and use cached results;
- inspect physical fields, spatial regions, tolerances and results;
- express engineering assertions;
- compare candidate revisions;
- commit deliberate revisions or discard experiments;
- preserve supervision provenance and evidence.

The supervision layer should support unit-like, integration-like, system-like, regression-like, and exploratory engineering programs.

### Acceptance
A supervision program can construct a parametric part with a sketch, rectangle, three circles and three holes; build it; observe a mandatory rule rejection; modify the design; rebuild; run/consume simulation; assert the engineering result; and commit or discard the resulting revision.

## FW-15 — Rule/package lifecycle and trust model

### Objective
Make executable engineering knowledge and supervision programs maintainable in real organizations.

### Work
Define package identity, version compatibility, trusted sources, installation/replacement audit, dependency declarations, reproducible builds, and resource/isolation policy.

### Acceptance
A rule/supervision package can be replaced without ambiguity, its exact implementation/version is recoverable from provenance, and executable engineering code cannot silently bypass semantic authority.

## FW-16 — Full vertical programmable-manufacturing proof

### Objective
Prove the entire architecture through one demanding scenario.

### Candidate scenario

~~~
Parametric CAD part
    -> sketch on specified face
    -> rectangle + three circles
    -> three holes
    -> build
    -> mandatory engineering rules
    -> rule requests thermal simulation
    -> simulation cache hit or execution
    -> HAZ + material/deviation fields
    -> rule rejects or creates a design restriction
    -> engineer/supervision program modifies revision
    -> recompute
    -> CAM re-plan / compensation
    -> final simulation
    -> tolerance verification
    -> deterministic NC
~~~

### Acceptance
The complete path is exercised through real component tests, red-team tests, Python/system E2E, deterministic identity checks, transaction rollback tests, rule replacement tests, simulation-cache tests, supervision revision tests, and exact evidence.

## Global testing law for FW

Every milestone follows the repository's established discipline:

~~~
Inspect
 -> authoritative RED
 -> implementation
 -> GREEN
 -> red-team
 -> complete E2E
 -> exact evidence
~~~

The programmable-engineering layer specifically requires adversarial coverage for:

~~~
uncaught rule exception
rule-local recovery
partial mutation + rollback
mandatory rule rejection
diagnostic completeness
recursive rules
cyclic rules
oscillating corrections
non-deterministic rule input
stale simulation result
invalid simulation cache hit
wrong field frame
wrong region provenance
ambiguous spatial query
invalid CAD mutation
constraint cycle
cache contamination
rule-version identity collision
override precedence collision
provider disagreement
simulation uncertainty / indeterminate result
supervision revision isolation
supervision rollback
supervision assertion failure
supervision-induced recompute loop
revision provenance loss
~~~

## Non-negotiable architectural outcome

At the end of this future-work sequence:

~~~
Engineer-written program
       |
       +--> reads engineering knowledge
       +--> calls simulations
       +--> reuses valid cached simulation results
       +--> reasons about geometry + physics + manufacturing
       +--> invokes CAD semantic functions
       +--> creates constraints / changes / decisions
       +--> handles its own exceptions
       +--> can cause the current build to be rejected
       v
UMLCAD Engineering Runtime
       |
       +--> transactional semantic control
       +--> deterministic evaluation
       +--> provenance / evidence
       +--> mandatory engineering-rule enforcement
       +--> authoritative kernel evaluation
       +--> phenomena simulation

Engineering Supervision Program
       |
       +--> constructs/revises test scenarios
       +--> drives normal CAD/rule/simulation services
       +--> asserts integrated engineering behavior
       +--> retains or discards revisions
       v
Validated System-CAD / Manufacturing result
~~~

The engineer is not forced to wait for UMLCAD developers to add a bespoke feature for every new engineering idea. The platform provides stable primitives and authority boundaries; engineering code provides the domain logic; Engineering Supervision provides executable system-level engineering verification and controlled revision.
