#!/usr/bin/env python3
import struct
import sys
import zlib
from pathlib import Path
from statistics import mean, pstdev


ROOT = Path(__file__).resolve().parents[2]
SCREENSHOT_DIR = ROOT / "docs/verification/diri-ui-screenshot"
REQUIRED = {
    "homie": SCREENSHOT_DIR / "homie-window-2026-08-07.png",
    "diri": SCREENSHOT_DIR / "diri-reference-2026-08-07.png",
    "report": SCREENSHOT_DIR / "visual-verification-report.md",
}


def reconstruct_png(path: Path) -> tuple[int, int, int, list[list[int]], int]:
    data = path.read_bytes()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError(f"{path} is not a PNG")
    pos = 8
    width = height = color_type = bit_depth = None
    idat = []
    while pos < len(data):
        length = struct.unpack(">I", data[pos : pos + 4])[0]
        kind = data[pos + 4 : pos + 8]
        chunk = data[pos + 8 : pos + 8 + length]
        pos += length + 12
        if kind == b"IHDR":
            width, height, bit_depth, color_type, _, _, _ = struct.unpack(">IIBBBBB", chunk)
        elif kind == b"IDAT":
            idat.append(chunk)
    raw = zlib.decompress(b"".join(idat))
    if bit_depth != 8 or color_type not in (2, 6):
        raise ValueError(f"{path} uses unsupported PNG format: bit_depth={bit_depth} color_type={color_type}")
    channels = 4 if color_type == 6 else 3
    stride = width * channels
    rows = []
    previous = [0] * stride
    cursor = 0
    for _ in range(height):
        filter_type = raw[cursor]
        cursor += 1
        scanline = list(raw[cursor : cursor + stride])
        cursor += stride
        row = [0] * stride
        for index, value in enumerate(scanline):
            left = row[index - channels] if index >= channels else 0
            up = previous[index]
            up_left = previous[index - channels] if index >= channels else 0
            if filter_type == 0:
                reconstructed = value
            elif filter_type == 1:
                reconstructed = value + left
            elif filter_type == 2:
                reconstructed = value + up
            elif filter_type == 3:
                reconstructed = value + ((left + up) // 2)
            elif filter_type == 4:
                predicted = paeth(left, up, up_left)
                reconstructed = value + predicted
            else:
                raise ValueError(f"{path} uses unsupported PNG filter: {filter_type}")
            row[index] = reconstructed & 0xFF
        rows.append(row)
        previous = row
    return width, height, channels, rows, len(raw)


def paeth(left: int, up: int, up_left: int) -> int:
    prediction = left + up - up_left
    left_distance = abs(prediction - left)
    up_distance = abs(prediction - up)
    up_left_distance = abs(prediction - up_left)
    if left_distance <= up_distance and left_distance <= up_left_distance:
        return left
    if up_distance <= up_left_distance:
        return up
    return up_left


def png_stats(path: Path) -> dict:
    width, height, channels, rows, raw_len = reconstruct_png(path)
    nonzero_raw_bytes = 0
    for row in rows:
        nonzero_raw_bytes += sum(1 for byte in row if byte)
    metrics = structural_metrics(width, height, channels, rows)
    return {
        "bytes": path.stat().st_size,
        "width": width,
        "height": height,
        "raw_bytes": raw_len,
        "nonzero_raw_bytes": nonzero_raw_bytes,
        **metrics,
    }


def luma(row: list[int], x: int, channels: int) -> float:
    offset = x * channels
    red = row[offset]
    green = row[offset + 1]
    blue = row[offset + 2]
    return 0.2126 * red + 0.7152 * green + 0.0722 * blue


def structural_metrics(width: int, height: int, channels: int, rows: list[list[int]]) -> dict:
    left = band_luma(rows, channels, 0, int(width * 0.22), 0, height)
    center = band_luma(rows, channels, int(width * 0.22), int(width * 0.78), 0, height)
    right = band_luma(rows, channels, int(width * 0.78), width, 0, height)
    peaks = vertical_edge_peaks(rows, channels, width, height)
    return {
        "left_luma": round(left, 2),
        "center_luma": round(center, 2),
        "right_luma": round(right, 2),
        "left_center_gap": round(abs(left - center), 2),
        "right_center_gap": round(abs(right - center), 2),
        "vertical_edge_peaks": [round(peak, 3) for peak in peaks],
    }


def band_luma(
    rows: list[list[int]],
    channels: int,
    x_start: int,
    x_end: int,
    y_start: int,
    y_end: int,
) -> float:
    width = len(rows[0]) // channels
    height = len(rows)
    x_start = max(0, min(width - 1, x_start))
    x_end = max(x_start + 1, min(width, x_end))
    y_start = max(0, min(height - 1, y_start))
    y_end = max(y_start + 1, min(height, y_end))
    x_step = max(1, (x_end - x_start) // 80)
    y_step = max(1, (y_end - y_start) // 80)
    values = [
        luma(rows[y], x, channels)
        for y in range(y_start, y_end, y_step)
        for x in range(x_start, x_end, x_step)
    ]
    return mean(values)


def vertical_edge_peaks(rows: list[list[int]], channels: int, width: int, height: int) -> list[float]:
    y_step = max(1, height // 160)
    edges = []
    for x in range(1, width):
        values = [
            abs(luma(rows[y], x, channels) - luma(rows[y], x - 1, channels))
            for y in range(0, height, y_step)
        ]
        edges.append(mean(values))
    threshold = mean(edges) + (2.5 * pstdev(edges))
    peaks = []
    for index, value in enumerate(edges):
        x = index + 1
        normalized = x / width
        if 0.04 < normalized < 0.96 and value > threshold:
            peaks.append((value, normalized))
    return [normalized for _, normalized in sorted(peaks, reverse=True)[:8]]


def require_structural_workbench(label: str, stats: dict) -> None:
    if stats["width"] < 900 or stats["height"] < 560:
        raise ValueError(f"{label} screenshot too small: {stats}")
    if stats["nonzero_raw_bytes"] < 100_000:
        raise ValueError(f"{label} screenshot appears blank: {stats}")
    if stats["left_center_gap"] < 30:
        raise ValueError(f"{label} screenshot lacks sidebar/workbench contrast: {stats}")
    if stats["right_center_gap"] < 30:
        raise ValueError(f"{label} screenshot lacks inspector/workbench contrast: {stats}")
    if len(stats["vertical_edge_peaks"]) < 2:
        raise ValueError(f"{label} screenshot lacks structural separator edges: {stats}")


def main() -> int:
    for label, path in REQUIRED.items():
        if not path.exists():
            print(f"missing {label}: {path}", file=sys.stderr)
            return 1
    for label in ["homie", "diri"]:
        stats = png_stats(REQUIRED[label])
        try:
            require_structural_workbench(label, stats)
        except ValueError as error:
            print(str(error), file=sys.stderr)
            return 1
        print(f"{label} screenshot structural ok: {stats}")
    report = REQUIRED["report"].read_text()
    for needle in [
        "First screenshot capture",
        "Accepted window capture",
        "Diri reference screenshot",
        "Structural comparison gate",
        "left/center/right luma",
        "vertical edge peaks",
        "black screen",
    ]:
        if needle not in report:
            print(f"visual report missing {needle!r}", file=sys.stderr)
            return 1
    print("visual report ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
