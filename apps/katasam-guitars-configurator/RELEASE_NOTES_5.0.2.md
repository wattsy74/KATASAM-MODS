## KATASAM Configurator v5.0.2

This patch release focuses on Windows connection reliability and post-flash hardware detection accuracy.

### Highlights

- Replaced COM-port ordering heuristics with a protocol handshake check so the app keeps the real API endpoint when duplicate serial ports appear.
- Improved post-flash hardware detection by validating signal transition against baseline readings before choosing V1 or V2 defaults.
- Reduced cases where the wrong endpoint is shown in device selection during startup and reconnect.

### What Changed

#### Device detection and COM endpoint selection

- Updated Windows duplicate-port handling to prioritize the endpoint that successfully responds to a quick device-name handshake.
- Removed dependence on COM number ordering for endpoint identity.
- Improved fallback behavior so non-responding duplicate endpoints are deprioritized.

#### Post-flash hardware version detection

- Added baseline and transition checks to middle-fret hardware detection.
- Prevented false-positive auto-selection of V1 defaults when no meaningful signal transition is detected.
- Improved detection diagnostics used by the flash follow-up flow.

### Known Issues

- If a device is in a transient reboot window, immediate post-flash detection can still require one additional retry before the final endpoint appears.

### Upgrade Notes

- App version is now 5.0.2.
- No config format changes are required for this update.
