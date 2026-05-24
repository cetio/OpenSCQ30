use openscq30_lib_macros::Has;

use crate::devices::soundcore::{
    a3954::structures::{AmbientSoundControl, EqualizerSettings},
    common::structures::{
        DualBattery, DualFirmwareVersion, FirmwareVersion, LimitHighVolume, SerialNumber,
        SoundLeakCompensation, TwsStatus, WearingDetection,
    },
};

use super::packets::A3954StateUpdatePacket;

#[derive(Debug, Clone, PartialEq, Eq, Has)]
pub struct A3954State {
    tws_status: TwsStatus,
    battery: DualBattery,
    dual_firmware_version: DualFirmwareVersion,
    serial_number: SerialNumber,
    case_firmware_version: FirmwareVersion,
    equalizer_settings: EqualizerSettings,
    ambient_sound_control: AmbientSoundControl,
    adaptive_mode: bool,
    limit_high_volume: LimitHighVolume,
    sound_leak_compensation: SoundLeakCompensation,
    wearing_detection: WearingDetection,
}

impl From<A3954StateUpdatePacket> for A3954State {
    fn from(packet: A3954StateUpdatePacket) -> Self {
        let A3954StateUpdatePacket {
            tws_status,
            battery,
            dual_firmware_version,
            serial_number,
            case_firmware_version,
            equalizer_settings,
            ambient_sound_control,
            adaptive_mode,
            limit_high_volume,
            sound_leak_compensation,
            wearing_detection,
            ..
        } = packet;

        Self {
            tws_status,
            battery,
            dual_firmware_version,
            serial_number,
            case_firmware_version,
            equalizer_settings,
            ambient_sound_control,
            adaptive_mode,
            limit_high_volume,
            sound_leak_compensation,
            wearing_detection,
        }
    }
}
