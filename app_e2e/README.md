# UMLCAD.V.7 Application-Layer E2E and Architecture Gate

This Python project owns architecture-level validation for the complete .NET application layer.

## Application-layer rule

All production .NET libraries live under `dotnet/src/` and are classified as the **Application Layer**.

The mathematical kernel is outside that layer under `kernel/`.

The current dedicated .NET kernel-access library is:

```
dotnet/src/UMLCAD.Kernel.Client/
```

It is the only production .NET library permitted to contain knowledge of the current kernel transport/implementation boundary.

The gateway name is intentionally transitional. The target stable public concept is a single concrete **UMLCAD Kernel API** whose implementation technology is invisible to the rest of the Application Layer.

## Architecture gates

The test suite verifies, from the actual repository tree:

- every production .NET project is inside `dotnet/src/`;
- no production project reference escapes the Application Layer;
- the complete production project-reference graph is acyclic;
- there is exactly one designated .NET kernel-access gateway;
- kernel transport/implementation-specific identifiers do not leak into other production libraries;
- production .NET code does not directly reference native kernel paths or invoke the current kernel host/transport;
- the gateway is not placed behind a reverse dependency that creates a kernel/application cycle.

These are repository architecture tests, not claims that a test exists merely because it is written. CI must execute this project successfully.

## Run locally

From repository root:

```bash
python3 app_e2e/run.py
```

No third-party Python package is required.
