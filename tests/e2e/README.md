# UMLCAD V6 E2E / Red-Team Gate

This directory owns the authoritative local integration harness for the V6 kernel boundary.

## One command

From the repository root on macOS/Linux:

```bash
python3 tests/e2e/run.py
```

For debug + release Rust API integration coverage:

```bash
python3 tests/e2e/run.py --release
```

Verbose child-process output:

```bash
python3 tests/e2e/run.py --release --verbose
```

The harness requires only Python 3, .NET 10, Rust/Cargo, and the repository's existing dependencies. No Python package installation is required.

## Test architecture

The Python runner starts the real `kernel_host` process on `127.0.0.1:8080`, waits for the socket, and guarantees process-group cleanup.

The Rust integration suite tests the typed kernel API operation-by-operation: geometry contracts, `Validate`, `AnalyzeLinearSystem`, `Solve`, `Analyze`, `EngineeringEvidence`, `Dimensions`, `ExportDxf`, and adversarial solver inputs.

The dedicated .NET integration suite tests the production `RustKernelService` over the real HTTP boundary using drawn geometry, graph identity assertions, invalid geometry, stale references, concurrency, malformed schema, and unsupported geometry.

The existing framework E2E tests remain part of the gate so the newer suite cannot accidentally hide regression coverage.

The Python red-team probes attack the raw wire protocol directly for unknown routes, invalid JSON, wrong schemas, missing semantic payloads, and unsupported geometry.

The demo E2E is also executed so the real Bench Vise application path is covered by the same process lifecycle.

## Output

The runner writes a JSON report to `tests/e2e/artifacts/e2e-report.json` and returns a non-zero exit code on any failed stage. The generated `artifacts/` directory is local test output and must not be committed.
