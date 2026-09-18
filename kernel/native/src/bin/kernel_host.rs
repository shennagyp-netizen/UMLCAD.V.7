use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use serde_json::{json, Value};
use umlcad_kernel_rust::api::{dispatch, KernelRequest, KernelResponse};
use umlcad_kernel_rust::functions::snapshot::{Constraint, GeometryItem, SemanticSnapshot};
use umlcad_kernel_rust::functions::validation::Severity;
use umlcad_kernel_rust::{Arc, Circle, Geometry, Line, Point};

const ADDRESS: &str = "127.0.0.1:8080";
const BUILD_SCHEMA: &str = "uml-cad-build-package/1.0.0";
const MODEL_SCHEMA: &str = "uml-cad-compiled-model/1.1.0";
const BOX_SOLID_SCHEMA: &str = "uml-cad-axis-aligned-box-solid/1.0.0";
const MAX_REQUEST_BYTES: usize = 32 * 1024 * 1024;

fn main() {
    let listener = TcpListener::bind(ADDRESS).expect("bind kernel host");
    for stream in listener.incoming().flatten() {
        thread::spawn(|| {
            let _ = handle(stream);
        });
    }
}

fn handle(mut stream: TcpStream) -> std::io::Result<()> {
    let request = read_request(&mut stream)?;
    let (method, path, body) = parse_request(&request)?;
    let (status, response) = match (method.as_str(), path.as_str()) {
        ("POST", "/v1/build/evaluate") => evaluate(&body),
        ("POST", "/v1/geometry/box-solid") => box_solid(&body),
        ("POST", "/v1/geometry/extrude-convex-planar-profile") => extrude_convex_planar_profile(&body),
        ("GET", "/health") => (200, json!({"status":"ok"})),
        _ => (404, failure("KERNEL_NOT_FOUND", "Unknown kernel endpoint.")),
    };
    write_response(&mut stream, status, response.to_string().as_bytes())
}

fn read_request(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    let mut data = Vec::with_capacity(8192);
    let mut chunk = [0_u8; 8192];
    loop {
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        data.extend_from_slice(&chunk[..n]);
        if data.len() > MAX_REQUEST_BYTES {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "HTTP request is too large",
            ));
        }
        if let Some(end) = header_end(&data) {
            let headers = String::from_utf8_lossy(&data[..end]);
            if is_chunked(&headers) {
                if chunked_body_complete(&data[end..]) {
                    break;
                }
            } else if let Some(length) = parse_content_length(&headers) {
                if data.len() >= end + length {
                    break;
                }
            } else {
                break;
            }
        }
    }
    Ok(data)
}

fn header_end(data: &[u8]) -> Option<usize> {
    data.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|p| p + 4)
}

fn parse_request(data: &[u8]) -> std::io::Result<(String, String, Vec<u8>)> {
    let end = header_end(data).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "HTTP headers are incomplete",
        )
    })?;
    let headers = String::from_utf8_lossy(&data[..end]);
    let first = headers.lines().next().unwrap_or_default();
    let mut parts = first.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let path = parts.next().unwrap_or_default().to_string();
    let raw_body = data[end..].to_vec();
    let body = if is_chunked(&headers) {
        decode_chunked_body(&raw_body)?
    } else if let Some(length) = parse_content_length(&headers) {
        if raw_body.len() < length {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "HTTP body is incomplete",
            ));
        }
        raw_body[..length].to_vec()
    } else {
        raw_body
    };
    Ok((method, path, body))
}

fn parse_content_length(headers: &str) -> Option<usize> {
    headers.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        if key.trim().eq_ignore_ascii_case("content-length") {
            value.trim().parse::<usize>().ok()
        } else {
            None
        }
    })
}

fn is_chunked(headers: &str) -> bool {
    headers.lines().any(|line| {
        let (key, value) = match line.split_once(':') {
            Some(parts) => parts,
            None => return false,
        };
        key.trim().eq_ignore_ascii_case("transfer-encoding")
            && value.to_ascii_lowercase().contains("chunked")
    })
}

fn chunked_body_complete(data: &[u8]) -> bool {
    data.windows(5).any(|w| w == b"0\r\n\r\n")
}

fn decode_chunked_body(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut cursor = 0;
    let mut output = Vec::new();
    loop {
        let line_end = data[cursor..]
            .windows(2)
            .position(|w| w == b"\r\n")
            .map(|p| cursor + p)
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Chunk size line is incomplete",
                )
            })?;
        let size_line = String::from_utf8_lossy(&data[cursor..line_end]);
        let size_text = size_line.split(';').next().unwrap_or("").trim();
        let size = usize::from_str_radix(size_text, 16).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid chunk size")
        })?;
        cursor = line_end + 2;
        if size == 0 {
            break;
        }
        if cursor + size + 2 > data.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Chunk data is incomplete",
            ));
        }
        output.extend_from_slice(&data[cursor..cursor + size]);
        cursor += size;
        if &data[cursor..cursor + 2] != b"\r\n" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Chunk terminator is invalid",
            ));
        }
        cursor += 2;
        if output.len() > MAX_REQUEST_BYTES {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "HTTP request body is too large",
            ));
        }
    }
    Ok(output)
}

fn write_response(stream: &mut TcpStream, status: u16, body: &[u8]) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        404 => "Not Found",
        422 => "Unprocessable Entity",
        _ => "Internal Server Error",
    };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)
}

fn extrude_convex_planar_profile(body: &[u8]) -> (u16, Value) {
    let root: Value = match serde_json::from_slice(body) {
        Ok(value) => value,
        Err(error) => {
            return (
                422,
                failure("KERNEL_INVALID_JSON", &format!("The request is not valid JSON: {error}.")),
            )
        }
    };

    if root.get("schema").and_then(Value::as_str)
        != Some("uml-cad-extrude-convex-planar-profile/1.0.0")
    {
        return (
            422,
            failure(
                "KERNEL_EXTRUSION_SCHEMA",
                "Unsupported convex planar extrusion schema.",
            ),
        );
    }

    let operation_identity = match root.get("operationIdentity").and_then(Value::as_str) {
        Some(value) if !value.trim().is_empty() => value.to_string(),
        _ => return (
            422,
            failure("KERNEL_EXTRUSION_SCHEMA", "operationIdentity is required."),
        ),
    };

    let origin = match vector3(root.get("origin")) {
        Ok(value) => value,
        Err(message) => return (422, failure("KERNEL_EXTRUSION_FRAME", message)),
    };
    let u_direction = match vector3(root.get("uDirection")) {
        Ok(value) => value,
        Err(message) => return (422, failure("KERNEL_EXTRUSION_FRAME", message)),
    };
    let v_direction = match vector3(root.get("vDirection")) {
        Ok(value) => value,
        Err(message) => return (422, failure("KERNEL_EXTRUSION_FRAME", message)),
    };

    let profile = match root.get("profile").and_then(Value::as_array) {
        Some(values) if values.len() >= 3 => {
            let mut points = Vec::with_capacity(values.len());
            for value in values {
                let object = match value.as_object() {
                    Some(object) => object,
                    None => return (
                        422,
                        failure(
                            "KERNEL_EXTRUSION_PROFILE",
                            "Profile point must be an object.",
                        ),
                    ),
                };
                let u = match object.get("u").and_then(Value::as_f64) {
                    Some(value) if value.is_finite() => value,
                    _ => return (422, failure("KERNEL_EXTRUSION_PROFILE", "Profile u must be finite.")),
                };
                let v = match object.get("v").and_then(Value::as_f64) {
                    Some(value) if value.is_finite() => value,
                    _ => return (422, failure("KERNEL_EXTRUSION_PROFILE", "Profile v must be finite.")),
                };
                points.push(umlcad_kernel_rust::math::vec::Vec2::new(u, v));
            }
            points
        }
        Some(_) => return (
            422,
            failure(
                "KERNEL_EXTRUSION_PROFILE",
                "Profile requires at least three points.",
            ),
        ),
        None => return (
            422,
            failure("KERNEL_EXTRUSION_PROFILE", "Profile array is required."),
        ),
    };

    let depth = match root.get("depth").and_then(Value::as_f64) {
        Some(value) if value.is_finite() && value > 0.0 => value,
        _ => return (
            422,
            failure(
                "KERNEL_EXTRUSION_GEOMETRY",
                "Depth must be finite and positive.",
            ),
        ),
    };

    let tolerance_value = root.get("tolerance").and_then(Value::as_object);
    let absolute = tolerance_value
        .and_then(|object| object.get("absolute"))
        .and_then(Value::as_f64)
        .unwrap_or(1.0e-9);
    let relative = tolerance_value
        .and_then(|object| object.get("relative"))
        .and_then(Value::as_f64)
        .unwrap_or(1.0e-9);

    let tolerance = match umlcad_kernel_rust::math::tolerance::Tolerance::new(absolute, relative) {
        Ok(value) => value,
        Err(_) => return (
            422,
            failure(
                "KERNEL_EXTRUSION_TOLERANCE",
                "Tolerance must be finite and non-negative.",
            ),
        ),
    };

    let region = umlcad_kernel_rust::math::brep::PlanarRegion3 {
        origin,
        u_dir: u_direction,
        v_dir: v_direction,
        outer: profile,
        holes: Vec::new(),
    };

    let response = dispatch(KernelRequest::ExtrudeConvexPlanarProfile {
        operation_identity,
        region,
        depth,
        tolerance,
    });

    match response {
        Ok(KernelResponse::Extrusion(report)) => (
            200,
            json!({
                "schema": "uml-cad-extrude-convex-planar-profile/1.0.0",
                "status": "succeeded",
                "succeeded": true,
                "resultId": report.result_id,
                "evidenceHash": report.evidence_hash,
                "topology": report.topology.iter().map(|entry| json!({
                    "kind": entry.kind,
                    "key": entry.key
                })).collect::<Vec<_>>(),
                "volume": report.volume,
                "surfaceArea": report.surface_area,
                "centroid": {
                    "x": report.centroid.x,
                    "y": report.centroid.y,
                    "z": report.centroid.z
                },
                "diagnostics": []
            }),
        ),
        Ok(_) => (
            500,
            failure("KERNEL_DISPATCH", "Extrusion dispatch returned the wrong response."),
        ),
        Err(error) => (
            200,
            json!({
                "schema": "uml-cad-extrude-convex-planar-profile/1.0.0",
                "status": "failed",
                "succeeded": false,
                "resultId": null,
                "evidenceHash": null,
                "topology": [],
                "volume": null,
                "surfaceArea": null,
                "centroid": null,
                "diagnostics": [{
                    "code": "KERNEL_EXTRUSION",
                    "severity": "error",
                    "message": error.to_string()
                }]
            }),
        ),
    }
}


fn box_solid(body: &[u8]) -> (u16, Value) {
    let root: Value = match serde_json::from_slice(body) {
        Ok(value) => value,
        Err(error) => {
            return (
                422,
                failure("KERNEL_INVALID_JSON", &format!("The request is not valid JSON: {error}.")),
            )
        }
    };

    if root.get("schema").and_then(Value::as_str) != Some(BOX_SOLID_SCHEMA) {
        return (422, failure("KERNEL_BOX_SOLID_SCHEMA", "Unsupported box-solid schema."));
    }

    let operation_identity = match root.get("operationIdentity").and_then(Value::as_str) {
        Some(value) if !value.trim().is_empty() => value.to_string(),
        _ => return (422, failure("KERNEL_BOX_SOLID_SCHEMA", "operationIdentity is required.")),
    };

    let min = match vector3(root.get("min")) {
        Ok(value) => value,
        Err(message) => return (422, failure("KERNEL_BOX_SOLID_GEOMETRY", message)),
    };
    let max = match vector3(root.get("max")) {
        Ok(value) => value,
        Err(message) => return (422, failure("KERNEL_BOX_SOLID_GEOMETRY", message)),
    };

    let tolerance_value = root.get("tolerance").and_then(Value::as_object);
    let absolute = tolerance_value
        .and_then(|object| object.get("absolute"))
        .and_then(Value::as_f64)
        .unwrap_or(1.0e-9);
    let relative = tolerance_value
        .and_then(|object| object.get("relative"))
        .and_then(Value::as_f64)
        .unwrap_or(1.0e-9);

    let tolerance = match umlcad_kernel_rust::math::tolerance::Tolerance::new(absolute, relative) {
        Ok(value) => value,
        Err(_) => return (422, failure("KERNEL_BOX_SOLID_TOLERANCE", "Tolerance must be finite and non-negative.")),
    };

    let response = dispatch(KernelRequest::BuildAxisAlignedBoxSolid {
        operation_identity,
        bounds: umlcad_kernel_rust::math::brep::AxisAlignedBox { min, max },
        tolerance,
    });

    match response {
        Ok(KernelResponse::BoxSolid(report)) => (
            200,
            json!({
                "schema": BOX_SOLID_SCHEMA,
                "status": "succeeded",
                "succeeded": true,
                "resultId": report.result_id,
                "evidenceHash": report.evidence_hash,
                "topology": report.topology.iter().map(|entry| json!({
                    "kind": entry.kind,
                    "key": entry.key
                })).collect::<Vec<_>>(),
                "volume": report.volume,
                "surfaceArea": report.surface_area,
                "centroid": {
                    "x": report.centroid.x,
                    "y": report.centroid.y,
                    "z": report.centroid.z
                },
                "diagnostics": []
            }),
        ),
        Ok(_) => (500, failure("KERNEL_DISPATCH", "Box-solid dispatch returned the wrong response.")),
        Err(error) => (
            200,
            json!({
                "schema": BOX_SOLID_SCHEMA,
                "status": "failed",
                "succeeded": false,
                "resultId": null,
                "evidenceHash": null,
                "topology": [],
                "volume": null,
                "surfaceArea": null,
                "centroid": null,
                "diagnostics": [{
                    "code": "KERNEL_BOX_SOLID",
                    "severity": "error",
                    "message": error.to_string()
                }]
            }),
        ),
    }
}

fn vector3(value: Option<&Value>) -> Result<umlcad_kernel_rust::math::vec::Vec3, &'static str> {
    let object = value
        .and_then(Value::as_object)
        .ok_or("3D vector object is required.")?;
    let x = object.get("x").and_then(Value::as_f64).ok_or("Vector x must be numeric.")?;
    let y = object.get("y").and_then(Value::as_f64).ok_or("Vector y must be numeric.")?;
    let z = object.get("z").and_then(Value::as_f64).ok_or("Vector z must be numeric.")?;
    if !x.is_finite() || !y.is_finite() || !z.is_finite() {
        return Err("Vector components must be finite.");
    }
    Ok(umlcad_kernel_rust::math::vec::Vec3::new(x, y, z))
}

fn evaluate(body: &[u8]) -> (u16, Value) {
    let root: Value = match serde_json::from_slice(body) {
        Ok(value) => value,
        Err(error) => {
            return (
                422,
                failure(
                    "KERNEL_INVALID_JSON",
                    &format!("The build package is not valid JSON: {error}."),
                ),
            )
        }
    };
    if root.get("schema").and_then(Value::as_str) != Some(BUILD_SCHEMA) {
        return (
            422,
            failure("KERNEL_BUILD_SCHEMA", "Unsupported build package schema."),
        );
    }
    let semantic = match root.get("semantic") {
        Some(value) if value.is_object() => value,
        _ => {
            return (
                422,
                failure(
                    "KERNEL_BUILD_SCHEMA",
                    "Build package semantic payload is missing.",
                ),
            )
        }
    };
    let snapshot = match snapshot_from_package(semantic) {
        Ok(snapshot) => snapshot,
        Err((code, message)) => return (422, failure(code, message)),
    };

    let diagnostics = match dispatch(KernelRequest::Validate {
        snapshot: snapshot.clone(),
    }) {
        Ok(KernelResponse::Diagnostics(value)) => value,
        Ok(_) => {
            return (
                500,
                failure(
                    "KERNEL_DISPATCH",
                    "Validate dispatch returned the wrong response.",
                ),
            )
        }
        Err(error) => return (500, failure("KERNEL_DISPATCH", &error.to_string())),
    };
    if diagnostics.iter().any(|d| d.severity == Severity::Error) {
        return (
            200,
            json!({
                "succeeded": false,
                "compiledModel": null,
                "diagnostics": diagnostics.iter().map(|d| diagnostic(&d.code, severity(d.severity.clone()), &d.message)).collect::<Vec<_>>()
            }),
        );
    }

    match dispatch(KernelRequest::Solve {
        snapshot: snapshot.clone(),
        options: Default::default(),
    }) {
        Ok(KernelResponse::Solve(_)) => {}
        Ok(_) => {
            return (
                500,
                failure(
                    "KERNEL_DISPATCH",
                    "Solve dispatch returned the wrong response.",
                ),
            )
        }
        Err(error) => return (200, failure("KERNEL_SOLVE", &error.to_string())),
    }
    match dispatch(KernelRequest::EngineeringEvidence { snapshot }) {
        Ok(KernelResponse::Engineering(evidence)) if evidence.engineering_rule_validity => {}
        Ok(KernelResponse::Engineering(_)) => {
            return (
                200,
                failure(
                    "KERNEL_ENGINEERING",
                    "Engineering validation rejected the build package.",
                ),
            )
        }
        Ok(_) => {
            return (
                500,
                failure(
                    "KERNEL_DISPATCH",
                    "Engineering dispatch returned the wrong response.",
                ),
            )
        }
        Err(error) => return (200, failure("KERNEL_ENGINEERING", &error.to_string())),
    }

    (
        200,
        json!({
            "succeeded": true,
            "compiledModel": compiled_package(&root, semantic),
            "diagnostics": []
        }),
    )
}

fn snapshot_from_package(
    semantic: &Value,
) -> Result<SemanticSnapshot, (&'static str, &'static str)> {
    let mut geometry = Vec::new();
    let mut constraints = Vec::new();
    let parts = semantic
        .get("parts")
        .and_then(Value::as_array)
        .ok_or(("KERNEL_BUILD_SCHEMA", "Parts must be an array."))?;
    for part in parts {
        let part_id = part
            .get("id")
            .and_then(Value::as_str)
            .ok_or(("KERNEL_BUILD_SCHEMA", "Part ID is missing."))?;
        for item in part
            .get("geometry")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let id = item
                .get("id")
                .and_then(Value::as_str)
                .ok_or(("KERNEL_BUILD_SCHEMA", "Geometry ID is missing."))?;
            let kind = item.get("kind").and_then(Value::as_str).unwrap_or_default();
            let props = item
                .get("properties")
                .and_then(Value::as_object)
                .ok_or(("KERNEL_BUILD_SCHEMA", "Geometry properties are missing."))?;
            let geom = match kind {
                "line" => Geometry::Line(Line {
                    start: point(props, "start")?,
                    end: point(props, "end")?,
                }),
                "circle" => Geometry::Circle(Circle {
                    center: point(props, "center")?,
                    radius: number(props, "radius")?,
                }),
                "arc" => Geometry::Arc(Arc {
                    center: point(props, "center")?,
                    radius: number(props, "radius")?,
                    start_angle: number(props, "startAngle")?,
                    end_angle: number(props, "endAngle")?,
                }),
                _ => return Err(("KERNEL_UNSUPPORTED_GEOMETRY", "Unsupported geometry kind.")),
            };
            geometry.push(GeometryItem {
                id: format!("{part_id}:{id}"),
                geometry: geom,
                parameter_dependencies: vec![],
            });
        }
        for item in part
            .get("constraints")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let id = item
                .get("id")
                .and_then(Value::as_str)
                .ok_or(("KERNEL_BUILD_SCHEMA", "Constraint ID is missing."))?;
            let kind = item.get("kind").and_then(Value::as_str).unwrap_or_default();
            let refs = item
                .get("references")
                .and_then(Value::as_array)
                .ok_or(("KERNEL_BUILD_SCHEMA", "Constraint references are missing."))?;
            let first = refs
                .first()
                .and_then(Value::as_str)
                .map(|v| format!("{part_id}:{v}"))
                .ok_or(("KERNEL_BUILD_SCHEMA", "Constraint reference is missing."))?;
            let constraint = match kind {
                "horizontal" => Constraint::Horizontal { entity_id: first },
                "vertical" => Constraint::Vertical { entity_id: first },
                "fixed" => Constraint::Fixed { entity_id: first },
                _ => {
                    return Err((
                        "KERNEL_UNSUPPORTED_CONSTRAINT",
                        "Unsupported constraint kind.",
                    ))
                }
            };
            constraints.push((format!("{part_id}:{id}"), constraint));
        }
    }
    Ok(SemanticSnapshot {
        parameters: vec![],
        geometry,
        constraints,
        relations: vec![],
    })
}

fn point(
    props: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Point, (&'static str, &'static str)> {
    let raw = props
        .get(key)
        .and_then(Value::as_str)
        .ok_or(("KERNEL_GEOMETRY_PROPERTIES", "Point property is missing."))?;
    let mut values = raw.split(',').map(str::trim);
    let x = values
        .next()
        .ok_or(("KERNEL_GEOMETRY_PROPERTIES", "Point X value is invalid."))?
        .parse::<f64>()
        .map_err(|_| ("KERNEL_GEOMETRY_PROPERTIES", "Point X value is invalid."))?;
    let y = values
        .next()
        .ok_or(("KERNEL_GEOMETRY_PROPERTIES", "Point Y value is invalid."))?
        .parse::<f64>()
        .map_err(|_| ("KERNEL_GEOMETRY_PROPERTIES", "Point Y value is invalid."))?;
    if !x.is_finite() || !y.is_finite() {
        return Err(("KERNEL_GEOMETRY_PROPERTIES", "Point values must be finite."));
    }
    Ok(Point { x, y })
}

fn number(
    props: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<f64, (&'static str, &'static str)> {
    let value = props
        .get(key)
        .and_then(Value::as_str)
        .ok_or(("KERNEL_GEOMETRY_PROPERTIES", "Numeric property is missing."))?;
    let number = value
        .parse::<f64>()
        .map_err(|_| ("KERNEL_GEOMETRY_PROPERTIES", "Numeric property is invalid."))?;
    if !number.is_finite() {
        return Err((
            "KERNEL_GEOMETRY_PROPERTIES",
            "Numeric property must be finite.",
        ));
    }
    Ok(number)
}

fn compiled_package(root: &Value, semantic: &Value) -> Value {
    let app_id = root
        .get("applicationId")
        .and_then(Value::as_str)
        .unwrap_or("");
    let app_version = root
        .get("applicationVersion")
        .and_then(Value::as_str)
        .unwrap_or("");
    let build_identity = root
        .get("buildIdentity")
        .and_then(Value::as_str)
        .unwrap_or("");
    let parts = semantic
        .get("parts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let assemblies = semantic
        .get("assemblies")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut nodes = Vec::new();
    let mut relationships = Vec::new();
    let mut all_part_ids = Vec::new();
    let mut all_assembly_ids = Vec::new();

    for part in &parts {
        if let Some(id) = part.get("id").and_then(Value::as_str) {
            all_part_ids.push(id.to_string());
            compile_part_definition(part, id, &mut nodes);
        }
    }
    for assembly in &assemblies {
        if let Some(id) = assembly.get("id").and_then(Value::as_str) {
            all_assembly_ids.push(id.to_string());
            compile_assembly_definition(
                assembly,
                id,
                &assemblies,
                &parts,
                &mut nodes,
                &mut relationships,
            );
        }
    }

    let mut roots = Vec::new();
    let referenced_assemblies: std::collections::HashSet<String> = assemblies
        .iter()
        .flat_map(|assembly| {
            assembly
                .get("occurrences")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter(|occurrence| occurrence.get("definitionKind").and_then(Value::as_str) == Some("assembly"))
                .filter_map(|occurrence| occurrence.get("definitionId").and_then(Value::as_str))
                .map(ToOwned::to_owned)
        })
        .collect();
    let referenced_parts: std::collections::HashSet<String> = assemblies
        .iter()
        .flat_map(|assembly| {
            assembly
                .get("occurrences")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter(|occurrence| occurrence.get("definitionKind").and_then(Value::as_str) == Some("part"))
                .filter_map(|occurrence| occurrence.get("definitionId").and_then(Value::as_str))
                .map(ToOwned::to_owned)
        })
        .collect();

    for id in &all_assembly_ids {
        if !referenced_assemblies.contains(id) {
            roots.push(definition_id("assembly", id));
        }
    }
    for id in &all_part_ids {
        if !referenced_parts.contains(id) {
            roots.push(definition_id("part", id));
        }
    }
    roots.sort();
    roots.dedup();
    nodes.sort_by(|a, b| {
        a.get("id")
            .and_then(Value::as_str)
            .cmp(&b.get("id").and_then(Value::as_str))
    });
    relationships.sort_by(|a, b| {
        a.get("id")
            .and_then(Value::as_str)
            .cmp(&b.get("id").and_then(Value::as_str))
    });

    json!({
        "schema": MODEL_SCHEMA,
        "applicationId": app_id,
        "applicationVersion": app_version,
        "buildIdentity": build_identity,
        "manifest": {
            "schema": MODEL_SCHEMA,
            "applicationId": app_id,
            "applicationVersion": app_version,
            "buildIdentity": build_identity,
            "rootNodeIds": roots,
            "nodes": nodes,
            "relationships": relationships,
            "representations": [],
            "topologyBindings": [],
            "sourceBindings": [],
            "diagnostics": []
        },
        "renderArtifact": null,
        "diagnostics": []
    })
}

fn compile_part_definition(part: &Value, part_id: &str, nodes: &mut Vec<Value>) {
    let root_id = definition_id("part", part_id);
    let mut children = Vec::new();
    for geometry in part
        .get("geometry")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let gid = geometry.get("id").and_then(Value::as_str).unwrap_or("");
        let child = format!("{root_id}/geometry:{}", encode(gid));
        children.push(child.clone());
        nodes.push(node(
            &child,
            gid,
            geometry.get("kind").and_then(Value::as_str).unwrap_or("geometry"),
            &root_id,
            Vec::new(),
        ));
    }
    for constraint in part
        .get("constraints")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let cid = constraint.get("id").and_then(Value::as_str).unwrap_or("");
        let child = format!("{root_id}/constraint:{}", encode(cid));
        children.push(child.clone());
        nodes.push(node(
            &child,
            cid,
            constraint.get("kind").and_then(Value::as_str).unwrap_or("constraint"),
            &root_id,
            Vec::new(),
        ));
    }
    nodes.push(node(
        &root_id,
        part.get("name").and_then(Value::as_str).unwrap_or(part_id),
        "Part",
        "",
        children,
    ));
}

fn compile_assembly_definition(
    assembly: &Value,
    assembly_id: &str,
    assemblies: &[Value],
    parts: &[Value],
    nodes: &mut Vec<Value>,
    relationships: &mut Vec<Value>,
) {
    let definition_id_value = definition_id("assembly", assembly_id);
    let mut children = Vec::new();
    for occurrence in assembly
        .get("occurrences")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(child_id) = compile_occurrence(
            assembly_id,
            &definition_id_value,
            occurrence,
            assemblies,
            parts,
            nodes,
            relationships,
        ) {
            children.push(child_id);
        }
    }
    nodes.push(node(
        &definition_id_value,
        assembly
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or(assembly_id),
        "Assembly",
        "",
        children,
    ));
}

fn compile_occurrence(
    containing_assembly_id: &str,
    parent_id: &str,
    occurrence: &Value,
    assemblies: &[Value],
    parts: &[Value],
    nodes: &mut Vec<Value>,
    relationships: &mut Vec<Value>,
) -> Option<String> {
    let occurrence_id = occurrence.get("id").and_then(Value::as_str)?;
    let node_id = if parent_id.starts_with("definition:assembly:") {
        format!(
            "occurrence:{}/{}",
            encode(containing_assembly_id),
            encode(occurrence_id)
        )
    } else {
        format!("{parent_id}/occurrence:{}", encode(occurrence_id))
    };
    let definition_kind = occurrence
        .get("definitionKind")
        .and_then(Value::as_str)
        .unwrap_or("");
    let definition_ref = occurrence
        .get("definitionId")
        .and_then(Value::as_str)
        .unwrap_or("");
    let definition_node_id = definition_id(
        if definition_kind == "assembly" { "assembly" } else { "part" },
        definition_ref,
    );

    let mut child_ids = Vec::new();
    if definition_kind == "assembly" {
        if let Some(definition) = assemblies
            .iter()
            .find(|candidate| candidate.get("id").and_then(Value::as_str) == Some(definition_ref))
        {
            for nested in definition
                .get("occurrences")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                if let Some(child) = compile_occurrence(
                    definition_ref,
                    &node_id,
                    nested,
                    assemblies,
                    parts,
                    nodes,
                    relationships,
                ) {
                    child_ids.push(child);
                }
            }
        }
    } else if definition_kind == "part" {
        if let Some(definition) = parts
            .iter()
            .find(|candidate| candidate.get("id").and_then(Value::as_str) == Some(definition_ref))
        {
            for geometry in definition
                .get("geometry")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let gid = geometry.get("id").and_then(Value::as_str).unwrap_or("");
                let child = format!("{node_id}/geometry:{}", encode(gid));
                child_ids.push(child.clone());
                nodes.push(node(
                    &child,
                    gid,
                    geometry.get("kind").and_then(Value::as_str).unwrap_or("geometry"),
                    &node_id,
                    Vec::new(),
                ));
            }
            for constraint in definition
                .get("constraints")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let cid = constraint.get("id").and_then(Value::as_str).unwrap_or("");
                let child = format!("{node_id}/constraint:{}", encode(cid));
                child_ids.push(child.clone());
                nodes.push(node(
                    &child,
                    cid,
                    constraint.get("kind").and_then(Value::as_str).unwrap_or("constraint"),
                    &node_id,
                    Vec::new(),
                ));
            }
        }
    }

    let mut metadata = serde_json::Map::new();
    metadata.insert("definitionId".to_string(), Value::String(definition_node_id.clone()));
    metadata.insert("definitionKind".to_string(), Value::String(definition_kind.to_string()));
    if let Some(value) = occurrence.get("quantity") {
        metadata.insert("quantity".to_string(), value.clone());
    }
    if let Some(value) = occurrence.get("configurationName") {
        metadata.insert("configuration".to_string(), value.clone());
    }
    if let Some(value) = occurrence.get("bomStructure") {
        metadata.insert("bomStructure".to_string(), value.clone());
    }
    if let Some(value) = occurrence.get("transform") {
        metadata.insert("transform".to_string(), value.clone());
    }
    if let Some(value) = occurrence.get("metadata").and_then(Value::as_object) {
        for (key, value) in value {
            metadata.insert(key.clone(), value.clone());
        }
    }

    let relationship_id = format!("relationship:{node_id}:instantiates");
    let name = occurrence
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(occurrence_id);
    nodes.push(Value::Object(
        [
            ("id".to_string(), Value::String(node_id.clone())),
            ("name".to_string(), Value::String(name.to_string())),
            ("kind".to_string(), Value::String("ComponentInstance".to_string())),
            ("parentId".to_string(), Value::String(parent_id.to_string())),
            ("childIds".to_string(), Value::Array(child_ids.into_iter().map(Value::String).collect())),
            ("metadata".to_string(), Value::Object(metadata)),
            ("relationshipIds".to_string(), Value::Array(vec![Value::String(relationship_id.clone())])),
            ("representationIds".to_string(), Value::Array(Vec::new())),
            (
                "capabilities".to_string(),
                json!({
                    "visible": occurrence.get("visible").and_then(Value::as_bool).unwrap_or(true)
                        && !occurrence.get("suppressed").and_then(Value::as_bool).unwrap_or(false),
                    "hideable": true,
                    "selectable": true,
                    "focusable": true
                }),
            ),
            ("source".to_string(), Value::Null),
            (
                "state".to_string(),
                json!({
                    "visible": occurrence.get("visible").and_then(Value::as_bool).unwrap_or(true).to_string().to_lowercase(),
                    "suppressed": occurrence.get("suppressed").and_then(Value::as_bool).unwrap_or(false).to_string().to_lowercase(),
                    "grounded": occurrence.get("grounded").and_then(Value::as_bool).unwrap_or(false).to_string().to_lowercase(),
                    "flexible": occurrence.get("flexible").and_then(Value::as_bool).unwrap_or(false).to_string().to_lowercase()
                }),
            ),
        ]
        .into_iter()
        .collect(),
    ));
    relationships.push(json!({
        "id": relationship_id,
        "kind": "instantiates",
        "sourceId": node_id,
        "targetIds": [definition_node_id],
        "metadata": {}
    }));
    Some(node_id)
}

fn node(id: &str, name: &str, kind: &str, parent: &str, children: Vec<String>) -> Value {
    let parent_value = if parent.is_empty() {
        Value::Null
    } else {
        Value::String(parent.to_string())
    };
    json!({"id":id,"name":name,"kind":kind,"parentId":parent_value,"childIds":children,"metadata":{},"relationshipIds":[],"representationIds":[],"capabilities":{"visible":true,"hideable":true,"selectable":true,"focusable":true},"source":null,"state":{}})
}

fn definition_id(kind: &str, id: &str) -> String {
    format!("definition:{kind}:{}", encode(id))
}
fn encode(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace(' ', "%20")
        .replace('/', "%2F")
        .replace(':', "%3A")
}
fn severity(value: Severity) -> &'static str {
    if value == Severity::Error {
        "error"
    } else {
        "warning"
    }
}
fn diagnostic(code: &str, severity: &str, message: &str) -> Value {
    json!({"code":code,"severity":severity,"message":message})
}
fn failure(code: &str, message: &str) -> Value {
    json!({"succeeded":false,"compiledModel":null,"diagnostics":[{"code":code,"severity":"error","message":message}]})
}
