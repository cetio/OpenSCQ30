use std::collections::HashMap;

use crate::devices::soundcore::{
    a3954::{packets::A3954StateUpdatePacket, state::A3954State},
    common::{
        device::fetch_state_from_state_update_packet,
        macros::soundcore_device,
        packet::outbound::{RequestState, ToPacket},
    },
};

mod packets;
mod state;
mod structures;

soundcore_device!(
    A3954State,
    async |packet_io| {
        fetch_state_from_state_update_packet::<A3954State, A3954StateUpdatePacket>(packet_io).await
    },
    async |builder| {
        builder.module_collection().add_state_update();
        builder.limit_high_volume();
        builder.serial_number_and_dual_firmware_version();
        builder.tws_status();
        builder.dual_battery(5);
        builder.sound_leak_compensation();
        builder.wearing_detection();
    },
    {
        HashMap::from([(
            RequestState::COMMAND,
            A3954StateUpdatePacket::default().to_packet(),
        )])
    },
);
