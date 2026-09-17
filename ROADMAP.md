# UMLCAD Roadmap

## Product boundary

UMLCAD is intentionally developed in two major generations.

### V5 — complete product solution around a hardened 2D engineering kernel

V5 is **not** the generation where the kernel is expanded toward full-featured 3D mechanical CAD.

V5 goal:

```text
native kernel
    ↓
stateless / immutable Kernel API
    ↓
project/build services
    ↓
routing / collaboration
    ↓
web semantic client
    ↓
production web viewer
```

V5 must deliver the complete end-to-end solution from the native kernel through the web viewer and surrounding protocol/services. The implemented 2D kernel must be scientifically valid, deterministic, fail-closed, transport-neutral, and equivalent to the relevant V4 engineering semantics that V5 advertises.

V5 kernel scope is therefore **hardening and V4-semantic recovery of its implemented 2D surface**, not indefinite feature expansion.

V5 completion requires:

- separate transport-neutral Kernel API;
- stateless kernel evaluation contract;
- immutable semantic snapshots and build identities;
- native 2D geometry/constraint/relation semantics;
- rigorous numerical solving, scaling, conditioning, and acceptance validation;
- semantic topology, references, dimensions, spatial/engineering validation, and deterministic DXF where advertised;
- source/build/project orchestration around the kernel;
- web protocol and production semantic viewer;
- end-to-end tests proving that clients consume kernel evidence rather than becoming secondary engineering authorities.

Unsupported capabilities must remain explicitly unsupported. V5 must never approximate a future kernel capability merely to make the web product appear complete.

### V6 — full mechanical CAD kernel generation

V6 is the generation where UMLCAD's kernel expands toward the breadth of a mature mechanical CAD system such as the functionality expected from SolidWorks-class modeling.

The V6 kernel roadmap includes, at the appropriate engineering stages:

- freeform curves including Bézier/B-spline/NURBS semantics;
- 3D analytic and freeform curves/surfaces;
- complete 3D B-Rep topology including vertices, edges, loops, faces, shells, and solids;
- robust Boolean and trimming operations;
- offsets, blends, fillets, chamfers, shells, drafts, sweeps, lofts, and related solid/surface modeling operations;
- broader 3D parametric constraints and assembly solving;
- industrial-strength geometric intersection, approximation, tolerance, and robustness infrastructure;
- complete mechanical modeling semantics and associated engineering validation.

#### V6 machine-design-first development direction

V6 should be developed around the actual workflow of mechanical design rather than around an arbitrary list of 3D features. The principal modeling path is expected to be:

```text
2D engineering sketch
        ↓
constrained profile
        ↓
extrusion / primary solid
        ↓
secondary features
        ↓
several independent parts
        ↓
assembly
        ↓
mechanism / motion
        ↓
engineering analysis / FEA
```

The 2D kernel remains the natural starting point for most machine-design parts. V6 should therefore preserve the mathematical and semantic strength of the V5 2D foundation and lift it into a 3D parametric system rather than replacing the 2D model with a purely geometric 3D workflow.

V6 should ultimately support, as a coherent engineering system:

- 2D-sketch-driven part creation, with extrusion as a primary transition from planar design to 3D solids;
- multiple parametric parts with persistent semantic references and rebuild-safe dependencies;
- assemblies composed from independently defined parts rather than monolithic models;
- mates/constraints and deterministic assembly solving;
- kinematic and motion simulation for mechanisms, including positions, velocities, accelerations, limits, and collision/interference analysis where advertised;
- transfer of validated geometry, materials, loads, contacts, and boundary conditions into finite-element analysis workflows;
- engineering evidence that identifies when a geometric or mechanical result is valid, indeterminate, unsupported, or numerically unreliable.

#### Robust-kernel research beyond OCCT

OCCT may be used as an interoperability, import/export, comparison, experimentation, or temporary implementation backend where useful, but **V6 must not make OCCT the irreversible semantic or engineering authority**.

This is a deliberate future-work requirement because difficult machine-design geometry can expose failures in Boolean operations, topology evolution, trimming, intersections, fillets, near-coincident geometry, small features, and numerical tolerance handling. V6 should therefore investigate whether a stronger open-source mathematical foundation can be built around exact or certified geometric methods rather than accepting ordinary floating-point failure as the final behavior.

The intended direction is a layered kernel in which:

```text
SemanticOperation / SemanticReference / OperationEvidence
                         ↓
             UMLCAD geometric abstraction
                         ↓
       exact / filtered / certified geometry layer
                         ↓
          topology and B-Rep authority
                         ↓
         surface / solid operations
                         ↓
      optional OCCT or other backend adapters
```

Candidate mathematical infrastructure such as CGAL may be evaluated for exact predicates, exact constructions, robust set/topology operations, and other components where its algorithms are appropriate. CGAL is **not** presumed to be a complete replacement for a mechanical CAD kernel; the V6 architecture must remain independent of any single external geometry library.

The target is not merely to reduce crash frequency. V6 should prefer a scientifically classified result over an apparently successful but geometrically wrong result:

```text
valid result
    or
certified/controlled result
    or
indeterminate / unsupported with evidence
```

rather than silently returning corrupted topology.

#### Topology and reference persistence

Persistent semantic references are a first-class V6 problem. Names must be assigned from semantic provenance and topology history, not reconstructed from current coordinates or display order. The architecture must handle the full range of rebuild evolution, including:

```text
one → one
one → zero
one → many
many → one
many → many
```

for faces, edges, vertices, and other topology where such evolution is mathematically meaningful.

The V6 kernel should investigate and benchmark established topology-history mechanisms, including OCCT's capabilities where useful, but the application-facing `SemanticReference` model must remain independent of the backend implementation.

#### Geometry robustness benchmark

V6 future work must establish an adversarial geometry benchmark before claiming industrial robustness. The benchmark should include, at minimum:

- near-coincident and tangent intersections;
- very small features relative to model scale;
- thin walls and high aspect-ratio geometry;
- repeated Booleans;
- topology-changing cuts and unions;
- fillets/chamfers near other features;
- self-intersections and degenerate input;
- complex freeform surfaces;
- imported models with imperfect tolerances;
- large-coordinate/small-feature combinations;
- rebuild sequences that repeatedly alter topology.

Results should be measured independently for geometric correctness, topological validity, numerical residual/error, deterministic behavior, and failure classification. OCCT and other kernels may be used as comparison baselines, but benchmark success must be defined by UMLCAD's own mathematical and engineering acceptance criteria.

#### Analysis and simulation direction

FEA and motion are downstream engineering consumers of the same authoritative model. V6 should avoid maintaining a second, unrelated geometric truth for analysis. The intended direction is:

```text
parametric CAD model
        ↓
validated topology / geometry
        ↓
analysis representation
        ↓
mesh + material + boundary conditions
        ↓
FEA result + evidence
```

The same principle applies to motion: the assembly solver, collision/interference model, and simulation layer should consume authoritative semantic geometry and constraints rather than duplicating or approximating the design state.

V6 is therefore not simply "3D CAD added to V5". It is the future generation in which the existing 2D scientific kernel becomes the foundation of a complete machine-design system: **part creation, robust 3D topology, assemblies, motion, and engineering analysis**.

V6 is a separate kernel-generation scope. V5 should not import V6 features merely because they are desirable later.

## Non-negotiable engineering rule

A version is complete only for the capability set it explicitly claims. Compilation and green CI are necessary evidence, not scientific equivalence by themselves.

The V5 kernel remains the engineering authority. V4 is used only as a semantic/scientific reference for capabilities being recovered natively; V5 must not depend on V4 at runtime.
