# Soundcore Liberty 4 Pro (A3954) Settings Packet Format Documentation

This document describes the structure of the Soundcore Liberty 4 Pro (A3954) state/status packets, focusing on how wearing detection, adaptive leakage compensation, and safe volume settings are encoded in the packet payload.

---

## 1. General Device Settings

The general earbud behavior features are mapped to the following bytes:

### `body[153]` — Adaptive Leakage Compensation
This feature automatically detects ear canal structure and seal state to adjust ANC parameters for compensation:
- **`1`**: Adaptive Leakage Compensation enabled.
- **`0`**: Adaptive Leakage Compensation disabled.

### `body[159]` — Wearing Detection Toggle
Determines whether smart playback auto-pause/resume and touch sensors are guided by the earbud wear state:
- **`1`**: Wearing Detection enabled.
- **`0`**: Wearing Detection disabled.

---

## 2. Safe Volume Settings

The Liberty 4 Pro features real-time volume limit safeguards to protect hearing health. The control and metering settings are located in the safe volume sub-range:

### `body[145]` — Limit High Volume Toggle
- **`1`**: High Volume Limit safeguard enabled.
- **`0`**: High Volume Limit safeguard disabled.

### `body[146]` — High Volume Limit Value
Specifies the maximum allowable decibel output when the volume limit safeguard is active:
- **Value**: Stored directly as a single-byte integer representing the decibel threshold (e.g., `90` representing a **90dB** limit).

### `body[147]` — Safe Volume Meter Refresh Rate
Controls how frequently the safe volume meter updates and polls the decibel status:
- **`0`**: Realtime updates
- **`1`**: 10-second updates
- **`2`**: 1-minute updates
