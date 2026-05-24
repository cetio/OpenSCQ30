use async_trait::async_trait;
use nom::{
    IResult, Parser,
    bytes::complete::take,
    combinator::{all_consuming, map},
    error::{ContextError, ParseError, context},
    number::complete::le_u8,
};
use tokio::sync::watch;

use crate::{
    api::device,
    devices::soundcore::{
        a3954::{
            state::A3954State,
            structures::{AmbientSoundControl, EqualizerSettings},
        },
        common::{
            modules::ModuleCollection,
            packet::{
                self,
                inbound::{FromPacketBody, TryToPacket},
                outbound::ToPacket,
                parsing::take_bool,
            },
            packet_manager::PacketHandler,
            structures::{
                DualBattery, DualFirmwareVersion, FirmwareVersion, LimitHighVolume, SerialNumber,
                SoundLeakCompensation, TwsStatus, WearingDetection,
            },
        },
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct A3954StateUpdatePacket {
    pub tws_status: TwsStatus,
    pub battery: DualBattery,
    pub dual_firmware_version: DualFirmwareVersion,
    pub serial_number: SerialNumber,
    pub case_firmware_version: FirmwareVersion,
    pub equalizer_settings: EqualizerSettings,
    pub ambient_sound_control: AmbientSoundControl,
    pub adaptive_mode: bool,
    pub limit_high_volume: LimitHighVolume,
    pub sound_leak_compensation: SoundLeakCompensation,
    pub wearing_detection: WearingDetection,
    unknown1: [u8; 7],
    unknown2: [u8; 1],
    unknown3: [u8; 1],
    unknown4: [u8; 11],
    unknown5: [u8; 57],
    unknown6: [u8; 15],
    unknown7: [u8; 1],
    unknown8: [u8; 5],
    unknown9: [u8; 3],
    trailer: [u8; 2],
}

impl Default for A3954StateUpdatePacket {
    fn default() -> Self {
        Self {
            tws_status: Default::default(),
            battery: Default::default(),
            dual_firmware_version: Default::default(),
            serial_number: Default::default(),
            case_firmware_version: Default::default(),
            equalizer_settings: Default::default(),
            ambient_sound_control: AmbientSoundControl::default(),
            adaptive_mode: false,
            limit_high_volume: Default::default(),
            sound_leak_compensation: Default::default(),
            wearing_detection: Default::default(),
            unknown1: [0u8; 7],
            unknown2: [0u8; 1],
            unknown3: [0u8; 1],
            unknown4: [0u8; 11],
            unknown5: [0u8; 57],
            unknown6: [0u8; 15],
            unknown7: [0u8; 1],
            unknown8: [0u8; 5],
            unknown9: [0u8; 3],
            trailer: [0x11, 0x11],
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
                    (
                        TwsStatus::take,
                        DualBattery::take,
                        DualFirmwareVersion::take,
                        SerialNumber::take,
                        FirmwareVersion::take,
                        take(7usize),
                        le_u8,
                        take(1usize),
                        take(8usize),
                        take(1usize),
                        le_u8,
                        take(11usize),
                        take_bool,
                        take(57usize),
                        AmbientSoundControl::take,
                        take(15usize),
                    ),
                    (
                        LimitHighVolume::take,
                        take_bool,
                        le_u8,
                        le_u8,
                        take(1usize),
                        take_bool,
                        SoundLeakCompensation::take,
                        take(5usize),
                        WearingDetection::take,
                        take(3usize),
                        take(2usize),
                    ),
                ),
                |(
                    (
                        tws_status,
                        battery,
                        dual_firmware_version,
                        serial_number,
                        case_firmware_version,
                        unknown1,
                        preset_selector,
                        unknown2,
                        eq_gains,
                        unknown3,
                        hear_id_offset,
                        unknown4,
                        preference_test_active,
                        unknown5,
                        ambient_sound_control,
                        unknown6,
                    ),
                    (
                        limit_high_volume,
                        spatial_audio,
                        equalizer_type,
                        preference_test_status,
                        unknown7,
                        adaptive_mode,
                        sound_leak_compensation,
                        unknown8,
                        wearing_detection,
                        unknown9,
                        trailer,
                    ),
                )| {
                    let unknown1: [u8; 7] = unknown1
                        .try_into()
                        .expect("take returns exactly the requested length");
                    let unknown2: [u8; 1] = unknown2
                        .try_into()
                        .expect("take returns exactly the requested length");
                    let unknown3: [u8; 1] = unknown3
                        .try_into()
                        .expect("take returns exactly the requested length");
                    let eq_gains: [u8; 8] = eq_gains
                        .try_into()
                        .expect("take returns exactly the requested length");
                    let unknown4: [u8; 11] = unknown4
                        .try_into()
                        .expect("take returns exactly the requested length");
                    let unknown5: [u8; 57] = unknown5
                        .try_into()
                        .expect("take returns exactly the requested length");
                    let unknown6: [u8; 15] = unknown6
                        .try_into()
                        .expect("take returns exactly the requested length");
                    let unknown7: [u8; 1] = unknown7
                        .try_into()
                        .expect("take returns exactly the requested length");
                    let unknown8: [u8; 5] = unknown8
                        .try_into()
                        .expect("take returns exactly the requested length");
                    let unknown9: [u8; 3] = unknown9
                        .try_into()
                        .expect("take returns exactly the requested length");
                    let trailer: [u8; 2] = trailer
                        .try_into()
                        .expect("take returns exactly the requested length");
                    Self {
                        tws_status,
                        battery,
                        dual_firmware_version,
                        serial_number,
                        case_firmware_version,
                        equalizer_settings: EqualizerSettings::new(
                            preset_selector,
                            eq_gains,
                            hear_id_offset,
                            preference_test_active,
                            spatial_audio,
                            equalizer_type,
                            preference_test_status,
                        ),
                        ambient_sound_control,
                        adaptive_mode,
                        limit_high_volume,
                        sound_leak_compensation,
                        wearing_detection,
                        unknown1,
                        unknown2,
                        unknown3,
                        unknown4,
                        unknown5,
                        unknown6,
                        unknown7,
                        unknown8,
                        unknown9,
                        trailer,
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
            .chain(self.case_firmware_version.bytes())
            .chain(self.unknown1)
            .chain([self.equalizer_settings.preset_selector])
            .chain(self.unknown2)
            .chain(self.equalizer_settings.gains)
            .chain(self.unknown3)
            .chain([self.equalizer_settings.hear_id_offset])
            .chain(self.unknown4)
            .chain([self.equalizer_settings.preference_test_active.into()])
            .chain(self.unknown5)
            .chain(self.ambient_sound_control.bytes())
            .chain(self.unknown6)
            .chain(self.limit_high_volume.bytes())
            .chain([self.equalizer_settings.spatial_audio.into()])
            .chain([self.equalizer_settings.equalizer_type])
            .chain([self.equalizer_settings.preference_test_status])
            .chain(self.unknown7)
            .chain([self.adaptive_mode.into()])
            .chain([self.sound_leak_compensation.0.into()])
            .chain(self.unknown8)
            .chain([self.wearing_detection.0.into()])
            .chain(self.unknown9)
            .chain(self.trailer)
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

    #[test]
    fn parse_initial_packet() {
        let body = [
            1, 1, 98, 100, 0, 0, 48, 52, 46, 50, 57, 48, 52, 46, 50, 57, 51, 57, 53, 52, 51, 56,
            55, 52, 52, 70, 56, 65, 57, 68, 70, 52, 48, 50, 46, 53, 56, 9, 244, 157, 138, 64, 240,
            11, 1, 0, 160, 130, 140, 140, 160, 160, 160, 140, 120, 60, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 0, 1, 180, 180, 180, 180, 180, 180, 180, 180, 180, 60, 180, 180,
            180, 180, 180, 180, 180, 180, 180, 60, 0, 0, 0, 0, 0, 180, 119, 60, 118, 180, 180,
            120, 179, 180, 60, 180, 119, 60, 118, 180, 180, 120, 179, 180, 60, 18, 0, 10, 102,
            102, 50, 51, 255, 255, 68, 68, 51, 2, 6, 0, 0, 0, 0, 0, 1, 255, 0, 0, 0, 0, 98, 1, 49,
            1, 1, 0, 1, 1, 2, 1, 90, 0, 0, 1, 2, 0, 0, 0, 1, 49, 1, 0, 1, 0, 0, 255, 0, 0, 17,
            17,
        ];
        let (_, packet) = A3954StateUpdatePacket::take::<VerboseError<_>>(&body).unwrap();
        assert_eq!(packet.case_firmware_version.to_string(), "02.58");
        assert_eq!(packet.equalizer_settings.preset_selector, 1);
        assert_eq!(
            packet.equalizer_settings.gains,
            [160, 130, 140, 140, 160, 160, 160, 140]
        );
        assert_eq!(packet.equalizer_settings.hear_id_offset, 60);
        assert!(!packet.equalizer_settings.preference_test_active);
        assert_eq!(packet.ambient_sound_control.ambient_mode as u8, 2);
        assert_eq!(packet.ambient_sound_control.intensity, 6);
        assert!(!packet.ambient_sound_control.airplane_adaptive);
        assert!(!packet.ambient_sound_control.wind_noise_reduction);
        assert_eq!(packet.ambient_sound_control.initialization_state, 0);
        assert!(!packet.adaptive_mode);
        assert_eq!(packet.limit_high_volume.enabled as u8, 1);
        assert_eq!(packet.limit_high_volume.db_limit, 90);
        assert_eq!(packet.limit_high_volume.refresh_rate as u8, 0);
        assert!(!packet.equalizer_settings.spatial_audio);
        assert_eq!(packet.equalizer_settings.equalizer_type, 1);
        assert_eq!(packet.equalizer_settings.preference_test_status, 2);
        assert!(packet.sound_leak_compensation.0);
        assert!(!packet.wearing_detection.0);
    }

    #[test]
    fn parse_anc_8_packet() {
        let body = [
            1, 1, 96, 100, 0, 0, 48, 52, 46, 50, 57, 48, 52, 46, 50, 57, 51, 57, 53, 52, 51, 56,
            55, 52, 52, 70, 56, 65, 57, 68, 70, 52, 48, 50, 46, 53, 56, 9, 244, 157, 138, 64, 240,
            11, 1, 0, 160, 130, 140, 140, 160, 160, 160, 140, 120, 60, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 0, 1, 180, 180, 180, 180, 180, 180, 180, 180, 180, 60, 180, 180,
            180, 180, 180, 180, 180, 180, 180, 60, 0, 0, 0, 0, 0, 180, 119, 60, 118, 180, 180,
            120, 179, 180, 60, 180, 119, 60, 118, 180, 180, 120, 179, 180, 60, 0, 9, 0, 0, 0, 255,
            255, 68, 68, 51, 2, 6, 0, 0, 0, 0, 0, 1, 255, 0, 0, 0, 0, 98, 1, 49, 1, 1, 0, 1, 1, 2,
            1, 90, 0, 0, 1, 2, 0, 0, 0, 1, 49, 1, 0, 1, 0, 0, 255, 0, 0, 17, 17,
        ];
        let (_, packet) = A3954StateUpdatePacket::take::<VerboseError<_>>(&body).unwrap();
        assert_eq!(packet.ambient_sound_control.ambient_mode as u8, 0);
        assert_eq!(packet.ambient_sound_control.intensity, 9);
    }
}
