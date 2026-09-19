# UMLCAD.V.7 — Detailed .NET Library Reference

> **Scope:** .NET libraries only.  
> **Baseline:** `milestone/cad-system-s1-vertical-slice`.  
> **Purpose:** exhaustive library-level documentation of the current System-CAD .NET implementation, including project boundaries, dependencies, source files, types, public contracts, validation rules, algorithms, integration points, and current limitations.
>
> This file is **additional documentation**. It does not replace, modify, or supersede the existing architecture, roadmap, TDD, handoff, or mathematical-authority documents.

---

## 1. Library landscape

The current System-CAD .NET layer is intentionally split into small libraries with explicit responsibility boundaries.

| Library | Responsibility | Project dependencies |
|---|---|---|
| `UMLCAD.Cad.Contracts` | Versioned CAD/kernel/evaluation/representation contracts | None |
| `UMLCAD.Cad.Expressions` | Immutable arithmetic expression AST and canonical expression identity | None |
| `UMLCAD.Cad.Semantics` | Immutable CAD semantics, references, feature specifications, product structure, authoritative results | Expressions |
| `UMLCAD.Cad.Engine` | Specification graph, reference resolution, evaluation planning, execution, cache, incremental recomputation, result integration | Expressions, Semantics, Contracts |
| `UMLCAD.Science` | Quantities, dimensions, materials, phenomena simulation contracts and service | Expressions |
| `UMLCAD.Engineering.Resources` | Machines, tools, manufacturing processes, capabilities and compatibility | Expressions, Science |
| `UMLCAD.Engineering.SheetMetal` | Sheet-metal semantic rules, bend semantics and manufacturing validation | Expressions, Semantics, Science, Resources |
| `UMLCAD.Engineering.Cam` | CAM operations, toolpaths, deterministic NC/G-code and phenomena integration | Expressions, Semantics, Science, Resources |
| `UMLCAD.Engineering.Drawing` | Associative drawing semantics, views, dimensions, annotations, dress-up, BOM and sheet presentation | Expressions, Semantics, Science |
| `UMLCAD.Integration.Simulation` | Adapter boundary from UMLCAD phenomena contracts to external simulation applications | Science |
| `UMLCAD.Framework` | Application composition, semantic authoring, build/package/compiled-model foundation | Microsoft.Extensions packages |
| `UMLCAD.Kernel.Client` | Outward adapter to the Rust mathematical authority and native geometry/sketch services | Contracts + transitional Framework dependency |

The intended dependency direction is:

```
Expressions
     ↓
Semantics / Science
     ↓
Contracts
     ↓
Engine
     ↓
application/domain composition
     ↓
Kernel.Client / external providers
```

The exact project references are authoritative for the current dependency graph. Architectural rules additionally prohibit the CAD Engine from depending directly on the concrete Rust client.

---

# 2. Global .NET conventions

## 2.1 Target framework and compiler policy

All current System-CAD projects target:

- `net10.0`
- nullable reference types enabled
- implicit usings enabled
- warnings treated as errors

The solution-level SDK is pinned by `dotnet/global.json`.

## 2.2 Record-first immutable domain model

The semantic and contract layer predominantly uses:

- `record`
- `record struct`
- immutable/read-only collections
- constructor validation
- deterministic ordering where identity or serialization depends on collections

The intent is to prevent accidental semantic mutation and make semantic identity suitable for hashing and caching.

## 2.3 Failure-closed behavior

The .NET layer distinguishes:

- success;
- invalid specification/input;
- missing reference;
- ambiguous reference;
- ambiguous evaluation;
- indeterminate result;
- unsupported capability;
- kernel/backend failure.

A consumer must not silently reinterpret a non-success result as a valid CAD result.

## 2.4 Authority split

The .NET layer owns CAD meaning and orchestration.

The Rust kernel owns certified mathematical execution.

The viewer and derived representations are not mathematical authorities.

---

# 3. UMLCAD.Cad.Contracts

**Project:** `dotnet/src/UMLCAD.Cad.Contracts`

**Purpose:** versioned contracts between the System-CAD evaluation layer and mathematical/derived providers.

**Dependencies:** none.

This library must remain small and stable. It is the boundary that allows domain libraries and evaluation orchestration to remain independent from a concrete kernel implementation.

## 3.1 Source files

- `AxisAlignedBoxSolidContracts.cs`
- `CadEvaluationContracts.cs`
- `CircularPrismSolidContracts.cs`
- `ExtrusionContracts.cs`
- `GeometryKernelStatus.cs`
- `KernelContracts.cs`
- `RepresentationContracts.cs`
- `SketchSolveContracts.cs`

## 3.2 AxisAlignedBoxSolidContracts

### `KernelVector3`

Immutable 3D numerical vector used by contract DTOs.

Properties:

- `X`
- `Y`
- `Z)

Derived members:

- `IsFinite`
- `Length`

The finite predicate is based on all three coordinates. The length is Euclidean.

### `KernelTolerance`

Stores:

- `Absolute`
- `Relative`

It is passed explicitly into authoritative operations rather than hidden inside an implementation.

### `AxisAlignedBoxSolidRequest`

Fields:

- `OperationIdentity`
- `Min`
- `Max`
- `Tolerance`
- `ContractVersion`

Stable constants:

- Contract ID: `UMLCAD.Geometry.AxisAlignedBoxSolid`
- Schema: `uml-cad-axis-aligned-box-solid/1.0.0`

The request identifies both the operation and the contract version.

### `AxisAlignedBoxSolidKernelTopology`

One returned topological item:

- `Kind`
- `Key`

### `AxisAlignedBoxSolidKernelResult`

Contains:

- `Status`
- `ResultId`
- `EvidenceHash`
- `Topology`
- `Volume`
- `SurfaceArea`
- `Centroid`
- `Diagnostics`

The result is data plus evidence; the caller can inspect both the geometric measures and the certified response state.

### `IAuthoritativeGeometryService`

Asynchronous abstraction for authoritative axis-aligned-box evaluation.

Implementations live outside Contracts.

---

## 3.3 CadEvaluationContracts

This file is the broader current System-CAD contract surface.

### `CadContractVersions`

Contract identifiers:

- `KernelEvaluation = uml-cad-kernel-evaluation/1.0.0`
- `Evaluation = uml-cad-evaluation/1.0.0`
- `Representation = uml-cad-representation/1.0.0`

### Identity value types

#### `CadId`

String-backed semantic identity with:

- `Value`
- `IsValid`

#### `CadResultId`

String-backed result identity with:

- `Value`
- `IsValid`

#### `TopologyEntityId`

String-backed topology identity with:

- `Value`
- `IsValid`

### `CadFrameKind`

Supported semantic frame categories:

- World
- Document
- Part
- Body
- Sketch
- Face
- Occurrence
- DrawingView
- Simulation

### `ReferenceKind`

- Geometric
- Topology
- Support
- Publication

### `ReferenceResolutionStatus`

- Resolved
- Missing
- Ambiguous
- Indeterminate
- Unsupported

### `TopologyEntityKind`

- Solid
- Shell
- Face
- Edge
- Vertex

### `FeatureBooleanOperation`

- Add
- Remove

This is semantic operation intent, not a promise that every Boolean operation is currently implemented.

### `SketchConstraintKind`

- Fixed
- FullyConstrained

### `CadEvaluationStatus`

- Succeeded
- InvalidSpecification
- MissingReference
- AmbiguousReference
- AmbiguousEvaluation
- Indeterminate
- Unsupported
- KernelFailure

### `CadFrame`

Represents an explicit right-handed orthonormal coordinate frame.

State:

- `Id`
- `Kind`
- `OriginX`
- `OriginY`
- `OriginZ`
- `XAxis`
- `YAxis`
- `ZAxis`

Important methods:

- `Validate()`
- `ToWorldPoint()`
- `ToWorldVector()`
- `ToLocalVector()`
- `ToLocalPoint()`

Validation guarantees:

1. frame ID is valid;
2. origin coordinates are finite;
3. all basis vectors are finite;
4. each basis vector has unit length to the defined tolerance;
5. basis vectors are mutually orthogonal;
6. X × Y = Z within the same tolerance.

This prevents a generic transformation matrix from being used where a specific CAD reference frame is expected.

### `CadVector3`

Properties:

- X, Y, Z
- IsFinite
- Length

Operation:

- `Normalize(name)`

Normalization fails for non-finite or zero-length vectors.

### `CadBoundingBox3`

Stores six extrema:

- MinX, MinY, MinZ
- MaxX, MaxY, MaxZ

`Validate()` rejects non-finite values and inverted intervals.

### `ReferenceContext`

Carries:

- frame kind;
- configuration;
- optional expected result identity;
- optional occurrence path.

Validation requires configuration and rejects an explicitly empty occurrence path.

### `TopologySelector`

Carries:

- entity kind;
- selector kind;
- parameter dictionary.

The helper `PlanarFaceByNormalAndPoint()` normalizes the supplied normal and canonicalizes its floating-point values using invariant round-trip formatting.

### `CadReference`

A concrete reference includes:

- reference ID;
- kind;
- target specification ID;
- topology selector;
- reference context.

`Validate()` validates the identity, selector and context.

### `ReferenceResolution`

Carries:

- reference;
- status;
- candidate topology IDs;
- optional diagnostic code;
- optional evidence.

Derived property:

- `IsResolved` is true only for status Resolved with exactly one candidate.

### `CadFeatureSpecification`

Abstract root for feature semantics.

Current extensible base identity:

- `Id`

Virtual validation ensures the feature ID is valid.

### `BoxFeatureSpecification`

Adds:

- frame;
- width;
- depth;
- height.

Validation requires:

- valid ID;
- valid frame;
- finite positive dimensions.

### `SketchCircle`

Stores:

- ID
- X
- Y
- Radius

Radius must be finite and strictly positive.

### `SketchConstraintSpecification`

Stores:

- ID
- constraint kind
- referenced geometry ID

Constraint identity and referenced geometry identity must be valid.

### `SketchFeatureSpecification`

Stores:

- support reference;
- frame;
- circles;
- constraints.

Validation requires:

- valid support and frame;
- at least one circle;
- unique circle IDs;
- unique constraint IDs;
- every constraint resolves to a circle in this sketch.

This is the current semantic boundary for the S1 circular-sketch slice.

### `ExtrusionFeatureSpecification`

Stores:

- profile sketch ID;
- support reference;
- direction;
- distance;
- Boolean operation.

Validation requires:

- valid profile sketch identity;
- valid support reference;
- finite non-zero direction;
- finite positive distance.

### `CadDocumentSpecification`

Stores:

- document ID;
- revision;
- configuration;
- ordered feature specifications;
- evaluation tolerance.

Validation rejects duplicate feature IDs and invalid document identity/configuration.

Default evaluation tolerance is:

```
absolute = 1e-9
relative = 1e-9
```

### `CadDiagnostic`

Carries:

- code;
- status;
- message.

### `TopologyProvenance`

Maps a result topology entity back to:

- producing feature;
- source result;
- evolution kind;
- source entities.

### `TopologyEntityResult`

Stores:

- topology ID;
- entity kind;
- optional normal;
- optional point;
- optional measure;
- use count;
- provenance.

Validation rejects invalid IDs, negative use counts, non-finite optional geometry values, and invalid provenance.

### `TopologySnapshot`

Carries:

- kernel contract version;
- topology entity list.

Validation requires the exact current kernel evaluation contract version and unique topology IDs.

Convenience member:

- `Faces`

### `AuthoritativeCadResult`

Carries:

- result ID;
- kernel contract version;
- bounding box;
- volume;
- surface area;
- topology snapshot.

Validation makes the result a strict authoritative object rather than an arbitrary DTO.

### `CadFeatureEvaluationResult`

Carries:

- feature identity;
- evaluation identity;
- evaluation status;
- optional result ID;
- reference resolutions;
- diagnostics;
- optional authoritative result;
- optional sketch-solver result.

Derived:

- `Succeeded`

### `KernelEvaluationRequest`

Carries:

- evaluation ID;
- feature specification;
- optional upstream authoritative result;
- resolved references;
- upstream evaluations;
- explicit tolerance.

### `KernelEvaluationResponse`

Carries:

- status;
- optional authoritative result;
- diagnostics;
- optional sketch-solver result.

### `ICadKernelEvaluator`

Asynchronous evaluation contract implemented by the Kernel Client adapter.

### `RecomputeMode`

- Full
- Incremental

### `CadRepresentation`

Derived representation metadata:

- contract version;
- representation ID;
- source result ID;
- representation kind;
- display policy;
- content identity.

### `CadEvaluationResult`

Top-level evaluation response:

- contract version;
- document ID;
- evaluation identity;
- recompute mode;
- evaluation plan;
- invalidated feature IDs;
- feature results;
- final authoritative result;
- optional representation;
- optional failure diagnostic.

The success predicate requires no failure, all feature evaluations successful, and a final authoritative result.

---

## 3.4 CircularPrismSolidContracts

### `CircularPrismSolidRequest`

Stores:

- operation identity;
- origin;
- axis;
- radius;
- depth;
- tolerance;
- contract version.

Constants:

- `UMLCAD.Geometry.CircularPrismSolid`
- `uml-cad-circular-prism-solid/1.0.0`

### `CircularPrismKernelTopology`

Stores a topology `Kind` and `Key`.

### `CircularPrismSolidKernelResult`

Stores:

- status;
- success flag;
- result ID;
- evidence hash;
- origin;
- axis;
- radius;
- depth;
- volume;
- surface area;
- centroid;
- bounds;
- topology;
- diagnostics.

### `ICircularPrismGeometryService`

Authoritative circular-prism service abstraction.

This is the narrow analytical-cylinder boundary required for the current circular-profile vertical slice. It does not imply support for arbitrary general-purpose solid Boolean operations.

---

## 3.5 ExtrusionContracts

### `PlanarProfilePoint`

Stores two-dimensional profile coordinates:

- U
- V

### `ExtrusionRequest`

Stores:

- operation identity;
- origin;
- U direction;
- V direction;
- ordered planar profile;
- depth;
- tolerance;
- contract version.

Contract ID:

`UMLCAD.Geometry.ExtrudeConvexPlanarProfile`

Schema:

`uml-cad-extrude-convex-planar-profile/1.0.0`

### `ExtrusionTopology`

Stores topology kind and key.

### `ExtrusionKernelResult`

Stores:

- status;
- result ID;
- evidence hash;
- topology;
- volume;
- surface area;
- centroid;
- diagnostics.

### `IExtrusionGeometryService`

Authoritative convex-planar-profile extrusion abstraction.

The certified implementation scope is deliberately narrow rather than pretending to be a universal sweep engine.

---

## 3.6 GeometryKernelStatus

Enum:

- Succeeded
- Failed
- Unsupported
- Ambiguous
- Indeterminate

All authoritative geometry adapters use this vocabulary.

---

## 3.7 KernelContracts

### `ContractVersion`

Immutable string value object.

Provides:

- `Value`
- `ToString()`

### `ContractResultId`

Immutable result identity value object.

### `ContractTopologyId`

Composite identity:

- result ID;
- topology kind;
- topology key.

### `KernelOperationKind`

Current operation families:

- EvaluateCurve
- EvaluateSurface
- Intersect
- Project
- BuildWire
- BuildSurface
- BuildSolid
- ExtrudeConvexPlanarProfile
- Boolean
- Tessellate
- SolveConstraints

### `KernelInputBinding`

Carries:

- role;
- optional result identity;
- optional topology identity;
- geometry contract key.

### `KernelRequest`

Generic request envelope:

- contract ID;
- contract version;
- operation;
- operation identity;
- inputs;
- canonical parameters.

### `KernelResultStatus`

Generic kernel result state.

### `KernelResult`

Generic result envelope:

- optional result ID;
- status;
- topology identities;
- evidence hash;
- diagnostics.

---

## 3.8 RepresentationContracts

### `RepresentationIdentity`

String identity value object.

### `RepresentationKind`

The representation taxonomy used to distinguish derived products from authoritative CAD results.

### `RepresentationBuildStatus`

Represents representation build state separately from CAD-result state.

### `RepresentationRequest`

Carries:

- representation kind;
- source result ID;
- display policy;
- representation policy version.

### `RepresentationResult`

Carries:

- representation identity;
- build status;
- source result;
- content hash;
- diagnostics.

Representation identity is deliberately distinct from authoritative-result identity.

---

## 3.9 SketchSolveContracts

### `SketchKernelCircle`

Stores:

- ID;
- X;
- Y;
- Radius.

### `SketchKernelFixedConstraint`

Stores:

- ID;
- GeometryId.

### `KernelSolveOptions`

Stores:

- MaxIterations;
- ResidualTolerance;
- StepTolerance;
- InitialDamping.

A static `Default` configuration exists.

### `SketchSolveRequest`

Stores:

- operation identity;
- circles;
- fixed constraints;
- tolerance;
- solve options.

Contract ID:

`UMLCAD.Geometry.SketchSolve`

Schema:

`uml-cad-sketch-solve/1.0.0`

### `SketchSolvedCircle`

Returned solved geometry item.

### `SketchSolveKernelResult`

Carries:

- status;
- success flag;
- reason;
- iteration count;
- initial/final residual norms;
- scaled residual norms;
- final step norm;
- variable count;
- equation count;
- rank;
- degrees of freedom;
- condition estimate;
- solved geometry;
- diagnostics.

The constructor explicitly verifies consistency between status and the success flag and rejects invalid numerical diagnostics.

### `ISketchConstraintService`

Asynchronous externalized sketch-solver boundary.

---

# 4. UMLCAD.Cad.Expressions

**Project:** `dotnet/src/UMLCAD.Cad.Expressions`

**Dependencies:** none.

**Purpose:** pure immutable expression semantics shared by CAD, engineering and manufacturing logic.

## 4.1 Source files

- `ArithmeticExpression.cs`
- `ExpressionIdentity.cs`

## 4.2 Operator enums

### `UnaryOperator`

Defines the currently supported unary arithmetic operators.

### `BinaryOperator`

Defines the currently supported binary arithmetic operators.

## 4.3 `ExpressionNode`

Abstract immutable expression node.

Mandatory operations:

- `Evaluate(IReadOnlyDictionary<string,double>)`
- `ToCanonicalString()`

All expression identities must be derived from semantic structure rather than object identity.

## 4.4 `ConstantExpression`

Stores a numeric constant.

Validation rejects non-finite values.

Canonicalization preserves a deterministic numeric representation.

## 4.5 `VariableExpression`

Stores:

- variable `Name`

Canonical form is exactly the variable name.

Evaluation requires the variable to exist in the supplied variable dictionary.

## 4.6 `UnaryExpression`

Stores:

- operator;
- operand.

Canonical representation is deterministic and parenthesized.

## 4.7 `BinaryExpression`

Stores:

- operator;
- left operand;
- right operand.

Canonical representation is fully parenthesized.

For example:

```
(a + (b * c))
```

This avoids precedence-dependent textual ambiguity.

Evaluation fails closed for invalid operations such as division by zero.

## 4.8 `ExpressionIdentity`

Static identity helper.

The expression identity is derived from a SHA-256 hash of its canonical representation.

This identity is suitable for:

- cache keys;
- deterministic references to expressions;
- semantic comparison;
- downstream manufacturing formulas.

The identity must never contain timestamps, object addresses, or rendering state.

---

# 5. UMLCAD.Cad.Semantics

**Project:** `dotnet/src/UMLCAD.Cad.Semantics`

**Dependencies:** Expressions.

**Purpose:** immutable CAD meaning independent of execution backend.

## 5.1 Source files

- `AuthoritativeResults.cs`
- `FeatureSpecifications.cs`
- `ProductStructure.cs`
- `References.cs`

## 5.2 Identity — `SemanticId`

GUID-backed value type.

Operations:

- create a new identity;
- deterministic string formatting.

The current `New()` helper uses a fresh GUID.

## 5.3 Feature specifications

### `FeatureSpecification`

Abstract semantic feature root.

Stores:

- `FeatureId`
- `PartId`
- `Name`
- `Dependencies`

Abstract members:

- `OperationKind`
- `CanonicalDefinition`

This class is semantic intent, not an execution object.

### `AxisAlignedBoxSolidSpecification`

Stores six box bounds:

- MinXmm
- MinYmm
- MinZmm
- MaxXmm
- MaxYmm
- MaxZmm

Operation kind:

`PartDesign.AxisAlignedBoxSolid`

Canonical definition deterministically serializes the geometry.

### `SemanticVector3`

Immutable semantic 3D vector:

- X
- Y
- Z

### `SketchProfilePoint`

Planar profile point:

- U
- V

### `ConvexSketchProfileDefinition`

Stores:

- ProfileId
- PartId
- Name
- OriginMm
- UDirection
- VDirection
- ordered profile points.

The semantic definition validates the profile plane basis and point structure.

It is an evaluation input. The profile object itself is not incorrectly treated as a feature graph dependency.

### `ExtrusionFeatureSpecification`

Stores:

- profile;
- depth;
- inherited feature identity data.

Operation kind:

`PartDesign.ExtrudeConvexPlanarProfile`

Canonical definition includes enough information to make the evaluation identity deterministic.

---

# 6. Reference semantics

## 6.1 `ReferenceTargetKind`

- Semantic
- Geometric
- Topology
- Publication
- Support

## 6.2 `ReferenceResolutionStatus`

- Resolved
- Missing
- Ambiguous
- Indeterminate
- Unsupported

## 6.3 `ReferenceContextId`

GUID-backed context identity.

## 6.4 `ReferencePathSegment`

Carries:

- OwnerId
- PublicationName

It represents one semantically named navigation step.

## 6.5 `ReferencePath`

Carries an immutable ordered list of path segments.

A reference path provides semantic addressing rather than a permanent numerical index.

## 6.6 `CadReference`

Abstract reference root:

- ReferenceId
- TargetKind
- ContextId
- optional Path

## 6.7 `SemanticReference`

Targets another semantic entity by semantic identity.

## 6.8 `GeometricReference`

Targets geometric meaning while preserving semantic ownership.

## 6.9 `TopologyReference`

Stores:

- authoritative result identity;
- topology key;
- topology kind.

This is the current mechanism for referencing an authoritative face/edge/vertex without relying on fragile indexes such as `Faces[12]`.

## 6.10 `ReferenceResolution`

Stores:

- reference;
- status;
- optional resolved result ID;
- diagnostic.

A resolution must be explicit. Missing or ambiguous references are never silently repaired.

## 6.11 `TopologyEvolution`

Maps topology identity across feature evaluation.

Fields:

- PreviousTopologyKey
- CurrentTopologyKey
- EvolutionKind

### `TopologyEvolutionKind`

- Preserved
- Replaced
- Split
- Merged
- Removed
- Introduced
- Ambiguous

This is essential for stable semantic references when upstream geometry changes.

---

# 7. Authoritative results

## 7.1 `AuthoritativeResultIdentity`

Stable string value object.

## 7.2 `AuthoritativeResultKind`

Current categories include:

- Wire
- Surface
- Solid
- Compound
- SketchSolution
- AssemblyState
- KinematicState
- ManufacturingState

## 7.3 `AuthoritativeResultStatus`

- Succeeded
- Failed
- Unsupported
- Ambiguous
- Indeterminate

## 7.4 `TopologyBinding`

Connects a topology item to its producing semantic feature:

- TopologyKind
- TopologyKey
- SourceSemanticId

## 7.5 `ResultEvidence`

Carries:

- ContractId
- ContractVersion
- EvidenceHash
- TopologyEvolution
- Diagnostics

Evidence is part of the authoritative boundary.

## 7.6 `AuthoritativeCadResult`

Carries:

- identity;
- result kind;
- status;
- producing semantic ID;
- topology bindings;
- evidence.

A successful geometric result is expected to contain topology bindings.

---

# 8. Product structure and BOM

## 8.1 `ProductComponent`

Stores:

- ComponentId
- PartNumber
- Revision
- Description

## 8.2 `ProductOccurrence`

Stores:

- OccurrenceId
- ComponentId
- Quantity
- Context

An occurrence is an instance/context, not a duplicate component definition.

## 8.3 `ProductDefinition`

Stores:

- ProductId
- PartNumber
- Revision
- Components
- Occurrences

Validation requires unique component IDs and references from occurrences only to existing components.

## 8.4 `BomLine`

Represents one generated BOM group.

## 8.5 `BomService`

Static BOM generator.

Generation rules:

1. group occurrences by component identity;
2. aggregate quantity;
3. preserve component metadata;
4. order deterministically by PartNumber, Revision and ComponentId.

The drawing layer consumes BOM semantics; it does not redefine product structure.

---

# 9. UMLCAD.Cad.Engine

**Project:** `dotnet/src/UMLCAD.Cad.Engine`

**Dependencies:** Expressions, Semantics, Contracts.

**Purpose:** execute the semantic-to-authoritative-result lifecycle without embedding a concrete kernel implementation.

## 9.1 Source files

- `AsyncEvaluationEngine.cs`
- `AuthoritativeCadReferenceResolver.cs`
- `AxisAlignedBoxSolidEvaluator.cs`
- `CadEvaluationEngine.cs`
- `CadModelEvaluator.cs`
- `ChangeSets.cs`
- `ConvexProfileExtrusionEvaluator.cs`
- `EvaluationCache.cs`
- `EvaluationEngine.cs`
- `EvaluationGraph.cs`
- `EvaluationIdentityBuilder.cs`
- `EvaluationStepFactory.cs`
- `FeatureEvaluation.cs`
- `IncrementalEvaluation.cs`
- `ReferenceResolver.cs`
- `ResultIntegration.cs`

There are two generations of closely related evaluation types in this branch: the broader current `CadEvaluationEngine.cs` contract set and the newer strongly typed execution path. They must converge rather than becoming competing semantic engines.

---

## 9.2 Evaluation graph

### `EvaluationInputIdentity`

Tuple-like identity:

- role;
- identity.

It makes all semantically relevant inputs explicit.

### `EvaluationStep`

Stores:

- step ID;
- operation kind;
- normalized definition;
- dependencies;
- input identities;
- configuration context;
- tolerance policy;
- kernel contract version;
- representation policy.

Validation rejects duplicate input roles and self-dependencies.

### `EvaluationPlan`

Contains ordered evaluation steps.

It validates:

- unique step IDs;
- deterministic topological order;
- dependency completeness.

### `EvaluationPlanner`

Creates a deterministic topological order.

The ordering is deterministic by semantic identity when multiple steps are otherwise available simultaneously.

Cycles and missing dependencies fail instead of being guessed around.

### `EvaluationIdentity`

String value object representing an evaluation identity.

---

# 10. Evaluation identity

## 10.1 `EvaluationIdentityBuilder`

Builds a SHA-256 identity over canonical evaluation inputs.

The canonical identity includes, where applicable:

- operation kind;
- normalized feature definition;
- sorted dependencies;
- sorted input identities;
- configuration context;
- tolerance policy;
- kernel contract version;
- representation policy.

A profile or other execution input must appear in the input identity when it can affect the result.

The identity excludes:

- timestamps;
- object addresses;
- viewer state;
- GPU resource handles;
- incidental collection order.

This is the mathematical basis for safe deterministic caching.

---

# 11. Evaluation cache

## 11.1 `IEvaluationCache`

Minimal cache contract:

- retrieve by evaluation identity;
- store evaluation result.

## 11.2 `DeterministicEvaluationCache`

Thread-safe implementation using a concurrent dictionary.

It:

- indexes by evaluation identity;
- rejects a cache-entry identity mismatch;
- reports entry count.

A cache hit is semantically equivalent to a fresh evaluation for the same identity.

---

# 12. Change sets and incremental evaluation

## 12.1 `RecomputeMode`

- Full
- Incremental

## 12.2 `CadChangeSet`

Stores the changed feature identities.

Incremental mode requires at least one changed feature.

`Full()` explicitly creates a full recomputation request.

## 12.3 `IncrementalEvaluationPlan`

Combines:

- original evaluation plan;
- recomputation step IDs.

## 12.4 `IncrementalEvaluationPlanner`

Calculates the minimal affected closure from changed features.

The key invariant is:

```
AuthoritativeResult(FullRecompute(M, Δ))
=
AuthoritativeResult(IncrementalRecompute(M, Δ))
```

Incremental execution is therefore an execution optimization, not a second semantics.

---

# 13. Reference resolution

## 13.1 `IAuthoritativeResultCatalog`

Catalog abstraction for authoritative results.

## 13.2 `AuthoritativeResultCatalog`

Registers authoritative results and resolves result/topology identities deterministically.

## 13.3 `ReferenceResolver`

Resolves semantic, geometric and topology references against:

- semantic identity;
- result identity;
- topology kind/key.

Unknown results, missing topology and ambiguity are returned explicitly.

## 13.4 `ICadReferenceResolver`

Higher-level reference abstraction used by the full CAD evaluator.

## 13.5 `AuthoritativeCadReferenceResolver`

Resolves references in the broader System-CAD contract model.

The resolver is deliberately separate from evaluation so reference semantics can be tested independently.

---

# 14. Evaluation execution

## 14.1 `EvaluationOutcomeStatus`

Execution-level state for one step.

## 14.2 `EvaluationOutcome`

Stores:

- status;
- diagnostics;
- optional authoritative result.

The authoritative result is carried as a first-class member rather than reconstructed from diagnostics.

## 14.3 `IEvaluationStepExecutor`

Synchronous execution abstraction.

## 14.4 `EvaluationEngine`

Executes a prevalidated plan step-by-step.

It does not resolve business meaning itself; it invokes the registered executor for each operation.

## 14.5 `IAsyncEvaluationStepExecutor`

Async version of step execution.

## 14.6 `AsyncEvaluationEngine`

Responsibilities:

1. execute steps in deterministic plan order;
2. verify dependency completion;
3. stop on unsuccessful dependencies;
4. invoke exactly the appropriate executor;
5. validate returned step/identity consistency;
6. require an authoritative result on a successful operation;
7. integrate cache hits without changing semantic meaning.

---

# 15. Feature evaluation

## 15.1 `FeatureEvaluationOptions`

Carries evaluation context such as:

- configuration;
- kernel tolerance;
- tolerance policy;
- representation policy.

## 15.2 `FeatureSpecificationCatalog`

Catalog of typed feature specifications.

## 15.3 `IAuthoritativeFeatureEvaluator`

Contract implemented once per authoritative feature operation family.

It exposes the operation kind and performs the semantic-to-contract mapping.

## 15.4 `AxisAlignedBoxFeatureEvaluator`

Operation kind:

`PartDesign.AxisAlignedBoxSolid`

Responsibilities:

1. consume typed box semantics;
2. create the versioned box request;
3. call `IAuthoritativeGeometryService`;
4. validate the returned contract;
5. convert it into an authoritative CAD result.

## 15.5 `ConvexProfileExtrusionFeatureEvaluator`

Operation kind:

`PartDesign.ExtrudeConvexPlanarProfile`

Responsibilities:

1. consume typed planar-profile semantics;
2. use explicit kernel tolerance;
3. call `IExtrusionGeometryService`;
4. verify operation identity/result evidence;
5. integrate the returned topology.

## 15.6 `AuthoritativeFeatureStepExecutor`

Binds exactly one evaluator to exactly one operation kind.

This prevents a generic executor registry from accidentally routing a feature through the wrong mathematical contract.

## 15.7 `FeatureEvaluationExecutorFactory`

Creates the correct step executor for the requested operation kind.

Unsupported operation kinds are rejected explicitly.

---

# 16. Specialized feature evaluators

## 16.1 `AxisAlignedBoxSolidEvaluator`

Adapts the typed box specification to the box-solid contract.

The evaluator is the domain/backend bridge; it does not calculate the B-Rep itself.

## 16.2 `ConvexProfileExtrusionEvaluator`

Adapts the typed profile/extrusion specification to the extrusion contract.

The mathematical kernel remains authoritative for exact prism topology.

---

# 17. Result integration

## `ResultIntegrator`

Converts strongly validated provider responses into `AuthoritativeCadResult`.

Integration responsibilities include:

- status propagation;
- result identity mapping;
- topology mapping;
- evidence creation;
- diagnostic translation;
- failure-closed validation.

A display mesh, drawing result, or simulation mesh cannot be promoted to authoritative CAD truth merely because it exists.

---

# 18. UMLCAD.Science

**Project:** `dotnet/src/UMLCAD.Science`

**Dependencies:** Expressions.

**Purpose:** shared scientific foundation and provider-neutral phenomena simulation contracts.

## 18.1 Source files

- `ScienceFoundation.cs`
- `PhenomenaSimulationService.cs`

## 18.2 `QuantityDimension`

Integer powers for:

- Length
- Mass
- Time
- Temperature

Static:

- `Dimensionless` = (0,0,0,0)

This allows units to be represented dimensionally without coupling the CAD core to a heavy unit library.

## 18.3 `Quantity`

Stores:

- SI value;
- dimensional signature;
- unit symbol.

Validation:

- value must be finite;
- unit symbol must be non-empty.

## 18.4 `MaterialFamily`

- Metal
- Polymer
- Ceramic
- Composite
- Wood
- Stone
- Other

## 18.5 `MaterialProperties`

Current physical properties:

- density;
- Young's modulus;
- yield strength;
- ultimate strength;
- ductility;
- thermal conductivity;
- electrical conductivity.

## 18.6 `Material`

Stores:

- name;
- family;
- properties.

It provides shared material identity for CAD engineering domains.

## 18.7 `PhenomenonKind`

Enumerates supported phenomenon categories.

The enum is the provider-neutral semantic vocabulary, not a solver implementation.

## 18.8 `PhenomenaSimulationRequest`

Carries:

- RequestId;
- Phenomenon;
- input dictionary.

## 18.9 `PhenomenaSimulationResult`

Carries:

- RequestId;
- Phenomenon;
- success flag;
- output dictionary;
- ProviderId;
- Diagnostic.

## 18.10 `IPhenomenaSimulationProvider`

Provider contract:

- provider identity;
- capability test;
- simulation execution.

## 18.11 `IPhenomenaSimulationService`

Provider-neutral service contract.

## 18.12 `PhenomenaSimulationService`

Chooses a capable provider deterministically rather than allowing arbitrary provider ordering to change semantic output.

---

# 19. UMLCAD.Engineering.Resources

**Project:** `dotnet/src/UMLCAD.Engineering.Resources`

**Dependencies:** Expressions, Science.

**Purpose:** manufacturing resource semantics.

## 19.1 Source file

- `EngineeringResources.cs`

## 19.2 `MachineKind`

- MachiningCenter
- Lathe
- WireEdm
- PressBrake
- LaserCutter
- Waterjet
- GrindingMachine

## 19.3 `ToolKind`

- EndMill
- Drill
- Reamer
- WireElectrode
- PressBrakePunch
- PressBrakeDie
- LaserNozzle
- WaterjetNozzle
- GrindingWheel

## 19.4 `ManufacturingProcessKind`

- Milling
- Turning
- WireEdmCutting
- SheetMetalBending
- LaserCutting
- WaterjetCutting
- Grinding

## 19.5 `MachineCapability`

Stores:

- process;
- minimum stock thickness;
- maximum stock thickness.

Validation rejects non-finite bounds, negative minimum thickness and inverted ranges.

Operation:

`SupportsThickness(thicknessMm)`

requires finite thickness within the inclusive range.

## 19.6 `ToolDefinition`

Stores:

- ToolId
- Kind
- InterfaceId
- NominalDiameterMm
- MinimumDiameterMm
- MaximumDiameterMm

Validation ensures:

- non-empty IDs;
- finite non-negative diameters;
- minimum <= nominal <= maximum.

Operation:

`SupportsDiameter(diameterMm)`

returns true only for finite in-range values.

## 19.7 `MachineProcessCompatibility`

Current deterministic machine/process mapping:

- MachiningCenter → Milling
- Lathe → Turning
- WireEdm → WireEdmCutting
- PressBrake → SheetMetalBending
- LaserCutter → LaserCutting
- Waterjet → WaterjetCutting
- GrindingMachine → Grinding

Unknown mappings fail closed.

## 19.8 `ToolProcessCompatibility`

Current mapping:

- Milling → EndMill / Drill / Reamer
- Turning → currently unsupported by this table
- WireEdmCutting → WireElectrode
- SheetMetalBending → PressBrakePunch / PressBrakeDie
- LaserCutting → LaserNozzle
- WaterjetCutting → WaterjetNozzle
- Grinding → GrindingWheel

## 19.9 `MachineDefinition`

Stores:

- MachineId
- MachineKind
- ToolInterfaceId
- capability list

Operation:

`SupportsProcess(process, thickness)`

requires both a process capability and supported thickness.

## 19.10 `MachineToolCompatibility`

Validates:

1. machine/tool interface IDs match exactly using ordinal comparison;
2. the tool type is compatible with the requested manufacturing process.

This is a fail-closed manufacturing safety boundary.

---

# 20. UMLCAD.Engineering.SheetMetal

**Project:** `dotnet/src/UMLCAD.Engineering.SheetMetal`

**Dependencies:** Expressions, Semantics, Science, Engineering.Resources.

**Purpose:** sheet-metal semantic rules and manufacturing feasibility checks.

## 20.1 Source files

- `SheetMetalExpressions.cs`
- `SheetMetalSemantics.cs`

## 20.2 `SheetMetalPartDefinition`

Stores:

- PartId
- Material
- ThicknessMm

Material is connected to the scientific material model rather than a private material enum.

## 20.3 `BendDefinition`

Stores:

- BendId
- AngleDegrees
- RadiusMm
- KFactor

These values are semantic inputs for bend allowance and manufacturability logic.

## 20.4 `SheetMetalValidationResult`

Encodes validation state and diagnostics.

## 20.5 `SheetMetalValidator`

Validates, where applicable:

- metal material family;
- positive/finite thickness;
- ductility;
- bend radius threshold;
- machine thickness capability;
- machine/process compatibility;
- tool/process compatibility;
- tool interface compatibility.

The validator does not fabricate geometry.

## 20.6 `SheetMetalExpressions`

Static formula library.

Current bend-allowance expression is represented as the shared expression AST:

```
(((pi / 180) * (R + (K * T))) * A)
```

The fully parenthesized AST is intentional: it provides deterministic structure and an identity that can be reused by engineering and manufacturing code.

## 20.7 Current implementation boundary

The library currently defines the semantic/manufacturing foundation.

It is not yet a complete exact sheet-metal geometric engine for:

- flanges;
- relief topology;
- folding;
- unfolding;
- flat-pattern generation;
- bend-zone B-Rep conversion.

Those require additional authoritative geometry contracts.

---

# 21. UMLCAD.Engineering.Cam

**Project:** `dotnet/src/UMLCAD.Engineering.Cam`

**Dependencies:** Expressions, Semantics, Science, Engineering.Resources.

**Purpose:** manufacturing operations and deterministic NC/G-code output.

## 21.1 Source file

- `CamAndGCode.cs`

## 21.2 `ToolpathPoint`

Stores a 3D tool position:

- X
- Y
- Z

The record is semantic path data; machine-specific output is produced by a postprocessor.

## 21.3 `ManufacturingOperation`

Stores:

- OperationId
- Process
- ToolId
- StockThicknessMm
- Path

The path is ordered and becomes part of deterministic NC generation.

## 21.4 `NcProgram`

Stores:

- ProgramId
- MachineId
- Lines

Operations:

- `Serialize()`
- `ContentHash()`

Serialization joins lines with newline separators.

The content hash is SHA-256 over the serialized NC content.

## 21.5 `INcPostprocessor`

Manufacturing postprocessor abstraction.

## 21.6 `DeterministicGCodePostprocessor`

Current postprocessor ID:

`UMLCAD.GCODE.BASIC.1`

Current generated program structure is deterministic and includes:

- metric unit selection;
- absolute positioning;
- tool identity comment;
- rapid move to first point;
- linear moves for subsequent path points;
- program end;
- terminal percent marker.

The implementation is deliberately deterministic: the same operation, resources and path must produce byte-equivalent NC text.

## 21.7 Failure rules

The postprocessor checks machine/process/tool compatibility before generating NC.

Unsupported manufacturing processes fail explicitly.

The implementation currently emits G-code for the bounded supported milling path. It is not yet a universal postprocessor for every listed process kind.

## 21.8 `CamPhenomenaAdvisor`

Uses `IPhenomenaSimulationService` to evaluate manufacturing-related physical phenomena.

This keeps CAM from embedding its own simulation engine.

## 21.9 Current implementation boundary

The current CAM library has deterministic NC generation.

It does not yet include a full B-Rep-driven toolpath planner for every machining strategy.

The intended future chain is:

```
Authoritative CAD Result
    ↓
Manufacturing feature recognition / operation planning
    ↓
toolpath generation
    ↓
machine/tool/fixture validation
    ↓
postprocessor
    ↓
deterministic G-code / NC
```

---

# 22. UMLCAD.Engineering.Drawing

**Project:** `dotnet/src/UMLCAD.Engineering.Drawing`

**Dependencies:** Expressions, Semantics, Science.

**Purpose:** CAD drawing semantics and associative drafting contracts.

## 22.1 Source files

- `DrawingSemantics.cs`
- `DrawingAdvancedSemantics.cs`

## 22.2 Drawing standards

### `DrawingStandard`

- Iso
- Ansi
- Jis

Standards affect drafting presentation/rules, not the underlying CAD geometry.

## 22.3 View kinds

### `DrawingViewKind`

Current list:

- Front
- Rear
- Top
- Bottom
- Left
- Right
- Isometric
- Auxiliary
- Section
- AlignedSection
- OffsetSection
- Detail
- CircularDetail
- ProfiledDetail
- Clipping
- Broken
- Unfolded

## 22.4 Dimensions

### `DimensionKind`

- Linear
- Angular
- Radius
- Diameter
- Coordinate
- Baseline
- Chain

## 22.5 Annotations

### `AnnotationKind`

- Text
- Note
- Leader
- Balloon
- Datum
- DatumTarget
- GeometricTolerance
- SurfaceRoughness
- WeldingSymbol
- FlagNote

## 22.6 Dress-up

### `DressUpKind`

- Centerline
- Axis
- SymmetryLine
- ThreadLine
- AreaFill
- Hatch
- BreakLine
- ConstructionGeometry
- MarkupArrow

## 22.7 `DrawingSheet`

Stores:

- SheetId
- Name
- Format
- Scale

Validation rejects empty identity/name/format and non-positive or non-finite scale.

## 22.8 `DrawingView`

Stores:

- ViewId
- Kind
- SourceResultId
- dimensionless scale `Quantity`

A drawing scale must be dimensionless and positive.

This establishes that a drawing view is attached to an authoritative CAD result rather than to arbitrary viewer geometry.

## 22.9 `DrawingDimension`

Stores:

- DimensionId
- dimension kind;
- ReferenceA;
- optional ReferenceB;
- Quantity value;
- tolerance text.

For linear, angular, radius and diameter dimensions, the value must not be dimensionless.

## 22.10 `DrawingAnnotation`

Stores:

- AnnotationId
- kind
- text

Non-empty annotation text is required.

## 22.11 `DrawingCapabilityMatrix`

Encodes the supported capabilities for one drafting standard:

- view kinds;
- dimension kinds;
- annotation kinds;
- dress-up kinds.

## 22.12 `MechanicalDraftingCapabilityProfile`

Creates the current baseline capability matrix containing the full enum sets documented above.

---

# 23. Advanced drawing semantics

## 23.1 `DrawingAssociativityState`

- Associative
- NeedsUpdate
- MissingReference
- AmbiguousReference
- Unsupported

This is critical: an invalidated drawing is represented explicitly instead of being silently repaired.

## 23.2 `ViewDisplayMode`

- Exact
- Shaded
- HiddenLine
- Sectioned
- Wireframe

These are display policies for drawing derivation/presentation, not authoritative geometry identities.

## 23.3 `ViewAxis`

Stores X/Y/Z and validates a finite non-zero vector.

## 23.4 `DrawingViewSpecification`

Stores:

- view identity;
- view kind;
- source authoritative result ID;
- view axis;
- display mode;
- associativity state;
- included occurrence identities.

This is the detailed semantic object that downstream projection/hidden-line services consume.

## 23.5 `DrawingBomItem`

Stores:

- item number;
- component identity;
- part number;
- revision;
- quantity;
- description.

## 23.6 `DrawingBomTable`

Stores:

- table identity;
- ordered BOM items.

The constructor copies the provided list, preventing later caller-side list mutation from changing the table.

## 23.7 `DrawingSheetPresentation`

Stores:

- border name;
- title block name;
- revision block name;
- scale.

All presentation names are required and the scale must be finite and positive.

## 23.8 Current drawing boundary

The library defines drawing meaning and associative contracts.

It does not yet implement the complete projection/section/hidden-line mathematics required for full generative drafting.

The intended chain is:

```
Authoritative 3D result
    ↓
Drawing view specification
    ↓
projection / section / hidden-line result
    ↓
dimensions / annotations / dress-up
    ↓
sheet composition / export
```

Broken or ambiguous references must remain explicit.

---

# 24. UMLCAD.Integration.Simulation

**Project:** `dotnet/src/UMLCAD.Integration.Simulation`

**Dependencies:** Science.

**Purpose:** clean external-provider boundary for simulation software.

## 24.1 Source file

- `SimulationApplicationAdapter.cs`

## 24.2 `ISimulationApplicationAdapter`

External application adapter contract.

The adapter owns:

- application identity;
- phenomenon capability;
- actual application invocation.

## 24.3 `SimulationApplicationProvider`

Implements `IPhenomenaSimulationProvider`.

Provider ID is derived from the adapter application ID.

The provider delegates support checks and execution to the adapter.

Returned results are validated against:

- expected request ID;
- expected phenomenon;
- expected provider identity.

This prevents an external adapter from accidentally returning a result belonging to a different request/provider.

---

# 25. UMLCAD.Framework

**Project:** `dotnet/src/UMLCAD.Framework`

**Target:** net10.0.

**External packages:**

- Microsoft.Extensions.Configuration
- Microsoft.Extensions.Configuration.Abstractions
- Microsoft.Extensions.DependencyInjection
- Microsoft.Extensions.DependencyInjection.Abstractions
- Microsoft.Extensions.Hosting.Abstractions
- Microsoft.Extensions.Options

**Purpose:** application composition and semantic build foundation.

This library is the original .NET application facade. It should not become the semantic owner of the newer System-CAD feature evaluation engine.

## 25.1 Source files

- `CadApplication.cs`
- `Configuration/CadConfiguration.cs`
- `Semantics/BuildHistory.cs`
- `Semantics/BuildPackage.cs`
- `Semantics/CompiledModel.cs`
- `Semantics/ISemanticRegistry.cs`
- `Semantics/SemanticEntity.cs`
- `Semantics/SemanticModels.cs`
- `Semantics/SemanticServices.cs`

## 25.2 `CadApplicationBuilder`

Primary mutable authoring/composition object.

Properties:

- application ID;
- version;
- `IServiceCollection` through `Services`;
- `IConfigurationManager` through `Configuration`.

Authoring methods:

- `AddPart()`
- `AddAssembly()`
- `AddDrawing()`

Build sequence includes:

1. validate definition identities;
2. validate definition references;
3. detect assembly cycles;
4. construct DI services;
5. publish configuration;
6. publish semantic state;
7. build deterministic identity;
8. record build history;
9. return `CadApplication`.

The DI container is created with build-time validation enabled and is disposed when the application is disposed.

## 25.3 `CadApplication`

Public runtime façade.

Properties:

- Semantic
- Configuration
- Services

Operations:

- `GetRequiredService<T>()`
- `CreateBuildPackage()`
- `CreateCompiledModelManifest()`
- `Dispose()`

The application owns the DI provider lifetime.

## 25.4 `PartBuilder`

Fluent authoring API:

- Name
- PartNumber
- Description
- Material
- Manufacturer
- Vendor
- Revision
- LifecycleState
- Author
- DocumentCode
- Property
- Parameter
- Geometry
- Constraint
- Reference
- Component

Definitions are normalized into deterministic arrays/dictionaries during build.

## 25.5 `DrawingBuilder`

Fluent authoring API:

- Setting
- Sheet
- PartReference

Sheets and references are normalized deterministically.

## 25.6 `AssemblyBuilder`

Fluent authoring API:

- Part occurrence
- Assembly occurrence
- generic Occurrence
- PartNumber
- Description
- Manufacturer
- Vendor
- Revision
- LifecycleState
- Author
- DocumentCode
- Property
- Setting

## 25.7 `AssemblyOccurrenceBuilder`

Occurrence-specific API:

- Transform
- Configuration
- Quantity
- BomStructure
- Visible
- Suppressed
- Grounded
- Flexible
- Property

Default state:

- identity transform;
- quantity 1;
- visible;
- not suppressed;
- not grounded;
- not flexible.

Quantity rejects zero, negative, non-finite values.

## 25.8 SemanticEntity

Abstract record:

- Id
- Kind
- optional Source
- Metadata dictionary

It provides generic semantic identity without attaching the entity to a geometry library.

## 25.9 Semantic models

### `ParameterSemantic`

- Name
- Value
- optional Unit

### `CadMetadata`

Structured engineering metadata:

- PartNumber
- Description
- Material
- Manufacturer
- Vendor
- Revision
- LifecycleState
- Author
- DocumentCode
- Custom

It implements read-only dictionary semantics with deterministic keys and ordinal ordering.

Custom keys use a stable `custom:<name>` namespace.

### `GeometrySemantic`

- Id
- Kind
- Properties

This describes geometry intent, not mathematical geometry implementation.

### `ConstraintSemantic`

- Id
- Kind
- References
- Properties

### `ComponentSemantic`

- Id
- ComponentType
- Children
- Parameters

### `TransformSemantic`

Stores a 16-value homogeneous transformation matrix.

Identity is the standard homogeneous 4×4 identity matrix.

`FromArray()` validates:

- exactly 16 entries;
- all entries finite;
- homogeneous element valid.

### `AssemblyOccurrenceSemantic`

Stores:

- Id
- Name
- DefinitionId
- DefinitionKind
- Transform
- Metadata
- ConfigurationName
- Quantity
- BomStructure
- Visible
- Suppressed
- Grounded
- Flexible

### `PartSemantic`

Stores:

- Id
- PartType
- Parameters
- Geometry
- Constraints
- References
- Components

Also exposes:

- Name
- StructuredMetadata

### `SheetSemantic`

Stores:

- Id
- Name
- DrawingReferences
- Settings

### `DrawingSemantic`

Stores:

- Id
- Name
- Sheets
- PartReferences
- Settings

### `AssemblySemantic`

Stores:

- Id
- Name
- component references
- settings
- structured metadata
- occurrences

### `SemanticApplication`

Top-level semantic snapshot:

- Id
- Version
- Configuration
- Parts
- Assemblies
- Drawings
- BuildIdentity

## 25.10 Semantic services

### `IPartSemanticService`

- Get(id)
- GetAll()

### `IDrawingSemanticService`

- Get(id)
- GetAll()

### `ISheetSemanticService`

- Get(drawingId, sheetId)

### `IAssemblySemanticService`

- Get(id)
- GetAll()

### `ISemanticApplication`

Exposes the current immutable application semantic snapshot.

### `SemanticApplicationState`

Internal state publisher.

Before build completion, current state is unavailable.

After build, all semantic lookup services reference the same current snapshot rather than duplicating state.

### Lookup implementations

- PartSemanticService
- DrawingSemanticService
- SheetSemanticService
- AssemblySemanticService

They perform deterministic identity lookups against the application snapshot.

## 25.11 Semantic registry

### `ISemanticRegistry`

Registration boundary:

- RegisterPart
- RegisterAssembly
- RegisterDrawing
- Snapshot

### `SemanticRegistry`

Internal implementation.

Maintains separate registries and rejects duplicate identities within each type.

Snapshot ordering is deterministic.

## 25.12 Build history

### `BuildSnapshot`

Stores:

- sequence;
- application semantic state;
- creation timestamp;
- package identity.

### `IBuildHistory`

Operations include:

- current snapshot access;
- all snapshots;
- lookup by sequence;
- record.

### `BuildHistory`

Thread-safe internal in-memory history using a private synchronization boundary.

Timestamps are UTC.

Build sequence is a history position, not semantic identity.

## 25.13 BuildPackage

### `BuildPackage`

Transport-oriented semantic package containing:

- schema/version information;
- semantic application identity/content;
- build identity and other package data.

### `IBuildPackageService`

Creates and serializes build packages.

### `BuildPackageService`

Serialization uses deterministic JSON settings.

Operations:

- `Serialize()`
- `SerializeToString()`

Package bytes are a transport representation, not authoritative geometric truth.

## 25.14 Compiled model

### `CompiledModelPackage`

Current compiled package container.

### `CompiledModelManifest`

Deterministic compiled semantic graph.

### `CompiledNode`

Represents a compiled semantic object.

### `CompiledRelationship`

Represents a typed relationship between compiled nodes.

### `CompiledRepresentation`

Represents a derived representation descriptor.

### `CompiledTopologyBinding`

Binds compiled nodes to topology identities.

### `CompiledSourceBinding`

Carries source location metadata:

- File
- Symbol
- Start
- End
- Revision

### `CompiledCapabilities`

Current flags:

- Visible
- Hideable
- Selectable
- Focusable

### `CompiledRenderArtifact`

Descriptor for a render artifact.

### `CompiledDiagnostic`

Carries:

- code;
- severity;
- message;
- optional target ID.

### `ICompiledModelService`

Creates the compiled model manifest.

### `CompiledModelService`

Deterministically transforms semantic data into a compiled graph/manifest.

Important boundary:

- compiled semantic graph is not the authoritative B-Rep;
- render artifacts are derived;
- topology binding retains provenance.

---

# 26. UMLCAD.Kernel.Client

**Project:** `dotnet/src/UMLCAD.Kernel.Client`

**Target:** net10.0.

**Packages:**

- Microsoft.Extensions.Http
- Microsoft.Extensions.Options

**Project references:**

- `UMLCAD.Cad.Contracts`
- transitional `UMLCAD.Framework` dependency

The Framework dependency is explicitly marked transitional and is intended to be removed by the stated transport-layer milestone. New System-CAD Engine code must not depend on the concrete Kernel Client.

## 26.1 Source files

- `RustCadKernelEvaluator.cs`
- `RustCircularPrismGeometryService.cs`
- `RustExtrusionGeometryService.cs`
- `RustGeometryKernelService.cs`
- `RustKernelService.cs`
- `RustSketchConstraintService.cs`

---

# 27. RustKernelOptions

Stores transport configuration:

- `BaseAddress`, default `http://localhost:8080/`
- `EvaluatePath`, default `v1/build/evaluate`
- `RequestTimeout`, default two minutes
- `MaxResponseBytes`, default 256 MiB

These are transport safeguards, not mathematical tolerances.

---

# 28. Legacy/general Rust kernel service

## 28.1 `IRustKernelService`

General build/evaluate transport abstraction.

## 28.2 `KernelEvaluationResult`

Contains compiled model/package result data from the legacy/general build endpoint.

## 28.3 `KernelDiagnostic`

Contains machine-readable diagnostic code/message/state.

## 28.4 `RustKernelService`

HTTP implementation.

Responsibilities:

1. build request payload;
2. apply configured endpoint and timeout;
3. send request;
4. read response with a hard size limit;
5. reject malformed/non-success transport;
6. parse JSON;
7. validate semantic response status;
8. return structured diagnostics.

Transport failure is not converted into an empty semantic package.

## 28.5 Response-size defense

The service reads response data under an explicit maximum size.

This protects the process from unbounded kernel responses.

## 28.6 `CompiledModelValidator`

Internal validator.

Validates that the returned compiled model:

- matches the expected build identity;
- satisfies the package/model consistency rules;
- contains internally coherent compiled information.

## 28.7 DI extension

### `AddRustKernel()`

Registers the transport client and typed options/services with `IServiceCollection`.

---

# 29. RustCadKernelEvaluator

Implements `ICadKernelEvaluator`.

It is the System-CAD adapter over narrow authoritative geometry services.

Current constructor paths support:

- axis-aligned box geometry service;
- combined box + circular-prism + extrusion + sketch services.

## 29.1 `EvaluateAsync`

Routes a typed `CadFeatureSpecification` to the appropriate mathematical adapter.

Current explicit paths include:

- sketch evaluation;
- box evaluation;
- circular-prism-related operations;
- convex extrusion.

Unsupported feature types return structured unsupported results.

## 29.2 Box evaluation

The evaluator:

1. validates the feature contract;
2. validates/normalizes the frame;
3. produces a kernel request;
4. invokes the authoritative box service;
5. validates returned topology;
6. converts it into `AuthoritativeCadResult`.

## 29.3 Sketch evaluation

The evaluator maps semantic circles/constraints to `SketchSolveRequest` and validates the returned solver result.

## 29.4 Failure behavior

The adapter does not silently synthesize geometry when the kernel cannot provide it.

---

# 30. RustGeometryKernelService

Implements `IAuthoritativeGeometryService`.

Endpoint:

```
POST /v1/geometry/box-solid
```

Responsibilities:

- map .NET request to wire DTO;
- enforce contract version;
- validate HTTP response;
- validate JSON schema shape;
- ensure status and success semantics are consistent;
- map topology and geometric metrics;
- return structured failure on transport, timeout or JSON faults.

Internal DTOs:

- AxisAlignedBoxSolidResponseDto
- AxisAlignedBoxSolidResponseTopologyDto
- AxisAlignedBoxSolidResponseVectorDto

The DTO layer exists so wire names and transport defaults cannot accidentally leak into the authoritative contract model.

---

# 31. RustExtrusionGeometryService

Implements `IExtrusionGeometryService`.

Endpoint:

```
POST /v1/geometry/extrude-convex-planar-profile
```

Responsibilities mirror the box adapter:

- contract/version validation;
- strict JSON mapping;
- status/success consistency;
- topology/measure mapping;
- structured transport failure.

Internal DTOs:

- ExtrusionResponseDto
- ExtrusionResponseTopologyDto
- ExtrusionResponseVectorDto

The explicit kernel tolerance used for the evaluation is preserved in the request so evaluation identity and actual execution cannot diverge.

---

# 32. RustCircularPrismGeometryService

Implements `ICircularPrismGeometryService`.

It adapts the analytical circular-prism kernel boundary.

The response model includes:

- status;
- success;
- result identity;
- evidence;
- origin;
- axis;
- radius;
- depth;
- volume;
- surface area;
- centroid;
- bounds;
- topology;
- diagnostics.

Internal DTOs:

- CircularPrismResponseDto
- CircularPrismResponseVectorDto
- CircularPrismResponseBoundsDto
- CircularPrismResponseTopologyDto

This is the current .NET-to-Rust boundary for exact analytical cylindrical/prismatic geometry needed by the circular vertical slice.

---

# 33. RustSketchConstraintService

Implements `ISketchConstraintService`.

Endpoint:

```
POST /v1/geometry/solve-sketch
```

The service maps semantic sketch circles and fixed constraints to the kernel solver contract.

It validates:

- response status;
- success/status consistency;
- numerical diagnostics;
- returned solved geometry;
- request/response identity.

Internal DTOs:

- SketchSolveResponseDto
- SketchSolveResponseCircleDto

A solver that cannot prove a valid solution returns a non-success state. The client does not invent a solved sketch.

---

# 34. Cross-library lifecycle

The current .NET System-CAD execution path is:

```
1. FeatureSpecification
       ↓
2. EvaluationStep
       ↓
3. EvaluationIdentity
       ↓
4. ReferenceResolution
       ↓
5. AuthoritativeFeatureEvaluator
       ↓
6. Contract DTO / versioned request
       ↓
7. Rust Kernel Client
       ↓
8. Authoritative kernel result
       ↓
9. ResultIntegration
       ↓
10. AuthoritativeCadResult
       ↓
11. Derived representation
       ↓
12. Drawing / CAM / simulation / viewer consumers
```

The semantic definition is never replaced by a kernel object.

The kernel result is never replaced by a render mesh.

A drawing view is never treated as the owner of the engineering truth.

A CAM toolpath is never treated as a replacement for the CAD result from which it was generated.

---

# 35. Dependency rules by library

## Contracts

Must remain dependency-free.

## Expressions

Must remain pure and independent of CAD execution.

## Semantics

May use Expressions, but not Engine or Kernel Client.

## Engine

Owns orchestration. It may depend on contracts and semantic/expression definitions, but it must not depend on concrete Rust transport.

## Science

Must remain provider-neutral.

## Engineering Resources

May depend on science and expressions, not on concrete kernel transport.

## Sheet Metal

May compose semantics, science and resources. Exact geometry must be mediated through versioned mathematical contracts.

## CAM

Consumes CAD semantics/results and resources, and may ask Science for phenomena evidence. It must not own a separate geometry kernel.

## Drawing

Consumes authoritative results and semantic references. Projection/drafting execution belongs behind a contract.

## Simulation Integration

Depends on Science only and adapts external providers.

## Framework

Owns application composition and older semantic/build infrastructure. It is not the newer evaluation kernel.

## Kernel Client

Is an outward adapter. It must not become the owner of CAD meaning.

---

# 36. Identity hierarchy

The .NET layer currently uses several different identity classes.

### Semantic identity

Answers:

> What engineering object/specification is this?

Examples:

- `SemanticId`
- `CadId`

### Evaluation identity

Answers:

> What exact semantic evaluation inputs define this result?

Includes normalized definition, references, configuration, tolerance and contract policy.

### Authoritative result identity

Answers:

> What mathematically certified result was produced?

Examples:

- `AuthoritativeResultIdentity`
- `CadResultId`
- `ContractResultId`

### Topology identity

Answers:

> Which face/edge/vertex/etc. in that result?

Uses explicit topology keys/bindings.

### Representation identity

Answers:

> Which derived representation belongs to this authoritative result?

This distinction is required for safe cache reuse and reference stability.

---

# 37. Coordinate-frame rules

The current contract model explicitly distinguishes:

- World
- Document
- Part
- Body
- Sketch
- Face
- Occurrence
- DrawingView
- Simulation

A reference that crosses a frame boundary must carry enough context to resolve the target deterministically.

A viewer transform is not sufficient semantic evidence.

---

# 38. Determinism rules

The .NET layer must preserve deterministic behavior in all identity-bearing operations.

Required practices include:

- sorted dictionaries for semantic serialization;
- deterministic ordering of graph nodes;
- deterministic BOM grouping;
- canonical fully parenthesized expression strings;
- SHA-256 identities over canonical content;
- ordinal string comparison for semantic keys;
- explicit operation/version contracts;
- explicit tolerance inputs.

Forbidden identity inputs include:

- timestamps;
- runtime object references;
- memory addresses;
- render resource IDs;
- incidental thread scheduling;
- unordered provider selection.

---

# 39. TDD and validation coverage

The .NET libraries are exercised by:

- `dotnet/tests/UMLCAD.Framework.Tests`
- `dotnet/tests/UMLCAD.Engineering.Tests`
- `dotnet/tests/UMLCAD.Kernel.Integration.Tests`

Representative coverage areas include:

- semantic build correctness;
- identity collision rejection;
- adversarial edge cases;
- compiled-model consistency;
- Rust client transport failures;
- Rust kernel end-to-end behavior;
- architecture dependency contracts;
- feature evaluation;
- asynchronous evaluation;
- evaluation-cache integrity;
- reference resolution;
- coordinate-frame contracts;
- box-solid boundaries;
- extrusion adapters;
- circular-prism adapter;
- sketch-constraint adapter;
- product/BOM determinism;
- drawing/provider boundaries;
- engineering-resource compatibility;
- S1 vertical-slice TDD.

The existence of a test is evidence of intended behavior, but an unexecuted CI run is not a passing test result.

---

# 40. Current implementation status versus target architecture

The presence of a type or enum in a .NET library does **not** mean that the corresponding complete CAD capability is finished.

Current implemented foundations include:

- immutable semantic feature specifications;
- explicit reference semantics;
- deterministic evaluation planning;
- cache/incremental execution primitives;
- exact axis-aligned box result integration;
- exact convex-planar-profile extrusion result integration;
- analytical circular-prism contract/client;
- sketch-solver contract/client;
- product structure/BOM foundation;
- drawing semantic foundation;
- CAM operation + deterministic G-code foundation;
- science/material/phenomena provider abstraction;
- engineering resource compatibility.

Important capabilities still requiring further certified implementation include:

- general exact Boolean remove/add;
- fully associative circular-hole feature evaluation;
- complete sketch constraint solving beyond the current bounded contract;
- full curved-face B-Rep integration;
- complete part-design feature families;
- complete drawing projection/hidden-line generation;
- full B-Rep-driven CAM toolpath planning;
- full sheet-metal topology/unfolding;
- broader assembly/kinematics feature execution;
- complete knowledge/configuration/template systems;
- complete PMI/GD&T execution and standards handling.

The documentation must not overstate these as complete merely because their semantic contracts exist.

---

# 41. Architectural interpretation

The .NET libraries collectively implement a **systems-engineering CAD layer**, not a duplicate monolithic geometry kernel.

The semantic architecture is:

```
Specification
   ↓
Evaluation
   ↓
Authoritative CAD Result
   ↓
Derived Representation
```

The three distinct graphs are:

1. **Specification graph** — engineering intent and references.
2. **Evaluation graph** — executable dependency/invalidation order.
3. **Result graph** — authoritative solids/surfaces/topology and provenance.

The library boundaries exist to keep these concepts separate.

---

# 42. Non-negotiable library invariants

1. Semantic definitions must be deterministic.
2. References must be explicit and resolvable.
3. Ambiguous/missing/unsupported references must remain explicit.
4. Evaluation identity must contain every semantic input that can alter the result.
5. Cache reuse must be semantically equivalent to fresh evaluation.
6. Incremental recomputation must equal full recomputation semantically.
7. Mathematical authority remains outside the .NET semantic model.
8. Derived representations cannot become authoritative truth.
9. External providers must be validated against request/provider identity.
10. Manufacturing output must not be generated for invalid machine/process/tool combinations.
11. Drawing associations must fail closed when source references are missing or ambiguous.
12. Unsupported capabilities must remain unsupported rather than being approximated silently.

---

# 43. File-to-library map

## UMLCAD.Cad.Contracts

```
AxisAlignedBoxSolidContracts.cs
CadEvaluationContracts.cs
CircularPrismSolidContracts.cs
ExtrusionContracts.cs
GeometryKernelStatus.cs
KernelContracts.cs
RepresentationContracts.cs
SketchSolveContracts.cs
```

## UMLCAD.Cad.Expressions

```
ArithmeticExpression.cs
ExpressionIdentity.cs
```

## UMLCAD.Cad.Semantics

```
AuthoritativeResults.cs
FeatureSpecifications.cs
ProductStructure.cs
References.cs
```

## UMLCAD.Cad.Engine

```
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
```

## UMLCAD.Science

```
ScienceFoundation.cs
PhenomenaSimulationService.cs
```

## UMLCAD.Engineering.Resources

```
EngineeringResources.cs
```

## UMLCAD.Engineering.SheetMetal

```
SheetMetalExpressions.cs
SheetMetalSemantics.cs
```

## UMLCAD.Engineering.Cam

```
CamAndGCode.cs
```

## UMLCAD.Engineering.Drawing

```
DrawingSemantics.cs
DrawingAdvancedSemantics.cs
```

## UMLCAD.Integration.Simulation

```
SimulationApplicationAdapter.cs
```

## UMLCAD.Framework

```
CadApplication.cs
Configuration/CadConfiguration.cs
Semantics/BuildHistory.cs
Semantics/BuildPackage.cs
Semantics/CompiledModel.cs
Semantics/ISemanticRegistry.cs
Semantics/SemanticEntity.cs
Semantics/SemanticModels.cs
Semantics/SemanticServices.cs
```

## UMLCAD.Kernel.Client

```
RustCadKernelEvaluator.cs
RustCircularPrismGeometryService.cs
RustExtrusionGeometryService.cs
RustGeometryKernelService.cs
RustKernelService.cs
RustSketchConstraintService.cs
```

---

# 44. Relationship to the Rust kernel

The .NET libraries document and preserve the **contractual boundary** to Rust.

They do not duplicate:

- B-Rep topology algorithms;
- NURBS evaluation;
- intersections;
- exact curve/surface mathematics;
- nonlinear numerical solver internals;
- GPU execution.

Instead:

```
.NET semantic/domain object
    ↓
versioned .NET contract
    ↓
Rust request
    ↓
CPU f64 normative mathematical result
    ↓
evidence/result contract
    ↓
.NET authoritative result
```

GPU implementations remain acceleration paths under CPU-reference conformance.

OCCT may be an oracle/backend in the larger system but is not elevated into .NET semantic authority by this library design.

---

# 45. Maintenance rules for this reference

When a .NET library changes:

1. add/remove the library in the landscape table;
2. update its dependency declaration;
3. update its source-file inventory;
4. document every new public type;
5. document every changed public member/contract;
6. document validation and failure semantics;
7. update the implementation-status section when capability status changes;
8. keep architecture documents synchronized without replacing this library reference;
9. add or update authoritative tests for the changed behavior.

This file is intended to be a **single library reference**, while the existing architecture/roadmap/TDD files remain the authoritative documents for their respective subjects.
