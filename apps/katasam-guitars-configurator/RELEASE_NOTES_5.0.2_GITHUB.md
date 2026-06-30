## 🚀 KATASAM Configurator v5.0.2

This patch release improves Windows serial endpoint selection and post-flash hardware detection reliability.

### Highlights

- Replaced COM-number heuristics with a protocol handshake check to keep the correct API endpoint when duplicate ports are detected
- Improved post-flash V1/V2 hardware detection by validating middle-fret signal transition against baseline readings
- Reduced startup and reconnect cases where the wrong duplicate endpoint could be preferred

### Included Improvements

- Added handshake-based scoring for duplicate COM endpoints on Windows
- Prioritized ports that return a valid quick device-name response
- Reduced dependence on COM ordering assumptions
- Added baseline and transition validation in hardware auto-detect after flash
- Prevented false-positive V1 auto-selection when no meaningful transition is present

### Known Issue

**Immediately after a reboot/flash window, one extra retry may still be needed before the final endpoint becomes available.**

### Notes

- App version is now 5.0.2
- No config format changes are required for this release
