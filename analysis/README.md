# Soundcore Liberty 4 Pro (A3954) Protocol & Packet Analysis

This workspace contains reverse-engineered state packets for the **Soundcore Liberty 4 Pro** (Model **A3954**), a premium set of noise-cancelling earbuds featuring a smart charging case with an integrated display and touch bar.

All packets are captured as raw `u8` byte arrays with a length of **165 bytes** and are stored in `@/home/cet/Documents/Modeling/soundcore/initial.rs` and the subfiles inside `@/home/cet/Documents/Modeling/soundcore/anc/`.

---

## 1. System Metadata & Firmware Mapping

By analyzing the ASCII conversions of the bytes in `@/home/cet/Documents/Modeling/soundcore/initial.rs`, we have mapped several authoritative hardware and software fields embedded directly in the packet body:

- **Primary Earbud Firmware**: **`04.29`**
  - **Byte Range**: `body[6..11]`
  - **Raw Values**: `[48, 52, 46, 50, 57]` (ASCII `"04.29"`)
- **Secondary Earbud Firmware (Repeated)**: **`04.29`**
  - **Byte Range**: `body[11..16]`
  - **Raw Values**: `[48, 52, 46, 50, 57]` (ASCII `"04.29"`)
- **Earbud Serial Number**: **`395438744F8A9DF4`**
  - **Byte Range**: `body[16..32]`
  - **Raw Values**: `[51, 57, 53, 52, 51, 56, 55, 52, 52, 70, 56, 65, 57, 68, 70, 52]` (ASCII `"395438744F8A9DF4"`)
  - *Note*: The prefix `3954` represents the model number **A3954** (Liberty 4 Pro), while the rest of the string is the unique hardware identifier.
- **Charging Case Firmware**: **`02.58`**
  - **Byte Range**: `body[32..37]`
  - **Raw Values**: `[48, 50, 46, 53, 56]` (ASCII `"02.58"`)
- **Charging Case Serial Number**: **`3954F49D8A40F00B`**
  - *Note*: This is the serial number of the smart case, which shares the `3954` model prefix.

---

## 2. Earbud Telemetry

The early bytes of the payload convey active runtime statuses:

- **Battery Levels**:
  - **Left Earbud**: `body[2]` (varies between `95`, `96`, `98` depending on the state file)
  - **Right Earbud**: `body[3]` (consistently `100` in our captures)

---

## 3. Other Identified Fields

- **Packet Frame Boundary**: `body[163..165]` is always `[17, 17]` (`0x11, 0x11`), marking the end of the state report transmission.
- **Initialization/Active Flag (`body[129]`)**: Set to `255` (`0xFF`) in all operating configurations, and `0` in `@/home/cet/Documents/Modeling/soundcore/initial.rs`.
