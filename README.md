# KATASAM MODS

This repository now contains the rebranded KATASAM desktop applications migrated from:

- `wattsy74/BumbleGum-Guitars-Configurator`
- `wattsy74/BumbleGum-Firmware-Switcher`

## Repository Layout

```
apps/
	katasam-guitars-configurator/
	katasam-firmware-switcher/
```

## Applications

### 1) KATASAM Guitars Configurator

Path: `apps/katasam-guitars-configurator`

- Package name: `katasam-guitars-configurator`
- Product name: `KATASAM Guitars Configurator`

Run locally:

```bash
cd apps/katasam-guitars-configurator
npm install
npm start
```

### 2) KATASAM Firmware Switcher

Path: `apps/katasam-firmware-switcher`

- Package name: `katasam-firmware-switcher`
- Product name: `KATASAM Firmware Switcher`

Run locally:

```bash
cd apps/katasam-firmware-switcher
npm install
npm start
```

## Notes

- User-facing BumbleGum branding has been rebranded to KATASAM across both apps.
- Firmware download URLs currently point to the existing firmware release source repo so flashing continues to work during migration.
- Configurator auto-update target is set to `wattsy74/KATASAM-MODS`.
