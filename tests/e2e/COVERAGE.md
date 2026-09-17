# Kernel E2E Coverage Matrix

The integration gate deliberately has three boundaries. No single test substitutes for the others.

| Boundary | Test layer | What is proven |
| --- | --- | --- |
| Kernel library API | `kernel_rust/tests/kernel_api_integration_e2e.rs` | Typed `KernelRequest` dispatch, numerical geometry behavior, linear solve, snapshot solve/analyze, dimensions, engineering evidence, DXF, adversarial solver inputs |
| Production .NET boundary | `dotnet/tests/UMLCAD.Kernel.Integration.Tests/KernelBlackBoxE2ETests.cs` | `RustKernelService` JSON/HTTP transport, compiled-model identity, graph nodes, diagnostics, concurrency, invalid geometry, stale references, malformed schema |
| Process/wire boundary | `tests/e2e/run.py` | Real `kernel_host` lifecycle, readiness, cleanup, unknown endpoint, malformed JSON, unsupported schema/geometry, aggregate failure reporting |
| Application path | `projects/demo/Demo.csproj -- --e2e` | Real Bench Vise semantic definition through package creation, Rust evaluation, compiled model validation, nested assembly assertions |

## KernelRequest operation coverage

`Validate` — valid and invalid snapshots.

`AnalyzeLinearSystem` — exact 2x2 solution, rank, degrees of freedom, conditioning, and invalid matrix/damping/tolerance inputs.

`Solve` — fixed geometry, convergence reason, residual, and materialized geometry.

`Analyze` — validity, satisfaction, variable/equation counts.

`EngineeringEvidence` — structural/reference/spatial/export evidence.

`Dimensions` — length, radius, diameter, endpoint distance, and angle.

`ExportDxf` — entity presence and terminal `EOF` marker.

## Red-team principles

Tests assert numerical or protocol invariants rather than only successful execution. Negative cases must fail closed with deterministic diagnostics/status codes. Concurrency tests use distinct application identities and verify that results do not cross-contaminate. Every process started by the harness is owned and terminated by the harness, including forced termination on timeout.
