#!/usr/bin/env python3
"""Summarize HID fixture captures for parity tracking.

This script reads canonical HID fixture files and prints:
- report counts
- unique report counts
- transition rates
- per-byte change rates
"""

from __future__ import annotations

import argparse
from collections import Counter
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Analyze HID fixture files")
    parser.add_argument(
        "--hid-dir",
        default="fixtures/hid",
        help="HID fixture directory (default: fixtures/hid)",
    )
    parser.add_argument(
        "--files",
        nargs="*",
        default=[
            "idle_YYYYMMDD.txt",
            "single_actions_YYYYMMDD.txt",
            "combo_actions_YYYYMMDD.txt",
        ],
        help="Fixture files to analyze",
    )
    return parser.parse_args()


def parse_report(line: str) -> list[int] | None:
    text = line.strip()
    if not text or text.startswith("#"):
        return None
    parts = text.split()
    try:
        return [int(part, 16) for part in parts]
    except ValueError:
        return None


def per_byte_change_counts(reports: list[list[int]]) -> tuple[list[int], int]:
    if len(reports) < 2:
        return [], 0

    width = max(len(r) for r in reports)
    counts = [0] * width
    transitions = 0

    prev = reports[0]
    for cur in reports[1:]:
        transitions += 1
        for i in range(width):
            prev_b = prev[i] if i < len(prev) else None
            cur_b = cur[i] if i < len(cur) else None
            if prev_b != cur_b:
                counts[i] += 1
        prev = cur

    return counts, transitions


def summarize_file(path: Path) -> dict[str, object]:
    lines = path.read_text(encoding="utf-8").splitlines()
    reports = [r for r in (parse_report(line) for line in lines) if r is not None]

    flattened = [" ".join(f"{b:02X}" for b in report) for report in reports]
    unique_counter = Counter(flattened)
    unique_reports = len(unique_counter)
    total_reports = len(reports)

    byte_changes, transitions = per_byte_change_counts(reports)

    return {
        "path": path,
        "total_reports": total_reports,
        "unique_reports": unique_reports,
        "unique_ratio": (unique_reports / total_reports) if total_reports else 0.0,
        "transitions": transitions,
        "byte_changes": byte_changes,
        "top_reports": unique_counter.most_common(5),
    }


def print_summary(summary: dict[str, object]) -> None:
    path = summary["path"]
    total_reports = int(summary["total_reports"])
    unique_reports = int(summary["unique_reports"])
    unique_ratio = float(summary["unique_ratio"])
    transitions = int(summary["transitions"])
    byte_changes = list(summary["byte_changes"])
    top_reports = list(summary["top_reports"])

    print(f"\\n== {path} ==")
    print(f"reports: {total_reports}")
    print(f"unique reports: {unique_reports} ({unique_ratio:.2%})")
    print(f"transitions: {transitions}")

    if transitions > 0 and byte_changes:
        change_rates = [c / transitions for c in byte_changes]
        top_indices = sorted(range(len(change_rates)), key=lambda i: change_rates[i], reverse=True)[:8]
        printable = ", ".join(
            f"b{i}:{byte_changes[i]}/{transitions} ({change_rates[i]:.1%})" for i in top_indices if byte_changes[i] > 0
        )
        print(f"most-active bytes: {printable or 'none'}")
    else:
        print("most-active bytes: none")

    print("top reports:")
    if not top_reports:
        print("  (none)")
    else:
        for report, count in top_reports:
            print(f"  {count:>6}  {report}")


def main() -> int:
    args = parse_args()
    hid_dir = Path(args.hid_dir)

    missing = []
    summaries = []

    for name in args.files:
        path = hid_dir / name
        if not path.exists():
            missing.append(path)
            continue
        summaries.append(summarize_file(path))

    if missing:
        print("Missing fixture files:")
        for path in missing:
            print(f"  {path}")
        return 1

    for summary in summaries:
        print_summary(summary)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
