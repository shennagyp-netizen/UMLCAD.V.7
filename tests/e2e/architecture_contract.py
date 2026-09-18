#!/usr/bin/env python3
"""Mechanical architecture contract for the UMLCAD V7 system boundary.

The logical architecture is authoritative in docs/architecture/architecture.json.
This test translates that logical model into the current repository projection.
The .csproj graph is therefore an implementation check, not the architecture
definition.
"""
from __future__ import annotations

import argparse
import json
import sys
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
SRC = ROOT / "dotnet" / "src"
MANIFEST = ROOT / "docs" / "architecture" / "architecture.json"

PROJECTS = {
    "UMLCAD.Cad.Expressions": SRC / "UMLCAD.Cad.Expressions" / "UMLCAD.Cad.Expressions.csproj",
    "UMLCAD.Cad.Semantics": SRC / "UMLCAD.Cad.Semantics" / "UMLCAD.Cad.Semantics.csproj",
    "UMLCAD.Cad.Contracts": SRC / "UMLCAD.Cad.Contracts" / "UMLCAD.Cad.Contracts.csproj",
    "UMLCAD.Cad.Engine": SRC / "UMLCAD.Cad.Engine" / "UMLCAD.Cad.Engine.csproj",
    "UMLCAD.Science": SRC / "UMLCAD.Science" / "UMLCAD.Science.csproj",
    "UMLCAD.Engineering.Resources": SRC / "UMLCAD.Engineering.Resources" / "UMLCAD.Engineering.Resources.csproj",
    "UMLCAD.Engineering.SheetMetal": SRC / "UMLCAD.Engineering.SheetMetal" / "UMLCAD.Engineering.SheetMetal.csproj",
    "UMLCAD.Engineering.Cam": SRC / "UMLCAD.Engineering.Cam" / "UMLCAD.Engineering.Cam.csproj",
    "UMLCAD.Engineering.Drawing": SRC / "UMLCAD.Engineering.Drawing" / "UMLCAD.Engineering.Drawing.csproj",
    "UMLCAD.Integration.Simulation": SRC / "UMLCAD.Integration.Simulation" / "UMLCAD.Integration.Simulation.csproj",
    "UMLCAD.Framework": SRC / "UMLCAD.Framework" / "UMLCAD.Framework.csproj",
    "UMLCAD.Kernel.Client": SRC / "UMLCAD.Kernel.Client" / "UMLCAD.Kernel.Client.csproj",
}

# Transitional only: the current kernel transport still consumes legacy
# Framework semantic/package types. This edge is removed by the later
# legacy-elimination milestone.
ALLOWED_LEGACY_EDGES = {
    ("UMLCAD.Kernel.Client", "UMLCAD.Framework"),
}

EXPECTED_REFERENCES = {
    "UMLCAD.Cad.Expressions": set(),
    "UMLCAD.Cad.Semantics": {"UMLCAD.Cad.Expressions"},
    "UMLCAD.Cad.Contracts": set(),
    "UMLCAD.Cad.Engine": {
        "UMLCAD.Cad.Expressions",
        "UMLCAD.Cad.Semantics",
        "UMLCAD.Cad.Contracts",
    },
    "UMLCAD.Science": {"UMLCAD.Cad.Expressions"},
    "UMLCAD.Engineering.Resources": {
        "UMLCAD.Cad.Expressions",
        "UMLCAD.Science",
    },
    "UMLCAD.Engineering.SheetMetal": {
        "UMLCAD.Cad.Expressions",
        "UMLCAD.Cad.Semantics",
        "UMLCAD.Science",
        "UMLCAD.Engineering.Resources",
    },
    "UMLCAD.Engineering.Cam": {
        "UMLCAD.Cad.Expressions",
        "UMLCAD.Cad.Semantics",
        "UMLCAD.Science",
        "UMLCAD.Engineering.Resources",
    },
    "UMLCAD.Engineering.Drawing": {
        "UMLCAD.Cad.Expressions",
        "UMLCAD.Cad.Semantics",
        "UMLCAD.Science",
    },
    "UMLCAD.Integration.Simulation": {
        "UMLCAD.Science",
    },
    "UMLCAD.Kernel.Client": {"UMLCAD.Cad.Contracts"},
}

FORBIDDEN_TARGETS = {
    "UMLCAD.Cad.Expressions": {
        "UMLCAD.Framework",
        "UMLCAD.Kernel.Client",
        "UMLCAD.Cad.Semantics",
        "UMLCAD.Cad.Contracts",
        "UMLCAD.Cad.Engine",
    },
    "UMLCAD.Cad.Semantics": {
        "UMLCAD.Framework",
        "UMLCAD.Kernel.Client",
        "UMLCAD.Cad.Contracts",
        "UMLCAD.Cad.Engine",
    },
    "UMLCAD.Cad.Contracts": {
        "UMLCAD.Framework",
        "UMLCAD.Kernel.Client",
        "UMLCAD.Cad.Semantics",
        "UMLCAD.Cad.Engine",
    },
    "UMLCAD.Cad.Engine": {
        "UMLCAD.Framework",
        "UMLCAD.Kernel.Client",
    },
    "UMLCAD.Science": {
        "UMLCAD.Framework",
        "UMLCAD.Kernel.Client",
        "UMLCAD.Cad.Semantics",
        "UMLCAD.Cad.Contracts",
        "UMLCAD.Cad.Engine",
        "UMLCAD.Engineering.Resources",
        "UMLCAD.Engineering.SheetMetal",
        "UMLCAD.Engineering.Cam",
        "UMLCAD.Engineering.Drawing",
    },
    "UMLCAD.Engineering.Resources": {
        "UMLCAD.Framework",
        "UMLCAD.Kernel.Client",
        "UMLCAD.Engineering.SheetMetal",
        "UMLCAD.Engineering.Cam",
        "UMLCAD.Engineering.Drawing",
    },
    "UMLCAD.Engineering.SheetMetal": {
        "UMLCAD.Framework",
        "UMLCAD.Kernel.Client",
        "UMLCAD.Engineering.Cam",
        "UMLCAD.Engineering.Drawing",
    },
    "UMLCAD.Engineering.Cam": {
        "UMLCAD.Framework",
        "UMLCAD.Kernel.Client",
        "UMLCAD.Engineering.SheetMetal",
        "UMLCAD.Engineering.Drawing",
    },
    "UMLCAD.Engineering.Drawing": {
        "UMLCAD.Framework",
        "UMLCAD.Kernel.Client",
        "UMLCAD.Engineering.SheetMetal",
        "UMLCAD.Engineering.Cam",
    },
    "UMLCAD.Integration.Simulation": {
        "UMLCAD.Framework",
        "UMLCAD.Kernel.Client",
        "UMLCAD.Cad.Semantics",
        "UMLCAD.Cad.Engine",
        "UMLCAD.Engineering.SheetMetal",
        "UMLCAD.Engineering.Cam",
        "UMLCAD.Engineering.Drawing",
    },
}


def project_name_from_path(path_text: str) -> str:
    normalized = Path(path_text.replace("\\", "/"))
    return normalized.stem


def parse_references(project_path: Path) -> set[str]:
    root = ET.fromstring(project_path.read_text(encoding="utf-8"))
    refs: set[str] = set()
    for ref in root.iter():
        if not ref.tag.endswith("ProjectReference"):
            continue
        include = ref.attrib.get("Include")
        if not include:
            raise AssertionError(f"{project_path}: ProjectReference has no Include.")
        candidate = (project_path.parent / include).resolve()
        candidate = candidate.parent / (candidate.stem + ".csproj")
        for name, known in PROJECTS.items():
            if candidate == known.resolve():
                refs.add(name)
                break
        else:
            refs.add(project_name_from_path(include))
    return refs


def source_files(project_path: Path) -> list[Path]:
    return [
        path
        for path in project_path.parent.rglob("*.cs")
        if "bin" not in path.parts and "obj" not in path.parts
    ]


def load_manifest() -> tuple[dict[str, Any] | None, list[str]]:
    errors: list[str] = []
    if not MANIFEST.is_file():
        return None, [f"missing architecture manifest: {MANIFEST}"]

    try:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        return None, [f"invalid architecture manifest JSON: {exc}"]

    for key in ("schemaVersion", "authority", "c4", "layers", "logicalUnits", "rules", "repositoryProjection"):
        if key not in manifest:
            errors.append(f"architecture manifest missing key: {key}")

    if errors:
        return None, errors

    return manifest, errors


def validate_manifest(manifest: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    layers = manifest["layers"]
    units = manifest["logicalUnits"]

    if not isinstance(layers, list) or not layers:
        return ["architecture manifest layers must be a non-empty list"]
    if not isinstance(units, dict) or not units:
        return ["architecture manifest logicalUnits must be a non-empty object"]

    layer_by_id: dict[str, dict[str, Any]] = {}
    for layer in layers:
        if not isinstance(layer, dict):
            errors.append("architecture manifest contains a non-object layer")
            continue
        layer_id = layer.get("id")
        if not isinstance(layer_id, str) or not layer_id:
            errors.append("architecture manifest layer has no valid id")
            continue
        if layer_id in layer_by_id:
            errors.append(f"duplicate architecture layer id: {layer_id}")
        layer_by_id[layer_id] = layer
        if not isinstance(layer.get("rank"), int):
            errors.append(f"layer {layer_id}: rank must be an integer")
        if not isinstance(layer.get("canDependOn"), list):
            errors.append(f"layer {layer_id}: canDependOn must be a list")

    for layer_id, layer in layer_by_id.items():
        for dependency in layer.get("canDependOn", []):
            if dependency not in layer_by_id:
                errors.append(f"layer {layer_id}: unknown dependency layer {dependency}")
            elif layer["rank"] <= layer_by_id[dependency]["rank"]:
                errors.append(
                    f"layer {layer_id}: dependency {dependency} must be strictly lower-ranked"
                )

    unit_layer: dict[str, str] = {}
    unit_kind: dict[str, str] = {}
    unit_deps: dict[str, set[str]] = {}

    for unit_id, unit in units.items():
        if not isinstance(unit, dict):
            errors.append(f"unit {unit_id}: definition is not an object")
            continue
        layer = unit.get("layer")
        kind = unit.get("kind")
        deps = unit.get("canDependOn", [])
        if layer not in layer_by_id:
            errors.append(f"unit {unit_id}: unknown layer {layer}")
            continue
        if not isinstance(kind, str) or not kind:
            errors.append(f"unit {unit_id}: kind must be non-empty")
        if not isinstance(deps, list):
            errors.append(f"unit {unit_id}: canDependOn must be a list")
            deps = []
        unit_layer[unit_id] = layer
        unit_kind[unit_id] = kind
        unit_deps[unit_id] = set(deps)

    for unit_id, deps in unit_deps.items():
        source_layer = layer_by_id[unit_layer[unit_id]]
        for dependency in deps:
            if dependency not in units:
                errors.append(f"unit {unit_id}: unknown dependency unit {dependency}")
                continue
            target_layer = layer_by_id[unit_layer[dependency]]
            if target_layer["rank"] > source_layer["rank"]:
                errors.append(
                    f"unit {unit_id}: depends upward on {dependency}"
                )
            elif target_layer["rank"] < source_layer["rank"]:
                if target_layer["id"] not in source_layer["canDependOn"]:
                    errors.append(
                        f"unit {unit_id}: layer {source_layer['id']} does not permit "
                        f"dependency on layer {target_layer['id']}"
                    )
            elif unit_kind.get(unit_id) == "domain" and unit_kind.get(dependency) == "domain":
                errors.append(
                    f"domain unit {unit_id}: direct peer-domain dependency on {dependency} is forbidden"
                )

    # Detect logical dependency cycles.
    state: dict[str, int] = {unit_id: 0 for unit_id in units}
    stack: list[str] = []

    def visit(unit_id: str) -> None:
        if state[unit_id] == 2:
            return
        if state[unit_id] == 1:
            cycle_start = stack.index(unit_id)
            cycle = stack[cycle_start:] + [unit_id]
            errors.append("logical dependency cycle: " + " -> ".join(cycle))
            return

        state[unit_id] = 1
        stack.append(unit_id)
        for dependency in unit_deps.get(unit_id, set()):
            if dependency in units:
                visit(dependency)
        stack.pop()
        state[unit_id] = 2

    for unit_id in units:
        visit(unit_id)

    projection = manifest["repositoryProjection"]
    if not isinstance(projection, dict):
        errors.append("repositoryProjection must be an object")
    else:
        for project_name, projection_entry in projection.items():
            if project_name not in PROJECTS:
                errors.append(f"repositoryProjection contains unknown project: {project_name}")
                continue
            if not isinstance(projection_entry, dict):
                errors.append(f"repositoryProjection {project_name}: entry must be an object")
                continue
            mapped = projection_entry.get("logicalUnits")
            if not isinstance(mapped, list) or not mapped:
                errors.append(
                    f"repositoryProjection {project_name}: logicalUnits must be a non-empty list"
                )
                continue
            for unit_id in mapped:
                if unit_id not in units:
                    errors.append(
                        f"repositoryProjection {project_name}: unknown logical unit {unit_id}"
                    )

    return errors


def validate_repository_projection(manifest: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    refs = {name: parse_references(path) for name, path in PROJECTS.items()}

    for name, expected in EXPECTED_REFERENCES.items():
        actual = refs[name] - {
            dst for src, dst in ALLOWED_LEGACY_EDGES if src == name
        }
        if expected != actual:
            errors.append(
                f"{name}: expected references {sorted(expected)}, found {sorted(actual)}"
            )

    for name, forbidden in FORBIDDEN_TARGETS.items():
        disallowed = refs[name] & forbidden
        if disallowed:
            errors.append(f"{name}: forbidden references {sorted(disallowed)}")

    for source_name, refs_for_source in refs.items():
        for target_name in refs_for_source:
            if (source_name, target_name) in ALLOWED_LEGACY_EDGES:
                continue
            if target_name not in PROJECTS:
                continue
            if source_name.startswith("UMLCAD.Cad.") and target_name in {
                "UMLCAD.Framework",
                "UMLCAD.Kernel.Client",
            }:
                errors.append(
                    f"{source_name}: new CAD layer may not depend on {target_name}"
                )

    # Source-level guard against concrete kernel-client/framework references in
    # new semantic/engine layers.
    new_layers = (
        "UMLCAD.Cad.Expressions",
        "UMLCAD.Cad.Semantics",
        "UMLCAD.Cad.Contracts",
        "UMLCAD.Cad.Engine",
        "UMLCAD.Science",
        "UMLCAD.Engineering.Resources",
        "UMLCAD.Engineering.SheetMetal",
        "UMLCAD.Engineering.Cam",
        "UMLCAD.Engineering.Drawing",
        "UMLCAD.Integration.Simulation",
    )
    for name in new_layers:
        for source in source_files(PROJECTS[name]):
            text = source.read_text(encoding="utf-8")
            if "UMLCAD.Kernel.Client" in text:
                errors.append(
                    f"{name}: {source.relative_to(ROOT)} references UMLCAD.Kernel.Client directly"
                )
            if "UMLCAD.Framework" in text:
                errors.append(
                    f"{name}: {source.relative_to(ROOT)} references UMLCAD.Framework directly"
                )

    # Ensure every projected project maps to a declared logical unit.
    projection = manifest["repositoryProjection"]
    for project_name in PROJECTS:
        if project_name in {"UMLCAD.Framework"}:
            continue
        if project_name not in projection:
            errors.append(f"{project_name}: missing repositoryProjection entry")

    return errors


def validate() -> list[str]:
    manifest, errors = load_manifest()
    if manifest is None:
        return errors

    errors.extend(validate_manifest(manifest))

    for name, path in PROJECTS.items():
        if not path.is_file():
            errors.append(f"missing required project: {path}")

    if errors:
        return errors

    errors.extend(validate_repository_projection(manifest))
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", required=True)
    _ = parser.parse_args()

    errors = validate()
    if errors:
        print("ARCHITECTURE CONTRACT FAIL")
        for error in errors:
            print(f"- {error}")
        return 1

    print("ARCHITECTURE CONTRACT PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
