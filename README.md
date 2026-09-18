# UMLCAD.V.7

UMLCAD.V.7 is currently a mathematical/native engineering foundation with a thin .NET semantic and kernel-integration layer.

## Current architecture

- CPU mathematical authority: `kernel/math`
- Rust native kernel and host: `kernel/native`
- .NET semantic/build model: `dotnet/src/UMLCAD.Framework`
- .NET Rust-kernel adapter: `dotnet/src/UMLCAD.Kernel.Client`
- Demo host: `projects/demo`
- System E2E and red-team harness: `tests/e2e`

CPU `f64` behavior is the mathematical reference. Metal and CUDA accelerate explicitly certified workloads and conform to backend-neutral contracts; they do not redefine UMLCAD semantics.

## Documentation

The main architecture handbook is [`docs/doc.tex`](docs/doc.tex). It distinguishes current implementation from target system-CAD architecture.

Mathematical roadmap and coverage are tracked in:

- [`docs/MATH_AUTHORITY_ROADMAP.md`](docs/MATH_AUTHORITY_ROADMAP.md)
- [`kernel/math/GAP_MATRIX.md`](kernel/math/GAP_MATRIX.md)
- [`docs/CONTINUATION_HANDOFF.md`](docs/CONTINUATION_HANDOFF.md)

## Validation

Rust kernel:

```bash
cargo test --manifest-path kernel/native/Cargo.toml
```

Full integration and red-team gate:

```bash
python3 tests/e2e/run.py
```

Release-mode gate:

```bash
python3 tests/e2e/run.py --release
```

The E2E harness starts the real `kernel_host`, executes Rust and .NET tests, runs the demo path, and performs raw HTTP red-team probes.

## Architecture rule

Do not add duplicate mathematical semantics to .NET, the viewer, an accelerator, or an external provider. New functionality must identify its semantic owner, contract, evaluation path, authoritative result, representation boundary, tests, and exact evidence.
