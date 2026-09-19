# UMLCAD Application Framework

The framework contains production Application Layer libraries and their tests.

## Boundary

\`\`\`
Application domain
    -> UMLCAD.Kernel
    -> mathematical kernel
\`\`\`

\`UMLCAD.Kernel\` is the only Application Layer gateway to kernel implementation mechanics.

Domain libraries must not know Rust, native process details, kernel transport internals, or accelerator implementation details.

## Test placement

All framework tests live under:

\`\`\`
app/framework/tests/
\`\`\`

The tests may be unit, contract, component, integration, architecture, or red-team tests.

