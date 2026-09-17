use super::geometry::{Geometry, Point};
use super::snapshot::{GeometryItem, SemanticSnapshot};

#[derive(Clone, Debug, PartialEq)]
pub struct TopologyVertex {
    pub id: String,
    pub point: Point,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TopologyEdge {
    pub id: String,
    pub geometry_id: String,
    pub start_vertex_id: Option<String>,
    pub end_vertex_id: Option<String>,
    pub geometry: Geometry,
    pub closed: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TopologyWire {
    pub id: String,
    pub edge_ids: Vec<String>,
    pub directions: Vec<bool>,
    pub closed: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TopologyModel {
    pub vertices: Vec<TopologyVertex>,
    pub edges: Vec<TopologyEdge>,
    pub wires: Vec<TopologyWire>,
}

fn endpoint(g: &Geometry, start: bool) -> Option<Point> {
    match g {
        Geometry::Line(l) => Some(if start { l.start } else { l.end }),
        Geometry::Arc(a) => Some(if start { a.start_point() } else { a.end_point() }),
        Geometry::Circle(_) => None,
    }
}
fn near(a: Point, b: Point, t: f64) -> bool { a.distance(b) <= t }

fn model_scale(items: &[GeometryItem]) -> f64 {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut count = 0usize;

    for item in items {
        for start in [true, false] {
            let Some(point) = endpoint(&item.geometry, start) else { continue };
            if !point.x.is_finite() || !point.y.is_finite() { continue; }
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
            count += 1;
        }
    }

    if count < 2 {
        return 0000.0;
    }
    let scale = (max_x - min_x).hypot(max_y - min_y);
    if scale.is_finite() && scale > 0.0 { scale } else { 0000.0 }
}

fn topology_tolerance(items: &[GeometryItem]) -> f64 {
    let scale = model_scale(items);
    if scale > 0.0 {
        // Vertex equivalence is relative to the model's geometric extent.
        // This preserves the same topology under uniform unit/scale changes.
        1.0e-8 * scale
    } else {
        0000.0
    }
}

fn incident_degree(edge: &TopologyEdge, vertex_id: &str) -> usize {
    if edge.closed { return 0; }
    usize::from(edge.start_vertex_id.as_deref() == Some(vertex_id))
        + usize::from(edge.end_vertex_id.as_deref() == Some(vertex_id))
}

pub fn build_topology(snapshot: &SemanticSnapshot) -> Result<TopologyModel, String> {
    let mut items = snapshot.geometry.clone();
    items.sort_by(|a, b| a.id.cmp(&b.id));
    let tolerance = topology_tolerance(&items);

    let mut vertices = Vec::<TopologyVertex>::new();
    let mut edges = Vec::<TopologyEdge>::new();
    let mut vertex = |p: Point| {
        if let Some(v) = vertices.iter().find(|v| near(v.point, p, tolerance)) {
            return v.id.clone();
        }
        let id = format!("v{}", vertices.len() + 1);
        vertices.push(TopologyVertex { id: id.clone(), point: p });
        id
    };

    for item in &items {
        let s = endpoint(&item.geometry, true);
        let e = endpoint(&item.geometry, false);
        let sv = s.map(&mut vertex);
        let ev = e.map(&mut vertex);
        edges.push(TopologyEdge {
            id: format!("e{}", edges.len() + 1),
            geometry_id: item.id.clone(),
            start_vertex_id: sv.clone(),
            end_vertex_id: ev.clone(),
            geometry: item.geometry,
            closed: sv.is_none() && ev.is_none(),
        });
    }

    for v in &vertices {
        let degree: usize = edges.iter().map(|e| incident_degree(e, &v.id)).sum();
        if degree > 2 {
            return Err(format!("Non-manifold topology: vertex {} has degree {}", v.id, degree));
        }
    }

    let mut wires = Vec::<TopologyWire>::new();
    let mut left: std::collections::BTreeSet<String> = edges
        .iter()
        .filter(|e| !e.closed)
        .map(|e| e.id.clone())
        .collect();

    for e in &edges {
        if e.closed {
            wires.push(TopologyWire {
                id: format!("w{}", wires.len() + 1),
                edge_ids: vec![e.id.clone()],
                directions: vec![true],
                closed: true,
            });
        }
    }

    while let Some(seed_id) = left.iter().next().cloned() {
        let seed = edges.iter().find(|e| e.id == seed_id).unwrap();
        let start = seed.start_vertex_id.clone();
        let end = seed.end_vertex_id.clone();

        if start.is_none() || end.is_none() {
            left.remove(&seed_id);
            wires.push(TopologyWire {
                id: format!("w{}", wires.len() + 1),
                edge_ids: vec![seed.id.clone()],
                directions: vec![true],
                closed: false,
            });
            continue;
        }

        let degree_of = |vertex_id: &str| -> usize {
            edges.iter().map(|e| incident_degree(e, vertex_id)).sum()
        };
        let sx = start.as_ref().unwrap();
        let ex = end.as_ref().unwrap();
        let origin = {
            let ds = degree_of(sx);
            let de = degree_of(ex);
            if ds == 1 && de != 1 { sx.clone() }
            else if de == 1 && ds != 1 { ex.clone() }
            else { sx.min(ex).clone() }
        };

        left.remove(&seed_id);
        let forward = seed.start_vertex_id.as_deref() == Some(&origin);
        let mut current = if forward { seed.end_vertex_id.clone() } else { seed.start_vertex_id.clone() };
        let mut edge_ids = vec![seed.id.clone()];
        let mut directions = vec![forward];

        while current.as_ref() != Some(&origin) {
            let candidates: Vec<&TopologyEdge> = edges.iter().filter(|e| {
                left.contains(&e.id) && !e.closed
                    && (e.start_vertex_id == current || e.end_vertex_id == current)
            }).collect();
            if candidates.len() > 1 {
                return Err(format!("Ambiguous topology at vertex {}", current.unwrap()));
            }
            let Some(next) = candidates.first().copied() else { break; };
            left.remove(&next.id);
            let forward = next.start_vertex_id == current;
            edge_ids.push(next.id.clone());
            directions.push(forward);
            current = if forward { next.end_vertex_id.clone() } else { next.start_vertex_id.clone() };
        }

        wires.push(TopologyWire {
            id: format!("w{}", wires.len() + 1),
            edge_ids,
            directions,
            closed: current == Some(origin),
        });
    }

    Ok(TopologyModel { vertices, edges, wires })
}