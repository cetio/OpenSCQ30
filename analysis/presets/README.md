# Soundcore Liberty 4 Pro (A3954) Presets & Equalizer Packet Format Documentation

This document describes the structure of the Soundcore Liberty 4 Pro (A3954) state/status packets, focusing on how audio presets, custom Equalizers (EQ), and Spatial Audio are encoded in the packet payload.

---

## 1. Equalizer & Preset Bytes

All audio profile configurations (Default Presets, Custom EQ, HearID Preference Test) are configured within the following bytes:

### `body[44]` — Default Preset Selector
Determines which built-in Soundcore EQ preset is active when `body[149] == 2` (Default Presets Mode):
- **`0`**: Soundcore Signature
- **`1`**: Acoustic
- **`2`**: Bass Booster
- **`3`**: Bass Reducer
- **`4`**: Classical

### `body[46..=53]` — Equalizer Gains (8 Bands)
These 8 consecutive bytes represent the gain values for the 8-band custom equalizer.
- **Value Range**: Each byte ranges from `0` to `240` (or similar), where `120` represents a flat gain (`0.0 dB`).
- **Bands**:
  - `body[46]`: Band 1
  - `body[47]`: Band 2
  - `body[48]`: Band 3
  - `body[49]`: Band 4
  - `body[50]`: Band 5
  - `body[51]`: Band 6
  - `body[52]`: Band 7
  - `body[53]`: Band 8

### `body[55]` — HearID EQ Gain Offset / Status
- **`60`**: Active when HearID Preference Test is selected.
- **`0`**: Active when a standard preset is selected.

### `body[67]` — Preference Test Active Indicator
- **`1`**: Preference Test (HearID) active.
- **`0`**: Preference Test disabled.

---

## 2. Audio Mode & Spatial Audio Selector Bytes

### `body[148]` — Spatial Audio Toggle
- **`1`**: Spatial Audio enabled.
- **`0`**: Spatial Audio disabled (Stereo).

### `body[149]` — Equalizer Type / Head Tracking Selector
The behavior of this byte depends on whether Spatial Audio is enabled:
- **When Spatial Audio is Disabled (`body[148] == 0`)**: Controls Equalizer category:
  - **`1`**: Preference Test / HearID Custom EQ active.
  - **`2`**: Default Preset mode active (uses the preset from `body[44]`).
- **When Spatial Audio is Enabled (`body[148] == 1`)**: Controls Head Tracking:
  - **`1`**: Fixed Spatial Audio (no head tracking).
  - **`2`**: Head Tracking Spatial Audio active.

### `body[150]` — Preference Test Status / Spatial Audio Submode
The behavior of this byte depends on whether Spatial Audio is enabled:
- **When Spatial Audio is Disabled (`body[148] == 0`)**: Controls Preference Test profile toggle:
  - **`0`**: Preference Test Profile inactive.
  - **`2`**: Preference Test Profile active.
- **When Spatial Audio is Enabled (`body[148] == 1`)**: Controls the active Spatial Audio environment profile:
  - **`0`**: Spatial Music
  - **`1`**: Spatial Podcast
  - **`2`**: Spatial Movie
  - **`3`**: Spatial Gaming
