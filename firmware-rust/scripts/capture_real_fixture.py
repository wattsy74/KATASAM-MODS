#!/usr/bin/env python3
"""Capture a real CircuitPython serial session for a protocol fixture.

Usage:
  ./scripts/capture_real_fixture.py capture-001
  ./scripts/capture_real_fixture.py capture-001 --port /dev/cu.usbmodem2101
"""

import argparse
import glob
import sys
import time
from pathlib import Path

try:
    import serial
except Exception as exc:  # pragma: no cover - runtime env dependent
    print(
        "pyserial is required. Install with: python3 -m pip install --user pyserial",
        file=sys.stderr,
    )
    raise SystemExit(1) from exc


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("fixture_id", help="fixture id like capture-001")
    parser.add_argument(
        "--port",
        default="",
        help="serial port path (default: first /dev/cu.usbmodem*)",
    )
    parser.add_argument("--baud", type=int, default=115200)
    parser.add_argument("--initial-wait", type=float, default=0.8)
    parser.add_argument("--post-write-delay", type=float, default=0.05)
    parser.add_argument("--max-read-seconds", type=float, default=2.0)
    parser.add_argument("--idle-gap-seconds", type=float, default=0.25)
    return parser.parse_args()


def discover_port(explicit_port: str) -> str:
    if explicit_port:
        if explicit_port.startswith("/"):
            return explicit_port
        # Accept shorthand like cu.usbmodem2103 on macOS.
        if explicit_port.startswith("cu.") or explicit_port.startswith("tty."):
            return f"/dev/{explicit_port}"
        return explicit_port
    candidates = sorted(glob.glob("/dev/cu.usbmodem*"))
    if not candidates:
        print("No /dev/cu.usbmodem* serial ports found.", file=sys.stderr)
        raise SystemExit(1)
    return candidates[0]


def read_until_idle(ser: serial.Serial, max_total: float, idle_gap: float) -> bytes:
    buf = bytearray()
    start = time.time()
    last_data = start
    while time.time() - start < max_total:
        chunk = ser.read(512)
        if chunk:
            buf.extend(chunk)
            last_data = time.time()
            continue
        if time.time() - last_data >= idle_gap:
            break
    return bytes(buf)


def main() -> int:
    args = parse_args()

    root_dir = Path(__file__).resolve().parent.parent
    fixtures_dir = root_dir / "fixtures" / "protocol"
    cmd_file = fixtures_dir / f"cmd-real-{args.fixture_id}-session.txt"
    golden_file = fixtures_dir / f"golden-real-{args.fixture_id}-session.txt"

    if not cmd_file.exists():
        print(f"Missing command fixture: {cmd_file}", file=sys.stderr)
        return 1

    commands = [
        line.strip()
        for line in cmd_file.read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    if not commands:
        print(f"No commands found in: {cmd_file}", file=sys.stderr)
        return 1

    port = discover_port(args.port)
    print(f"Using serial port: {port}")

    captured = []
    try:
        with serial.Serial(port, args.baud, timeout=0.05) as ser:
            time.sleep(args.initial_wait)
            ser.reset_input_buffer()
            ser.reset_output_buffer()

            for cmd in commands:
                ser.write((cmd + "\n").encode("utf-8"))
                ser.flush()
                time.sleep(args.post_write_delay)
                raw = read_until_idle(ser, args.max_read_seconds, args.idle_gap_seconds)
                text = raw.decode("utf-8", errors="replace")
                text = text.replace("\r\n", "\n").replace("\r", "\n")
                for line in text.split("\n"):
                    if line:
                        captured.append(line)
    except serial.SerialException as exc:
        print(f"Could not open serial port {port}: {exc}", file=sys.stderr)
        print("Tip: use a full path like /dev/cu.usbmodem2103", file=sys.stderr)
        return 1

    golden_file.write_text(
        "\n".join(captured) + ("\n" if captured else ""),
        encoding="utf-8",
    )

    print(f"Wrote {len(captured)} lines to {golden_file}")
    if not captured:
        print(
            "No response lines were captured. If this is unexpected, try --port with another /dev/cu.usbmodem* device.",
            file=sys.stderr,
        )
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
