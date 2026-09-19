# UMLCAD.V.7 Application Layer

The production Application Layer is a typed semantic execution system.

It does not model CAD as a persistent Feature Tree. User actions become typed semantic operations. Operations consume explicit semantic inputs and previous results and produce new authoritative results.

Canonical composition:

Part
 -> Sketch
 -> SketchResult
 -> Extrusion(SketchResult)
 -> BodyResult_1
 -> Hole(BodyResult_1)
 -> BodyResult_2 = Current Body

Previous results are immutable lineage. The newest BodyResult is the current solid state.

The Application Layer owns semantic meaning, functional composition, references/publications, dependency planning, invalidation, evaluation identity, engineering orchestration, and domain contracts.

The UMLCAD kernel is the mathematical authority and realization boundary.

The previous experimental implementation is preserved under app_old/.
