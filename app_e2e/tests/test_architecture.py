from __future__ import annotations

import re
import unittest
import xml.etree.ElementTree as ET
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
DOTNET_ROOT = ROOT / "dotnet"
SOURCE_ROOT = DOTNET_ROOT / "src"
TEST_ROOT = DOTNET_ROOT / "tests"

# Transitional implementation name. The architecture contract is "exactly one gateway";
# migration may rename this project to UMLCAD.Kernel without changing the rule.
KERNEL_GATEWAY_NAMES = {"UMLCAD.Kernel.Client"}

KERNEL_IMPLEMENTATION_MARKERS = (
    "RustKernelService",
    "IRustKernelService",
    "RustKernelOptions",
    "AddRustKernel",
    "kernel_host",
    "v1/build/evaluate",
    "UMLCAD_KERNEL_URL",
)

NATIVE_KERNEL_MARKERS = (
    "kernel/native/Cargo.toml",
    "kernel\\native\\Cargo.toml",
    "kernel/native/",
    "kernel\\native\\",
)

DIRECT_KERNEL_PROCESS_PATTERNS = (
    re.compile(r"cargo\\s+(?:run|test).*kernel_host", re.IGNORECASE),
    re.compile(r"Process\\.Start(?:AsUser|Async)?[^\\n]*kernel_host", re.IGNORECASE),
    re.compile(r"(?:DllImport|LibraryImport)[^\\n]*(?:kernel|uml?cad)", re.IGNORECASE),
)


@dataclass(frozen=True)
class Project:
    path: Path
    name: str
    references: tuple[Path, ...]


def local_name(tag: str) -> str:
    return tag.rsplit("}", 1)[-1]


def project_name(project_file: Path, tree: ET.ElementTree) -> str:
    for element in tree.getroot().iter():
        if local_name(element.tag) == "AssemblyName" and element.text and element.text.strip():
            return element.text.strip()
    return project_file.stem


def load_projects() -> dict[Path, Project]:
    projects: dict[Path, Project] = {}

    for project_file in sorted(SOURCE_ROOT.rglob("*.csproj")):
        project_file = project_file.resolve()
        tree = ET.parse(project_file)
        references: list[Path] = []

        for element in tree.getroot().iter():
            if local_name(element.tag) != "ProjectReference":
                continue

            include = element.attrib.get("Include")
            if not include:
                raise AssertionError(f"{project_file}: ProjectReference has no Include attribute.")

            if "$(" in include:
                raise AssertionError(
                    f"{project_file}: dynamic ProjectReference '{include}' cannot be proven safe."
                )

            target = (project_file.parent / include).resolve()
            if target.suffix.lower() != ".csproj":
                raise AssertionError(
                    f"{project_file}: ProjectReference '{include}' does not resolve to a .csproj."
                )

            references.append(target)

        projects[project_file] = Project(
            path=project_file,
            name=project_name(project_file, tree),
            references=tuple(references),
        )

    if not projects:
        raise AssertionError("No production .NET projects were discovered under dotnet/src.")

    return projects


def application_project_set(projects: dict[Path, Project]) -> set[Path]:
    return set(projects)


def cycle_in_graph(projects: dict[Path, Project]) -> list[Path] | None:
    state: dict[Path, int] = {path: 0 for path in projects}
    stack: list[Path] = []

    def visit(node: Path) -> list[Path] | None:
        state[node] = 1
        stack.append(node)

        for target in projects[node].references:
            if target not in projects:
                continue
            if state[target] == 0:
                cycle = visit(target)
                if cycle is not None:
                    return cycle
            elif state[target] == 1:
                start = stack.index(target)
                return [*stack[start:], target]

        stack.pop()
        state[node] = 2
        return None

    for node in projects:
        if state[node] == 0:
            cycle = visit(node)
            if cycle is not None:
                return cycle

    return None


def is_under(path: Path, parent: Path) -> bool:
    try:
        path.relative_to(parent)
        return True
    except ValueError:
        return False


class ApplicationArchitectureTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.projects = load_projects()
        cls.application_projects = application_project_set(cls.projects)

    def test_all_production_dotnet_projects_are_application_layer_projects(self) -> None:
        source_projects = sorted(self.projects)
        unexpected = [
            project
            for project in source_projects
            if not is_under(project, SOURCE_ROOT.resolve())
        ]
        self.assertFalse(unexpected, "Production .NET projects escaped dotnet/src: " + ", ".join(map(str, unexpected)))

        stray_projects = [
            project
            for project in DOTNET_ROOT.rglob("*.csproj")
            if not is_under(project.resolve(), SOURCE_ROOT.resolve())
            and not is_under(project.resolve(), TEST_ROOT.resolve())
        ]
        self.assertFalse(
            stray_projects,
            "A .NET project exists outside the declared Application Layer or test tree: "
            + ", ".join(map(str, sorted(stray_projects))),
        )

    def test_project_references_never_escape_application_layer(self) -> None:
        violations: list[str] = []

        for project in self.projects.values():
            for target in project.references:
                if target not in self.application_projects:
                    violations.append(f"{project.name} -> {target}")

        self.assertFalse(
            violations,
            "Production ProjectReference escapes dotnet/src or resolves to an unknown project:\n"
            + "\n".join(violations),
        )

    def test_complete_application_project_graph_is_acyclic(self) -> None:
        cycle = cycle_in_graph(self.projects)
        rendered = " -> ".join(self.projects[node].name for node in cycle) if cycle else ""
        self.assertIsNone(
            cycle,
            "CIRCULAR APPLICATION DEPENDENCY DETECTED"
            + (f": {rendered}" if rendered else ""),
        )

    def test_exactly_one_kernel_access_gateway_exists(self) -> None:
        gateways = [project for project in self.projects.values() if project.name in KERNEL_GATEWAY_NAMES]
        self.assertEqual(
            len(gateways),
            1,
            "Exactly one production .NET kernel-access gateway is required; "
            f"found {[project.name for project in gateways]}",
        )

        gateway = gateways[0]
        self.assertTrue(is_under(gateway.path, SOURCE_ROOT.resolve()))

    def test_kernel_transport_knowledge_isolated_to_gateway(self) -> None:
        gateway = next(project for project in self.projects.values() if project.name in KERNEL_GATEWAY_NAMES)
        violations: list[str] = []

        for source_file in sorted(SOURCE_ROOT.rglob("*.cs")):
            if is_under(source_file.resolve(), gateway.path.parent / gateway.path.name):
                continue

            text = source_file.read_text(encoding="utf-8")
            for marker in KERNEL_IMPLEMENTATION_MARKERS:
                if marker in text:
                    violations.append(f"{source_file.relative_to(ROOT)} contains forbidden kernel marker '{marker}'")

        self.assertFalse(
            violations,
            "Kernel transport/implementation details leaked outside the dedicated gateway:\n"
            + "\n".join(violations),
        )

    def test_no_direct_native_kernel_path_or_process_access_outside_gateway(self) -> None:
        gateway = next(project for project in self.projects.values() if project.name in KERNEL_GATEWAY_NAMES)
        violations: list[str] = []

        for source_file in sorted(SOURCE_ROOT.rglob("*.cs")):
            if is_under(source_file.resolve(), gateway.path.parent / gateway.path.name):
                continue

            text = source_file.read_text(encoding="utf-8")

            for marker in NATIVE_KERNEL_MARKERS:
                if marker in text:
                    violations.append(f"{source_file.relative_to(ROOT)} contains native kernel path '{marker}'")

            for pattern in DIRECT_KERNEL_PROCESS_PATTERNS:
                if pattern.search(text):
                    violations.append(f"{source_file.relative_to(ROOT)} directly invokes kernel/native transport")

        self.assertFalse(
            violations,
            "Direct kernel/native access escaped the dedicated gateway:\n" + "\n".join(violations),
        )

    def test_gateway_has_no_reverse_dependency_cycle_with_its_consumers(self) -> None:
        gateway = next(project for project in self.projects.values() if project.name in KERNEL_GATEWAY_NAMES)
        consumers = {
            project.path
            for project in self.projects.values()
            if gateway.path in project.references
        }

        reverse_edges = [
            f"{project.name} -> {self.projects[target].name}"
            for project in self.projects.values()
            if project.path == gateway.path
            for target in project.references
            if target in consumers
        ]

        self.assertFalse(
            reverse_edges,
            "The kernel gateway must not depend back on an Application Layer consumer: "
            + ", ".join(reverse_edges),
        )

    def test_project_names_are_unique(self) -> None:
        by_name: dict[str, list[Path]] = {}
        for project in self.projects.values():
            by_name.setdefault(project.name, []).append(project.path)

        duplicates = {
            name: paths
            for name, paths in by_name.items()
            if len(paths) > 1
        }

        self.assertFalse(
            duplicates,
            "Duplicate production assembly names are not permitted: "
            + "; ".join(f"{name}: {', '.join(map(str, paths))}" for name, paths in duplicates.items()),
        )


if __name__ == "__main__":
    unittest.main()
