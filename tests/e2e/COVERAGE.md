# Kernel E2E Coverage Matrix

The integration gate deliberately has multiple boundaries. No single test substitutes for the others.

| Boundary | Test layer | What is proven |
| --- | --- | --- |
| Kernel library API | `kernel/native/tests/kernel_api_integration_e2e.rs` | Typed `KernelRequest` dispatch, numerical geometry behavior, linear solve, snapshot solve/analyze, dimensions, engineering evidence, DXF, adversarial solver inputs |
| .NET Framework contract/unit layer | `dotnet/tests/UMLCAD.Framework.Tests` | Complete Framework test project, including semantic builds, compiled models, identity collisions, client behavior, adversarial cases, and real-kernel E2E tests |
| Production .NET boundary | `dotnet/tests/UMLCAD.Kernel.Integration.Tests/KernelBlackBoxE2ETests.cs` | `RustKernelService` JSON/HTTP transport, compiled-model identity, graph nodes, diagnostics, concurrency, invalid geometry, stale references, malformed schema |
| Process/wire boundary | `tests/e2e/run.py` | Real `kernel_host` lifecycle, readiness, cleanup, explicit repository-path validation, test discovery, unknown endpoint, malformed JSON, unsupported schema/geometry, aggregate failure reporting |
| Typed CAD sketch solver boundary | `dotnet/tests/UMLCAD.Engineering.Tests/SketchEvaluationTransportTests.cs` + `tests/e2e/run.py` | Typed circle/constraint request mapping, deterministic result identity/evidence, invalid frame rejection, unsupported constraint rejection, repeated-request determinism |
| Application path | `projects/demo/Demo.csproj -- --e2e` | Real Bench Vise semantic definition through package creation, Rust evaluation, compiled model validation, nested assembly assertions |

## Test discovery invariant

`tests/e2e/run.py` resolves the repository root from its own location and validates the Rust manifest, both .NET test projects, and the demo project before execution. It explicitly runs `dotnet test --list-tests` for `UMLCAD.Framework.Tests` and fails the gate if the project exposes zero tests or does not expose `RustKernelEndToEndTests`.

The Framework project is then executed **without a test filter**. The previous `Category=KernelEndToEnd` filter selected only the five real-kernel tests in `RustKernelEndToEndTests.cs` and silently excluded the rest of the Framework suite. Those tests remain covered because the full project now runs after the kernel host is started.

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
