# V6 S18.x Next Kernel Gates

S18 is the final numbered V6 station. The work below is not a sequence of new stations; it is the remaining implementation required to satisfy the single S18 **full-system kernel completion gate**.

## S18.x implementation families

1. generalized NURBS/freeform surface intersection tracing and boundary derivation;
2. generalized sweep/pipe families, including broader path and profile transport;
3. generalized offset, shell/thickness, blend, and imported-shape healing operations;
4. broader deterministic tessellation and mesh-quality contracts;
5. broader STEP/IGES exchange coverage and round-trip evidence;
6. broader arbitrary-B-Rep topology operations and pathology handling;
7. kernel API/integration expansion that preserves immutable/stateless semantic authority;
8. cross-crate/cross-layer system-integration scenarios covering the complete supported V6 kernel path.

The already-merged S19A/S19B/S19C changes are historical post-S18 implementation identifiers. They are treated as S18.x work and do not introduce a new V6 station.

## Mandatory S18.x gate

Every slice must follow:

```text
mathematical / semantic authority
        ↓
backend-neutral contract
        ↓
independent semantic tests
        ↓
native realization (OCCT reference backend)
        ↓
independent native conformance
        ↓
cross-system integration
        ↓
adversarial + determinism + release CI
```

Unsupported cases remain explicit. No renderer behavior, native identity, tolerance widening, or heuristic candidate selection may silently become semantic authority.

## S18 final completion boundary

S18 is complete only when the applicable contracts required by the declared V6 kernel boundary are implemented **and integrated as one system**, with the complete validation matrix green on the exact `main` commit.

The final matrix must include, as applicable:

- all V6 workspace crate tests;
- all separate kernel tests;
- mathematical/semantic contract tests;
- adversarial and determinism tests;
- native OCCT realization and independent conformance tests;
- release/debug validation and Clippy/format gates;
- dedicated cross-crate/cross-layer system-integration tests;
- integrated scenarios for the complete supported kernel pipeline.

A green feature branch, isolated crate test, or renderer demonstration is not sufficient for S18 completion. Required system-integration coverage must be explicitly executed by authoritative CI against `main`.

Assemblies, kinematics, drawings, FEA, and machine-design application behavior remain outside the V6 part-geometry-kernel boundary unless explicitly brought into the V6 scope by a separate roadmap revision.

See `docs/S18_SYSTEM_INTEGRATION_GATE.md` for the authoritative full-system exit rule.
