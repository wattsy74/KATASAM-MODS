#!/usr/bin/env python3
"""Capture HID baseline fixtures from a connected controller.

This script records three phases to the canonical fixture files:
- fixtures/hid/idle_YYYYMMDD.txt
- fixtures/hid/single_actions_YYYYMMDD.txt
- fixtures/hid/combo_actions_YYYYMMDD.txt
"""

from __future__ import annotations

import argparse
import sys
import time
from pathlib import Path
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


DEFAULT_VID = 0x6997
DEFAULT_PID = 0xB528
DEFAULT_REPORT_LEN = 64


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Capture HID fixture streams")
    parser.add_argument("--vid", type=lambda x: int(x, 0), default=DEFAULT_VID)
    parser.add_argument("--pid", type=lambda x: int(x, 0), default=DEFAULT_PID)
    parser.add_argument("--report-len", type=int, default=DEFAULT_REPORT_LEN)
    parser.add_argument("--idle-seconds", type=int, default=10)
    parser.add_argument("--single-seconds", type=int, default=30)
    parser.add_argument("--combo-seconds", type=int, default=30)
    return parser.parse_args()


def hex_line(report: list[int]) -> str:
    return " ".join(f"{b:02X}" for b in report)


def capture_phase(dev: Any, seconds: int, report_len: int, out_path: Path) -> int:
    end = time.time() + seconds
    lines: list[str] = []

    while time.time() < end:
        report = dev.read(report_len, timeout_ms=100)
        if not report:
            continue
        lines.append(hex_line(report))

    out_path.write_text("\n".join(lines) + ("\n" if lines else ""), encoding="utf-8")
    return len(lines)


def main() -> int:
    args = parse_args()
    root = Path(__file__).resolve().parent.parent
    hid_dir = root / "fixtures" / "hid"

    idle_file = hid_dir / "idle_YYYYMMDD.txt"
    single_file = hid_dir / "single_actions_YYYYMMDD.txt"
    combo_file = hid_dir / "combo_actions_YYYYMMDD.txt"

    print(
        f"Opening HID device VID:PID {args.vid:04X}:{args.pid:04X} "
        f"(report_len={args.report_len})"
    )

    dev = hid.device()
    try:
        dev.open(args.vid, args.pid)
    except Exception as exc:  # pragma: no cover - hardware dependent
        print(
            "Could not open HID device. Confirm controller is connected and not exclusively "
            f"claimed by another app. Error: {exc}",
            file=sys.stderr,
        )
        return 1

    dev.set_nonblocking(True)

    try:
        print(f"[idle] Capture for {args.idle_seconds}s. Leave controller untouched.")
        idle_count = capture_phase(dev, args.idle_seconds, args.report_len, idle_file)
        print(f"[idle] wrote {idle_count} reports to {idle_file}")

        print(
            f"[single_actions] Capture for {args.single_seconds}s. "
            "Perform single button/axis actions now."
        )
        single_count = capture_phase(dev, args.single_seconds, args.report_len, single_file)
        print(f"[single_actions] wrote {single_count} reports to {single_file}")

        print(
            f"[combo_actions] Capture for {args.combo_seconds}s. "
            "Perform combos/chords/diagonal transitions now."
        )
        combo_count = capture_phase(dev, args.combo_seconds, args.report_len, combo_file)
        print(f"[combo_actions] wrote {combo_count} reports to {combo_file}")
    finally:
        dev.close()

    if idle_count == 0 or single_count == 0 or combo_count == 0:
        print("One or more phases captured zero reports.", file=sys.stderr)
        return 2

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
