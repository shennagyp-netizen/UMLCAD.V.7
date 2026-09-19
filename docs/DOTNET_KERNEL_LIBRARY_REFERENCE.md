# UMLCAD.V.7 — Current .NET Libraries Reference

Authoritative baseline: main at merge commit ccdf4a6b7122d945456432a91cfc7398fa1f9738.

Scope: production .NET libraries currently present under dotnet/src/.

This document describes current implementation only. Roadmaps and target-architecture documents remain separate.

IMPORTANT KERNEL BOUNDARY
The engineering-library integration was selective. It changed no kernel/** files and no existing UMLCAD.Kernel.Client source files.

---

# 1. Current production library set

dotnet/src currently contains exactly 12 production .NET libraries:

| Library | Project | Responsibility |
|---|---|---|
| UMLCAD.Framework | dotnet/src/UMLCAD.Framework | Application composition, authoring API, semantic application state, build identity, package generation, history and compiled semantic graph |
| UMLCAD.Kernel.Client | dotnet/src/UMLCAD.Kernel.Client | Existing .NET-to-Rust build/evaluation transport boundary |
| UMLCAD.Cad.Contracts | dotnet/src/UMLCAD.Cad.Contracts | Versioned CAD, kernel-geometry, sketch-solver and representation contracts |
| UMLCAD.Cad.Expressions | dotnet/src/UMLCAD.Cad.Expressions | Pure arithmetic expressions and deterministic expression identity |
| UMLCAD.Cad.Semantics | dotnet/src/UMLCAD.Cad.Semantics | Feature semantics, references, authoritative-result semantics and product structure/BOM |
| UMLCAD.Cad.Engine | dotnet/src/UMLCAD.Cad.Engine | Evaluation graph, planning, invalidation, execution, caching, references and result integration |
| UMLCAD.Science | dotnet/src/UMLCAD.Science | Quantities, dimensions, materials and phenomena-simulation contracts |
| UMLCAD.Engineering.Resources | dotnet/src/UMLCAD.Engineering.Resources | Machines, tools, processes, capabilities and compatibility |
| UMLCAD.Engineering.SheetMetal | dotnet/src/UMLCAD.Engineering.SheetMetal | Sheet-metal semantics, bend expressions and manufacturing validation |
| UMLCAD.Engineering.Cam | dotnet/src/UMLCAD.Engineering.Cam | CAM operations, toolpaths, postprocessing and deterministic NC/G-code |
| UMLCAD.Engineering.Drawing | dotnet/src/UMLCAD.Engineering.Drawing | Drawing semantics, views, dimensions, annotations, dress-up, BOM presentation and associativity |
| UMLCAD.Integration.Simulation | dotnet/src/UMLCAD.Integration.Simulation | Adapter boundary to external simulation applications |

Test projects are not included in this production-library count.

---

# 2. Global .NET policy

All current production projects target net10.0, enable nullable reference types, enable implicit usings and treat warnings as errors.

The SDK is pinned through dotnet/global.json.

The semantic/domain layer uses immutable records, explicit validation, deterministic ordering and failure-closed states.

The fundamental authority split is:

.NET = engineering meaning, contracts and orchestration
Rust = mathematical authority
Derived representations = non-authoritative products

---

# 3. UMLCAD.Cad.Contracts

Project: dotnet/src/UMLCAD.Cad.Contracts/UMLCAD.Cad.Contracts.csproj
Dependencies: none.

Purpose: stable versioned boundaries used by CAD evaluation and authoritative providers.

Source files:

~~~
AxisAlignedBoxSolidContracts.cs
CadEvaluationContracts.cs
CircularPrismSolidContracts.cs
ExtrusionContracts.cs
GeometryKernelStatus.cs
KernelContracts.cs
RepresentationContracts.cs
SketchSolveContracts.cs
UMLCAD.Cad.Contracts.csproj
~~~

Principal contract groups:

- CadId, CadResultId and TopologyEntityId
- CadFrame, CadVector3 and CadBoundingBox3
- ReferenceContext, TopologySelector, CadReference and ReferenceResolution
- Box solid request/result and IAuthoritativeGeometryService
- Circular-prism request/result and ICircularPrismGeometryService
- Convex planar extrusion request/result and IExtrusionGeometryService
- SketchSolveRequest, SketchSolveKernelResult and ISketchConstraintService
- Generic kernel request/result and input-binding contracts
- Representation identity, request and result contracts

CadFrame validates finite origin, unit orthogonal axes and right-handed orientation.

Reference states remain explicit: resolved, missing, ambiguous, indeterminate or unsupported.

These contracts describe boundaries; they do not implement a duplicate mathematical kernel.

---

# 4. UMLCAD.Cad.Expressions

Project: dotnet/src/UMLCAD.Cad.Expressions/UMLCAD.Cad.Expressions.csproj
Dependencies: none.

Source files:

~~~
ArithmeticExpression.cs
ExpressionIdentity.cs
UMLCAD.Cad.Expressions.csproj
~~~

Principal types:

- ExpressionNode
- ConstantExpression
- VariableExpression
- UnaryExpression
- BinaryExpression
- ExpressionIdentity

Expressions support deterministic canonicalization and numerical evaluation.

ExpressionIdentity is derived from SHA-256 over canonical expression structure.

Identity must not depend on timestamps, object addresses, rendering state or incidental collection order.

---

# 5. UMLCAD.Cad.Semantics

Project: dotnet/src/UMLCAD.Cad.Semantics/UMLCAD.Cad.Semantics.csproj
Dependencies: UMLCAD.Cad.Expressions.

Source files:

~~~
AuthoritativeResults.cs
FeatureSpecifications.cs
ProductStructure.cs
References.cs
UMLCAD.Cad.Semantics.csproj
~~~

Principal responsibilities:

## Feature semantics

Typed feature intent, including the current bounded box/sketch/profile/extrusion structures.

These are semantic definitions and are never Rust geometry objects.

## Reference semantics

Current concepts include:

- ReferenceTargetKind
- ReferenceResolutionStatus
- ReferenceContextId
- ReferencePathSegment
- ReferencePath
- CadReference
- SemanticReference
- GeometricReference
- TopologyReference
- ReferenceResolution
- TopologyEvolution
- TopologyEvolutionKind

Topology evolution distinguishes preserved, replaced, split, merged, removed, introduced and ambiguous states.

## Authoritative result semantics

Current concepts include:

- AuthoritativeResultIdentity
- AuthoritativeResultKind
- AuthoritativeResultStatus
- TopologyBinding
- ResultEvidence
- AuthoritativeCadResult

## Product structure/BOM

Current concepts include:

- SemanticId
- ProductComponent
- ProductOccurrence
- ProductDefinition
- BomLine
- BomService

BOM generation groups occurrences by component identity, aggregates quantities and uses deterministic ordering.

---

# 6. UMLCAD.Cad.Engine

Project: dotnet/src/UMLCAD.Cad.Engine/UMLCAD.Cad.Engine.csproj
Dependencies:

- UMLCAD.Cad.Expressions
- UMLCAD.Cad.Semantics
- UMLCAD.Cad.Contracts

Source files:

~~~
AsyncEvaluationEngine.cs
AuthoritativeCadReferenceResolver.cs
AxisAlignedBoxSolidEvaluator.cs
CadEvaluationEngine.cs
CadModelEvaluator.cs
ChangeSets.cs
ConvexProfileExtrusionEvaluator.cs
EvaluationCache.cs
EvaluationEngine.cs
EvaluationGraph.cs
EvaluationIdentityBuilder.cs
EvaluationStepFactory.cs
FeatureEvaluation.cs
IncrementalEvaluation.cs
ReferenceResolver.cs
ResultIntegration.cs
UMLCAD.Cad.Engine.csproj
~~~

Purpose: execute semantic-to-authoritative-result orchestration without embedding a concrete mathematical implementation.

Major concepts:

- EvaluationInputIdentity
- EvaluationStep
- EvaluationPlan
- EvaluationPlanner
- EvaluationIdentity
- EvaluationCache
- CadChangeSet
- IncrementalEvaluationPlan
- IncrementalEvaluationPlanner
- IAuthoritativeResultCatalog
- AuthoritativeResultCatalog
- ReferenceResolver
- ICadReferenceResolver
- AuthoritativeCadReferenceResolver
- EvaluationOutcome
- EvaluationEngine
- AsyncEvaluationEngine
- FeatureSpecificationCatalog
- IAuthoritativeFeatureEvaluator
- AxisAlignedBoxFeatureEvaluator
- ConvexProfileExtrusionFeatureEvaluator
- AuthoritativeFeatureStepExecutor
- FeatureEvaluationExecutorFactory
- ResultIntegrator

Evaluation identity is based on canonical semantic inputs, dependencies, configuration, tolerance and contract policy.

The cache is keyed by evaluation identity.

Incremental recomputation is an optimization and must remain semantically equivalent to full recomputation.

The Engine orchestrates authoritative operations; it does not implement B-Rep mathematics, NURBS mathematics or the numerical solver.

---

# 7. UMLCAD.Science

Project: dotnet/src/UMLCAD.Science/UMLCAD.Science.csproj
Dependencies: UMLCAD.Cad.Expressions.

Source files:

~~~
PhenomenaSimulationService.cs
ScienceFoundation.cs
UMLCAD.Science.csproj
~~~

Principal concepts:

- QuantityDimension
- Quantity
- MaterialFamily
- MaterialProperties
- Material
- PhenomenonKind
- PhenomenaSimulationRequest
- PhenomenaSimulationResult
- IPhenomenaSimulationProvider
- IPhenomenaSimulationService
- PhenomenaSimulationService

Quantity dimensions currently cover length, mass, time and temperature powers.

Material semantics provide shared physical properties.

Phenomena simulation is provider-neutral. This library defines the service boundary rather than becoming a general-purpose physics solver.

---

# 8. UMLCAD.Engineering.Resources

Project: dotnet/src/UMLCAD.Engineering.Resources/UMLCAD.Engineering.Resources.csproj
Dependencies:

- UMLCAD.Cad.Expressions
- UMLCAD.Science

Source file:

~~~
EngineeringResources.cs
~~~

Principal concepts:

- MachineKind
- ToolKind
- ManufacturingProcessKind
- MachineCapability
- ToolDefinition
- MachineDefinition
- MachineProcessCompatibility
- ToolProcessCompatibility
- MachineToolCompatibility

Current machine categories include machining center, lathe, wire EDM, press brake, laser cutter, waterjet and grinding machine.

Current manufacturing process categories include milling, turning, wire EDM cutting, sheet-metal bending, laser cutting, waterjet cutting and grinding.

Compatibility is fail-closed.

Machine capability checks process and stock-thickness range.

Tool checks include interface and diameter constraints.

---

# 9. UMLCAD.Engineering.SheetMetal

Project: dotnet/src/UMLCAD.Engineering.SheetMetal/UMLCAD.Engineering.SheetMetal.csproj
Dependencies:

- UMLCAD.Cad.Expressions
- UMLCAD.Cad.Semantics
- UMLCAD.Science
- UMLCAD.Engineering.Resources

Source files:

~~~
SheetMetalExpressions.cs
SheetMetalSemantics.cs
UMLCAD.Engineering.SheetMetal.csproj
~~~

Principal concepts:

- SheetMetalPartDefinition
- BendDefinition
- SheetMetalValidationResult
- SheetMetalValidator

The current bend-allowance expression is represented through the shared expression AST using the relationship:

((pi / 180) * (R + (K * T)) * A)

Validation connects material, thickness, ductility, bend radius and manufacturing-resource compatibility.

Current scope is a semantic/manufacturing foundation.

It is not yet a complete exact sheet-metal geometric engine for all flange, relief, folding, unfolding and flat-pattern B-Rep behavior.

---

# 10. UMLCAD.Engineering.Cam

Project: dotnet/src/UMLCAD.Engineering.Cam/UMLCAD.Engineering.Cam.csproj
Dependencies:

- UMLCAD.Cad.Expressions
- UMLCAD.Cad.Semantics
- UMLCAD.Science
- UMLCAD.Engineering.Resources

Source files:

~~~
CamAndGCode.cs
UMLCAD.Engineering.Cam.csproj
~~~

Principal concepts:

- ToolpathPoint
- ManufacturingOperation
- NcProgram
- INcPostprocessor
- DeterministicGCodePostprocessor
- CamPhenomenaAdvisor

NcProgram supports deterministic serialization and SHA-256 content hashing.

DeterministicGCodePostprocessor currently emits bounded milling G-code after machine/tool/process compatibility checks.

Current output semantics include metric units, absolute positioning, tool identification, first-point rapid motion, subsequent linear moves and program termination.

CAM may use Science phenomena services but does not own a second simulation engine.

Current scope is a CAM manufacturing foundation, not a complete B-Rep-driven toolpath planner for every machining strategy.

---

# 11. UMLCAD.Engineering.Drawing

Project: dotnet/src/UMLCAD.Engineering.Drawing/UMLCAD.Engineering.Drawing.csproj
Dependencies:

- UMLCAD.Cad.Expressions
- UMLCAD.Cad.Semantics
- UMLCAD.Science

Source files:

~~~
DrawingAdvancedSemantics.cs
DrawingSemantics.cs
UMLCAD.Engineering.Drawing.csproj
~~~

Current drafting vocabulary includes:

- DrawingStandard: ISO, ANSI, JIS
- DrawingViewKind: orthographic, isometric, auxiliary, section, detail, clipping, broken, unfolded and related modes
- DimensionKind: linear, angular, radius, diameter, coordinate, baseline, chain
- AnnotationKind: text, note, leader, balloon, datum, datum target, geometric tolerance, surface roughness, welding symbol and flag note
- DressUpKind: centerline, axis, symmetry line, thread line, hatch/fill, break line and markup

Principal semantic types include:

- DrawingSheet
- DrawingView
- DrawingDimension
- DrawingAnnotation
- DrawingCapabilityMatrix
- MechanicalDraftingCapabilityProfile
- DrawingAssociativityState
- ViewDisplayMode
- ViewAxis
- DrawingViewSpecification
- DrawingBomItem
- DrawingBomTable
- DrawingSheetPresentation

Drawing views are associated with authoritative CAD-result identities.

Associativity explicitly supports Associative, NeedsUpdate, MissingReference, AmbiguousReference and Unsupported states.

Current scope is drawing meaning and association. Complete projection, section and hidden-line mathematical generation remains separate work.

---

# 12. UMLCAD.Integration.Simulation

Project: dotnet/src/UMLCAD.Integration.Simulation/UMLCAD.Integration.Simulation.csproj
Dependencies: UMLCAD.Science.

Source files:

~~~
SimulationApplicationAdapter.cs
UMLCAD.Integration.Simulation.csproj
~~~

Principal concepts:

- ISimulationApplicationAdapter
- SimulationApplicationProvider

The provider delegates capability and simulation execution to an external application adapter.

Results are checked against request identity, phenomenon identity and provider identity.

This library is the external-provider boundary and does not own simulation mathematics.

---

# 13. UMLCAD.Framework

Project: dotnet/src/UMLCAD.Framework/UMLCAD.Framework.csproj

External Microsoft.Extensions dependencies:

- Configuration
- Configuration.Abstractions
- DependencyInjection
- DependencyInjection.Abstractions
- Hosting.Abstractions
- Options

Source files currently on main:

~~~
CadApplication.cs
Configuration/CadConfiguration.cs
Semantics/BuildHistory.cs
Semantics/BuildPackage.cs
Semantics/CompiledModel.cs
Semantics/ISemanticRegistry.cs
Semantics/SemanticEntity.cs
Semantics/SemanticModels.cs
Semantics/SemanticServices.cs
UMLCAD.Framework.csproj
~~~

Principal responsibilities:

- CadApplication and CadApplicationBuilder
- PartBuilder
- DrawingBuilder
- AssemblyBuilder
- AssemblyOccurrenceBuilder
- semantic snapshot publication
- semantic lookup services
- deterministic build identity
- build history
- BuildPackage creation
- compiled semantic graph generation
- configuration abstraction

Core semantic types include:

- SemanticEntity
- ParameterSemantic
- CadMetadata
- GeometrySemantic
- ConstraintSemantic
- ComponentSemantic
- TransformSemantic
- AssemblyOccurrenceSemantic
- PartSemantic
- SheetSemantic
- DrawingSemantic
- AssemblySemantic
- SemanticApplication

Core application services include:

- ISemanticApplication
- IPartSemanticService
- IDrawingSemanticService
- ISheetSemanticService
- IAssemblySemanticService
- ISemanticRegistry
- IBuildHistory
- IBuildPackageService
- ICompiledModelService
- ICadConfiguration

BuildPackage currently uses the uml-cad-build-package/1.0.0 bridge contract.

CompiledModelPackage and CompiledModelManifest represent a compiled semantic graph plus derived representation metadata.

Framework remains application/build infrastructure. It is not the newer mathematical evaluation kernel.

---

# 14. UMLCAD.Kernel.Client

Project: dotnet/src/UMLCAD.Kernel.Client/UMLCAD.Kernel.Client.csproj

Current project dependency on main:

- UMLCAD.Framework

Source files currently on main:

~~~
RustKernelService.cs
UMLCAD.Kernel.Client.csproj
~~~

This is intentionally the existing pre-S1 transport library. The S1-only Rust geometry adapter files were not imported by the engineering-library merge.

Principal types:

- RustKernelOptions
- IRustKernelService
- KernelEvaluationResult
- KernelDiagnostic
- RustKernelService
- CompiledModelValidator
- RustKernelServiceCollectionExtensions

Current endpoint configuration:

- BaseAddress defaults to http://localhost:8080/
- EvaluatePath defaults to v1/build/evaluate
- RequestTimeout defaults to two minutes
- MaxResponseBytes defaults to 256 MiB

RustKernelService responsibilities:

1. submit BuildPackage;
2. enforce timeout/cancellation policy;
3. bound response size;
4. handle transport/HTTP failure;
5. reject empty or invalid JSON;
6. validate returned compiled-model identity and graph consistency;
7. return structured diagnostics.

The client does not own CAD semantics, feature evaluation, B-Rep algorithms, solver mathematics, drawing, CAM or simulation.

---

# 15. Current project dependency map

~~~
UMLCAD.Cad.Expressions
    -> none

UMLCAD.Cad.Contracts
    -> none

UMLCAD.Cad.Semantics
    -> UMLCAD.Cad.Expressions

UMLCAD.Cad.Engine
    -> UMLCAD.Cad.Expressions
    -> UMLCAD.Cad.Semantics
    -> UMLCAD.Cad.Contracts

UMLCAD.Science
    -> UMLCAD.Cad.Expressions

UMLCAD.Engineering.Resources
    -> UMLCAD.Cad.Expressions
    -> UMLCAD.Science

UMLCAD.Engineering.SheetMetal
    -> UMLCAD.Cad.Expressions
    -> UMLCAD.Cad.Semantics
    -> UMLCAD.Science
    -> UMLCAD.Engineering.Resources

UMLCAD.Engineering.Cam
    -> UMLCAD.Cad.Expressions
    -> UMLCAD.Cad.Semantics
    -> UMLCAD.Science
    -> UMLCAD.Engineering.Resources

UMLCAD.Engineering.Drawing
    -> UMLCAD.Cad.Expressions
    -> UMLCAD.Cad.Semantics
    -> UMLCAD.Science

UMLCAD.Integration.Simulation
    -> UMLCAD.Science

UMLCAD.Framework
    -> Microsoft.Extensions.*

UMLCAD.Kernel.Client
    -> UMLCAD.Framework
~~~

This table reflects the current project references in the source projects on main.

---

# 16. Current data flow

~~~
Authoring
    ↓
UMLCAD.Framework
    ↓
SemanticApplication
    ↓
BuildPackage
    ↓
UMLCAD.Kernel.Client
    ↓
existing Rust process boundary
    ↓
Rust mathematical authority
    ↓
validated result
~~~

Engineering-domain flow:

~~~
CAD semantics
    ↓
UMLCAD.Cad.Engine
    ↓
authoritative CAD result
    ├── UMLCAD.Engineering.Drawing
    ├── UMLCAD.Engineering.Cam
    ├── UMLCAD.Engineering.SheetMetal
    ├── UMLCAD.Engineering.Resources
    └── UMLCAD.Science / UMLCAD.Integration.Simulation
~~~

The engineering layer composes around mathematical authority; it does not replace it.

---

# 17. Identity and determinism

Identity levels are intentionally distinct:

1. Semantic identity — identifies an engineering object or definition.
2. Evaluation identity — identifies exact semantic evaluation inputs.
3. Authoritative result identity — identifies a mathematically certified result.
4. Topology identity — identifies topology within a result.
5. Representation identity — identifies a derived representation.
6. Manufacturing program identity — identifies deterministic NC content where applicable.

Identity-bearing operations should use canonical serialization, stable ordering, ordinal comparison, explicit contracts and SHA-256 where hashing is required.

Forbidden identity inputs include timestamps, object addresses, viewer state and incidental execution ordering.

---

# 18. Failure-closed behavior

Current libraries preserve explicit failure/uncertainty states such as:

- invalid specification;
- missing reference;
- ambiguous reference;
- indeterminate result;
- unsupported capability;
- kernel/backend failure;
- incompatible machine/tool/process;
- stale drawing association;
- invalid external simulation response.

The rule is:

DO NOT silently fabricate authoritative engineering meaning when the responsible provider cannot prove it.

---

# 19. Current implementation status

Implemented foundations on main now include:

- versioned CAD contracts;
- deterministic expressions;
- CAD semantics and references;
- product structure/BOM foundation;
- evaluation planning/cache/invalidation primitives;
- authoritative-result integration structures;
- science/material/phenomena foundation;
- engineering resource semantics;
- sheet-metal engineering semantics;
- CAM operation and deterministic NC/G-code foundation;
- drawing semantics and associativity;
- external simulation adapter boundary.

This does not claim that all final product capabilities are complete.

Examples of broader work that remain separate include:

- general exact Boolean feature evolution;
- complete feature-tree families;
- complete topology/reference migration;
- full generative drawing projection;
- full sheet-metal unfolding/B-Rep behavior;
- full B-Rep-driven CAM planning;
- complete PMI/GD&T execution;
- PLM/PDM lifecycle storage;
- broader simulation-provider implementations.

---

# 20. Exact current source inventory

## UMLCAD.Cad.Contracts

~~~
AxisAlignedBoxSolidContracts.cs
CadEvaluationContracts.cs
CircularPrismSolidContracts.cs
ExtrusionContracts.cs
GeometryKernelStatus.cs
KernelContracts.cs
RepresentationContracts.cs
SketchSolveContracts.cs
UMLCAD.Cad.Contracts.csproj
~~~

## UMLCAD.Cad.Expressions

~~~
ArithmeticExpression.cs
ExpressionIdentity.cs
UMLCAD.Cad.Expressions.csproj
~~~

## UMLCAD.Cad.Semantics

~~~
AuthoritativeResults.cs
FeatureSpecifications.cs
ProductStructure.cs
References.cs
UMLCAD.Cad.Semantics.csproj
~~~

## UMLCAD.Cad.Engine

~~~
AsyncEvaluationEngine.cs
AuthoritativeCadReferenceResolver.cs
AxisAlignedBoxSolidEvaluator.cs
CadEvaluationEngine.cs
CadModelEvaluator.cs
ChangeSets.cs
ConvexProfileExtrusionEvaluator.cs
EvaluationCache.cs
EvaluationEngine.cs
EvaluationGraph.cs
EvaluationIdentityBuilder.cs
EvaluationStepFactory.cs
FeatureEvaluation.cs
IncrementalEvaluation.cs
ReferenceResolver.cs
ResultIntegration.cs
UMLCAD.Cad.Engine.csproj
~~~

## UMLCAD.Science

~~~
PhenomenaSimulationService.cs
ScienceFoundation.cs
UMLCAD.Science.csproj
~~~

## UMLCAD.Engineering.Resources

~~~
EngineeringResources.cs
UMLCAD.Engineering.Resources.csproj
~~~

## UMLCAD.Engineering.SheetMetal

~~~
SheetMetalExpressions.cs
SheetMetalSemantics.cs
UMLCAD.Engineering.SheetMetal.csproj
~~~

## UMLCAD.Engineering.Cam

~~~
CamAndGCode.cs
UMLCAD.Engineering.Cam.csproj
~~~

## UMLCAD.Engineering.Drawing

~~~
DrawingAdvancedSemantics.cs
DrawingSemantics.cs
UMLCAD.Engineering.Drawing.csproj
~~~

## UMLCAD.Integration.Simulation

~~~
SimulationApplicationAdapter.cs
UMLCAD.Integration.Simulation.csproj
~~~

## UMLCAD.Framework

~~~
CadApplication.cs
Configuration/CadConfiguration.cs
Semantics/BuildHistory.cs
Semantics/BuildPackage.cs
Semantics/CompiledModel.cs
Semantics/ISemanticRegistry.cs
Semantics/SemanticEntity.cs
Semantics/SemanticModels.cs
Semantics/SemanticServices.cs
UMLCAD.Framework.csproj
~~~

## UMLCAD.Kernel.Client

~~~
RustKernelService.cs
UMLCAD.Kernel.Client.csproj
~~~

---

# 21. Maintenance rule

When dotnet/src changes, this reference must be updated with the same architectural intent:

- library inventory;
- project references;
- source-file inventory;
- public semantic contracts;
- responsibility/ownership;
- implementation status;
- authority boundaries.

This file documents current implementation. Future library names or target architecture must not be presented here as current production code.

---

# 22. Final authority statement

The current production .NET stack is:

~~~
UMLCAD.Framework
    = application/build foundation

UMLCAD.Cad.Contracts
    = versioned CAD/kernel/representation boundaries

UMLCAD.Cad.Expressions
    = deterministic expression semantics

UMLCAD.Cad.Semantics
    = CAD meaning, references, authoritative-result semantics and product structure

UMLCAD.Cad.Engine
    = evaluation orchestration

UMLCAD.Science
    = scientific/material/phenomena foundation

UMLCAD.Engineering.Resources
    = engineering resource semantics

UMLCAD.Engineering.SheetMetal
    = sheet-metal engineering semantics

UMLCAD.Engineering.Cam
    = CAM semantics and deterministic NC/G-code foundation

UMLCAD.Engineering.Drawing
    = drawing semantics

UMLCAD.Integration.Simulation
    = external simulation adapter boundary

UMLCAD.Kernel.Client
    = existing transport boundary

Rust
    = mathematical authority
~~~

The .NET engineering libraries are therefore documented as the current systems-engineering layer represented by dotnet/src on main, while preserving the existing mathematical-kernel authority.
