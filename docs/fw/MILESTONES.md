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
- Rules can invoke controlled CAD semantic operations.
- Rules can implement their own exception handling and recovery.
- Rule-controlled CAD changes are transactional and provenance-preserving.
- Simulation produces facts; rules interpret facts; CAD services apply semantic decisions.
- Spatial regions and fields are first-class engineering knowledge.

### Acceptance
Architecture review of dependency direction, API ownership, and at least three nontrivial conceptual scenarios: laser HAZ, milling tool wear, and tolerance-driven design restriction.

## FW-01 — Kernel-facing .NET boundary cleanup

### Objective
Refine the current transitional UMLCAD.Kernel.Client design so engineering libraries depend only on the UMLCAD Kernel API rather than Rust-specific names.

### Work
- define stable kernel operation/result contracts at the correct boundary;
- isolate transport/process details;
- rename or move Rust-specific adapters to infrastructure;
- ensure engineering projects contain zero Rust/HTTP implementation knowledge;
- preserve the current mathematical authority semantics;
- add boundary tests proving implementation replacement does not alter domain contracts.

### Acceptance
A test kernel implementation can satisfy the same UMLCAD Kernel API as the real kernel without changes to CAD/CAM/Science rule code.

## FW-02 — Generic Engineering Context

### Objective
Create the typed information surface available to engineering rules.

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
~~~

### Acceptance
Rules can retrieve a line/curve/face, its surrounding influence regions, relevant PMI/tolerance information, and associated manufacturing/simulation provenance without stringly typed dictionaries.

## FW-03 — CAD Control API

### Objective
Allow engineer-written code to control CAD through semantic operations.

### Work
Define command/change services for feature creation, modification, suppression, deletion, parameter changes, constraints, references, and recomputation.

Rules must never mutate internal semantic collections or raw kernel objects directly.

### Acceptance
A custom rule changes a CAD parameter and creates a design constraint, then the normal evaluation engine recomputes the affected semantic closure and produces the authoritative result.

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
A rule that mutates three CAD objects and then throws leaves no partial mutation after rollback; a rule that catches its exception may intentionally recover and commit its chosen result.

## FW-05 — Rule registration, replacement and precedence

### Objective
Make default engineering knowledge replaceable.

### Work
Support default, company, project, part, and process-specific rule registrations, with explicit precedence and versioning.

### Acceptance
The same rule contract can execute the built-in implementation or a custom implementation without changing callers. Selection is deterministic and provenance records the selected implementation.

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
A laser simulation result exposes a HAZ region and at least one spatial material-property field; a rule can query whether a candidate feature intersects the region and evaluate a threshold over the field.

## FW-07 — Phenomena-simulation foundation

### Objective
Expand the current generic phenomena service into a serious provider-neutral simulation boundary.

### Work
Model phenomena, models, solver executions, inputs, assumptions, validity, convergence/evidence, fields, and results. Support thermal, structural, fluid, and coupled/multiphysics concepts without making any one solver application semantic authority.

### Acceptance
At least one complete phenomena path returns spatial/physical evidence that is consumable by a manufacturing simulation and an engineering rule. External-solver adapters remain outside the semantic model.

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

## FW-14 — Rule/package lifecycle and trust model

### Objective
Make executable engineering knowledge maintainable in real organizations.

### Work
Define package identity, version compatibility, trusted sources, installation/replacement audit, dependency declarations, reproducible builds, and resource/isolation policy.

### Acceptance
A rule package can be replaced without ambiguity, its exact implementation/version is recoverable from provenance, and execution of untrusted code cannot silently bypass semantic authority.

## FW-15 — Full vertical manufacturing proof

### Objective
Prove the entire architecture through one demanding manufacturing scenario.

### Candidate scenario

~~~
Parametric CAD part
    -> PMI tolerance / functional region
    -> laser operation
    -> tool/process/machine definition
    -> phenomena simulation (thermal + material response)
    -> HAZ + deviation fields
    -> custom engineer rule
    -> design restriction / parameter change
    -> recompute
    -> CAM re-plan / compensation
    -> final simulation
    -> tolerance verification
    -> deterministic NC
~~~

### Acceptance
The complete path is exercised through real component tests, red-team tests, Python/system E2E, deterministic identity checks, transaction rollback tests, rule replacement tests, and exact evidence.

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

The new rule engine specifically requires adversarial coverage for:

~~~
uncaught rule exception
rule-local recovery
partial mutation + rollback
recursive rules
cyclic rules
oscillating corrections
non-deterministic rule input
stale simulation result
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
~~~

## Non-negotiable architectural outcome

At the end of this future-work sequence:

~~~
Engineer-written code
       |
       +--> reads engineering knowledge
       |
       +--> reasons about geometry + physics + manufacturing
       |
       +--> invokes CAD semantic functions
       |
       +--> creates constraints / changes / decisions
       |
       +--> handles its own exceptions
       v
UMLCAD Engineering Runtime
       |
       +--> transactional semantic control
       +--> deterministic evaluation
       +--> provenance / evidence
       +--> authoritative kernel evaluation
       +--> phenomena simulation
       v
Validated System-CAD / Manufacturing result
~~~

The engineer is not forced to wait for UMLCAD developers to add a bespoke feature for every new engineering idea. The platform provides stable primitives and authority boundaries; engineering code provides the domain logic.
