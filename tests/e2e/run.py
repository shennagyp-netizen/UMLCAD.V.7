#!/usr/bin/env python3
"""Single-command UMLCAD V7 integration/red-team gate.

The runner owns the real kernel-host lifecycle and executes:
  1. Rust regression tests.
  2. Rust typed public-API integration E2E tests.
  3. Full .NET Framework test-project discovery.
  4. Full .NET Framework test-project execution, including real kernel E2E tests.
  5. Dedicated .NET black-box HTTP integration tests.
  6. Demo end-to-end build/evaluate test.
  7. Adversarial raw HTTP protocol probes.

Repository paths are resolved from this script, not from the caller's working
 directory. Required project files are checked before any test process starts.
No third-party Python packages are required.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import signal
import socket
import subprocess
import time
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Sequence


ROOT = Path(__file__).resolve().parents[2]
DEFAULT_URL = "http://127.0.0.1:8080"
FRAMEWORK_TEST_PROJECT = ROOT / "dotnet/tests/UMLCAD.Framework.Tests/UMLCAD.Framework.Tests.csproj"
BLACKBOX_TEST_PROJECT = ROOT / "dotnet/tests/UMLCAD.Kernel.Integration.Tests/UMLCAD.Kernel.Integration.Tests.csproj"
DEMO_PROJECT = ROOT / "projects/demo/Demo.csproj"
RUST_MANIFEST = ROOT / "kernel/native/Cargo.toml"


@dataclass
class Result:
    name: str
    command: list[str]
    returncode: int
    duration_seconds: float
    output_tail: str


class Runner:
    def __init__(self, release: bool, verbose: bool) -> None:
        self.release = release
        self.verbose = verbose
        self.results: list[Result] = []
        self.kernel: subprocess.Popen[str] | None = None
        self.kernel_log = ROOT / "tests" / "e2e" / ".kernel-host.log"

    def log(self, text: str) -> None:
        print(f"[E2E] {text}", flush=True)

    def run(self, name: str, command: Sequence[str], timeout: int = 600) -> bool:
        started = time.monotonic()
        self.log(f"RUN {name}: {' '.join(command)}")
        try:
            completed = subprocess.run(
                list(command),
                cwd=ROOT,
                env=os.environ.copy(),
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                timeout=timeout,
                check=False,
            )
            returncode = completed.returncode
            output = completed.stdout or ""
        except subprocess.TimeoutExpired as exc:
            returncode = 124
            output = (exc.stdout or "") if isinstance(exc.stdout, str) else ""
            output += f"\nTIMEOUT after {timeout}s"
        except OSError as exc:
            returncode = 127
            output = f"PROCESS ERROR: {exc}"
        duration = time.monotonic() - started
        tail = output[-8000:]
        self.results.append(Result(name, list(command), returncode, duration, tail))
        if self.verbose or returncode != 0:
            print(output, end="" if output.endswith("\n") else "\n")
        status = "PASS" if returncode == 0 else "FAIL"
        self.log(f"{status} {name} ({duration:.1f}s)")
        return returncode == 0

    def validate_paths(self) -> None:
        required = {
            "repository root": ROOT,
            "Rust manifest": RUST_MANIFEST,
            "Framework test project": FRAMEWORK_TEST_PROJECT,
            "black-box test project": BLACKBOX_TEST_PROJECT,
            "demo project": DEMO_PROJECT,
        }
        missing = [f"{label}: {path}" for label, path in required.items() if not path.exists()]
        if missing:
            raise FileNotFoundError("Required repository paths are missing:\n" + "\n".join(missing))
        self.log(f"repository root: {ROOT}")
        self.log("required repository paths: OK")

    def discover_framework_tests(self) -> bool:
        name = "dotnet-framework-discovery"
        started = time.monotonic()
        command = ["dotnet", "test", str(FRAMEWORK_TEST_PROJECT), "--list-tests"]
        self.log(f"RUN {name}: {' '.join(command)}")
        try:
            completed = subprocess.run(
                command,
                cwd=ROOT,
                env=os.environ.copy(),
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                timeout=900,
                check=False,
            )
            output = completed.stdout or ""
            returncode = completed.returncode
        except subprocess.TimeoutExpired as exc:
            returncode = 124
            output = (exc.stdout or "") if isinstance(exc.stdout, str) else ""
            output += "\nTIMEOUT after 900s"
        except OSError as exc:
            returncode = 127
            output = f"PROCESS ERROR: {exc}"

        duration = time.monotonic() - started
        tail = output[-8000:]
        self.results.append(Result(name, command, returncode, duration, tail))
        if self.verbose or returncode != 0:
            print(output, end="" if output.endswith("\n") else "\n")

        listed_tests = re.findall(r"^\s{2,}([^\r\n]+?)\s*$", output, re.MULTILINE)
        has_test_names = any("RustKernelEndToEndTests" in test for test in listed_tests)
        has_total = re.search(r"Total tests:\s*(\d+)", output)
        total = int(has_total.group(1)) if has_total else 0
        ok = returncode == 0 and total > 0 and has_test_names
        if not ok and returncode == 0:
            output += "\nDISCOVERY ERROR: Framework test project did not expose a non-zero test list containing RustKernelEndToEndTests."
            self.results[-1].output_tail = output[-8000:]
        self.log(("PASS" if ok else "FAIL") + f" {name}: discovered {total} tests")
        return ok

    def start_kernel(self) -> None:
        if self.kernel is not None:
            return
        self.kernel_log.parent.mkdir(parents=True, exist_ok=True)
        log = self.kernel_log.open("w", encoding="utf-8")
        command = [
            "cargo",
            "run",
            "--release",
            "--bin",
            "kernel_host",
            "--manifest-path",
            "kernel/native/Cargo.toml",
        ]
        self.log("START kernel_host: " + " ".join(command))
        self.kernel = subprocess.Popen(
            command,
            cwd=ROOT,
            stdout=log,
            stderr=subprocess.STDOUT,
            text=True,
            start_new_session=True,
        )
        deadline = time.monotonic() + 120
        while time.monotonic() < deadline:
            if self.kernel.poll() is not None:
                raise RuntimeError(f"kernel_host exited with code {self.kernel.returncode}; see {self.kernel_log}")
            try:
                with socket.create_connection(("127.0.0.1", 8080), timeout=1):
                    self.log("kernel_host is listening on 127.0.0.1:8080")
                    return
            except OSError:
                time.sleep(0.5)
        raise TimeoutError(f"kernel_host did not become ready; see {self.kernel_log}")

    def stop_kernel(self) -> None:
        process = self.kernel
        self.kernel = None
        if process is None:
            return
        if process.poll() is None:
            try:
                os.killpg(process.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                process.wait(timeout=5)
        self.log("kernel_host stopped")

    def write_report(self, path: Path) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            "schema": "uml-cad-e2e-report/1.0.0",
            "passed": all(x.returncode == 0 for x in self.results),
            "results": [asdict(x) for x in self.results],
            "kernel_log": str(self.kernel_log.relative_to(ROOT)),
        }
        path.write_text(json.dumps(payload, indent=2), encoding="utf-8")


def http_request(payload: bytes, request: str) -> bytes:
    raw = request.encode("ascii") + payload
    with socket.create_connection(("127.0.0.1", 8080), timeout=10) as sock:
        sock.sendall(raw)
        chunks: list[bytes] = []
        while True:
            chunk = sock.recv(8192)
            if not chunk:
                break
            chunks.append(chunk)
    return b"".join(chunks)


def status_and_body(response: bytes) -> tuple[int, bytes]:
    head, _, body = response.partition(b"\r\n\r\n")
    first = head.splitlines()[0].decode("ascii")
    return int(first.split()[1]), body


def raw_redteam_checks(runner: Runner) -> bool:
    checks = [
        (
            "unknown endpoint",
            b"",
            "GET /v1/does-not-exist HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
            404,
            b"KERNEL_NOT_FOUND",
        ),
        (
            "invalid JSON",
            b"{not-json",
            "POST /v1/build/evaluate HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: 9\r\nConnection: close\r\n\r\n",
            422,
            b"KERNEL_INVALID_JSON",
        ),
        (
            "wrong schema",
            json.dumps({"schema": "attack/0", "semantic": {"parts": []}}).encode(),
            None,
            422,
            b"KERNEL_BUILD_SCHEMA",
        ),
        (
            "missing semantic",
            json.dumps({"schema": "uml-cad-build-package/1.0.0"}).encode(),
            None,
            422,
            b"KERNEL_BUILD_SCHEMA",
        ),
        (
            "unsupported geometry",
            json.dumps(
                {
                    "schema": "uml-cad-build-package/1.0.0",
                    "applicationId": "attack",
                    "applicationVersion": "1.0.0",
                    "buildIdentity": "unused",
                    "semantic": {
                        "id": "attack",
                        "version": "1.0.0",
                        "buildIdentity": "unused",
                        "parts": [
                            {
                                "id": "p",
                                "geometry": [
                                    {"id": "g", "kind": "ellipse", "properties": {"center": "0,0", "radius": "5"}}
                                ],
                            }
                        ],
                    },
                }
            ).encode(),
            None,
            422,
            b"KERNEL_UNSUPPORTED_GEOMETRY",
        ),
    ]

    passed = True
    for name, payload, request, expected_status, expected_token in checks:
        if request is None:
            request = (
                "POST /v1/build/evaluate HTTP/1.1\r\n"
                "Host: localhost\r\n"
                "Content-Type: application/json\r\n"
                f"Content-Length: {len(payload)}\r\n"
                "Connection: close\r\n\r\n"
            )
        try:
            response = http_request(payload, request)
            status, body = status_and_body(response)
            ok = status == expected_status and expected_token in body
        except Exception as exc:  # noqa: BLE001 - red-team harness must report probe failures.
            ok = False
            body = str(exc).encode()
        runner.results.append(
            Result(
                f"python-redteam/{name}",
                ["raw-socket-http-probe", name],
                0 if ok else 1,
                0.0,
                body.decode("utf-8", errors="replace")[-2000:],
            )
        )
        runner.log(("PASS" if ok else "FAIL") + f" python-redteam/{name}")
        passed &= ok
    return passed


def main() -> int:
    parser = argparse.ArgumentParser(description="UMLCAD V7 one-command integration/red-team gate")
    parser.add_argument("--release", action="store_true", help="also use release Rust integration tests")
    parser.add_argument("--verbose", action="store_true", help="print full child process output")
    parser.add_argument(
        "--report",
        type=Path,
        default=ROOT / "tests" / "e2e" / "artifacts" / "e2e-report.json",
        help="JSON report path",
    )
    args = parser.parse_args()

    runner = Runner(args.release, args.verbose)
    passed = True
    try:
        runner.validate_paths()

        passed &= runner.run("rust-regression", ["cargo", "test", "--manifest-path", str(RUST_MANIFEST)], 900)
        passed &= runner.run(
            "rust-kernel-api-e2e",
            ["cargo", "test", "--manifest-path", str(RUST_MANIFEST), "--test", "kernel_api_integration_e2e", "--", "--nocapture"],
            900,
        )
        if args.release:
            passed &= runner.run(
                "rust-kernel-api-e2e-release",
                ["cargo", "test", "--release", "--manifest-path", str(RUST_MANIFEST), "--test", "kernel_api_integration_e2e", "--", "--nocapture"],
                900,
            )

        passed &= runner.discover_framework_tests()

        runner.start_kernel()
        os.environ["UMLCAD_KERNEL_URL"] = DEFAULT_URL + "/"

        passed &= runner.run(
            "dotnet-framework-full-suite",
            ["dotnet", "test", str(FRAMEWORK_TEST_PROJECT)],
            900,
        )
        passed &= runner.run(
            "dotnet-kernel-blackbox-e2e",
            ["dotnet", "test", str(BLACKBOX_TEST_PROJECT)],
            900,
        )
        passed &= runner.run(
            "demo-e2e",
            ["dotnet", "run", "--project", str(DEMO_PROJECT), "--", "--e2e"],
            900,
        )
        passed &= raw_redteam_checks(runner)
    except Exception as exc:  # noqa: BLE001 - top-level gate must always emit a report.
        runner.log(f"HARNESS ERROR: {exc}")
        passed = False
    finally:
        runner.stop_kernel()
        runner.write_report(args.report)

    runner.log(f"REPORT {args.report}")
    runner.log("E2E GATE PASS" if passed else "E2E GATE FAIL")
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
