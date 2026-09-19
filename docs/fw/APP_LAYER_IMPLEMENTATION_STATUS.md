# UMLCAD.V.7 — New Application Layer Implementation Status

## Current implementation boundary

The new Application Layer is implemented under app/. The legacy dotnet/ tree remains untouched and is not a dependency.

## Implemented foundations

- app/framework/libraries/UMLCAD.Kernel
  - single concrete .NET gateway;
  - transport/process implementation is private to the gateway;
  - sealed public API;
  - bounded response handling;
  - explicit timeout, transport, schema, and contradictory-result failures.

- UMLCAD.Cad.Contracts
  - stable CAD identifiers;
  - build identity;
  - kernel build definition.

- UMLCAD.Cad.Expressions
  - finite typed expression values with explicit units.

- UMLCAD.Cad.Semantics
  - immutable CAD feature definitions;
  - explicit feature dependencies;
  - immutable document definitions and snapshots;
  - typed planar line/circle/arc semantics;
  - typed planar constraints;
  - part validation.

- UMLCAD.Cad.Engine
  - semantic CAD command API;
  - transaction-controlled CAD mutation;
  - atomic commit and rollback;
  - deterministic dependency planning;
  - complete cycle-path reporting;
  - deterministic transitive invalidation closure;
  - typed semantic-to-kernel build package generation;
  - typed CAD evaluation through the concrete kernel gateway.

- UMLCAD.Science
  - typed phenomena;
  - engineering frames;
  - deterministic simulation identity;
  - simulation requests/results;
  - spatial regions;
  - scalar fields and finite samples.

- UMLCAD.Engineering.Runtime
  - executable engineering rules;
  - typed EngineeringContext;
  - CAD control without transaction-control leakage;
  - framework-managed commit/rollback;
  - exception-to-diagnostic conversion;
  - atomic multi-rule build validation;
  - deterministic rule registration and precedence;
  - concurrent simulation cache with reusable-result validation.

- UMLCAD.Framework
  - top-level application facade;
  - typed CAD build entry point;
  - engineering validation entry point;
  - kernel creation remains below the framework facade.

- app/application
  - application host;
  - typed Bench Vise E2E demo;
  - application hosts remain outside framework.

## Test coverage authored

Framework tests cover:

- kernel gateway API shape;
- malformed and contradictory kernel responses;
- real loopback kernel transport;
- typed CAD request through the kernel gateway;
- semantic ID and collection invariants;
- CAD transaction commit;
- explicit rollback;
- rollback after a later command failure;
- closed transaction behavior;
- dependency-first evaluation order;
- cycle-path reporting;
- unknown dependency rejection;
- deterministic invalidation closure;
- programmable rule commit;
- rule rejection rollback;
- uncaught rule exception rollback;
- transaction-control isolation from rule code;
- rule -> simulation -> CAD change integration;
- atomic build-time multi-rule validation;
- previous-rule state visibility;
- simulation cache deduplication;
- non-reusable simulation result eviction;
- semantic simulation identity changes;
- deterministic rule precedence and duplicate registration rejection;
- deterministic typed CAD-to-kernel package generation;
- planar geometry and constraint validation;
- framework-level engineering validation integration.

## Validation state

The repository app_e2e architecture gate is the authoritative architecture test and is wired into CI.

The development container has Python 3.13 but no dotnet, csc, mono, or msbuild. Therefore .NET compilation and xUnit execution cannot honestly be reported as locally executed in this environment.

GitHub Actions is configured to run:

~~~
Application architecture gate
Application framework build
Application framework tests
Application host and demo build
~~~

No hardware is required for this Application Layer increment.

## Next declared work

The next implementation increment should extend the semantic CAD model and command set toward:

1. richer Part/Body/Sketch/Feature semantics;
2. semantic references and publications;
3. parameter/expression binding;
4. dependency-aware recomputation;
5. result/provenance identities;
6. EngineeringContext access to materials, resources, regions, and fields;
7. rule-driven CAD constraints;
8. Engineering Supervision revision APIs.
