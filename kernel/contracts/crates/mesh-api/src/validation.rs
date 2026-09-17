use super::{Mesh, MeshError};

pub fn triangle_area2(mesh: &Mesh, a: usize, b: usize, c: usize) -> Result<f64, MeshError> {
    if a >= mesh.vertices.len() || b >= mesh.vertices.len() || c >= mesh.vertices.len() { return Err(MeshError::IndexOutOfBounds); }
    let p = mesh.vertices[a]; let q = mesh.vertices[b]; let r = mesh.vertices[c];
    let ux = q.x-p.x; let uy=q.y-p.y; let uz=q.z-p.z;
    let vx = r.x-p.x; let vy=r.y-p.y; let vz=r.z-p.z;
    let cx = uy*vz-uz*vy; let cy=uz*vx-ux*vz; let cz=ux*vy-uy*vx;
    Ok(cx*cx+cy*cy+cz*cz)
}
