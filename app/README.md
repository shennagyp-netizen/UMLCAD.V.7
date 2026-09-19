# UMLCAD.V.7 Application Layer

Rebuilt around a functional semantic operation/result pipeline.

    Part
      -> Sketch -> SketchResult
      -> Extrusion(SketchResult) -> BodyResult_1
      -> Hole(BodyResult_1) -> BodyResult_2 = CurrentBody

There is no traditional CAD Feature Tree as semantic authority.
Earlier results remain immutable lineage for provenance, references, invalidation and recomputation.

The former implementation is preserved under app_old/.