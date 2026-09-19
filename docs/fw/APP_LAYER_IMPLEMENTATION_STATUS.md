# UMLCAD.V.7 — New Application Layer Implementation Status

## Active implementation boundary

The production Application Layer is under `app/`.

The previous experimental Application Layer is preserved under `app_old/`.

The legacy `dotnet/` implementation remains an architectural reference and is not a dependency of the new application tree.

## Semantic architecture actually implemented

The active CAD semantic authority is a typed operation/result pipeline:

```
Part
  -> Sketch
      -> SketchResult
  -> Extrusion(SketchResult)
      -> BodyResult_1
  -> Hole(BodyResult_1)
      -> BodyResult_2 = CurrentBody
```

There is no traditional CAD Feature Tree in the active `app/` semantic model.

A user-visible action may be named Sketch, Extrusion, Hole, Fillet, Boolean, Pattern, etc., but the semantic abstraction is:

```
Operation(input results, references, expressions, configuration)
    -> typed Result
```

The latest BodyResult is the current body state. Earlier results are retained only as immutable lineage when required for references, provenance, cache reuse, invalidation, recomputation, revision, or diagnostics.

## Current application libraries

`UMLCAD.Cad.Contracts`
- shared identifiers, status values, kernel operation request/response contracts and topology bindings.

`UMLCAD.Cad.Expressions`
- deterministic expression values, parameter references and expression identity.

`UMLCAD.Cad.Semantics`
- Part, Body, Sketch, geometry, constraints, References, Publications, Operation types, Result types and functional Part authoring.

`UMLCAD.Cad.Engine`
- dependency graph;
- deterministic topological evaluation order;
- transitive invalidation closure;
- evaluation identity construction;
- cache-backed incremental reuse;
- full/incremental kernel operation requests;
- authoritative result integration into the semantic pipeline.

`UMLCAD.Kernel`
- isolated concrete kernel gateway contract/transport boundary;
- HTTP implementation is intentionally separate from semantic meaning.

`UMLCAD.Science`
- phenomena and deterministic simulation identity;
- concurrent in-flight simulation deduplication.

`UMLCAD.Engineering.Resources`
- initial machine/tool/fixture/process capability contracts.

`UMLCAD.Engineering.SheetMetal`
- initial bend/process semantic contracts.

`UMLCAD.Engineering.Cam`
- initial CAM operation/toolpath contracts.

`UMLCAD.Engineering.Drawing`
- initial drawing/sheet/view contracts.

`UMLCAD.Integration.Simulation`
- external phenomena-provider adapter boundary.

`UMLCAD.Engineering.Runtime`
- initial executable rule contracts and deterministic rule execution order.

`UMLCAD.Framework`
- application facade over the semantic evaluation engine.

## Tests authored

The active test project verifies:

- Sketch -> SketchResult -> Extrusion -> BodyResult -> Hole -> final Body composition;
- deterministic semantic dependency planning;
- transitive Sketch-change invalidation;
- cycle rejection;
- incremental rebuild that reuses unchanged upstream operations and reevaluates the changed downstream operation.

## Validation state

The environment used for this implementation has no .NET SDK, compiler, msbuild, or mono.

Therefore C# compilation and xUnit execution are not claimed as locally executed.

The project graph was statically inspected:

- 16 active .NET projects;
- no project-reference cycle;
- active `app/` contains no Feature-named implementation file;
- previous app implementation is preserved under `app_old/`.

## Kernel integration boundary

The existing Rust host currently exposes the older build-package endpoint. The new Application Layer defines an operation-level kernel contract so semantic operations and incremental realization are not forced into the old package model.

A real Rust operation-level endpoint/adaptor is the next kernel-integration increment. The application code does not falsely claim that the existing legacy endpoint already implements this new contract.
