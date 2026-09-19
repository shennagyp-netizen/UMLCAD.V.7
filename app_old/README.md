# UMLCAD.V.7 — New Application Layer

This is the clean implementation tree for the new UMLCAD Application Layer.

The legacy \`dotnet/\` and \`projects/demo/\` trees are intentionally left untouched and are not dependencies of this tree.

## Structure

\`\`\`
app/
├── Directory.Build.props
├── .editorconfig
├── framework/
│   ├── libraries/
│   │   ├── UMLCAD.Kernel
│   │   ├── UMLCAD.Cad.Contracts
│   │   ├── UMLCAD.Cad.Expressions
│   │   ├── UMLCAD.Cad.Semantics
│   │   ├── UMLCAD.Cad.Engine
│   │   ├── UMLCAD.Science
│   │   ├── UMLCAD.Engineering.Resources
│   │   ├── UMLCAD.Engineering.SheetMetal
│   │   ├── UMLCAD.Engineering.Cam
│   │   ├── UMLCAD.Engineering.Drawing
│   │   ├── UMLCAD.Integration.Simulation
│   │   └── UMLCAD.Framework
│   └── tests/
│       └── UMLCAD.ApplicationLayer.Tests
└── application/
    ├── UMLCAD.Application
    └── demos/
        └── BenchVise
\`\`\`

All production .NET libraries in this tree are Application Layer libraries.

The mathematical kernel remains outside \`app/\`.

The only library that owns kernel transport/implementation mechanics is \`UMLCAD.Kernel\`. Other libraries consume only its concrete API.

Tests for the framework are kept under \`app/framework/tests/\`.

Application hosts and demos are kept outside \`framework/\`.

