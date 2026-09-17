pub mod analytic;
pub mod analytic_surfaces;
pub mod bezier;
pub mod bspline;
pub mod constraints;
pub mod conics3d;
pub mod conditioning;
pub mod constants;
pub mod convergence;
pub mod curve_differential;
pub mod curves3d;
pub mod diagnostics;
pub mod dimensions;
pub mod distance;
pub mod dxf;
pub mod engineering;
pub mod geometry;
pub mod interval;
pub mod intersections;
pub mod jacobian;
pub mod jacobian_authority_exhaustive;
pub mod linalg;
#[cfg(test)]
mod linalg_authority_exhaustive;
pub mod linear_consistency;
pub mod mat;
pub mod nurbs;
pub mod nurbs3d;
pub mod nurbs_ops;
pub mod nurbs_surface;
pub mod nurbs_surface_differential;
pub mod nurbs_surface_ops;
pub mod offsets;
pub mod polynomial;
pub mod predicates;
pub mod quaternion;
pub mod relation_jacobian;
pub mod relations;
pub mod scalar;
pub mod segments;
pub mod snapshot;
pub mod solid;
pub mod spatial;
pub mod spatial_accel;
#[path = "solver.rs"]
mod solver_legacy;
pub mod terminal_authority;
pub mod solver {
    pub use super::terminal_authority::*;
}
pub mod solver_status;
pub mod surfaces;
pub mod surface_differential;
pub mod sweeps;
pub mod tessellation;
pub mod tolerance;
pub mod topology;
pub mod transform;
pub mod trim;
pub mod validation;
pub mod vec;
pub mod vec4;

#[cfg(test)]
mod convergence_metamorphic;