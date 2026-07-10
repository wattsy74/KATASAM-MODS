#!/usr/bin/env python3
"""Send a BOOTSEL command over USB CDC serial to the Rust firmware."""

from __future__ import annotations

import argparse
import glob
import sys
import time

try:
    import serial
    from serial.tools import list_ports
except ImportError:
    print("pyserial is required: pip install pyserial", file=sys.stderr)
    raise


DEFAULT_VID = 0x2E8A
DEFAULT_PID = 0x1031
DEFAULT_COMMAND = "KATASAM_BOOTSEL_V1"


def discover_ports(vid: int | None, pid: int | None) -> list[str]:
    preferred: list[str] = []
    fallback: list[str] = []

    for info in list_ports.comports():
        if info.device is None:
            continue

        fallback.append(info.device)
        if vid is not None and pid is not None and info.vid == vid and info.pid == pid:
            preferred.append(info.device)

    if preferred:
        return sorted(set(preferred))

    pattern_matches = sorted(
        set(
            glob.glob("/dev/cu.usbmodem*")
            + glob.glob("/dev/tty.usbmodem*")
            + glob.glob("/dev/cu.usbserial*")
            + glob.glob("/dev/tty.usbserial*")
        )
    )
    if pattern_matches:
        return pattern_matches

    return sorted(set(fallback))


def choose_port(user_port: str | None, vid: int | None, pid: int | None) -> str:
    if user_port:
        if user_port.startswith("/dev/"):
            return user_port

        # Accept shorthand values (e.g. 00011) and resolve to a detected device path.
        candidates = discover_ports(vid, pid)
        exact = [path for path in candidates if path == user_port]
        if exact:
            return exact[0]

        suffix = [path for path in candidates if path.endswith(user_port)]
        if len(suffix) == 1:
            return suffix[0]
        if len(suffix) > 1:
            raise RuntimeError(
                f"Port shorthand '{user_port}' is ambiguous: {', '.join(suffix)}"
            )

        contains = [path for path in candidates if user_port in path]
        if len(contains) == 1:
            return contains[0]
        if len(contains) > 1:
            raise RuntimeError(
                f"Port shorthand '{user_port}' matches multiple ports: {', '.join(contains)}"
            )

        raise RuntimeError(
            f"Port '{user_port}' not found. Run with --list-ports and use a full /dev/... path."
        )

    candidates = discover_ports(vid, pid)
    if not candidates:
        raise RuntimeError(
            "No serial ports found. Confirm USB data cable, board power, and that firmware is running."
        )

    return candidates[0]


def send_bootsel(port: str, baudrate: int, command: str) -> None:
    with serial.Serial(port=port, baudrate=baudrate, timeout=0.25, write_timeout=1.0) as ser:
        time.sleep(0.15)
        ser.reset_input_buffer()
        ser.reset_output_buffer()
        ser.write((command + "\n").encode("ascii"))
        ser.flush()


def main() -> int:
    parser = argparse.ArgumentParser(description="Trigger BOOTSEL mode on Rust firmware via USB CDC.")
    parser.add_argument("--port", help="Serial port path, e.g. /dev/cu.usbmodem12345")
    parser.add_argument("--baud", type=int, default=115200, help="Serial baud rate (default: 115200)")
    parser.add_argument("--vid", type=lambda x: int(x, 0), default=DEFAULT_VID, help="USB VID filter")
    parser.add_argument("--pid", type=lambda x: int(x, 0), default=DEFAULT_PID, help="USB PID filter")
    parser.add_argument("--list-ports", action="store_true", help="List detected serial ports and exit")
    parser.add_argument(
        "--command",
        default=DEFAULT_COMMAND,
        choices=[
            "KATASAM_BOOTSEL_V1",
            "KATASAM_REBOOT_BOOTSEL_V1",
            "BOOTSEL",
            "REBOOT_BOOTSEL",
        ],
        help="Command string to send (default: KATASAM_BOOTSEL_V1)",
    )
    args = parser.parse_args()

    if args.list_ports:
        ports = discover_ports(args.vid, args.pid)
        if not ports:
            print("No serial ports detected.")
            return 1
        print("Detected serial ports:")
        for path in ports:
            print(f"  {path}")
        return 0

    try:
        port = choose_port(args.port, args.vid, args.pid)
        send_bootsel(port, args.baud, args.command)
    except Exception as exc:  # pragma: no cover - command utility path
        print(f"Failed to enter BOOTSEL: {exc}", file=sys.stderr)
        return 1

    print(f"Sent {args.command} to {port}")
    print("If successful, device should remount as RPI-RP2 in BOOTSEL mode.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
