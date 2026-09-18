# UMLCAD.V.7 Continuation Handoff

## Snapshot
- Repository: `shennagyp-netizen/UMLCAD.V.7`
- Default branch: `main`
- Main head before PR #30 merge: `d267d9cb5d277da6d655fefd9d122c1c165e9a53`
- Latest completed roadmap family after merge: M16 — Performance + final red team
- Latest merged M10 closure PR: #27, `math: complete solver result status authority`
- PR #27 merge commit: `8fea60633c4f95f0175d18faa54878e7af9f44a6`
- Latest merged M8 closure PR: #28, `math: complete M8 construction authority`
- PR #28 merge commit: `fdb34d5a1e03ed399f900848b6dbe96da71d1a13`
- M9 closure PR: #30, `math: complete M8/M9 authority hardening`
- PR #30 validated implementation head: `9d04c05f67f4e5542df56b8612806fe8296f624b`

## Completed mathematical-authority state
PR #30 completes M9 for the declared certified domains. M0-M16 is now closed after the M16 post-merge verification.

Completed and validated stations now include:
- analytic constraint Jacobian authority;
- exhaustive analytic Jacobian relation coverage for the supported relation/constraint families;
- production solver use of the complete analytic Jacobian for the supported equation set;
- terminal convergence certification as an authoritative production result contract;
- exact terminal residual/step evidence checks with fail-closed contradictory-state handling;
- rank/nullity and augmented-rank consistency authority;
- wide null-space and pseudoinverse authority tests;
- conditioning authority with explicit conventional singular-system semantics;
- solver-status authority with fail-closed rank/conditioning semantics;
- linear-solver conformance tests against the independent consistency authority;
- zero-damping truncated pseudoinverse handling for numerically null singular directions;
- backend-neutral row-scaling contract for linearized systems;
- production row scaling of each analytic Jacobian row and matching residual row by the same explicit semantic scale before the existing nalgebra-backed damped SVD solve.
- truthful accepted-step convergence history authority that excludes rejected trials while preserving the authoritative attempted-iteration count;
- adversarial mixed-unit nonlinear solver regression coverage combining fixed geometry, coincident endpoints, model-unit distance, and angular relation equations across scales through `1e9`;

PR #23 specifically closed the previous mathematical mismatch in which convergence/acceptance used dimensionless scaled residuals while the nonlinear linearization step consumed raw mixed-unit residual rows. The production solve now applies the same semantic row normalization to both the Jacobian and residual before the existing solver backend.

## Analytic Jacobian policy
`kernel/math/jacobian.rs` is the production Jacobian authority in exact residual order: analytic constraint rows followed by analytic relation rows. The production solver does not fall back to generic finite differences for the supported equation set.

`SolveOptions::finite_difference_step` is retained for API compatibility, but it is not used for production Jacobian generation. Central/finite differences remain independent verification techniques only.

## Linear and conditioning authority
`kernel/math/linear_consistency.rs` distinguishes unique, underdetermined-consistent, inconsistent, and indeterminate systems using coefficient rank versus augmented rank under the declared numerical tolerance.

`kernel/math/conditioning.rs` provides explicit conventional conditioning evidence. Rank-deficient systems are not treated as conventionally finite-conditioned merely because a reduced nonzero singular spectrum has finite spread.

`kernel/math/solver_status.rs` consumes these proofs conservatively: `Inconsistent` is only claimed from explicit dimension-matched inconsistency evidence, and rank-deficient systems cannot be promoted to healthy conditioning.

## Accepted-step convergence, convergence and terminal authority
`kernel/math/convergence.rs` contains two complementary contracts:
- `evaluate` verifies accepted residual history, monotonicity, stagnation, progress, iteration cap, and joint residual/step convergence;
- `verify_terminal` verifies final residual and final step evidence without manufacturing an iteration history when an implementation rejects trial steps.

`kernel/math/terminal_authority.rs` is the public terminal-convergence authority used by the solver-facing module. It validates the terminal evidence returned by the wrapped solver and rejects contradictory convergence state.

The production iteration-control algorithm now stores the initial scaled residual and only residuals at actually accepted iterates. Rejected trial steps are not inserted into the history. The accepted-history authority therefore evaluates monotonicity and convergence truthfully while the solver's attempted-iteration count remains the authoritative count for iteration caps and result reporting.

## Production linearization scaling
`kernel/math/linearization.rs` defines the backend-neutral `RowScaledLinearSystem` contract and `scale_linearization` operation. A positive finite semantic row scale is applied identically to each Jacobian row and its corresponding residual component.

The native production solver integration is source-anchored and deterministic:
- `kernel/math/solver.rs` remains the sole solver implementation corpus;
- `kernel/native/build.rs` generates the native compile-time solver variant with the controlled row-scaling transformation;
- the generated solver calls the shared backend-neutral row-scaling contract, then the existing nalgebra-based damped SVD solver;
- a production regression checks uniform geometry scaling invariance of the terminal result evidence.

No second numerical library, duplicate solver implementation, GPU semantic path, or silent precision downgrade was introduced.

## Verification gate
Every mathematical station must pass the exact-head Rust kernel gate and comprehensive E2E/red-team gate before merge. Post-merge Rust + E2E must also be green on the resulting `main` commit.

For PR #23:
- exact-head Rust kernel validation: PASS;
- exact-head comprehensive E2E/red-team gate: PASS;
- post-merge Rust kernel validation on `201637b7...`: PASS;
- post-merge comprehensive E2E/red-team gate on `201637b7...`: PASS.

For PR #24:
- exact-head Rust kernel validation: PASS;
- exact-head comprehensive E2E/red-team gate: PASS;
- post-merge Rust kernel validation on `f014d8fd...`: PASS;
- post-merge comprehensive E2E/red-team gate on `f014d8fd...`: PASS.

For PR #25:
- exact-head Rust kernel validation: PASS;
- exact-head comprehensive E2E/red-team gate: PASS;
- post-merge Rust kernel validation on `ceea20ba...`: PASS;
- post-merge comprehensive E2E/red-team gate on `ceea20ba...`: PASS.

For PR #27:
- exact-head Rust kernel validation: PASS;
- exact-head comprehensive E2E/red-team gate: PASS;
- post-merge Rust kernel validation on `8fea6063...`: PASS;
- post-merge comprehensive E2E/red-team gate on `8fea6063...`: PASS.

M10 family status: **Implemented / Tested / CPU-validated by authoritative CI**.

M8 family status: **Implemented / Tested / CPU-validated by authoritative CI** for its declared certified construction domains.

M9 family status at PR #30 head: **Implemented / Tested** for its declared certified B-Rep/solid domains. The implementation head `9d04c05f67f4e5542df56b8612806fe8296f624b` includes bounded AABB tolerance hardening; its post-change Rust + E2E gates are now mandatory before merge.

The comprehensive E2E gate exercises the repository's Rust, typed API, .NET, black-box HTTP, Demo, raw HTTP red-team, release-path, and ignored-test checks.

## Immediate continuation
1. Verify PR #30's final exact-head Rust and comprehensive E2E/red-team gates on the documentation head, then merge it.
2. Verify post-merge main Rust and comprehensive E2E/red-team gates on the resulting main commit.
3. After post-merge green, begin M12 GPU abstraction from the updated main head.
4. M16 is merged and post-merge green on `3d494ebbffb6f944e3e574ab04234deeffd9f91a`. The current M0-M16 roadmap is complete; no further roadmap station is defined by this plan. CPU `f64` remains the semantic reference and no precision/fast-math shortcut may redefine authority.

## M11 completion boundary

M11 is now documented as `Implemented / Tested` for its declared certified domain. Current branch head `0db7d42694465b9d7146c90b99ba1c62c51fdc11` records the closure in `kernel/math/GAP_MATRIX.md`.

The certified scope is adaptive curve/surface tessellation; convex line/arc trim-aware outer-loop tessellation with preserved UV boundary samples, explicit interior classification, deterministic refinement, and chord/angular/parameter metadata; conservative AABB/bounding-sphere volumes; deterministic parameter-space bounds and 8-way spatial subdivision; and deterministic BVH query/candidate traversal with brute-force equivalence coverage.

The closure deliberately excludes general concave/holed/freeform trim filling, exact curvature-bound certification, and OBB as a semantic requirement. OBB is deferred because the current certified acceleration workload has no measured need that justifies another numerical authority.

Exact-head Rust and comprehensive E2E/red-team validation are green on `c9e542bb99122fa15fb86e8e86049de1b8ee5ca8`. The immediately preceding implementation head `f585dddc83471556fc8950e9138cd53f05bb6261` was also green in both authoritative gates.

## M9 completion boundary

M9 is closed for the certified explicit-topology planar-face/solid domains recorded in `kernel/math/GAP_MATRIX.md`. General curved-face sewing, unrestricted topology-aware Boolean construction, and exact holed-solid decomposition require separate mathematical contracts and are fail-closed where not certified. Bounded AABB Boolean/split/imprint math is certified, including sub-unit and translated-scale tolerance behavior.

## M8 completion boundary

M8 is closed for the certified domains recorded in `kernel/math/GAP_MATRIX.md`. Arbitrary freeform offset/fillet/chamfer/topology construction requires new explicit mathematical contracts and is not silently included.

## M10 completion boundary

M10 is now treated as a closed roadmap family for the current supported equation/solver scope. Any future new constraint/relation or solver mode requires a new mathematical contract and its own authority tests; it is not silently added under the closed M10 status.

## Non-negotiable authority rules
- CPU math is normative.
- Existing `nalgebra` remains numerical implementation infrastructure; do not introduce a second library without a demonstrated architectural need and a compatible proof/verification plan.
- GPU is acceleration only, never semantic authority.
- Unsupported, ambiguous, singular, degenerate and indeterminate cases fail closed.
- Semantic state and inputs remain immutable/stateless.
- Never claim hardware validation unless it actually ran.
- A green milestone is not completion of M0-M16.
- Never weaken a mathematical contract merely to obtain green CI.

## Continuation protocol
Before starting the next station:
1. Read `README.md`, `docs/MATH_AUTHORITY_ROADMAP.md`, this handoff, the current math-module sources, and the relevant tests.
2. Establish the exact `main` head and current authoritative CI state.
3. Change only the mathematical capability required by the next station, plus technically required native build/test exposure.
4. Test independently, then through native and repository-wide gates.
5. Merge only the exact validated head.
6. Validate both authoritative gates again on the resulting `main` commit.
7. Update this handoff with the exact commit, capability state, tests, and remaining work.

## Final M0-M16 acceptance reminder
Mathematical-authority completion still requires explicit reporting of:
1. mathematical capabilities implemented;
2. existing capabilities reused;
3. new algorithms added;
4. CPU reference status;
5. Metal status;
6. CUDA status;
7. TDD coverage;
8. red-team coverage;
9. determinism status;
10. known limitations;
11. hardware validation status;
12. performance measurements;
13. exact files changed;
14. tests executed;
15. remaining work.

Every capability must be distinguished as `Implemented`, `Tested`, `Hardware-validated`, `Not yet hardware-validated`, or `Unsupported`.

## Governing principles
> Mathematics first. Verification second. Optimization third.

> CPU defines the reference behavior. Metal and CUDA accelerate it. OCCT can challenge it. Nothing downstream defines it.

> Do not chase feature count. Build strong mathematical primitives from which professional CAD features can be constructed.

## M12 completion boundary

M12 is now documented as `Implemented / Tested` for its declared backend-neutral abstraction domain. The validated implementation head is `8a3043b5c8e216845d281dfe6c6f620d30d868cb`.

The M12 abstraction defines conceptual Auto/CPU/Metal/CUDA backends; explicit capability discovery; f64 precision and determinism requirements; automatic/preferred/explicit backend selection with diagnostic CPU fallback; checked batch-memory sizing and transfer accounting; immutable workgroup/dispatch planning; a backend-neutral executor submission interface; and CPU-versus-accelerator f64 conformance comparison with explicit tolerances and fail-closed mismatch/non-finite handling.

No Metal or CUDA hardware backend is implemented or hardware-validated by M12. No GPU handle/resource enters mathematical semantic state. The deprecated legacy selector remains only for compatibility; authority code uses the diagnostic selection result. M13 adds real Apple Silicon Metal execution; M14 adds real NVIDIA CUDA execution; M15 adds cross-backend hardware conformance.

The source implementation was green in the authoritative Rust kernel and comprehensive E2E/red-team gates before this documentation closure. The documentation-inclusive branch head must pass both gates again before PR #32 is merged.

## M13 completion boundary

M13 is now documented as `Implemented / Tested / Hardware-validated` for its declared Apple Silicon Metal domain. The validated implementation head was `625fae34f858c940085712ba315cff8a57b26abb`; M13 post-merge `main` validation passed on `7b55ee99a3ae122dbbff398df5287c8e020bb726`.

The Metal backend lives at `kernel/native/src/gpu/metal.rs` and uses the modern `objc2-metal` binding. It implements a real compute pipeline for conservative AABB candidate generation. Authoritative CPU `f64` AABBs are converted with outward-rounded `f32` bounds only for this broad-phase workload; the GPU output is required to contain every exact CPU `f64` overlap, and the adapter fails closed on a false negative. Candidate pairs are reconstructed deterministically on the CPU, and the hardware test repeats the same workload to verify stable output.

Apple Metal's lack of native hardware `f64` is represented explicitly in the backend capability. M13 therefore does not claim Metal execution of authoritative f64 vector/matrix/transform/NURBS geometry and does not silently downgrade those semantics. Those remain CPU-authoritative.

Exact-head validation on `b32e8c9...`: Rust kernel workflow #492 PASS; comprehensive E2E/red-team workflow #414 PASS; Metal hardware workflow #11 PASS on the `macos-14` Apple Silicon runner. Earlier failures on the same milestone were corrected before closure and are not part of the validated head.

M16 is the final roadmap family for the current mathematical-authority plan; M14 and M15 remain historical closure records.

## M14 completion boundary

M14 is implemented and tested for the declared CUDA AABB candidate-generation domain. The CPU f64 implementation remains authoritative. CUDA uses device-native f64 for the certified predicate; candidate ordering is reconstructed deterministically on the CPU; exact CPU overlap omission fails closed. The CUDA driver/NVRTC absence path is explicitly handled as unavailable rather than panic-prone. Hardware validation is **Not yet hardware-validated** because the repository has no confirmed NVIDIA runner execution record for this milestone.

## M14 post-merge closure record

M14 is closed on `main` for the declared NVIDIA CUDA acceleration domain at merge commit `f5095cdf53e5db1ef610c8e2fd3620920456e2da`.

- PR #34 exact documentation-inclusive head: `f4009a4abd17ad4bc78de11e216a7fcac70f1e8a`.
- PR #34 exact-head authoritative gates: Rust kernel PASS, comprehensive E2E/red-team PASS, Metal hardware PASS.
- Post-merge `main` authoritative gates on `f5095cdf53e5db1ef610c8e2fd3620920456e2da`: Rust kernel PASS, comprehensive E2E/red-team PASS, Metal hardware PASS.
- CUDA implementation: real device-native-f64 AABB candidate generation via `cudarc`, CPU f64 acceptance/reference authority, deterministic CPU reconstruction, fail-closed false-negative detection, and explicit no-driver handling.
- Normal repository gates contain zero ignored CUDA tests; hardware cases are feature-gated behind `cuda-hardware` and must be explicitly enabled on an NVIDIA runner.
- CUDA hardware status: **Not yet hardware-validated**. No confirmed NVIDIA hardware execution record was available during M14 closure.
- M15 cross-backend conformance is now closed for the declared common AABB candidate-generation workload; M16 is the final performance/crossover and full red-team closure.

## M15 post-merge closure record

M15 is closed on `main` for the declared common CPU/Metal/CUDA AABB candidate-generation workload at merge commit `b88862004debfda016d11c3d7da4dced0501065a`.

- PR #35 validated implementation head: `a6eded2c7de66b6708e819e17916d82798c26a8a`.
- PR #35 exact-head authoritative gates: Rust kernel PASS; comprehensive E2E/red-team PASS; Metal hardware PASS.
- Post-merge `main` authoritative gates on `b88862004debfda016d11c3d7da4dced0501065a`: Rust kernel PASS; comprehensive E2E/red-team PASS; Metal hardware PASS.
- The backend-neutral conformance contract reports false negatives separately from permitted broad-phase false positives and verifies repeat determinism.
- One canonical adversarial fixture is reused for CPU reference, Metal, and CUDA paths.
- Metal hardware executes and validates the shared fixture against the CPU f64 reference on Apple Silicon CI.
- CUDA is wired to the same conformance suite behind the explicit `cuda-hardware` feature, but no NVIDIA hardware execution record is currently available; CUDA remains **Not yet hardware-validated**.
- The closure does not certify cross-backend equivalence for the broader vector/matrix/transform/NURBS stack; those operations remain CPU-authoritative until additional common hardware contracts are established.

## M16 completion boundary

M16 is now the final roadmap family and is closed on `main` at merge commit `3d494ebbffb6f944e3e574ab04234deeffd9f91a` after exact-head and post-merge Rust, comprehensive E2E/red-team, Metal hardware, and release-performance validation all passed.

The final validated M16 implementation head was `3be00ca87b054000724cabd1d3393786ad78d334`; the resulting `main` merge is `3d494ebbffb6f944e3e574ab04234deeffd9f91a`.

M16 performance evidence on Apple Silicon (`macos-14`) measured the declared AABB candidate-generation workload at batches 32, 64, 128, 256, and 512. Median CPU/Metal latencies were respectively 1.25/745.79 µs, 4.79/1260.67 µs, 17.33/1136.58 µs, 61.88/2035.33 µs, and 229.75/10602.17 µs. No CPU/Metal crossover occurred in that tested range. The report also captured host preparation, buffer setup/upload, device execution, readback, CPU post-processing, transfer/setup overhead, throughput, and batch efficiency.

Post-merge main gates on `3d494ebbffb6f944e3e574ab04234deeffd9f91a`: Rust kernel workflow #554 PASS; comprehensive E2E/red-team workflow #476 PASS; Metal hardware workflow #73 PASS; M16 performance workflow #11 PASS.

The final M16 red-team matrix covers:
- scale invariance across 1e-12 through 1e12;
- near-parallel, tangent, and coincident intersection behavior;
- non-finite intersection inputs;
- extreme positive finite NURBS weights and invalid parameter/weight domains;
- rank-deficient linear systems and deterministic GPU candidate ordering;
- malformed/duplicate GPU candidate sets.

M16 CUDA status remains **hardware-unverified** because no NVIDIA runner was available for execution. No claim of CUDA hardware performance or CUDA hardware conformance is made.

M0-M16 is now closed as a roadmap of certified mathematical capabilities, not as a claim that every conceivable CAD operation is complete. Future changes to the mathematical authority layer require new explicit capability contracts and fresh gates.

## System CAD transition — M-S0

The mathematical-authority roadmap M0-M16 is closed at main commit `f1ab581140453c087beb50aac2216684196d4bdf`. System-CAD implementation now begins from a separate architecture track.

The governing system documents are:

- `docs/CATIA_SYSTEM_IMPLEMENTATION_ROADMAP.md`
- `docs/UMLCAD_V7_IMPLEMENTATION_PROMPT.md`

The first system milestone is M-S0 — boundary/dependency architecture. It does not expand mathematical authority. It establishes repository-level enforcement for the new .NET layer split:

```
Cad.Expressions
      ↓
Cad.Semantics

Cad.Engine → Cad.Expressions + Cad.Semantics + Cad.Contracts

Kernel.Client → Cad.Contracts
```

The existing `Kernel.Client → Framework` dependency is retained only as an explicit transitional legacy edge until the later legacy-elimination milestone. No new CAD semantic/engine code may depend on Framework or Kernel.Client.

M-S0 acceptance evidence is owned by `tests/e2e/architecture_contract.py` and the authoritative Python E2E runner.


## System-CAD architecture and engineering foundation snapshot

The system-CAD track now includes a normative abstraction/dependency architecture plus C4 documentation:

- `docs/architecture/ABSTRACTION_AND_DEPENDENCY_MODEL.md`
- `docs/architecture/architecture.json`
- `docs/architecture/c4/01-system-context.md`
- `docs/architecture/c4/02-containers.md`
- `docs/architecture/c4/03-components.md`
- `docs/architecture/c4/04-critical-flows.md`

The governing abstraction is:

```
Platform Foundation
    ↓
Mathematics
    ↓
Science
    ↓
CAD Engineering Core
    ↓
Engineering Resource Model
    ↓
Specialized Engineering Domains
    ↓
Application / Workflow / Presentation
```

External technology is reached through outward provider/adapters. The architecture distinguishes semantic ownership, reusable service dependency, published result/data flow, and private implementation dependency.

Key ownership decisions now recorded:
- Science owns material identity/properties, physical models, quantities/units, and the Phenomena Simulation Service.
- Phenomena Simulation is a reusable scientific service with multiple interchangeable providers, including external-application adapters. CAM may consume the service.
- CAD Product Structure owns Product/Occurrence relationships and the BOM service/view.
- Machine, Tool, Fixture, Process, and capability models are shared Engineering Resource semantics.
- Sheet Metal is a bounded engineering domain and a strong independent library boundary.
- CAM is a manufacturing domain and must terminate in deterministic machine-specific postprocessing to G-code/NC.
- Drawing is a bounded drafting domain with associative views, sections/details/clipping, dimensions, GD&T/annotations, dress-up, BOM, standards, and related CATIA-mapped capability families.
- The viewer and downstream representations are never semantic authorities.
- Rust remains the mathematical authority only.

Current implementation projects added on the system branch:
```
UMLCAD.Science
UMLCAD.Engineering.Resources
UMLCAD.Engineering.SheetMetal
UMLCAD.Engineering.Cam
UMLCAD.Engineering.Drawing
UMLCAD.Integration.Simulation
UMLCAD.Engineering.Tests
```

Implemented foundation slice:
- immutable arithmetic expression AST and deterministic SHA-256 expression identity;
- typed scientific Quantity/QuantityDimension and Material model;
- provider-backed PhenomenaSimulationService with deterministic provider selection and result identity checks;
- Machine/Tool/Process compatibility rules;
- typed Product Structure and deterministic BOM grouping/order;
- Sheet Metal material/machine/tool/process validation plus shared bend-allowance expression;
- CAM toolpath and machine-aware deterministic G-code/NC generator with content hashing and fail-closed compatibility checks;
- CAM use of the Phenomena Simulation Service;
- Drawing capability enums/profile plus associative view state, display mode, occurrence filters, drawing BOM, and sheet presentation semantics;
- outward SimulationApplicationProvider adapter boundary;
- .NET engineering foundation tests executed by the authoritative E2E runner.

Important CATIA drafting capability mapping is documented from the official Dassault Systèmes Generative Drafting and Interactive Drafting capability descriptions. The mapped surface includes associative 3D-to-2D drafting, front/side/top/isometric views, sections and aligned/offset sections, detail/circular/profiled detail views, clipping, associative dimensions, GD&T, annotations, BOM, assembly filtering, standards, and DXF/DWG interoperability.

Current system-CAD branch:
- branch: `milestone/cad-system-s0-boundaries`
- current implementation head: `c0c22cb590195c9fc95bc317cd6102923b5dbe91`
- PR #38 remains open and unmerged.

Validation status at this snapshot:
- Logical architecture manifest validation found no upward dependency or logical dependency cycle.
- Repository project references were inspected against the architecture projection.
- Local container does not provide the .NET SDK, so local C# compilation could not be executed.
- GitHub Actions for the branch repeatedly fail before any job step executes: jobs have zero steps, runner_id 0, and terminate within seconds. The job-log endpoint currently returns `BlobNotFound`. This is recorded as CI infrastructure/unavailable execution evidence, not as proof that the new C# code compiles.
- Do not merge the PR or claim a green milestone until the authoritative Rust, comprehensive E2E, Metal, and performance workflows execute normally and pass on the exact head.

Next implementation boundary after infrastructure recovery:
1. Integrate the new typed CAD specification/reference/result contracts with the real Rust kernel through a dedicated ICadKernelEvaluator adapter; do not route them through the legacy V4 build model.
2. Replace the contract-test kernel in the S1 tests with real kernel-backed evaluation for the first certified geometry subset.
3. Establish authoritative B-Rep/topology provenance and semantic reference resolution from the real kernel result.
4. Connect Sheet Metal/CAM/Drawing/BOM to authoritative CAD results, preserving their existing domain boundaries.
5. Execute the authoritative Rust/.NET/E2E/Metal/performance gates on the exact branch head before declaring S1/S0 green or merging PR #38.

## System-CAD S1 production adapter progress

The current system-CAD implementation head is `6bc7641f2a7b2f6997e134c3469e8841e876181e`.

The .NET system-CAD boundary now contains a production `ICadKernelEvaluator` implementation and a separate CAD-owned semantic reference resolver:
- `dotnet/src/UMLCAD.Kernel.Client/RustCadKernelEvaluator.cs`
- registered by `AddRustKernel()` as the production evaluator;
- delegates the first certified geometry subset (axis-aligned box solid) to the existing typed Rust geometry transport;
- maps the kernel result into the new `Cad.Contracts.AuthoritativeCadResult` with deterministic bounds and topology provenance;
- resolves semantic planar-face references independently of topology-array ordering;
- fails closed on expected-result mismatch, malformed/unsupported selectors, ambiguous topology evidence, incomplete kernel results, and unsupported feature types;
- does not use or disguise the legacy `uml-cad-build-package/1.0.0` route.

Tests added in `dotnet/tests/UMLCAD.Engineering.Tests/RustCadKernelEvaluatorTests.cs` cover production-adapter box evaluation, semantic face resolution, ambiguity fail-closed behavior, unsupported-feature fail-closed behavior, and CadEvaluationEngine integration.

The production adapter currently certifies the axis-aligned box through the new S1 `ICadKernelEvaluator` path. A typed convex-planar-extrusion transport also exists, but it is not yet composed through that canonical S1 evaluator path and accepts polygonal profiles only. The mandatory circle-sketch → additive/subtractive solid composition therefore remains incomplete. No geometry is approximated and no CAD semantic operation is routed through the legacy `uml-cad-build-package/1.0.0` path.

Validation status:
- local container: `dotnet` SDK unavailable;
- exact-head authoritative workflows for `c0c22cb...`: all failed before job execution with `runner_id=0` and zero steps, matching the existing GitHub Actions infrastructure failure pattern;
- therefore the adapter is **Implemented / Test-authored; compilation and runtime E2E remain unverified**.

Next .NET implementation boundary:
1. establish the typed sketch/constraint transport required for the S1 semantic sketch without using the legacy build route;
2. establish a certified profile/result contract that can feed the existing convex-planar-extrusion authority without geometric approximation;
3. replace the remaining S1 contract-test kernel only when the corresponding production contracts exist;
4. then connect result provenance and downstream Sheet Metal/CAM/Drawing/BOM consumers to the real authoritative CAD-result graph.


## System-CAD coordinate-frame hardening

Commit `75dc2ea...` separated semantic reference resolution from kernel execution. The follow-on frame-hardening work adds explicit immutable right-handed `XAxis/YAxis/ZAxis` basis vectors to `CadFrame`, validates orthonormality/handedness, and includes the complete basis in deterministic evaluation identity.

This remains a .NET semantic contract change only. The existing Rust mathematical authority is unchanged.


## System-CAD tolerance authority

The .NET S1 evaluation path now carries document-level tolerance semantics end-to-end. `CadDocumentSpecification.EvaluationTolerance` participates in deterministic document/feature identity, and `KernelEvaluationRequest.Tolerance` is consumed by the production box adapter. The kernel remains unchanged.


## System-CAD frame transformation progress

The oriented `CadFrame` contract now provides deterministic local/world point and vector transforms. This makes frame orientation operational for future sketch planes, occurrence transforms, drawing views, and semantic reference context rather than storing orientation as passive metadata.

## System-CAD correction — kernel changes from previous pass reverted

A prior implementation pass incorrectly broadened the Rust/native transport with additional face-evidence fields and corresponding kernel-host/test changes. That violated the current system-CAD instruction that the existing Rust mathematical authority remain unchanged during .NET system development.

Those changes were reverted from the system branch. The code branch is restored to `c0c22cb590195c9fc95bc317cd6102923b5dbe91`, with the correction documentation committed at `6bc7641f2a7b2f6997e134c3469e8841e876181e`. The existing Rust commits already present in the branch before that point are retained; no new mathematical algorithm or Rust semantic authority was introduced by the correction.

The next implementation work must remain in the .NET semantic/evaluation/application layers unless a narrowly scoped transport exposure of an already-existing Rust capability is demonstrably required by an explicit contract.
## System-CAD runtime composition correction

A follow-on experiment briefly composed the new CAD Engine directly inside `projects/demo/Program.cs` and added a dedicated Python E2E invocation. This was also reverted because `projects/demo` remains the legacy/demo application composition root and is not yet the normative Application Host boundary defined by the C4 architecture.

No runtime composition changes from that experiment remain on the branch.

The correct current state is therefore:
- `UMLCAD.Cad.Engine` remains the system-CAD orchestration layer.
- `UMLCAD.Kernel.Client` remains the outward adapter to explicit kernel contracts, with its documented transitional Framework dependency.
- `UMLCAD.Framework` / `projects/demo` remains the legacy application path until a deliberate application-host migration milestone establishes the replacement root.
- S1 production execution is still incomplete beyond the certified box operation.
- No new Rust kernel/math change was introduced by the current correction work.

## S1 sketch/constraint transport station

- Current system-CAD branch head: `f39bd1a43ebf5bd515edd95e63eb53d9b7fa30d9`.
- The next S1 boundary is now implemented as a typed sketch-solver transport around the existing Rust constraint solver; no new solver or geometric approximation was introduced.
- `uml-cad-sketch-solve/1.0.0` is an explicit contract for circular sketch geometry, sketch frame metadata, fixed constraints, tolerance, solver terminal evidence, result identity, and diagnostics.
- `RustSketchGeometryService` provides the .NET adapter, and `RustCadKernelEvaluator` now accepts an optional typed sketch service while retaining the existing box-only constructor for compatibility.
- `CadFeatureEvaluationResult` and `KernelEvaluationResponse` can now carry `CadSketchEvaluationResult` without pretending a sketch is a solid B-Rep.
- Sketch result identity is content-addressed by operation identity, sketch identity, frame, sorted circle geometry, sorted constraint content, and tolerance. Repeated identical requests therefore have a deterministic result identity/evidence contract.
- Existing Rust solver authority remains the source of constraint mathematics. The transport supports only `Fixed` circle constraints and rejects `FullyConstrained` until a dedicated mathematical/semantic contract exists.
- Test paradigms added: contract/unit tests, transport mapping tests, Rust endpoint unit tests, protocol-level Python E2E, fail-closed negative tests, repeated-request determinism, scale metamorphic testing, and result-identity mismatch tests.
- The production circular-sketch result is intentionally **not** yet promoted to a solid B-Rep. The existing certified extrusion contract is polygonal, while the current mathematical B-Rep construction set does not certify exact circular-profile solid construction. No polygonal approximation is introduced to bridge that gap.
- Next S1 gap: establish the exact analytic profile-to-solid operation required for circle-driven additive/subtractive features, through an explicit mathematical contract only after verifying/reusing an existing Rust authority capability. Then integrate additive/subtractive feature composition, topology evolution, and the mandatory full vertical slice.
- CI status at this head: repository Actions jobs are still failing before executable steps are exposed (job step lists empty and log endpoint returns GitHub `BlobNotFound`). Therefore this head is **Implemented / Test-authored / CI-unverified**, not GREEN.
