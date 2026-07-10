#!/usr/bin/env python3
"""Verify live HID report behavior from a connected controller.

This script performs a short capture and checks:
- minimum report count
- minimum unique report count
- per-byte activity threshold
"""

from __future__ import annotations

import argparse
import sys
import time
from collections import Counter
from typing import Any

try:
    import hid
except Exception as exc:  # pragma: no cover - environment dependent
    print(f"hid import error: {exc}", file=sys.stderr)
    print(
        "Use macOS-friendly backend:\n"
        "  python3 -m pip uninstall -y hid\n"
        "  python3 -m pip install --user --force-reinstall hidapi",
        file=sys.stderr,
    )
    raise SystemExit(1) from exc


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Verify live HID report stream")
    parser.add_argument("--vid", type=lambda x: int(x, 0), required=True, help="USB VID, e.g. 0x2E8A")
    parser.add_argument("--pid", type=lambda x: int(x, 0), required=True, help="USB PID, e.g. 0x1031")
    parser.add_argument("--duration", type=float, default=5.0, help="Capture duration in seconds")
    parser.add_argument("--report-len", type=int, default=64, help="HID report length")
    parser.add_argument("--timeout-ms", type=int, default=100, help="Read timeout in milliseconds")
    parser.add_argument("--min-reports", type=int, default=30, help="Minimum total reports")
    parser.add_argument("--min-unique", type=int, default=5, help="Minimum unique reports")
    parser.add_argument(
        "--min-active-bytes",
        type=int,
        default=2,
        help="Minimum number of bytes that must change at least once",
    )
    parser.add_argument(
        "--min-change-rate",
        type=float,
        default=0.05,
        help="Minimum change rate for active bytes (0.0 to 1.0)",
    )
    parser.add_argument("--print-samples", type=int, default=5, help="How many top report samples to print")
    return parser.parse_args()


def hex_line(report: list[int]) -> str:
    return " ".join(f"{b:02X}" for b in report)


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


def open_device(vid: int, pid: int) -> Any:
    dev = hid.device()
    dev.open(vid, pid)
    dev.set_nonblocking(True)
    return dev


def collect_reports(dev: Any, duration: float, report_len: int, timeout_ms: int) -> list[list[int]]:
    reports: list[list[int]] = []
    end = time.time() + duration
    while time.time() < end:
        report = dev.read(report_len, timeout_ms=timeout_ms)
        if report:
            reports.append(report)
    return reports


def verify(reports: list[list[int]], args: argparse.Namespace) -> tuple[bool, list[str]]:
    errors: list[str] = []

    flattened = [hex_line(report) for report in reports]
    unique_count = len(set(flattened))

    byte_changes, transitions = per_byte_change_counts(reports)
    active_indices = [i for i, c in enumerate(byte_changes) if c > 0]

    if len(reports) < args.min_reports:
        errors.append(f"only {len(reports)} reports captured (< {args.min_reports})")

    if unique_count < args.min_unique:
        errors.append(f"only {unique_count} unique reports (< {args.min_unique})")

    if len(active_indices) < args.min_active_bytes:
        errors.append(
            f"only {len(active_indices)} active bytes changed (< {args.min_active_bytes})"
        )

    if transitions > 0:
        too_static = []
        for i in active_indices:
            rate = byte_changes[i] / transitions
            if rate < args.min_change_rate:
                too_static.append((i, rate))
        if too_static and len(active_indices) <= args.min_active_bytes:
            formatted = ", ".join(f"b{i}:{rate:.2%}" for i, rate in too_static)
            errors.append(f"active bytes below min-change-rate: {formatted}")

    return len(errors) == 0, errors


def main() -> int:
    args = parse_args()

    print(
        f"Opening HID device {args.vid:04X}:{args.pid:04X} for {args.duration:.1f}s "
        f"(report_len={args.report_len})"
    )

    try:
        dev = open_device(args.vid, args.pid)
    except Exception as exc:  # pragma: no cover - hardware dependent
        print(f"Could not open HID device: {exc}", file=sys.stderr)
        return 2

    try:
        reports = collect_reports(dev, args.duration, args.report_len, args.timeout_ms)
    finally:
        dev.close()

    counter = Counter(hex_line(r) for r in reports)
    unique_count = len(counter)
    byte_changes, transitions = per_byte_change_counts(reports)
    active_indices = [i for i, c in enumerate(byte_changes) if c > 0]

    print(f"reports: {len(reports)}")
    print(f"unique reports: {unique_count}")
    print(f"transitions: {transitions}")
    print(f"active bytes: {len(active_indices)}")

    if transitions > 0 and active_indices:
        top_bytes = sorted(active_indices, key=lambda i: byte_changes[i], reverse=True)[:8]
        detail = ", ".join(
            f"b{i}:{byte_changes[i]}/{transitions} ({(byte_changes[i]/transitions):.1%})"
            for i in top_bytes
        )
        print(f"most-active bytes: {detail}")

    print("top reports:")
    for report, count in counter.most_common(max(args.print_samples, 0)):
        print(f"  {count:>6}  {report}")

    ok, errors = verify(reports, args)
    if ok:
        print("PASS: HID stream meets minimum activity thresholds.")
        return 0

    print("FAIL: HID stream verification failed.", file=sys.stderr)
    for err in errors:
        print(f"  - {err}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
