#!/usr/bin/env python3
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SPEC_DIR = ROOT / "specs"
REQUIRED_SPECS = [
    "agent-adapter-contract",
    "desktop-shell",
    "intent-orchestrator",
    "llm-proxy",
    "mcp-automation",
    "memory-controller",
    "observability",
    "packaging-updater",
    "remote-node-handoff",
    "runtime-supervisor",
    "session-context-store",
    "storage-indexing",
    "task-controller",
    "virtual-key-credentials",
]


def main() -> int:
    errors = []
    for name in REQUIRED_SPECS:
        path = SPEC_DIR / name / "README.md"
        if not path.exists():
            errors.append(f"missing spec: {path}")
            continue
        text = path.read_text()
        for needle in [
            "## Diri Parity Mapping",
            "Owned feature atoms",
            "Required Diri test mapping",
            "Pre-implementation gaps",
        ]:
            if needle not in text:
                errors.append(f"{path} missing {needle!r}")
        if "M" not in text and "Homie extension" not in text:
            errors.append(f"{path} has no Diri atom or extension marker")
    root = (SPEC_DIR / "README.md").read_text()
    for needle in [
        "## Diri Feature Atom Ownership",
        "Cross-spec mandatory gates",
        "M01-F001",
        "M20-F002",
    ]:
        if needle not in root:
            errors.append(f"specs/README.md missing {needle!r}")
    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    print("diri spec mapping ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
