#!/usr/bin/env python3
"""Mechanical architecture contract for the UMLCAD V7 .NET system boundary."""
from __future__ import annotations

import argparse
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SRC = ROOT / "dotnet" / "src"

PROJECTS = {
    "UMLCAD.Cad.Expressions": SRC / "UMLCAD.Cad.Expressions" / "UMLCAD.Cad.Expressions.csproj",
    "UMLCAD.Cad.Semantics": SRC / "UMLCAD.Cad.Semantics" / "UMLCAD.Cad.Semantics.csproj",
    "UMLCAD.Cad.Contracts": SRC / "UMLCAD.Cad.Contracts" / "UMLCAD.Cad.Contracts.csproj",
    "UMLCAD.Cad.Engine": SRC / "UMLCAD.Cad.Engine" / "UMLCAD.Cad.Engine.csproj",
    "UMLCAD.Framework": SRC / "UMLCAD.Framework" / "UMLCAD.Framework.csproj",
    "UMLCAD.Kernel.Client": SRC / "UMLCAD.Kernel.Client" / "UMLCAD.Kernel.Client.csproj",
}

# Transitional only: the current transport/client still consumes legacy Framework
# semantic/package types. M-S31 removes this edge.
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
}


def project_name_from_path(path_text: str) -> str:
    normalized = Path(path_text.replace("\\", "/"))
    stem = normalized.stem
    return stem


def parse_references(project_path: Path) -> set[str]:
    root = ET.fromstring(project_path.read_text(encoding="utf-8"))
    refs: set[str] = set()
    for ref in root.iter():
        if ref.tag.endswith("ProjectReference"):
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


def validate() -> list[str]:
    errors: list[str] = []

    for name, path in PROJECTS.items():
        if not path.is_file():
            errors.append(f"missing required project: {path}")

    if errors:
        return errors

    refs = {name: parse_references(path) for name, path in PROJECTS.items()}

    for name, expected in EXPECTED_REFERENCES.items():
        actual = refs[name] - {dst for src, dst in ALLOWED_LEGACY_EDGES if src == name}
        if expected != actual:
            errors.append(
                f"{name}: expected references {sorted(expected)}, found {sorted(actual)}"
            )

    for name, forbidden in FORBIDDEN_TARGETS.items():
        disallowed = refs[name] & forbidden
        if disallowed:
            errors.append(
                f"{name}: forbidden references {sorted(disallowed)}"
            )

    for source_name, refs_for_source in refs.items():
        for target_name in refs_for_source:
            if (source_name, target_name) not in ALLOWED_LEGACY_EDGES and target_name in PROJECTS:
                # Any edge into the application host or concrete transport from a
                # new semantic/engine layer is forbidden even if the project graph
                # happens to compile.
                if source_name.startswith("UMLCAD.Cad.") and target_name in {
                    "UMLCAD.Framework",
                    "UMLCAD.Kernel.Client",
                }:
                    errors.append(
                        f"{source_name}: new CAD layer may not depend on {target_name}"
                    )

    # Source-level guard against concrete kernel-client references in new CAD layers.
    for name in (
        "UMLCAD.Cad.Expressions",
        "UMLCAD.Cad.Semantics",
        "UMLCAD.Cad.Contracts",
        "UMLCAD.Cad.Engine",
    ):
        project_path = PROJECTS[name]
        for source in source_files(project_path):
            text = source.read_text(encoding="utf-8")
            if "UMLCAD.Kernel.Client" in text:
                errors.append(
                    f"{name}: source file {source.relative_to(ROOT)} references "
                    "UMLCAD.Kernel.Client directly"
                )

    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", required=True)
    args = parser.parse_args()
    _ = args

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
