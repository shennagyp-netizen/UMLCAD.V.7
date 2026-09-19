# Rebuilt semantic architecture

The old feature-oriented app has been moved to app_old/ and is not part of the new build.

The semantic authority is:

Part -> Operation -> Result -> Operation -> Result

Sketch -> SketchResult
Extrusion(SketchResult) -> BodyResult_1
Hole(BodyResult_1) -> BodyResult_2 = CurrentBody

The engine plans the operation DAG, computes invalidation closure, builds deterministic evaluation identities, reuses unchanged results during incremental rebuild, and sends the current operation/result context to the kernel boundary.

No Feature Tree is permitted as semantic authority.

Each FP operation advances the authoritative operation/result lineage and the kernel history/state; an operation node does not request rendering. The kernel history is the realization context for incremental execution. Representation/rendering is downstream and is requested separately from an authoritative current result when a consumer requires it.


## Kernel boundary

`UMLCAD.Cad.Engine` depends only on the stable `IKernelGateway` contract. The only concrete kernel-access implementation in the active application layer is `UMLCAD.Kernel.Client`. Rust, HTTP, endpoint, process, serialization and native implementation details stay inside that library.
