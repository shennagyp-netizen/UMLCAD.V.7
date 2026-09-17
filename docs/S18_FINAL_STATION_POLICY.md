# UMLCAD V6 — S18 Final Station Policy

## Purpose

S18 is the final numbered station of the V6 kernel program.

S19 and later station numbers are not part of the V6 station map. Advanced implementation work that remains necessary to satisfy the S18 completion gate is treated as **S18.x implementation slices**, each with its own semantic contract, tests, native conformance, adversarial coverage, determinism coverage, release validation, and authoritative CI gate.

## Meaning of S18 completion

S18 is the V6 geometry-kernel completion gate. Completion is reached only when the applicable bounded and generalized geometry/B-Rep/freeform contracts required by the declared V6 **part-geometry-kernel** product boundary are implemented and green.

This does not mean that every possible CAD operation is implemented. The product boundary must remain explicit, and unsupported cases must remain explicit. Assemblies, kinematics, drawings, FEA, and machine-design application behavior remain outside the V6 part-geometry kernel boundary.

## Authority order

Every S18.x slice follows the same authority order:

```text
mathematical / semantic authority
        ↓
backend-neutral contract
        ↓
independent semantic tests
        ↓
native OCCT realization
        ↓
independent native conformance
        ↓
adversarial + determinism + release CI
```

OCCT remains a realization/conformance backend. It does not define semantic truth, topology meaning, reference identity, tolerance policy, or candidate selection.

## Already-merged S19-labelled slices

The repository contains S19A/S19B/S19C implementation slices that were developed after the S18 reference-evolution/migration gate. These labels are historical implementation identifiers, not new V6 stations.

They are therefore considered part of the continuing S18 completion work unless and until the V6 scope is explicitly changed by a new roadmap decision.

## Freeze on station numbering

Do not create a new `S19`, `S20`, or later numbered V6 station for ordinary completion work. Use `S18.x` for subsequent implementation slices and keep the single S18 completion gate authoritative.

A change to this policy requires an explicit roadmap revision, not an incremental implementation PR.
