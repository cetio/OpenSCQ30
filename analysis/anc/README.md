# Soundcore Liberty 4 Pro (A3954) ANC Packet Format Documentation

This document describes the structure of the Soundcore Liberty 4 Pro (A3954) state/status packets, focusing on how Active Noise Cancellation (ANC), Transparency, and environmental options are encoded in the packet payload.

Setting the atmosphere level for manual airplane mode is not supported yet.

---

## 1. Packet Metadata & Header Information
The Soundcore Liberty 4 Pro state packet has a body length of **165 bytes**. Inside the payload, ASCII metadata identifies the firmware, device model, and hardware chip:

- **Bytes 6–10**: Firmware Version (e.g., `04.29` represented as ASCII `48, 52, 46, 50, 57`)
- **Bytes 11–14**: Device Model (e.g., `3954` representing model **A3954** as ASCII `48, 52, 46, 50`)
- **Bytes 15–28**: Serial / Unique Chip Identifier (ASCII characters)
- **Bytes 2–3**: Earbud Battery Status (Left & Right charge percentages)
- **Bytes 163–164**: Packet Trailer (always `17` or `0x11`)

---

## 2. Ambient Sound Control Bytes
All ambient audio state properties (ANC, Transparency, Off, Airplane, Wind Reduction) are configured within a compact byte range:

### `body[125]` — Ambient Mode Selector
Determines the active high-level listening profile:
- **`0`**: Noise Cancellation (ANC)
- **`1`**: Transparency
- **`2`**: Normal / Off
- **`3`**: Airplane Mode

### `body[126]` — Intensity / Step Slider
Encodes the current strength/value of the selected mode:
- **In ANC mode (`0`)**: `level + 1` (where manual slider levels 1–10 map to `2` through `11`). E.g., ANC Level 8 is stored as `9`.
- **In Transparency mode (`1`)**: Maps directly to the Transparency level (values `1` through `5` or similar).
- **In Normal mode (`2`)**: Defaults to a placeholder value of `6`.
- **In Airplane mode (`3`)**: Defaults to a placeholder value of `13`.

### `body[127]` — Airplane Adaptive Toggle
- **`1`**: Adaptive pressure-based Airplane ANC enabled (adjusts based on barometric sensor).
- **`0`**: Manual/Standard Airplane ANC (fixed setting).

### `body[128]` — Wind Noise Reduction
- **`1`**: Wind reduction filter enabled.
- **`0`**: Wind reduction filter disabled.

### `body[129]` — Initialization State
- **`255` (`0xFF`)**: Active / fully-initialized state.
- **`0`**: Device uninitialized / offline state.

### `body[152]` — General Adaptive Mode Toggle
Controls automatic adjustment for the current primary ambient mode:
- **`1`**: Adaptive mode active (Adaptive ANC / Adaptive Transparency).
- **`0`**: Manual mode active (Fixed ANC / Standard Transparency).