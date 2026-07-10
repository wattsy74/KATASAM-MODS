#!/usr/bin/env python3
"""Send a BOOTSEL command over USB CDC serial to the Rust firmware."""

from __future__ import annotations

import argparse
import glob
import sys
import time

try:
    import serial
except ImportError:
    print("pyserial is required: pip install pyserial", file=sys.stderr)
    raise


def choose_port(user_port: str | None) -> str:
    if user_port:
        return user_port

    candidates = sorted(glob.glob("/dev/cu.usbmodem*"))
    if not candidates:
        raise RuntimeError("No /dev/cu.usbmodem* ports found. Pass --port explicitly.")

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
    parser.add_argument(
        "--command",
        default="BOOTSEL",
        choices=["BOOTSEL", "REBOOT_BOOTSEL"],
        help="Command string to send (default: BOOTSEL)",
    )
    args = parser.parse_args()

    try:
        port = choose_port(args.port)
        send_bootsel(port, args.baud, args.command)
    except Exception as exc:  # pragma: no cover - command utility path
        print(f"Failed to enter BOOTSEL: {exc}", file=sys.stderr)
        return 1

    print(f"Sent {args.command} to {port}")
    print("If successful, device should remount as RPI-RP2 in BOOTSEL mode.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
