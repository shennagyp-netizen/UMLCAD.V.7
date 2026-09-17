# UMLCAD V5 — Mathematical Model of the Current Native Rust Kernel

> **Status:** implementation-extracted mathematical model. This describes what `kernel_rust` computes **now**, not the intended future kernel. Items marked **DEFECT/APPROXIMATION** must not be silently treated as exact mathematics.

## 0. State space

Current production kernel: `kernel_rust/`; geometry domain: 2D line segments, circles, circular arcs. A semantic snapshot is
\[
S=(P,G,C,R)
\]
with parameters `P`, geometry `G`, constraints `C`, relations `R`.

Solver state:
\[
x=[x_1,\ldots,x_n]^T
\]
with line `[x0,y0,x1,y1]`, circle `[cx,cy,r]`, arc `[cx,cy,r,a0,a1]`.

Primitive geometry tolerance: \(\varepsilon=10^{-9}\). Topology/spatial gates commonly use \(10^{-8}\).

## 1. Geometry

Point:
\[
\|p\|=\sqrt{x^2+y^2},\quad p\cdot q=x_p x_q+y_p y_q,
\]
\[
\operatorname{cross}(p,q)=x_py_q-y_px_q,
\quad d(p,q)=\|p-q\|.
\]

Line:
\[
L(t)=p_0+t(p_1-p_0),\ t\in[0,1],\qquad \ell(L)=\|p_1-p_0\|.
\]
Valid iff endpoints are finite and \(\ell(L)>\varepsilon\).

Circle:
\[
C(\theta)=c+r(\cos\theta,\sin\theta),\quad r>\varepsilon,
\]
\[
\operatorname{circ}(C)=2\pi r.
\]

Arc:
\[
A(t)=c+r(\cos\theta(t),\sin\theta(t)),
\quad \theta(t)=a_0+t(a_1-a_0),
\]
\[
\ell(A)=r|a_1-a_0|.
\]
Valid iff all finite, \(r>\varepsilon\), \(|a_1-a_0|>\varepsilon\).

Line tangent:
\[
\tau_L=(p_1-p_0)/\|p_1-p_0\|.
\]
Circle tangent at parameter \(t\):
\[
\tau_C(t)=(-\sin2\pi t,\cos2\pi t).
\]
Circles have no semantic endpoints.

## 2. Primitive projection

Point to segment: with \(d=p_1-p_0\),
\[
t^*=\operatorname{clamp}\left(\frac{(q-p_0)\cdot d}{d\cdot d},0,1\right),
\quad q^*=p_0+t^*d.
\]
Point-to-circle distance:
\[
|\|q-c\|-r|.
\]
Point-to-arc uses radial projection, angular-span membership, then endpoint comparison.

## 3. Constraint residuals

The solver seeks
\[
R_C(x)=[r_1(x),\ldots,r_m(x)]^T\approx0.
\]

Horizontal:
\[
r=y_1-y_0.
\]
Vertical:
\[
r=x_1-x_0.
\]
Coincident endpoints:
\[
r=[a_x-b_x,\ a_y-b_y]^T.
\]
Fixed geometry: coordinate-wise difference from original geometry (4/3/5 scalar rows for line/circle/arc).

Distance:
\[
r=d(a,b)-d_0
\]
for two endpoints, or \(r=\ell(L)-d_0\), \(r=r_C-d_0\), \(r=\ell(A)-d_0\) for single line/circle/arc.

## 4. Relation residuals

Parallel line directions:
\[
r=\operatorname{cross}(a,b).
\]
Perpendicular:
\[
r=a\cdot b.
\]
Equal length:
\[
r=\ell(a)-\ell(b).
\]
Angle:
\[
r=\operatorname{wrap}_{(-\pi,\pi]}(\operatorname{atan2}(b_y,b_x)-\operatorname{atan2}(a_y,a_x)-\theta_0).
\]
Collinear lines:
\[
r_1=\operatorname{cross}(d,b_s-a_s),\quad r_2=\operatorname{cross}(d,b_e-a_s),\quad d=a_e-a_s.
\]
Concentric:
\[
r=[c_{ax}-c_{bx},c_{ay}-c_{by}]^T.
\]
Equal radius: \(r=r_a-r_b\).
Radius: \(r=r_g-r_0\). Diameter: \(r=2r_g-d_0\).

Line-circle tangent:
\[
r=\operatorname{dist}(c,L_{segment})-r_C.
\]
Circle-circle tangent, \(d=\|c_a-c_b\|\):
\[
r_{ext}=d-r_a-r_b,\qquad r_{int}=d-|r_a-r_b|.
\]
`Any` selects whichever has smaller absolute residual; this is non-smooth at switching points.

Midpoint:
\[
m=(a+b)/2,\quad r=p-m.
\]
Point-on-line:
\[
r=|\operatorname{cross}(p-a,d)|/\|d\|.
\]
Point-on-circle:
\[
r=\|p-c\|-r_C.
\]
Point-distance:
\[
r=\|p_a-p_b\|-d_0.
\]
Symmetry intended equation:
\[
(a+b)/2-o=0.
\]
**Current defect:** code returns this plus the same equation multiplied by 2, so two residual rows are algebraically redundant.

## 5. Residual scaling actually implemented

Relations define scales and
\[
\|r\|_s=\sqrt{\sum_i(r_i/s_i)^2}.
\]
Examples: parallel/perpendicular scale by \(\max(\|a\|,\|b\|,1)^2\); equal-length by \(\max(|\ell_a|,|\ell_b|,1)\); concentric by \(\max(r_a,r_b,1)\); point-on-line by \(\max(\ell(L),1)\).

**Current defect:** ordinary constraint residuals are copied unchanged into `scaled`; the solver therefore lacks complete physical residual scaling.

## 6. Nonlinear solver actually running

Raw residual vector:
\[
R(x)=\begin{bmatrix}R_C(x)\\R_R(x)\end{bmatrix}.
\]

Forward finite-difference Jacobian:
\[
h_j=10^{-7}\max(|x_j|,1),
\qquad J_{ij}\approx[R_i(x+h_je_j)-R_i(x)]/h_j.
\]

Column normalization:
\[
c_j=\|J_{:,j}\|_2,
\quad A_{:,j}=J_{:,j}/c_j\ (c_j>0).
\]

Despite the function name `scaled_damped_qr`, implementation uses **SVD**, not QR:
\[
A=U\Sigma V^T.
\]
With damping \(\lambda=d\):
\[
z=\sum_k v_k\frac{\sigma_k}{\sigma_k^2+\lambda}(u_k^T(-R)),
\qquad \Delta x_j=z_j/c_j.
\]
This is SVD-based damped least squares/Tikhonov after Jacobian column normalization.

Rank threshold:
\[
\tau_{rank}=10^{-10}\max(\sigma_{max},1).
\]
Rank = number of finite singular values above threshold.

Condition estimate:
\[
\kappa=\sigma_{max}/\sigma_{min,retained}.
\]
No retained singular value ⇒ \(\kappa=\infty\). Current `well_conditioned` means \(\kappa<10^{10}\).

Candidate \(x'=x+\Delta x\) is accepted only if the computed scaled residual norm decreases. Damping becomes `0.3*d` after success and `10*d` after failure, clipped to \([10^{-12},10^{12}]\).

Defaults:
\[
N_{max}=100,\quad \tau_R=10^{-8},\quad \tau_{step}=10^{-10},\quad d_0=10^{-3},\quad h=10^{-7}.
\]

**Current defect:** `step_tolerance` is defined but not used to terminate/accept iterations. Convergence is residual-based.

**Current defect:** if initial residual is already small, analysis reports rank 0 and condition 1 without SVD; these are placeholders, not measurements.

## 7. DOF

Current local numerical DOF:
\[
DOF=n_{var}-\operatorname{rank}(J_{normalized}).
\]
This is a local Jacobian estimate, not a global configuration-space proof.

## 8. Spatial model

AABB is broad phase only.

Line-line: segment intersection via line parameters, otherwise minimum of four endpoint-to-segment distances; exact within tolerance.

Circle-circle with center distance \(d\): zero for intersection/touching when
\[
|r_a-r_b|-\varepsilon\le d\le r_a+r_b+\varepsilon.
\]

Line-circle:
\[
\max(0,\operatorname{dist}(c,L_{segment})-r).
\]

**APPROXIMATION:** line-arc uses circle-level proximity plus selected candidates; arc-circle uses center/endpoint candidates; arc-arc uses endpoints and midpoint samples. These are not complete analytic arc-pair minimum-distance/intersection algorithms.

## 9. Topology

Line/arc endpoints create vertices; circles create closed edges. Vertices merge if
\[
\|p_i-p_j\|\le10^{-8}.
\]
Vertex degree:
\[
d(v)=\#\{e:e\text{ incident on }v\}.
\]
Current gate rejects \(d(v)>2\); wire traversal rejects more than one unused continuation.

This is a wire/connectivity test, not a proof of planar faces, shells, or solids.

## 10. Validation and engineering gate

Structural validity:
\[
E_s=\text{no Error diagnostic from snapshot validation}.
\]

Current engineering evidence computes
\[
E_c=\neg(\text{contradictory constraint IDs}),
\]
\[
E_r=\text{every relation evaluation returns finite residuals},
\]
\[
E_n=true,
\]
\[
E_{ref}=\neg(\text{stale-reference Error}),
\]
\[
E_t=E_s\land buildTopology(S)\text{ succeeds},
\]
\[
E_x=E_s\land\text{all spatial distances are finite},
\]
\[
E_{dxf}=\text{DXF contains `EOF`}.
\]
Final gate:
\[
E_{engineering}=E_s\land E_c\land E_r\land E_n\land E_{ref}\land E_t\land E_x\land E_{dxf}.
\]

### Critical current defects

1. `E_n` is hard-coded true; conditioning is not wired into acceptance.
2. `E_r` checks finiteness, not relation satisfaction.
3. Engineering evidence is independent of `Solve` result in the dispatcher.
4. A failed/poorly-conditioned solve can therefore coexist with an engineering pass.
5. Spatial validity checks finiteness, not a semantic intersection rule.
6. DXF validity is only an `EOF` check.

Thus the current gate is structural validation, not a mathematical proof of a fully solved engineering state.

## 11. Dimensions

Length:
\[
D_L(g)=
\begin{cases}
\|p_1-p_0\|&Line\\
2\pi r&Circle\\
r|a_1-a_0|&Arc
\end{cases}
\]
Radius: \(D_R=r\) for circle/arc. Diameter: \(D_D=2r\).

Current distance dimensions are endpoint-to-endpoint for line/arc entities; circles are invalid for this path.

Angle for lines:
\[
D_\theta=\operatorname{atan2}(a_xb_y-a_yb_x,a_xb_x+a_yb_y).
\]

## 12. Mathematical status

### Strong/exact current core

Primitive vector algebra; line length/projection; circle radius/circumference; arc parameterization/length; line-line and circle-circle spatial core; core constraint/relation residuals; SVD damped least-squares step; deterministic state encoding.

### Approximate/insufficient

Forward finite-difference Jacobian; incomplete physical residual scaling; non-smooth `Any` tangent; redundant symmetry rows; arc-pair spatial narrow phases; wire-only topology; conditioning absent from engineering gate; relation finiteness mistaken for satisfaction; engineering acceptance independent of solve success; zero-residual placeholder rank/condition; `EOF` export gate.

## 13. Mathematical hardening protocol

Do not patch formulas ad hoc. Use
\[
\boxed{\text{mathematical model}}
\rightarrow
\boxed{\text{domains/invariants}}
\rightarrow
\boxed{\text{exact vs approximation}}
\rightarrow
\boxed{\text{numerical policy}}
\rightarrow
\boxed{\text{implementation}}
\rightarrow
\boxed{\text{property tests}}
\rightarrow
\boxed{\text{differential tests}}
\rightarrow
\boxed{\text{adversarial tests}}.
\]

For every primitive/relation define domain, parameterization, residual, units, scale, Jacobian, singular/degenerate cases, tolerance, acceptance predicate, invariant, properties and independent reference oracle.

Recommended nonlinear target:
\[
\min_{\Delta x}
\frac12\|W_r(R(x)+J\Delta x)\|_2^2
+\frac{\lambda}{2}\|W_x\Delta x\|_2^2,
\]
with explicit residual scale \(W_r\) and variable-characteristic scale \(W_x\). The current implementation does not yet implement this complete formulation.

Expose at least
\[
rank,\sigma_{min},\sigma_{max},\kappa,\|R\|_2,\|W_rR\|_2,\|W_x\Delta x\|_2.
\]
Accepted state must require all applicable structural, constraint, relation, conditioning, topology, reference and engineering predicates.

## 14. Verification basis

Independent mathematical oracles should test translation/rotation invariance; scale behavior; tangent/non-tangent cases; coincident/separated/nested circles; segment intersection/separation; arc branch cuts; degenerate and near-degenerate primitives; singular/nearly-singular Jacobians; redundant/contradictory/under-constrained systems; stale references; topology degree 0/1/2/>2; deterministic re-evaluation/serialization.

The expected result must not be generated by the same implementation under test.
