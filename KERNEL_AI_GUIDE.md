# V5 Kernel Guide for AI Clients

AI clients should treat the kernel as a typed engineering oracle.

1. Query capabilities before attempting an unsupported operation.
2. Identify the exact project revision and immutable part/assembly build identity.
3. Address objects only by semantic IDs, never by array positions or screen coordinates.
4. Use the smallest explicit operation that matches the requested intent.
5. Treat solver output, diagnostics, topology, and dimensions as evidence returned by the kernel.
6. Never infer engineering validity from SVG, screenshots, rendered pixels, or DXF text.
7. A rejected operation leaves the previous immutable build unchanged.
8. New state must receive a new build identity.
9. Cached data may be reused only when its input identity matches exactly.
10. Transport, authentication, Git, source editing, and LLM orchestration are outside the kernel.

The stable AI-facing primitives are capability discovery, immutable build identity, semantic snapshots, analysis, solve, topology, dimensions, spatial queries, reference migration, and deterministic artifact projection.
