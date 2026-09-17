# UMLCAD V5 Demo — Bench Vise

This project is a code-first UMLCAD V5 demonstration of nested assemblies.

The top-level assembly is **`demo`** (`Bench Vise Demo`). It contains multiple assemblies:

```text
demo
├── Body Assembly
│   ├── Vise Base
│   └── Fixed Jaw
├── Screw Assembly
│   └── Lead Screw
├── Handle Assembly
│   └── Operating Handle
└── Sliding Jaw
```

## One-command run

From the repository root:

```bash
dotnet run --project projects/demo/Demo.csproj
```

The demo now performs the complete local flow:

```text
.NET demo
   ↓
build semantic model
   ↓
start Rust kernel host
   ↓
submit BuildPackage over HTTP
   ↓
Rust kernel validates/evaluates
   ↓
validated CompiledModelPackage
   ↓
write runtime package for viewer
   ↓
start Vite viewer
   ↓
open browser
```

The browser viewer loads the **package returned by the kernel**, not the old fixture. The fixture remains available when the viewer is opened without a `package` query parameter.

## Requirements

- .NET 10 SDK
- Rust/Cargo
- Node.js 20+
- npm

The demo starts the Rust kernel on `127.0.0.1:8080` and the viewer on `127.0.0.1:4173`.
