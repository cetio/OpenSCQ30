use openscq30_lib_macros::Has;

use crate::devices::soundcore::common::structures::{
    DualBattery, DualFirmwareVersion, LimitHighVolume, SerialNumber, TwsStatus,
};

use super::packets::A3954StateUpdatePacket;

#[derive(Debug, Clone, PartialEq, Eq, Has)]
pub struct A3954State {
    tws_status: TwsStatus,
    battery: DualBattery,
    dual_firmware_version: DualFirmwareVersion,
    serial_number: SerialNumber,
    limit_high_volume: LimitHighVolume,
}

impl From<A3954StateUpdatePacket> for A3954State {
    fn from(packet: A3954StateUpdatePacket) -> Self {
        let A3954StateUpdatePacket {
            tws_status,
            battery,
            dual_firmware_version,
            serial_number,
            limit_high_volume,
            ..
        } = packet;

        Self {
            tws_status,
            battery,
            dual_firmware_version,
            serial_number,
            limit_high_volume,
        }
    }
}
