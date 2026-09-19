# UMLCAD.V.7 .NET Kernel — Complete Library and Class Reference

## 1. Scope

This document describes every current production .NET library in dotnet/src on the audited main branch, and then maps every validation assembly that exercises those libraries.

The current production set contains exactly two libraries:

| Library | Project | Architectural role |
|---|---|---|
| UMLCAD.Framework | dotnet/src/UMLCAD.Framework | Semantic application/build foundation. Owns authoring, semantic state, deterministic identity, registry/snapshot construction, build history, package generation, and compiled graph generation. |
| UMLCAD.Kernel.Client | dotnet/src/UMLCAD.Kernel.Client | Process/HTTP integration boundary between .NET semantic state and the Rust kernel. Owns transport policy, cancellation, response-size defense, diagnostics, and compiled-result integrity validation. |

The following are test assemblies, not production libraries:

- UMLCAD.Framework.Tests
- UMLCAD.Kernel.Integration.Tests

The mathematical kernel itself is Rust. The .NET side must not be interpreted as a second mathematical kernel.

---

# 2. Solution and build environment

The .NET solution is dotnet/UMLCAD.sln.

The solution currently includes:

- UMLCAD.Framework
- UMLCAD.Kernel.Client
- UMLCAD.Framework.Tests
- UMLCAD.Demo

The separate Kernel Integration test project exists in the repository and is executed by the system E2E harness even though the solution file is not its architectural authority.

The SDK pin is dotnet/global.json:

- SDK version: 10.0.401
- rollForward: latestPatch
- prerelease SDKs disabled

Production projects use:

- TargetFramework: net10.0
- nullable enabled
- implicit usings enabled
- TreatWarningsAsErrors=true

The Framework project uses Microsoft.Extensions configuration, dependency injection, hosting abstractions, and options.

The Kernel Client project uses Microsoft.Extensions.Http and Microsoft.Extensions.Options and references UMLCAD.Framework.

---

# 3. Architectural segmentation

The current .NET libraries are intentionally segmented by responsibility.

| Segment | Owner | Main types |
|---|---|---|
| Application entry/facade | Framework | CadApplication |
| Application authoring/composition | Framework | CadApplicationBuilder |
| Part authoring | Framework | PartBuilder |
| Drawing authoring | Framework | DrawingBuilder |
| Assembly authoring | Framework | AssemblyBuilder, AssemblyOccurrenceBuilder |
| Semantic contracts | Framework | SemanticEntity and semantic records |
| Semantic lookup | Framework | semantic service interfaces and implementations |
| Registry/snapshot | Framework | ISemanticRegistry, SemanticRegistry |
| History | Framework | IBuildHistory, BuildHistory, BuildSnapshot |
| Package | Framework | IBuildPackageService, BuildPackageService, BuildPackage |
| Compiled model | Framework | ICompiledModelService, CompiledModelService and result records |
| Configuration boundary | Framework | ICadConfiguration, CadConfiguration |
| Kernel transport configuration | Kernel.Client | RustKernelOptions |
| Kernel service contract | Kernel.Client | IRustKernelService |
| Kernel request result | Kernel.Client | KernelEvaluationResult, KernelDiagnostic |
| Kernel transport implementation | Kernel.Client | RustKernelService |
| Returned-result validator | Kernel.Client | CompiledModelValidator |
| DI composition | Kernel.Client | RustKernelServiceCollectionExtensions |

This is more important than the namespace names. The type segmentation defines the intended dependency direction.

---

# 4. UMLCAD.Framework

## 4.1 Library responsibility

UMLCAD.Framework is the current .NET semantic/build kernel.

It creates a deterministic semantic application state from authoring calls and produces two important downstream forms:

1. BuildPackage for the current Rust-kernel transport contract.
2. CompiledModelManifest for a deterministic graph representation of the semantic state.

It is not currently a full feature-evaluation engine.

It does not implement:

- general sketch solving;
- B-Rep construction;
- CAD topology algorithms;
- CAM;
- Sheet Metal;
- physical simulation;
- drawing projection mathematics;
- rendering;
- PLM.

Those capabilities belong above or behind the appropriate domain/contracts.

---

# 5. Application facade

## 5.1 CadApplication

File: dotnet/src/UMLCAD.Framework/CadApplication.cs

Kind: public sealed partial class.

### Purpose

CadApplication is the runtime object returned after the authoring/build phase has succeeded.

It is the stable façade for consuming the built semantic state.

### Public creation entry point

CreateBuilder() returns a new CadApplicationBuilder.

### Public state

Semantic:
the built SemanticApplication snapshot.

Configuration:
the current IConfiguration.

Services:
the ServiceProvider created by the build process.

### Public operations

GetRequiredService<T>()
Retrieves a registered service from the built provider.

CreateBuildPackage()
Obtains IBuildPackageService and creates the current BuildPackage.

CreateCompiledModelManifest()
Obtains ICompiledModelService and builds the current compiled semantic graph.

Dispose()
Disposes the owned ServiceProvider.

### Ownership boundary

CadApplication owns the lifetime of the DI container created for the application. The semantic snapshot itself is immutable record state.

### Invariants

A CadApplication is returned only after:

- definition validation;
- service-container construction;
- semantic registration;
- deterministic configuration normalization;
- deterministic semantic ordering;
- SHA-256 identity calculation;
- semantic snapshot creation;
- build-history recording.

### Non-responsibilities

CadApplication does not perform direct Rust calls. The transport boundary is UMLCAD.Kernel.Client.

---

# 6. Application authoring

## 6.1 CadApplicationBuilder

File: dotnet/src/UMLCAD.Framework/CadApplication.cs

Kind: public sealed class.

CadApplicationBuilder is the primary application composition object.

### Mutable authoring state

- user service registrations;
- ConfigurationManager;
- BuildHistory instance;
- application ID;
- application version;
- pending part definitions;
- pending assembly definitions;
- pending drawing definitions.

### Properties

Services:
IServiceCollection used to register application-owned services.

Configuration:
IConfigurationManager used to build application configuration.

ApplicationId:
required non-whitespace application identity.

Version:
required non-whitespace application version.

### AddPart

Accepts:

- part ID;
- part type;
- optional PartBuilder configuration delegate.

The builder is created, configured, converted into an internal PartDefinition, and retained until Build().

### AddAssembly

Accepts:

- assembly ID;
- assembly name;
- optional AssemblyBuilder delegate.

### AddDrawing

Accepts:

- drawing ID;
- drawing name;
- optional DrawingBuilder delegate.

### Build

Build is the most architecturally important method in the Framework.

The sequence is:

1. Validate global semantic definitions.
2. Construct a new DI service collection.
3. Copy user service registrations.
4. Register configuration.
5. Register build history.
6. Register semantic application state.
7. Register semantic lookup services.
8. Register package service.
9. Register semantic registry.
10. Register compiled-model service.
11. Build the service provider with ValidateScopes=true and ValidateOnBuild=true.
12. Register the internal PartDefinition, AssemblyDefinition and DrawingDefinition values as semantic records.
13. Resolve configuration into a sorted dictionary.
14. Create the canonical pre-identity object.
15. Serialize it using camel-case, compact JSON.
16. Compute SHA-256.
17. Create SemanticApplication.
18. Publish that snapshot through SemanticApplicationState.
19. Record the build.
20. Return CadApplication.

### Failure handling

If post-provider construction fails, the provider is disposed before rethrowing. This avoids leaking the DI container on build failure.

---

# 7. Definition validation

CadApplicationBuilder validates before publishing semantic state.

## 7.1 Global definition identity

Part IDs and assembly IDs share one global definition-ID namespace.

Therefore:

Part ID = X
and
Assembly ID = X

is invalid.

This prevents ambiguous cross-type definition references.

## 7.2 Occurrence identity

Occurrence IDs must be unique within a containing assembly.

The uniqueness scope is deliberately local to the parent assembly.

## 7.3 Definition references

An occurrence can reference:

- a part definition;
- an assembly definition.

Unknown definitions are rejected.

Unknown definition kinds are rejected.

## 7.4 Assembly cycles

A depth-first visiting/visited algorithm detects circular assembly references.

A cycle such as:

Assembly A -> Assembly B -> Assembly A

fails the build.

This is a semantic graph invariant, not a renderer restriction.

---

# 8. PartBuilder

File: dotnet/src/UMLCAD.Framework/CadApplication.cs

Kind: public sealed class.

PartBuilder is a mutable authoring object.

## 8.1 Stored authoring domains

- name
- part number
- description
- material
- manufacturer
- vendor
- revision
- lifecycle state
- author
- document code
- custom metadata
- parameters
- geometry descriptors
- constraints
- references
- component identifiers

## 8.2 Fluent API

Name(value)
sets the displayed semantic name.

PartNumber(value)
sets part number.

Description(value)
sets description.

Material(value)
sets material metadata.

Manufacturer(value)
sets manufacturer.

Vendor(value)
sets vendor.

Revision(value)
sets revision metadata.

LifecycleState(value)
sets lifecycle state.

Author(value)
sets author.

DocumentCode(value)
sets document/documentation code.

Property(name, value)
adds or replaces one custom metadata property.

Parameter(name, value, unit)
adds a semantic parameter.

Geometry(id, kind, properties)
adds a semantic geometry descriptor.

Constraint(id, kind, references, properties)
adds a semantic constraint descriptor.

Reference(id)
adds a semantic reference identifier.

Component(id)
adds a component identifier.

## 8.3 Deterministic conversion

The internal Build() operation sorts:

- parameters by Name;
- geometry by ID;
- constraints by ID;
- references by ID;
- components by ID.

It then creates CadMetadata using sorted custom properties.

## 8.4 Important boundary

Geometry() does not receive Rust geometry objects.

The Framework stores descriptions such as:

- kind = line;
- start = 0,0;
- end = 100,0.

The authoritative mathematical layer interprets mathematical geometry.

This is the critical separation between CAD semantics and numerical implementation.

---

# 9. DrawingBuilder

File: dotnet/src/UMLCAD.Framework/CadApplication.cs

Kind: public sealed class.

## Purpose

Defines drawing intent and sheet structure.

## State

- drawing settings;
- part references;
- sheets.

## API

Setting(name, value)
sets a drawing setting.

Sheet(id, name, drawingReferences)
adds a sheet.

PartReference(partId)
adds a part reference.

## Determinism

Sheets are sorted by ID.

Part references are sorted.

Settings are emitted in a sorted dictionary.

## Non-responsibility

DrawingBuilder does not calculate projections, dimensions, views, sections, or rendering.

---

# 10. AssemblyBuilder

File: dotnet/src/UMLCAD.Framework/CadApplication.cs

Kind: public sealed class.

## Purpose

Defines assembly-level product structure and metadata.

## Occurrence methods

Part(occurrenceId, partId, name, configure)
adds a part occurrence.

Assembly(occurrenceId, assemblyId, name, configure)
adds a nested assembly occurrence.

Occurrence(occurrenceId, name, definitionId, definitionKind, configure)
provides the fully generic occurrence construction method.

## Metadata

AssemblyBuilder supports:

- part number;
- description;
- manufacturer;
- vendor;
- revision;
- lifecycle state;
- author;
- document code;
- custom properties;
- settings.

## Determinism

Occurrences are sorted by occurrence ID before being transferred to AssemblyDefinition.

---

# 11. AssemblyOccurrenceBuilder

File: dotnet/src/UMLCAD.Framework/CadApplication.cs

Kind: public sealed class.

This is the contextual instance object inside an assembly.

## Fields/defaults

Transform:
identity 4x4 matrix.

Configuration:
null.

Quantity:
1.

BOM structure:
null.

Visible:
true.

Suppressed:
false.

Grounded:
false.

Flexible:
false.

## API

Transform(matrix)
sets a validated TransformSemantic.

Configuration(value)
sets configuration name.

Quantity(value)
sets occurrence quantity.

BomStructure(value)
sets BOM structure metadata.

Visible(value)
changes visibility.

Suppressed(value)
changes suppression.

Grounded(value)
marks grounded state.

Flexible(value)
marks flexible state.

Property(name, value)
adds occurrence metadata.

## Quantity rules

Quantity rejects:

- zero;
- negative values;
- NaN;
- infinity.

## Architectural meaning

An occurrence is not a second definition.

A PartDefinition is reusable semantic definition state.

An AssemblyOccurrenceSemantic is placement/contextual instance state.

That distinction is fundamental for product structure.

---

# 12. Internal definition records

## 12.1 PartDefinition

Internal record.

Fields:

- ID
- Name
- PartType
- Parameters
- Geometry
- Constraints
- References
- Components
- CadMetadata

ToSemantic() converts it to PartSemantic.

Its internal nature prevents mutable builder objects from being leaked as public semantic state.

## 12.2 DrawingDefinition

Internal record.

Fields:

- ID
- Name
- Sheets
- PartReferences
- Settings

ToSemantic() produces DrawingSemantic.

## 12.3 AssemblyDefinition

Internal record.

Fields:

- ID
- Name
- Occurrences
- Settings
- CadMetadata

ToSemantic() produces AssemblySemantic and preserves occurrence data and structured metadata.

---

# 13. SemanticEntity

File: dotnet/src/UMLCAD.Framework/Semantics/SemanticEntity.cs

Kind: public abstract record.

Common semantic identity base.

Fields:

- Id
- Kind
- Source
- Metadata

## Architectural role

Provides shared identity and generic metadata without tying semantic data to a specific geometry implementation, rendering backend, or vendor library.

---

# 14. Semantic records

File: dotnet/src/UMLCAD.Framework/Semantics/SemanticModels.cs

## 14.1 ParameterSemantic

Fields:

- Name
- Value
- Unit

Current storage is string-based. The record describes semantic intent; numerical interpretation happens at the appropriate evaluation boundary.

## 14.2 CadMetadata

Structured engineering metadata.

Standard properties:

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

CadMetadata implements IReadOnlyDictionary<string,string>.

Known properties are converted to stable keys.

Custom properties use the prefix:

custom:<name>

The generated dictionary is sorted with ordinal comparison.

## 14.3 GeometrySemantic

Fields:

- Id
- Kind
- Properties

Derives from SemanticEntity.

This is a semantic geometry descriptor, not a mathematical geometry object.

## 14.4 ConstraintSemantic

Fields:

- Id
- Kind
- References
- Properties

References identify semantic targets.

## 14.5 ComponentSemantic

Fields:

- Id
- ComponentType
- Children
- Parameters

Represents generic semantic component information.

It is not the compiled assembly occurrence graph.

## 14.6 TransformSemantic

Field:

Matrix

The matrix contains 16 double values.

### Identity

Identity is the standard 4x4 homogeneous identity matrix.

### FromArray

Validates:

- non-null input;
- exactly 16 values;
- no NaN;
- no infinity;
- homogeneous element at index 15 is non-zero.

This validates structural transform input.

It does not claim that every matrix is a valid rigid-body transformation.

## 14.7 AssemblyOccurrenceSemantic

Fields:

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

This is the core current occurrence contract.

## 14.8 PartSemantic

Fields:

- Id
- PartType
- Parameters
- Geometry
- Constraints
- References
- Components

Additional properties:

- Name
- StructuredMetadata

This is the semantic part definition, not the resulting B-Rep.

## 14.9 SheetSemantic

Fields:

- Id
- Name
- DrawingReferences
- Settings

## 14.10 DrawingSemantic

Fields:

- Id
- Name
- Sheets
- PartReferences
- Settings

## 14.11 AssemblySemantic

Fields:

- Id
- Name
- ComponentReferences
- Settings

Additional properties:

- StructuredMetadata
- Occurrences

## 14.12 SemanticApplication

Top-level snapshot.

Fields:

- Id
- Version
- Configuration
- Parts
- Assemblies
- Drawings
- BuildIdentity

This record is the Framework's central semantic state.

---

# 15. Semantic application access

File: dotnet/src/UMLCAD.Framework/Semantics/SemanticServices.cs

## 15.1 IPartSemanticService

Get(id)
returns one PartSemantic or null.

GetAll()
returns all current parts.

## 15.2 IDrawingSemanticService

Get(id)
returns one DrawingSemantic or null.

GetAll()
returns all current drawings.

## 15.3 ISheetSemanticService

Get(drawingId, sheetId)
resolves a sheet through its containing drawing.

## 15.4 IAssemblySemanticService

Get(id)
returns one AssemblySemantic or null.

GetAll()
returns all current assemblies.

## 15.5 ISemanticApplication

Exposes:

Current

This is the controlled state publication boundary.

---

# 16. SemanticApplicationState

Internal implementation of ISemanticApplication.

Before Build() completes, Current throws because no valid semantic application exists.

After Build():

SetCurrent(application)

publishes the semantic snapshot.

## Architectural purpose

Lookup services do not own duplicate semantic state.

They all read from the same published SemanticApplication.

---

# 17. Semantic lookup implementations

## PartSemanticService

Internal.

Performs ordinal-ID lookup through the current application.

## DrawingSemanticService

Internal.

Performs ordinal-ID lookup through current drawings.

## SheetSemanticService

Internal.

Resolves:

drawing ID -> drawing -> sheet ID -> sheet.

## AssemblySemanticService

Internal.

Performs ordinal-ID lookup through current assemblies.

None of these implementations introduce an independent persistent cache.

---

# 18. Semantic registry

File: dotnet/src/UMLCAD.Framework/Semantics/ISemanticRegistry.cs

## 18.1 ISemanticRegistry

Public construction-time interface.

RegisterPart(part)
RegisterAssembly(assembly)
RegisterDrawing(drawing)

Snapshot(applicationId, version, configuration, buildIdentity)

## 18.2 SemanticRegistry

Internal implementation.

Maintains three ordinal-keyed dictionaries:

- parts;
- assemblies;
- drawings.

### Registration invariants

- IDs cannot be empty;
- duplicate IDs in each collection are rejected;
- snapshot order is deterministic.

### Snapshot behavior

Configuration is converted to a sorted dictionary.

Parts, assemblies and drawings are sorted ordinally.

The supplied build identity is carried into the resulting SemanticApplication.

---

# 19. Build history

File: dotnet/src/UMLCAD.Framework/Semantics/BuildHistory.cs

## 19.1 BuildSnapshot

Fields:

- Sequence
- Application
- CreatedAt
- PackageIdentity

Sequence is the position in the in-memory history.

PackageIdentity is the semantic/package identity.

They have different meanings and must not be merged.

## 19.2 IBuildHistory

Current:
returns the latest BuildSnapshot.

GetAll():
returns all snapshots.

TryGet(sequence):
looks up a snapshot by sequence.

Record(application, packageIdentity):
records a new snapshot.

## 19.3 BuildHistory

Internal, thread-safe implementation.

Uses a private gate for:

- current access;
- list retrieval;
- lookup;
- record.

Each record increments the sequence.

CreatedAt uses UTC.

### Scope

This is application-memory history.

It is not yet persistent PDM/PLM lifecycle history.

---

# 20. Build package

File: dotnet/src/UMLCAD.Framework/Semantics/BuildPackage.cs

## 20.1 BuildPackage

Fields:

- Schema
- ApplicationId
- ApplicationVersion
- BuildIdentity
- Semantic

Current schema:

uml-cad-build-package/1.0.0

This is the current bridge contract to kernel evaluation.

## 20.2 IBuildPackageService

CreatePackage(application)

Serialize(package)

SerializeToString(package)

## 20.3 BuildPackageService

Internal implementation.

Serialization uses:

- camel-case naming;
- compact JSON;
- explicit null handling.

It can produce UTF-8 bytes or a string.

### Architectural limitation

This package is the current transport bridge. It should not automatically become the permanent canonical System-CAD interchange format.

---

# 21. Compiled model contract

File: dotnet/src/UMLCAD.Framework/Semantics/CompiledModel.cs

## 21.1 CompiledModelPackage

Fields:

- Schema
- ApplicationId
- ApplicationVersion
- BuildIdentity
- Manifest
- RenderArtifact
- Diagnostics

## 21.2 CompiledModelManifest

Fields:

- Schema
- ApplicationId
- ApplicationVersion
- BuildIdentity
- RootNodeIds
- Nodes
- Relationships
- Representations
- TopologyBindings
- SourceBindings
- Diagnostics

This is the current compiled semantic graph contract.

---

# 22. CompiledNode

Fields:

- Id
- Name
- Kind
- ParentId
- ChildIds
- Metadata
- RelationshipIds
- RepresentationIds
- Capabilities
- Source
- State

A node is a semantic graph object after compilation.

The node does not derive its identity from array position.

---

# 23. CompiledRelationship

Fields:

- Id
- Kind
- SourceId
- TargetIds
- Metadata

Relationships are explicit graph edges.

They are not reconstructed from child order.

---

# 24. CompiledRepresentation

Fields:

- Id
- Kind
- ArtifactId
- RenderNodeId
- Bounds
- Capabilities
- SelectableSubTargets

Represents derived consumer/presentation information.

It does not become geometry authority.

---

# 25. CompiledTopologyBinding

Fields:

- Id
- TopologyKind
- SemanticNodeId
- ArtifactId
- RenderPrimitiveId
- Name
- Nomenclature
- Number
- Bounds
- Metadata

This type bridges semantic/topological meaning to a representation primitive.

It is not a declaration that the render primitive defines topology.

---

# 26. CompiledSourceBinding

Fields:

- File
- Symbol
- Start
- End
- Revision

This is provenance metadata.

It answers where source information came from without making a source path the semantic identity.

---

# 27. CompiledCapabilities

Fields:

- Visible
- Hideable
- Selectable
- Focusable

These describe presentation capabilities.

They are not engineering properties.

---

# 28. CompiledRenderArtifact

Fields:

- ArtifactId
- BuildIdentity
- Format
- MediaType
- AssetIdentity
- SizeBytes
- IntegritySha256
- Uri
- InlineBase64
- Properties

The BuildIdentity link is critical: derived assets are attached to the same semantic build identity as the authoritative result.

---

# 29. CompiledDiagnostic

Fields:

- Code
- Severity
- Message
- TargetId

Diagnostics provide structured non-success and informational evidence at compiled-model level.

---

# 30. ICompiledModelService

Public interface.

Create(application)

Produces a CompiledModelManifest from SemanticApplication.

This is the Framework semantic-to-compiled-graph boundary.

---

# 31. CompiledModelService

Internal implementation.

## 31.1 Part compilation

For each part, ordered by ID:

1. create definition ID;
2. compile geometry child nodes;
3. compile constraint child nodes;
4. create constraint-reference relationships;
5. create the part definition node.

## 31.2 Assembly compilation

For each assembly, ordered by ID:

1. create assembly definition node;
2. process each occurrence in sorted order;
3. recursively compile nested assemblies;
4. recursively compile part geometry/constraint children;
5. create instantiation relationships.

## 31.3 Root calculation

An assembly is a root if it is not referenced by another assembly.

A part is a root if it is not referenced by an assembly occurrence.

The root list is deterministic.

## 31.4 Graph validation

The compiler rejects:

- missing root;
- missing parent;
- missing child;
- child whose ParentId does not point back;
- missing relationship source;
- missing relationship target;
- duplicate node identity;
- duplicate relationship identity.

## 31.5 Identity construction

Definition IDs use:

definition:part:<encoded-id>
definition:assembly:<encoded-id>

Occurrence IDs use:

occurrence:<assembly-id>/<occurrence-id>

Child IDs use parent-scoped paths.

User ID segments are URI-encoded.

This is specifically designed to avoid collisions from user IDs containing slash, colon or other structural delimiters.

---

# 32. Configuration library segment

File: dotnet/src/UMLCAD.Framework/Configuration/CadConfiguration.cs

## ICadConfiguration

Public abstraction.

Current:
returns IConfiguration.

## CadConfiguration

Internal adapter around Microsoft.Extensions.Configuration.

It contains no CAD business semantics.

Its architectural purpose is dependency inversion: consumers can depend on the Framework abstraction instead of introducing a custom configuration implementation.

---

# 33. UMLCAD.Kernel.Client

## 33.1 Library responsibility

This library is the .NET-to-Rust process boundary.

It must remain thin.

It does not own:

- CAD semantic definitions;
- feature-tree authority;
- mathematical geometry;
- solver algorithms;
- B-Rep;
- rendering;
- CAM;
- simulation.

It owns the communication contract and the integrity checks required when data crosses the process boundary.

---

# 34. RustKernelOptions

File: dotnet/src/UMLCAD.Kernel.Client/RustKernelService.cs

Kind: public sealed class.

Properties:

BaseAddress
Default: http://localhost:8080/

EvaluatePath
Default: v1/build/evaluate

RequestTimeout
Default: two minutes.

MaxResponseBytes
Default: 256 MiB.

## Registration validation

BaseAddress:

- must be absolute;
- scheme must be HTTP or HTTPS.

EvaluatePath:

- cannot be empty;
- must be relative.

RequestTimeout:

- must be greater than zero;
- must be at most one hour.

MaxResponseBytes:

- must be greater than zero;
- must be at most 2 GiB.

---

# 35. IRustKernelService

Public interface.

EvaluateAsync(package, cancellationToken)

is the only required operation.

It deliberately does not expose:

- HttpClient;
- sockets;
- process handles;
- JSON stream implementation;
- retry implementation.

---

# 36. KernelEvaluationResult

Fields:

- Succeeded
- CompiledModel
- Diagnostics

The result distinguishes successful execution from the presence/absence of a compiled model and diagnostic evidence.

---

# 37. KernelDiagnostic

Fields:

- Code
- Severity
- Message
- TargetId

The code is the stable machine-readable category.

---

# 38. RustKernelService

Public sealed implementation of IRustKernelService.

## 38.1 Request sequence

1. Reject null package.
2. Create a linked cancellation token source.
3. Apply configured timeout.
4. POST BuildPackage JSON.
5. Check advertised Content-Length.
6. Stream response into a bounded buffer.
7. Handle non-success HTTP status.
8. Reject empty body.
9. Deserialize KernelEvaluationResult.
10. Validate CompiledModelPackage when present.
11. Return structured result.

## 38.2 Caller cancellation versus timeout

Caller cancellation is preserved and re-thrown.

A service timeout becomes KERNEL_TIMEOUT.

This distinction is deliberate: cancellation is not equivalent to a failed kernel evaluation.

## 38.3 Resource-limit behavior

Response size is guarded twice:

- declared Content-Length;
- actual streamed body size.

This prevents an oversized response from passing merely because its final byte count was checked late.

## 38.4 Stable diagnostic mapping

KERNEL_RESPONSE_TOO_LARGE:
response exceeds configured limit.

KERNEL_HTTP:
kernel returned non-success HTTP status.

KERNEL_EMPTY_RESPONSE:
response body was empty/unusable.

KERNEL_TRANSPORT:
HTTP transport failure.

KERNEL_TIMEOUT:
configured timeout expired.

KERNEL_INVALID_JSON:
response is not valid JSON.

KERNEL_CLIENT:
unexpected client-side exception.

KERNEL_INVALID_RESULT:
returned compiled model violates integrity validation.

---

# 39. CompiledModelValidator

Internal static class.

This is the result-integrity firewall.

## Identity checks

Requires:

package BuildIdentity == expected BuildIdentity

Manifest BuildIdentity == expected BuildIdentity

Manifest ApplicationId == package ApplicationId

Manifest ApplicationVersion == package ApplicationVersion

## Graph checks

Rejects:

- duplicate node IDs;
- unresolved root IDs;
- unresolved relationship source IDs;
- unresolved relationship target IDs.

## Representation check

If a render artifact exists:

RenderArtifact.BuildIdentity must equal the submitted BuildIdentity.

## Why this matters

Valid JSON is not equivalent to a valid CAD result.

A server can return well-formed JSON containing:

- stale geometry;
- another application's graph;
- another build;
- wrong roots;
- broken relationship references.

CompiledModelValidator prevents those values from being silently accepted as the current build.

---

# 40. RustKernelServiceCollectionExtensions

Public static extension class.

Main method:

AddRustKernel(services, configure)

Responsibilities:

1. create default options;
2. apply optional configuration;
3. validate options;
4. register RustKernelOptions;
5. configure HttpClient;
6. register IRustKernelService -> RustKernelService.

This is the composition-root adapter.

The application layer depends on the interface, while DI selects the concrete HTTP implementation.

---

# 41. Dependency graph

Current production dependency graph:

UMLCAD.Kernel.Client
    -> UMLCAD.Framework

UMLCAD.Framework
    -> Microsoft.Extensions.Configuration
    -> Microsoft.Extensions.DependencyInjection
    -> Microsoft.Extensions.Hosting.Abstractions
    -> Microsoft.Extensions.Options

UMLCAD.Kernel.Client
    -> Microsoft.Extensions.Http
    -> Microsoft.Extensions.Options

UMLCAD.Kernel.Client
    -> HTTP
    -> Rust kernel host

There is no direct Framework dependency on Rust implementation assemblies.

---

# 42. Semantic data-flow

The complete current .NET path is:

Authoring
    -> CadApplicationBuilder
    -> PartBuilder / AssemblyBuilder / DrawingBuilder
    -> internal Definition records
    -> SemanticRegistry
    -> SemanticApplication

Identity
    -> canonical ordered content
    -> compact camel-case JSON
    -> SHA-256
    -> BuildIdentity

Transport
    -> SemanticApplication
    -> BuildPackage
    -> IRustKernelService
    -> RustKernelService
    -> HTTP
    -> Rust kernel
    -> KernelEvaluationResult
    -> CompiledModelValidator
    -> caller

Compiled graph
    -> SemanticApplication
    -> CompiledModelService
    -> nodes + relationships
    -> graph validation
    -> CompiledModelManifest

---

# 43. Validation assemblies

## 43.1 UMLCAD.Framework.Tests

This assembly validates Framework and client components.

### SemanticBuildTests

Verifies:

- application construction;
- application ID/version;
- configuration transfer;
- part semantics;
- parameter storage;
- geometry descriptors;
- constraints;
- drawing definitions;
- sheet definitions;
- service registration.

### CompiledModelTests

Verifies:

- nested assemblies;
- part occurrences;
- metadata;
- BOM-related occurrence metadata;
- compiled nodes;
- compiled relationships;
- product structure survival.

### IdentityCollisionTests

Specifically attacks:

- path-like user IDs;
- colon-containing IDs;
- slash-containing IDs;
- compiled definition collisions;
- duplicate geometry identity.

This validates URI-encoded structural identity.

### AdversarialEdgeCaseTests

Attacks:

- empty application;
- whitespace identifiers;
- global definition collisions;
- duplicate occurrence identity;
- other semantic edge conditions.

### RustKernelClientTests

Tests:

- successful response;
- HTTP errors;
- empty response;
- invalid JSON;
- transport errors;
- timeout;
- response-size limits;
- invalid compiled result.

### RustKernelEndToEndTests

Uses a real running Rust kernel.

It verifies:

- Framework package creation;
- real process boundary;
- compiled model schema;
- BuildIdentity preservation;
- application identity;
- expected semantic nodes.

### Test philosophy

This assembly is the component-level and client-level authority. It must not be treated as a substitute for the black-box system tests.

---

# 44. UMLCAD.Kernel.Integration.Tests

This assembly validates the production process boundary.

## 44.1 KernelBlackBoxE2ETests

Verifies:

- production client round-trip;
- rectangle semantic graph;
- build identity preservation;
- compiled node identity;
- invalid geometry rejection;
- stale reference rejection;
- concurrent request isolation;
- schema rejection;
- unsupported geometry rejection.

### Concurrency test meaning

Twelve distinct applications are evaluated concurrently.

The test requires:

- each result remains associated with the correct application;
- no BuildIdentity crossover;
- no result contamination between requests.

This is essential for a process-level kernel service.

## 44.2 AssemblyGraphE2ETests

Verifies that nested product structure survives the real HTTP boundary.

The scenario contains:

- a part;
- a subassembly;
- a top-level assembly;
- a nested assembly occurrence;
- a direct part occurrence.

The expected compiled graph is checked after crossing the real kernel boundary.

This test is important because it proves assembly meaning is not merely an in-memory Framework artifact.

---

# 45. Class segmentation by ownership

## Public authoring classes

- CadApplicationBuilder
- PartBuilder
- DrawingBuilder
- AssemblyBuilder
- AssemblyOccurrenceBuilder

## Public application façade

- CadApplication

## Public semantic records

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

## Public semantic service contracts

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

## Public package/result contracts

- BuildSnapshot
- BuildPackage
- CompiledModelPackage
- CompiledModelManifest
- CompiledNode
- CompiledRelationship
- CompiledRepresentation
- CompiledTopologyBinding
- CompiledSourceBinding
- CompiledCapabilities
- CompiledRenderArtifact
- CompiledDiagnostic

## Public kernel-client contracts

- RustKernelOptions
- IRustKernelService
- KernelEvaluationResult
- KernelDiagnostic

## Internal orchestration

- PartDefinition
- DrawingDefinition
- AssemblyDefinition
- SemanticApplicationState
- PartSemanticService
- DrawingSemanticService
- SheetSemanticService
- AssemblySemanticService
- SemanticRegistry
- BuildHistory
- BuildPackageService
- CompiledModelService
- CadConfiguration
- CompiledModelValidator

## Public extension/composition infrastructure

- RustKernelServiceCollectionExtensions

## Transport implementation

- RustKernelService

---

# 46. What belongs where

The intended ownership boundary can be stated precisely.

### Framework owns

- CAD semantic meaning;
- application identity;
- product definitions currently represented by parts/assemblies/drawings;
- authoring structures;
- semantic metadata;
- configuration;
- deterministic BuildIdentity;
- semantic snapshot;
- build history;
- bridge package;
- compiled graph.

### Kernel Client owns

- transport;
- HTTP policy;
- response limits;
- cancellation;
- diagnostics;
- returned-result validation;
- DI registration.

### Rust mathematics owns

- exact supported geometry mathematics;
- numerical linear algebra;
- solver;
- topology mathematics;
- tessellation;
- GPU conformance;
- mathematical diagnostics.

A future System-CAD feature must be added at the layer that owns its meaning rather than inserted into Kernel Client because that library is convenient.

---

# 47. Current library limitations

The production .NET libraries currently do not implement:

- full feature trees;
- generalized sketch authoring;
- generalized CAD evaluation graphs;
- semantic topology evolution;
- generalized reference migration;
- full B-Rep semantic ownership;
- production CAM;
- production Sheet Metal;
- production Drawing/PMI engine;
- production Science provider service;
- Engineering Resource catalog;
- generalized Phenomena Simulation orchestration;
- PLM/PDM lifecycle storage.

These are target System-CAD domains.

The current Framework is therefore a semantic/build foundation rather than the completed System-CAD application.

---

# 48. Rules for extending the libraries

Every new type must answer four questions before implementation:

1. What semantic truth does it own?
2. Which existing contract does it consume?
3. Which result does it produce?
4. Which test proves its boundary?

Then answer:

- What is its deterministic identity?
- What references can point into it?
- What frame does it use?
- What happens when input is invalid?
- What happens when evidence is ambiguous?
- What happens when a provider fails?
- What part can be cached?
- What exact input changes invalidate the result?
- Which representation is derived?
- Which E2E test proves the real boundary?

## No catch-all rule

Do not add a feature to UMLCAD.Framework merely because Framework already exists.

Framework should remain infrastructure.

A large domain belongs in a dedicated library when it has independent semantic ownership, lifecycle, testing, and dependency boundaries.

---

# 49. Target future .NET library segmentation

The following is the architectural target, not current implementation.

UMLCAD.Framework
    application and common semantic infrastructure

UMLCAD.Cad.Core
    Product, Part, Body, Sketch, Feature, references, publications, frames

UMLCAD.Cad.Evaluation
    dependency graph, evaluator, invalidation, cache, authoritative-result integration

UMLCAD.Cad.Representation
    derived representation contracts and artifact management

UMLCAD.Cad.ProductStructure
    product, assembly, occurrence, configuration, BOM

UMLCAD.Cad.Drawing
    drawing, sheet, views, PMI and documentation semantics

UMLCAD.Science
    material/physical facts and Phenomena Simulation Service

UMLCAD.Engineering.Resources
    Machine, Tool, Fixture, Process, Capability

UMLCAD.Engineering.SheetMetal
    sheet-metal design and flat-pattern semantics

UMLCAD.Engineering.Cam
    manufacturing definition, toolpath, postprocessor, G-code/NC

UMLCAD.Lifecycle
    revision, release, lifecycle, PDM/PLM integration

UMLCAD.Kernel.Client
    Rust-kernel transport only

The exact project split may evolve, but ownership boundaries should remain stable.

---

# 50. Definition of Done for a new .NET library

A library is not complete because it compiles.

Minimum completion requires:

- explicit ownership;
- stable public contract;
- correct dependency direction;
- deterministic identity where needed;
- reference semantics;
- frame semantics;
- failure semantics;
- provider boundary where applicable;
- component tests;
- red-team tests;
- integration/E2E tests;
- exact-head validation;
- documentation of limitations.

For authoritative CAD behavior, add:

- full-vs-incremental equivalence;
- cache-vs-fresh equivalence;
- topology/reference provenance;
- appropriate mathematical authority tests;
- representation regeneration rules.

---

# 51. Final architectural statement

The current .NET kernel is correctly understood as:

UMLCAD.Framework
    = semantic application foundation
    + deterministic build identity
    + product/drawing/assembly semantic construction
    + semantic registry
    + history
    + package generation
    + compiled graph generation

UMLCAD.Kernel.Client
    = transport boundary
    + timeout/cancellation policy
    + response-size defense
    + stable diagnostics
    + compiled-result integrity firewall

Rust
    = mathematical authority

Future System-CAD libraries
    = higher-level semantic ownership built above this foundation

The critical architectural rule is that these layers must not collapse into one catch-all library. Mathematical authority, CAD meaning, transport, derived representation, and specialized engineering domains must remain separately owned and testable.
