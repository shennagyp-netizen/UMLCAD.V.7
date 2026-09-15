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
    let mut nodes = Vec::new();
    let mut roots = Vec::new();
    for part in semantic
        .get("parts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let part_id = part.get("id").and_then(Value::as_str).unwrap_or("");
        let root_id = definition_id("part", part_id);
        roots.push(root_id.clone());
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
                geometry
                    .get("kind")
                    .and_then(Value::as_str)
                    .unwrap_or("geometry"),
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
                constraint
                    .get("kind")
                    .and_then(Value::as_str)
                    .unwrap_or("constraint"),
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
            "relationships": [],
            "representations": [],
            "topologyBindings": [],
            "sourceBindings": [],
            "diagnostics": []
        },
        "renderArtifact": null,
        "diagnostics": []
    })
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
