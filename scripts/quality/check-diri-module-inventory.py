#!/usr/bin/env python3
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
DOC = ROOT / "docs/research/diri-module-inventory.md"


def table_rows(section: str) -> list[str]:
    rows = []
    in_table = False
    for line in section.splitlines():
        if line.startswith("|"):
            in_table = True
            if not line.startswith("|-") and not line.startswith("| Feature ID") and not line.startswith("| Module") and not line.startswith("| Diri test") and not line.startswith("| Homie module"):
                rows.append(line)
        elif in_table:
            break
    return rows


def section_after(text: str, heading: str) -> str:
    marker = f"## {heading}"
    if marker not in text:
        raise AssertionError(f"missing section {marker}")
    return text.split(marker, 1)[1]


def main() -> int:
    text = DOC.read_text()
    required_sections = [
        "2. 模块总览",
        "7. Feature Atom Matrix",
        "8. Diri Test Coverage Matrix",
        "9. Dependency Matrix",
        "10. Verification Environment Matrix",
    ]
    for section in required_sections:
        section_after(text, section)

    for index in range(1, 21):
        module = f"M{index:02d}"
        if module not in text:
            raise AssertionError(f"missing module {module}")
        if not re.search(rf"\| {module}-F\d{{3}} \|", text):
            raise AssertionError(f"module {module} has no feature atom")

    required_sources = [
        "Sources/CDirijorPTY",
        "Sources/DirijorClient",
        "Sources/DirijorCore",
        "Sources/DirijorDaemonKit",
        "Sources/DirijorDetection",
        "Sources/DirijorGit",
        "Sources/DirijorHolderKit",
        "Sources/DirijorMCP",
        "Sources/DirijorProtocol",
        "Sources/dirijor-cli",
        "Sources/dirijord",
        "Sources/dirijord-holder",
        "diri/diri/crates/diri-app",
        "diri/diri/crates/diri-client",
        "diri/diri/crates/diri-engine",
        "diri/diri/crates/diri-node",
        "diri/diri/crates/diri-proto",
        "diri/diri/crates/diri-term",
        "diri/diri/crates/diri-ui",
        "diri/diri/crates/diri-updater",
        "diri/diri/crates/diri-usage",
        "diri/diri/crates/dirijor-mcp",
    ]
    for source in required_sources:
        if source not in text:
            raise AssertionError(f"missing source mapping {source}")

    required_tests = [
        "DirijorCoreTests",
        "DirijorProtocolTests",
        "DirijorDetectionTests",
        "SessionIntegrationTests",
        "CommandGrammarTests",
    ]
    for test in required_tests:
        if test not in text:
            raise AssertionError(f"missing test mapping {test}")

    forbidden = ["TBD", "TODO", "待补"]
    for token in forbidden:
        if token in text:
            raise AssertionError(f"forbidden placeholder {token}")

    print("diri module inventory ok")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except AssertionError as error:
        print(str(error), file=sys.stderr)
        raise SystemExit(1)
