use super::geometry::Geometry;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParameterDefinition {
    pub id: String,
    pub default_value: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeometryItem {
    pub id: String,
    pub geometry: Geometry,
    pub parameter_dependencies: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Endpoint {
    Start,
    End,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Constraint {
    Horizontal {
        entity_id: String,
    },
    Vertical {
        entity_id: String,
    },
    Coincident {
        first_geometry_id: String,
        first_point: Endpoint,
        second_geometry_id: String,
        second_point: Endpoint,
    },
    Fixed {
        entity_id: String,
    },
    Distance {
        first_geometry_id: String,
        second_geometry_id: Option<String>,
        first_endpoint: Option<Endpoint>,
        second_endpoint: Option<Endpoint>,
        value: f64,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RelationPoint {
    Endpoint {
        geometry_id: String,
        point: Endpoint,
    },
    Center {
        geometry_id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Relation {
    Parallel {
        first_geometry_id: String,
        second_geometry_id: String,
    },
    Perpendicular {
        first_geometry_id: String,
        second_geometry_id: String,
    },
    EqualLength {
        first_geometry_id: String,
        second_geometry_id: String,
    },
    Angle {
        first_geometry_id: String,
        second_geometry_id: String,
        radians: f64,
    },
    Collinear {
        first_geometry_id: String,
        second_geometry_id: String,
    },
    Concentric {
        first_geometry_id: String,
        second_geometry_id: String,
    },
    EqualRadius {
        first_geometry_id: String,
        second_geometry_id: String,
    },
    Radius {
        geometry_id: String,
        value: f64,
    },
    Diameter {
        geometry_id: String,
        value: f64,
    },
    Tangent {
        first_geometry_id: String,
        second_geometry_id: String,
        mode: TangentMode,
    },
    Midpoint {
        point: RelationPoint,
        line_geometry_id: String,
    },
    PointOnLine {
        point: RelationPoint,
        line_geometry_id: String,
    },
    PointOnCircle {
        point: RelationPoint,
        circle_geometry_id: String,
    },
    DistancePoints {
        first: RelationPoint,
        second: RelationPoint,
        value: f64,
    },
    Symmetric {
        first: RelationPoint,
        second: RelationPoint,
        about: RelationPoint,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TangentMode {
    External,
    Internal,
    Any,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SemanticSnapshot {
    pub parameters: Vec<ParameterDefinition>,
    pub geometry: Vec<GeometryItem>,
    pub constraints: Vec<(String, Constraint)>,
    pub relations: Vec<(String, Relation)>,
}

impl SemanticSnapshot {
    pub fn deterministic(mut self) -> Self {
        self.parameters.sort_by(|a, b| a.id.cmp(&b.id));
        self.geometry.sort_by(|a, b| a.id.cmp(&b.id));
        self.constraints.sort_by(|a, b| a.0.cmp(&b.0));
        self.relations.sort_by(|a, b| a.0.cmp(&b.0));
        self
    }

    pub fn geometry(&self, id: &str) -> Option<&Geometry> {
        self.geometry
            .iter()
            .find(|g| g.id == id)
            .map(|g| &g.geometry)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RelationSnapshot {
    pub base: SemanticSnapshot,
    pub relations: Vec<(String, Relation)>,
}
