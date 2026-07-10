# Legacy Protocol Command Matrix (CircuitPython Baseline)

Source reference: firmware/serial_handler.py

## Handshake and Identity

- `FIRMWARE_READY?`
- `READY?`
- `READVERSION`
- `READDEVICENAME`
- `READUID`

## File Operations

- `READFILE:/config.json`
- `READFILE:/user_presets.json`
- `WRITEFILE:/config.json`
- `WRITEFILE:/user_presets.json`
- `MKDIR:/updates`

## Input/Pin/Calibration Operations

- `READWHAMMY`
- `READJOYSTICK`
- `READPIN:<name>`
- `DETECTPIN:<name>`
- `SAVEPIN:<name>:<pin>`
- `CANCELPINDETECT`

## LED and Visual Operations

- `PREVIEWLED:<led-name>:<#RRGGBB>`
- `SETLED:<led-name>:<#RRGGBB>`
- `LEDRESTORE`
- `TILTWAVE`
- `TILTWAVE_ENABLE:ON`
- `TILTWAVE_ENABLE:OFF`
- `DEMO`

## Preset Import Operation

- `IMPORTUSER`

## Reboot Operations

- `REBOOT`
- `REBOOTBOOTSEL`

## Notes For Rust Parity

- Capture exact ACK and response text, not just semantics.
- Capture `END` delimiters for multi-line response commands.
- Treat unknown command behavior as part of compatibility contract.
