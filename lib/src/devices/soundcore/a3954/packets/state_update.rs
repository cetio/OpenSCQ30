use async_trait::async_trait;
use nom::{
    IResult, Parser,
    bytes::complete::take,
    combinator::{all_consuming, map},
    error::{ContextError, ParseError, context},
};
use tokio::sync::watch;

use crate::{
    api::device,
    devices::soundcore::{
        a3954::state::A3954State,
        common::{
            modules::ModuleCollection,
            packet::{
                self,
                inbound::{FromPacketBody, TryToPacket},
                outbound::ToPacket,
            },
            packet_manager::PacketHandler,
            structures::{
                DualBattery, DualFirmwareVersion, LimitHighVolume, SerialNumber, TwsStatus,
            },
        },
    },
};

// Bytes between SerialNumber (ends at body[32]) and LimitHighVolume (starts at body[145]).
// Includes charging case firmware, eq, anc, button config, ambient sound modes, etc.
// Their layout has not been reverse engineered yet, so they are kept opaque so the rest
// of the packet can still be parsed deterministically.
const UNKNOWN_PRE_LIMIT_HIGH_VOLUME_LEN: usize = 113;
// Bytes between LimitHighVolume (ends at body[148]) and end of packet (body[165]).
// Includes the 0x11, 0x11 frame boundary at body[163..165] and other unknowns.
const UNKNOWN_POST_LIMIT_HIGH_VOLUME_LEN: usize = 17;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct A3954StateUpdatePacket {
    pub tws_status: TwsStatus,
    pub battery: DualBattery,
    pub dual_firmware_version: DualFirmwareVersion,
    pub serial_number: SerialNumber,
    pub limit_high_volume: LimitHighVolume,
    unknown_pre: [u8; UNKNOWN_PRE_LIMIT_HIGH_VOLUME_LEN],
    unknown_post: [u8; UNKNOWN_POST_LIMIT_HIGH_VOLUME_LEN],
}

impl Default for A3954StateUpdatePacket {
    fn default() -> Self {
        let mut unknown_post = [0u8; UNKNOWN_POST_LIMIT_HIGH_VOLUME_LEN];
        // Frame boundary observed at body[163..165].
        unknown_post[UNKNOWN_POST_LIMIT_HIGH_VOLUME_LEN - 2] = 0x11;
        unknown_post[UNKNOWN_POST_LIMIT_HIGH_VOLUME_LEN - 1] = 0x11;
        Self {
            tws_status: Default::default(),
            battery: Default::default(),
            dual_firmware_version: Default::default(),
            serial_number: Default::default(),
            limit_high_volume: Default::default(),
            unknown_pre: [0u8; UNKNOWN_PRE_LIMIT_HIGH_VOLUME_LEN],
            unknown_post,
        }
    }
}

impl FromPacketBody for A3954StateUpdatePacket {
    type DirectionMarker = packet::InboundMarker;

    fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "a3954 state update packet",
            all_consuming(map(
                (
                    TwsStatus::take,
                    DualBattery::take,
                    DualFirmwareVersion::take,
                    SerialNumber::take,
                    take(UNKNOWN_PRE_LIMIT_HIGH_VOLUME_LEN),
                    LimitHighVolume::take,
                    take(UNKNOWN_POST_LIMIT_HIGH_VOLUME_LEN),
                ),
                |(
                    tws_status,
                    battery,
                    dual_firmware_version,
                    serial_number,
                    unknown_pre,
                    limit_high_volume,
                    unknown_post,
                ): (_, _, _, _, &[u8], _, &[u8])| {
                    let unknown_pre: [u8; UNKNOWN_PRE_LIMIT_HIGH_VOLUME_LEN] = unknown_pre
                        .try_into()
                        .expect("take returns exactly the requested length");
                    let unknown_post: [u8; UNKNOWN_POST_LIMIT_HIGH_VOLUME_LEN] = unknown_post
                        .try_into()
                        .expect("take returns exactly the requested length");
                    Self {
                        tws_status,
                        battery,
                        dual_firmware_version,
                        serial_number,
                        limit_high_volume,
                        unknown_pre,
                        unknown_post,
                    }
                },
            )),
        )
        .parse_complete(input)
    }
}

impl ToPacket for A3954StateUpdatePacket {
    type DirectionMarker = packet::InboundMarker;

    fn command(&self) -> packet::Command {
        packet::inbound::STATE_COMMAND
    }

    fn body(&self) -> Vec<u8> {
        self.tws_status
            .bytes()
            .into_iter()
            .chain(self.battery.bytes())
            .chain(self.dual_firmware_version.bytes())
            .chain(self.serial_number.bytes())
            .chain(self.unknown_pre)
            .chain(self.limit_high_volume.bytes())
            .chain(self.unknown_post)
            .collect()
    }
}

struct StateUpdatePacketHandler;

#[async_trait]
impl PacketHandler<A3954State> for StateUpdatePacketHandler {
    async fn handle_packet(
        &self,
        state: &watch::Sender<A3954State>,
        packet: &packet::Inbound,
    ) -> device::Result<()> {
        let packet: A3954StateUpdatePacket = packet.try_to_packet()?;
        state.send_modify(|state| *state = packet.into());
        Ok(())
    }
}

impl ModuleCollection<A3954State> {
    pub fn add_state_update(&mut self) {
        self.packet_handlers.set_handler(
            packet::inbound::STATE_COMMAND,
            Box::new(StateUpdatePacketHandler {}),
        );
    }
}

#[cfg(test)]
mod tests {
    use nom_language::error::VerboseError;

    use crate::devices::soundcore::common::packet::inbound::TryToPacket;

    use super::*;

    #[test]
    fn serialize_and_deserialize() {
        let bytes = A3954StateUpdatePacket::default()
            .to_packet()
            .bytes_with_checksum();
        let (_, packet) = packet::Inbound::take_with_checksum::<VerboseError<_>>(&bytes).unwrap();
        let _: A3954StateUpdatePacket = packet.try_to_packet().unwrap();
    }

    #[test]
    fn body_length_is_165() {
        let body = A3954StateUpdatePacket::default().body();
        assert_eq!(body.len(), 165);
    }
}
