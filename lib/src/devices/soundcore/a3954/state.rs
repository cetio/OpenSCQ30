use openscq30_lib_macros::Has;

use crate::devices::soundcore::common::structures::{
    AdaptiveLeakageCompensation, AdaptiveMode, DualBattery, DualFirmwareVersion, LimitHighVolume,
    SerialNumber, TwsStatus, WearingDetection,
};

use super::{
    packets::A3954StateUpdatePacket,
    structures::{A3954AmbientState, A3954SpatialState},
};

#[derive(Debug, Clone, PartialEq, Eq, Has)]
pub struct A3954State {
    pub tws_status: TwsStatus,
    pub battery: DualBattery,
    pub dual_firmware_version: DualFirmwareVersion,
    pub serial_number: SerialNumber,
    pub charging_case_firmware: A3954CaseFirmware,
    pub default_preset: A3954DefaultPreset,
    pub equalizer_bands: A3954EqualizerBands,
    pub ambient: A3954AmbientState,
    pub limit_high_volume: LimitHighVolume,
    pub spatial: A3954SpatialState,
    pub adaptive_mode: AdaptiveMode,
    pub adaptive_leakage_compensation: AdaptiveLeakageCompensation,
    pub wearing_detection: WearingDetection,
}

/// Thin wrapper so the state can derive `Has` for the charging-case firmware bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct A3954CaseFirmware(pub [u8; 5]);

impl Default for A3954CaseFirmware {
    fn default() -> Self {
        Self(*b"00.00")
    }
}

impl A3954CaseFirmware {
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap_or("?????")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct A3954DefaultPreset(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct A3954EqualizerBands(pub [u8; 8]);

impl Default for A3954EqualizerBands {
    fn default() -> Self {
        Self([120; 8])
    }
}

impl From<A3954StateUpdatePacket> for A3954State {
    fn from(packet: A3954StateUpdatePacket) -> Self {
        let A3954StateUpdatePacket {
            tws_status,
            battery,
            dual_firmware_version,
            serial_number,
            charging_case_firmware,
            default_preset,
            equalizer_bands,
            ambient,
            limit_high_volume,
            spatial,
            adaptive_mode,
            adaptive_leakage_compensation,
            wearing_detection,
            ..
        } = packet;

        Self {
            tws_status,
            battery,
            dual_firmware_version,
            serial_number,
            charging_case_firmware: A3954CaseFirmware(charging_case_firmware),
            default_preset: A3954DefaultPreset(default_preset),
            equalizer_bands: A3954EqualizerBands(equalizer_bands),
            ambient,
            limit_high_volume,
            spatial,
            adaptive_mode,
            adaptive_leakage_compensation,
            wearing_detection,
        }
    }
}
