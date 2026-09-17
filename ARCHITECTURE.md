# UMLCAD V5 — Software Architecture

## 1. Production architecture

UMLCAD V5 has one production engineering authority: the existing Rust kernel project.

The application/authoring framework is .NET. UMLCAD does not recreate the .NET dependency-injection, configuration, hosting, or service model; it builds on the Microsoft.Extensions ecosystem.

```text
Developer C# CAD program
        │
        ▼
┌──────────────────────────────────────────┐
│ UMLCAD .NET Framework                    │
│                                          │
│ Microsoft.Extensions DI                  │
│ Microsoft.Extensions Configuration       │
│ Parts / Components / Assemblies           │
│ Drawings / Sheets                         │
│ Semantic services                         │
│ Build()                                   │
│ Build history                             │
│ Canonical Build Package                   │
└────────────────────┬─────────────────────┘
                     │
                     │ IRustKernelService
                     │ API call
                     ▼
┌──────────────────────────────────────────┐
│ Existing Rust Kernel Project              │
│ Engineering authority                    │
│ Geometry / constraints / solving         │
│ topology / validation / engineering      │
└──────────────────────────────────────────┘
```

The Rust kernel is not embedded into the .NET framework and does not become a .NET service implementation. The .NET side owns only the client/service boundary used to call it.

---

## 2. Authority boundaries

### .NET UMLCAD Framework

The framework is authoritative for the semantic meaning produced by the user's C# application:

- application composition;
- dependency injection and service registration;
- configuration;
- parts;
- components;
- assemblies;
- drawings;
- sheets;
- parameters;
- semantic geometry declarations;
- semantic constraints and references;
- build snapshots and history;
- canonical semantic package generation.

### Rust kernel

The Rust project is authoritative for engineering meaning and computation:

- geometry evaluation;
- constraint solving;
- topology;
- spatial/engineering rules;
- validation;
- engineering diagnostics;
- supported authoritative CAD exports.

The .NET framework must never become a competing engineering solver.

---

## 3. .NET application model

The user's program is a normal .NET application. The framework uses the real .NET application-composition model rather than recreating it.

```csharp
var builder = CadApplication.CreateBuilder(args);

builder.Services.AddSingleton<IMyService, MyService>();
builder.Configuration["cad:units"] = "mm";

builder.AddPart("bracket", "mechanical-part");
builder.AddDrawing("bracket-drawing", "Bracket Drawing");

using var app = builder.Build();
```

`Build()` executes the application composition and materializes an immutable semantic snapshot.

The framework may use Microsoft.Extensions facilities for:

- `IServiceCollection` / `IServiceProvider`;
- singleton/scoped/transient service lifetimes;
- `IConfiguration` / `IConfigurationManager`;
- options;
- `HttpClientFactory`;
- hosting/logging facilities as the application grows.

UMLCAD-specific APIs should add CAD semantics rather than reproduce generic .NET infrastructure.

---

## 4. Semantic build model

Every successful `Build()` produces a `SemanticApplication` containing the complete semantic meaning required by the next stage.

```text
SemanticApplication
 ├── application identity
 ├── version
 ├── resolved configuration
 ├── parts
 │    ├── parameters
 │    ├── geometry declarations
 │    ├── constraints
 │    ├── references
 │    └── components
 ├── assemblies
 └── drawings
      └── sheets
```

Semantic records are immutable value-oriented data. Authoring builders may be mutable during construction; `Build()` is the boundary at which the result becomes the application snapshot.

The semantic layer should remain deterministic and should not include runtime-only values such as wall-clock timestamps.

---

## 5. Semantic services

The framework exposes semantic meaning through ordinary injected services, for example:

```text
IPartSemanticService
IDrawingSemanticService
ISheetSemanticService
IAssemblySemanticService
ISemanticApplication
IBuildHistory
IBuildPackageService
```

These services query the immutable snapshot. They do not solve geometry.

This allows other .NET services, renderers, exporters, diagnostics, and future UI clients to consume the same semantic model rather than reconstructing CAD meaning independently.

---

## 6. Build history

A `CadApplicationBuilder` records every successful `Build()` as a `BuildSnapshot`.

```text
Build #1 → semantic snapshot A
Build #2 → semantic snapshot B
Build #3 → semantic snapshot C
```

History records include metadata such as sequence number and build time, but those runtime metadata fields are intentionally excluded from the canonical semantic payload and package identity.

A completed `CadApplication` retains its own snapshot even when the same builder is subsequently built again.

---

## 7. Canonical Build Package

`IBuildPackageService` converts a semantic snapshot into the versioned transport package:

```text
uml-cad-build-package/1.0.0
```

The package contains only deterministic semantic data and identity. Equivalent builds with equivalent semantic meaning must serialize to identical package bytes.

Build identity is calculated from canonical semantic input, including resolved configuration and deterministic ordering. Build identity must not include sequence numbers, timestamps, or other process-local values.

---

## 8. Rust kernel integration

The framework calls the Rust kernel through a normal .NET service abstraction:

```csharp
public interface IRustKernelService
{
    Task<KernelEvaluationResult> EvaluateAsync(
        BuildPackage package,
        CancellationToken cancellationToken = default);
}
```

The concrete `RustKernelService` uses `HttpClientFactory` and configurable endpoint options.

The framework therefore depends on a client contract, not on the Rust implementation itself.

```text
.NET Framework
     │
     └── IRustKernelService
              │
              ▼
       HTTP/API transport
              │
              ▼
       Existing Rust kernel
```

The transport endpoint is a deployment concern. The semantic package is the framework/kernel contract.

---

## 9. Functional design discipline

The production .NET framework follows a functional-core / application-shell discipline rather than attempting to make all of .NET functional.

Authoring builders and dependency-injection composition are intentionally imperative. Semantic records, normalization, canonical ordering, identity calculation, and package serialization should be pure/deterministic wherever practical.

The important invariant is:

```text
mutable application construction
        ↓ Build()
immutable semantic snapshot
        ↓
deterministic package
        ↓
Rust engineering authority
```

---

## 10. Previous TypeScript framework/build-engine work

The historical TypeScript framework/build-engine implementation remains in the repository for reference and migration history. It is not the production authoring framework.

New production authoring work must target the .NET framework. The existing Rust kernel remains the single engineering authority.
