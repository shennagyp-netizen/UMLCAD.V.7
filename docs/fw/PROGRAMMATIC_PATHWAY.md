# UMLCAD.V.7 — Future Work: Programmatic Engineering Pathway

This document makes the execution pathway between engineer-written programs, engineering knowledge, CAD control, simulation, and the UMLCAD Kernel explicit.

It is a target architecture document under `docs/fw/`; it is not a claim of current implementation.

## 1. Normal build-time pathway

~~~text
CAD specification
      |
      v
Build transaction
      |
      v
CAD semantic evaluation
      |
      v
Mandatory engineering programs
      |
      +--> query knowledge
      +--> call simulation
      |       |
      |       +--> valid cache hit
      |       |
      |       +--> simulation execution
      |
      +--> reason
      +--> issue CAD semantic commands
      |
      v
Semantic draft / affected dependency closure
      |
      v
Recompute
      |
      v
UMLCAD Kernel API
      |
      v
Authoritative CAD result
      |
      +--> updated geometry/topology
      +--> updated provenance
      +--> updated engineering context
      +--> updated simulation inputs/results
      |
      v
Engineering programs evaluate again where dependencies require it
      |
      +--> accepted -> commit
      |
      +--> rejected -> build refusal + diagnostics
~~~

A geometrically valid CAD result is therefore not sufficient for build success. Mandatory engineering programs are part of the build contract.

## 2. What the engineer can read

The program can access typed semantic knowledge, including:

~~~text
CAD entities
geometry
topology
references/publications
Product / Assembly structure
Drawing / PMI / tolerances
materials
machines
tools
fixtures
processes
manufacturing state
simulation results
spatial regions
physical fields
inspection/measurement
configuration
rule policy
~~~

The program can reason about an entity and its surrounding spatial/physical context.

Example:

~~~text
line L
  -> find regions around L
  -> discover HAZ R
  -> query strength field S(x,y,z)
  -> find nearby torque-bearing hole H
  -> test H against R and S
~~~

## 3. What the engineer can do

The program controls CAD through semantic commands, not internal object mutation.

~~~text
CreateFeature
DeleteFeature
MoveFeature
SuppressFeature
SetParameter
AddConstraint
RemoveConstraint
CreateReference
ChangeConfiguration
Recompute
CreateRevision
~~~

The exact command vocabulary is a future contract. The architectural invariant is:

~~~text
program logic
    != CAD internal mutation

program logic
    -> CAD semantic command
    -> CAD evaluation
    -> kernel proof
~~~

## 4. Simulation calls

The program can request phenomena simulation while evaluating a design.

~~~text
Program
  -> simulation request
  -> simulation identity
  -> cache lookup
      |
      +--> valid result -> return
      |
      +--> miss/stale -> execute
  -> simulation result
  -> program reasons over fields/regions/evidence
~~~

The program should not need to know the simulation application's implementation.

## 5. Simulation cache correctness

The cache is reusable only for the same semantic simulation identity.

The identity includes every applicable input that can affect the result, including geometry/result identity, material state, process state, machine/tool state, boundary conditions, environment, model/solver version, relevant numerical/discretization settings, configuration, and uncertainty/validity policy.

A cache hit is therefore a semantic result reuse, not merely a performance shortcut.

## 6. CAD revision after simulation

A common engineering loop is:

~~~text
design revision R0
      |
      v
simulate
      |
      v
physical fields / regions
      |
      v
engineering program
      |
      +--> accept R0
      |
      +--> generate CAD commands
               |
               v
          revision R1
               |
               v
            recompute
               |
               v
            simulate
               |
               v
        inspect R1
~~~

The rule can repeat this process within explicit execution and transaction limits.

## 7. Build refusal

A mandatory rule may terminate the normal build:

~~~text
Candidate CAD
      |
      v
Engineering rule
      |
      +--> acceptable
      |       -> continue
      |
      +--> unacceptable
              |
              v
       Engineering diagnostic
              |
              v
          build refused
~~~

Diagnostics should identify the rule/version, affected entities/regions, relevant evidence and the engineering reason. The engineer can then modify the design.

## 8. Rule-local exception handling

Engineer-written code may contain its own exception handling.

~~~csharp
try
{
    // inspect knowledge
    // call simulation
    // issue CAD commands
}
catch (SpecificEngineeringException)
{
    // engineer-defined recovery
}
~~~

If the program catches and recovers, its chosen semantic operations may continue inside the transaction.

If an exception escapes the program boundary, the framework records the failure and rolls back the active transaction.

## 9. Engineering Supervision pathway

Engineering Supervision uses the same APIs but has a different purpose.

~~~text
Supervision program
      |
      v
create isolated revision
      |
      v
construct / modify CAD
      |
      v
build
      |
      v
invoke normal engineering rules
      |
      v
invoke/reuse simulations
      |
      v
assert expected engineering behavior
      |
      +--> revise again
      |
      +--> commit selected revision
      |
      +--> discard revision
~~~

This permits software-like unit/integration/system/regression scenarios to be written as engineering programs.

Example:

~~~text
Create sketch on face F
Create rectangle
Create three circles
Create three holes
Build
Expect rule X to reject
Inspect diagnostic
Modify hole locations
Build again
Run thermal simulation
Check HAZ
Check tolerance
Commit or discard revision
~~~

## 10. Same command path for UI, automation, rules and supervision

The semantic command pathway should converge:

~~~text
Human UI
Automation
Engineering Rule
Engineering Supervision
        |
        v
Common semantic CAD command API
        |
        v
Transaction / dependency / evaluation
        |
        v
UMLCAD Kernel API
~~~

This prevents a testing or scripting interface from becoming a second, less trustworthy CAD implementation.

## 11. Loop control and termination

Because programs can both observe and modify CAD, the system must explicitly control feedback:

~~~text
rule
 -> change
 -> recompute
 -> rule
 -> change
 -> ...
~~~

The runtime therefore needs:

- execution identity;
- dependency closure;
- transaction generation;
- repeated-mutation detection;
- cycle detection;
- oscillation detection;
- execution budget;
- cancellation;
- deterministic termination state.

Failure to terminate safely is an explicit engineering-program failure, not an implicit infinite loop.

## 12. Authority boundary

The programmatic path ends at the UMLCAD Kernel API:

~~~text
Engineering program
        |
        v
.NET semantic services
        |
        v
UMLCAD Kernel API
        |
        v
kernel implementation
        |
        +--> Rust/internal native implementation
        +--> internal accelerators/adapters as appropriate
~~~

Engineering code never needs to know the kernel programming language.

The kernel remains responsible for mathematical proof. The engineering program remains responsible for engineering reasoning and decisions. The CAD semantic engine remains responsible for turning those decisions into valid semantic evaluation.

## 13. Architectural equation

~~~text
Knowledge
   +
Engineer-written program
   +
Semantic CAD commands
   +
Phenomena simulation
   +
Transactional evaluation
   +
UMLCAD Kernel authority
   =
Programmable System-CAD
~~~
