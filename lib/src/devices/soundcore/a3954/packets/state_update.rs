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
        a3954::{
            state::A3954State,
            structures::{A3954AmbientState, A3954SpatialState},
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
                AdaptiveLeakageCompensation, AdaptiveMode, DualBattery, DualFirmwareVersion,
                LimitHighVolume, SerialNumber, TwsStatus, WearingDetection,
            },
        },
    },
};

// Layout of the 165-byte A3954 state body (zero-indexed):
// 0..2   TwsStatus
// 2..6   DualBattery
// 6..16  DualFirmwareVersion (left + right)
// 16..32 SerialNumber
// 32..37 Charging case firmware version (ASCII "MM.mm")
// 37..44 Opaque (7 bytes)
// 44     Default preset selector
// 45     Opaque
// 46..54 8-band equalizer gains
// 54..125 Opaque (71 bytes; HearID/buttons/etc.)
// 125..129 A3954AmbientState (mode, intensity, airplane_adaptive, wind_noise_reduction)
// 129..145 Opaque (16 bytes; includes init flag at 129)
// 145..148 LimitHighVolume (enabled, db_limit, refresh_rate)
// 148..151 A3954SpatialState (enabled, eq_or_head_tracking, submode)
// 151     Opaque
// 152     AdaptiveMode flag
// 153     AdaptiveLeakageCompensation flag
// 154..159 Opaque (5 bytes)
// 159     WearingDetection flag
// 160..163 Opaque (3 bytes)
// 163..165 Trailer (0x11, 0x11)

const CASE_FIRMWARE_LEN: usize = 5;
const POST_SERIAL_OPAQUE_LEN: usize = 7;
const POST_PRESET_OPAQUE_LEN: usize = 1;
const EQUALIZER_BAND_COUNT: usize = 8;
const POST_EQUALIZER_OPAQUE_LEN: usize = 71;
const POST_AMBIENT_OPAQUE_LEN: usize = 16;
const POST_SPATIAL_OPAQUE_LEN: usize = 1;
const POST_LEAKAGE_OPAQUE_LEN: usize = 5;
const POST_WEARING_OPAQUE_LEN: usize = 3;
const TRAILER_LEN: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct A3954StateUpdatePacket {
    pub tws_status: TwsStatus,
    pub battery: DualBattery,
    pub dual_firmware_version: DualFirmwareVersion,
    pub serial_number: SerialNumber,
    pub charging_case_firmware: [u8; CASE_FIRMWARE_LEN],
    pub default_preset: u8,
    pub equalizer_bands: [u8; EQUALIZER_BAND_COUNT],
    pub ambient: A3954AmbientState,
    pub limit_high_volume: LimitHighVolume,
    pub spatial: A3954SpatialState,
    pub adaptive_mode: AdaptiveMode,
    pub adaptive_leakage_compensation: AdaptiveLeakageCompensation,
    pub wearing_detection: WearingDetection,
    opaque_post_serial: [u8; POST_SERIAL_OPAQUE_LEN],
    opaque_post_preset: [u8; POST_PRESET_OPAQUE_LEN],
    opaque_post_equalizer: [u8; POST_EQUALIZER_OPAQUE_LEN],
    opaque_post_ambient: [u8; POST_AMBIENT_OPAQUE_LEN],
    opaque_post_spatial: [u8; POST_SPATIAL_OPAQUE_LEN],
    opaque_post_leakage: [u8; POST_LEAKAGE_OPAQUE_LEN],
    opaque_post_wearing: [u8; POST_WEARING_OPAQUE_LEN],
    trailer: [u8; TRAILER_LEN],
}

impl Default for A3954StateUpdatePacket {
    fn default() -> Self {
        Self {
            tws_status: Default::default(),
            battery: Default::default(),
            dual_firmware_version: Default::default(),
            serial_number: Default::default(),
            charging_case_firmware: *b"00.00",
            default_preset: 0,
            equalizer_bands: [120; EQUALIZER_BAND_COUNT],
            ambient: Default::default(),
            limit_high_volume: Default::default(),
            spatial: Default::default(),
            adaptive_mode: Default::default(),
            adaptive_leakage_compensation: Default::default(),
            wearing_detection: Default::default(),
            opaque_post_serial: [0; POST_SERIAL_OPAQUE_LEN],
            opaque_post_preset: [0; POST_PRESET_OPAQUE_LEN],
            opaque_post_equalizer: [0; POST_EQUALIZER_OPAQUE_LEN],
            opaque_post_ambient: [0; POST_AMBIENT_OPAQUE_LEN],
            opaque_post_spatial: [0; POST_SPATIAL_OPAQUE_LEN],
            opaque_post_leakage: [0; POST_LEAKAGE_OPAQUE_LEN],
            opaque_post_wearing: [0; POST_WEARING_OPAQUE_LEN],
            trailer: [0x11, 0x11],
        }
    }
}

fn take_array<'a, const N: usize, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], [u8; N], E> {
    map(take(N), |bytes: &[u8]| {
        bytes.try_into().expect("take returns exactly N bytes")
    })
    .parse_complete(input)
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
                        take_array::<CASE_FIRMWARE_LEN, E>,
                        take_array::<POST_SERIAL_OPAQUE_LEN, E>,
                        map(nom::number::complete::le_u8, |value| value),
                        take_array::<POST_PRESET_OPAQUE_LEN, E>,
                        take_array::<EQUALIZER_BAND_COUNT, E>,
                        take_array::<POST_EQUALIZER_OPAQUE_LEN, E>,
                    ),
                    (
                        A3954AmbientState::take,
                        take_array::<POST_AMBIENT_OPAQUE_LEN, E>,
                        LimitHighVolume::take,
                        A3954SpatialState::take,
                        take_array::<POST_SPATIAL_OPAQUE_LEN, E>,
                        map(take_bool, AdaptiveMode),
                        map(take_bool, AdaptiveLeakageCompensation),
                        take_array::<POST_LEAKAGE_OPAQUE_LEN, E>,
                        map(take_bool, WearingDetection),
                        take_array::<POST_WEARING_OPAQUE_LEN, E>,
                        take_array::<TRAILER_LEN, E>,
                    ),
                ),
                |(
                    (
                        tws_status,
                        battery,
                        dual_firmware_version,
                        serial_number,
                        charging_case_firmware,
                        opaque_post_serial,
                        default_preset,
                        opaque_post_preset,
                        equalizer_bands,
                        opaque_post_equalizer,
                    ),
                    (
                        ambient,
                        opaque_post_ambient,
                        limit_high_volume,
                        spatial,
                        opaque_post_spatial,
                        adaptive_mode,
                        adaptive_leakage_compensation,
                        opaque_post_leakage,
                        wearing_detection,
                        opaque_post_wearing,
                        trailer,
                    ),
                )| Self {
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
                    opaque_post_serial,
                    opaque_post_preset,
                    opaque_post_equalizer,
                    opaque_post_ambient,
                    opaque_post_spatial,
                    opaque_post_leakage,
                    opaque_post_wearing,
                    trailer,
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
            .chain(self.charging_case_firmware)
            .chain(self.opaque_post_serial)
            .chain([self.default_preset])
            .chain(self.opaque_post_preset)
            .chain(self.equalizer_bands)
            .chain(self.opaque_post_equalizer)
            .chain(self.ambient.bytes())
            .chain(self.opaque_post_ambient)
            .chain(self.limit_high_volume.bytes())
            .chain(self.spatial.bytes())
            .chain(self.opaque_post_spatial)
            .chain([self.adaptive_mode.0 as u8])
            .chain([self.adaptive_leakage_compensation.0 as u8])
            .chain(self.opaque_post_leakage)
            .chain([self.wearing_detection.0 as u8])
            .chain(self.opaque_post_wearing)
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

    use crate::devices::soundcore::{
        a3954::structures::A3954AmbientMode,
        common::packet::inbound::TryToPacket,
    };

    use super::*;

    #[test]
    fn serialize_and_deserialize_default() {
        let packet = A3954StateUpdatePacket::default();
        let bytes = packet.to_packet().bytes_with_checksum();
        let (_, parsed) =
            packet::Inbound::take_with_checksum::<VerboseError<_>>(&bytes).unwrap();
        let parsed: A3954StateUpdatePacket = parsed.try_to_packet().unwrap();
        assert_eq!(packet, parsed);
    }

    #[test]
    fn default_body_length_is_165() {
        assert_eq!(A3954StateUpdatePacket::default().body().len(), 165);
    }

    fn parse_body(body: &[u8]) -> A3954StateUpdatePacket {
        let (_, packet) =
            A3954StateUpdatePacket::take::<VerboseError<_>>(body).expect("parse");
        packet
    }

    fn initial_body() -> Vec<u8> {
        vec![
            1, 1, 98, 100, 0, 0, 48, 52, 46, 50, 57, 48, 52, 46, 50, 57, 51, 57, 53, 52, 51, 56,
            55, 52, 52, 70, 56, 65, 57, 68, 70, 52, 48, 50, 46, 53, 56, 9, 244, 157, 138, 64, 240,
            11, 1, 0, 160, 130, 140, 140, 160, 160, 160, 140, 120, 60, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 0, 1, 180, 180, 180, 180, 180, 180, 180, 180, 180, 60, 180,
            180, 180, 180, 180, 180, 180, 180, 180, 60, 0, 0, 0, 0, 0, 180, 119, 60, 118, 180,
            180, 120, 179, 180, 60, 180, 119, 60, 118, 180, 180, 120, 179, 180, 60, 18, 0, 10,
            102, 102, 50, 51, 255, 255, 68, 68, 51, 2, 6, 0, 0, 0, 1, 255, 0, 0, 0, 0, 98, 1, 49,
            1, 1, 0, 1, 1, 2, 1, 90, 0, 0, 1, 2, 0, 0, 1, 49, 1, 0, 1, 0, 0, 255, 0, 0, 17, 17,
        ]
    }

    #[test]
    fn parses_initial_capture() {
        let body = initial_body();
        assert_eq!(body.len(), 165);
        let packet = parse_body(&body);
        assert_eq!(packet.serial_number.to_string(), "395438744F8A9DF4");
        assert_eq!(&packet.charging_case_firmware, b"02.58");
        assert_eq!(packet.default_preset, 1);
        assert_eq!(
            packet.equalizer_bands,
            [160, 130, 140, 140, 160, 160, 160, 140]
        );
        assert_eq!(packet.ambient.mode, A3954AmbientMode::Normal);
        assert_eq!(packet.ambient.intensity, 6);
        assert!(!packet.ambient.airplane_adaptive);
        assert!(!packet.ambient.wind_noise_reduction);
        assert!(packet.limit_high_volume.enabled);
        assert_eq!(packet.limit_high_volume.db_limit, 90);
        assert!(!packet.spatial.enabled);
        assert_eq!(packet.spatial.eq_or_head_tracking, 1);
        assert_eq!(packet.spatial.submode, 2);
        assert!(!packet.adaptive_mode.0);
        // body[153] = 1 in initial capture
        assert!(packet.adaptive_leakage_compensation.0);
        assert!(!packet.wearing_detection.0);
        assert_eq!(packet.trailer, [0x11, 0x11]);
    }

    #[test]
    fn parses_anc_capture() {
        // analysis/anc/anc_8.rs: ANC level 8 stored as 9 (level + 1)
        let body = [
            1u8, 1, 96, 100, 0, 0, 48, 52, 46, 50, 57, 48, 52, 46, 50, 57, 51, 57, 53, 52, 51, 56,
            55, 52, 52, 70, 56, 65, 57, 68, 70, 52, 48, 50, 46, 53, 56, 9, 244, 157, 138, 64, 240,
            11, 1, 0, 160, 130, 140, 140, 160, 160, 160, 140, 120, 60, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 0, 1, 180, 180, 180, 180, 180, 180, 180, 180, 180, 60, 180,
            180, 180, 180, 180, 180, 180, 180, 180, 60, 0, 0, 0, 0, 0, 180, 119, 60, 118, 180,
            180, 120, 179, 180, 60, 180, 119, 60, 118, 180, 180, 120, 179, 180, 60, 18, 0, 10,
            102, 102, 50, 51, 255, 255, 68, 68, 51, 0, 9, 0, 0, 255, 1, 255, 0, 0, 0, 0, 98, 1,
            49, 1, 1, 0, 1, 1, 2, 1, 90, 0, 0, 1, 2, 0, 1, 1, 49, 1, 0, 1, 0, 0, 255, 0, 0, 17,
            17,
        ];
        let packet = parse_body(&body);
        assert_eq!(packet.ambient.mode, A3954AmbientMode::NoiseCanceling);
        assert_eq!(packet.ambient.intensity, 9);
        assert!(packet.adaptive_mode.0);
    }

    #[test]
    fn parses_wind_reduction_capture() {
        let body = [
            1u8, 1, 95, 100, 0, 0, 48, 52, 46, 50, 57, 48, 52, 46, 50, 57, 51, 57, 53, 52, 51, 56,
            55, 52, 52, 70, 56, 65, 57, 68, 70, 52, 48, 50, 46, 53, 56, 9, 244, 157, 138, 64, 240,
            11, 1, 0, 160, 130, 140, 140, 160, 160, 160, 140, 120, 60, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 0, 1, 180, 180, 180, 180, 180, 180, 180, 180, 180, 60, 180,
            180, 180, 180, 180, 180, 180, 180, 180, 60, 0, 0, 0, 0, 0, 180, 119, 60, 118, 180,
            180, 120, 179, 180, 60, 180, 119, 60, 118, 180, 180, 120, 179, 180, 60, 18, 0, 10,
            102, 102, 50, 51, 255, 255, 68, 68, 51, 2, 6, 0, 1, 255, 1, 255, 0, 0, 0, 0, 98, 1,
            49, 1, 1, 0, 1, 1, 2, 1, 90, 0, 0, 2, 0, 0, 0, 1, 49, 1, 0, 1, 0, 0, 255, 0, 0, 17,
            17,
        ];
        let packet = parse_body(&body);
        assert!(packet.ambient.wind_noise_reduction);
    }

    #[test]
    fn parses_wearing_detection_capture() {
        let body = [
            1u8, 1, 98, 100, 0, 0, 48, 52, 46, 50, 57, 48, 52, 46, 50, 57, 51, 57, 53, 52, 51, 56,
            55, 52, 52, 70, 56, 65, 57, 68, 70, 52, 48, 50, 46, 53, 56, 9, 244, 157, 138, 64, 240,
            11, 4, 0, 150, 150, 100, 100, 120, 140, 150, 160, 120, 60, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 0, 1, 180, 180, 180, 180, 180, 180, 180, 180, 180, 60, 180,
            180, 180, 180, 180, 180, 180, 180, 180, 60, 0, 0, 0, 0, 0, 180, 119, 60, 118, 180,
            180, 120, 179, 180, 60, 180, 119, 60, 118, 180, 180, 120, 179, 180, 60, 18, 0, 10,
            102, 102, 50, 51, 255, 255, 68, 68, 51, 2, 6, 0, 0, 255, 1, 255, 0, 0, 0, 0, 98, 1,
            49, 1, 1, 0, 1, 1, 2, 1, 90, 0, 0, 2, 0, 0, 0, 1, 49, 1, 0, 1, 0, 1, 255, 0, 0, 17,
            17,
        ];
        let packet = parse_body(&body);
        assert_eq!(packet.default_preset, 4);
        assert_eq!(
            packet.equalizer_bands,
            [150, 150, 100, 100, 120, 140, 150, 160]
        );
        assert!(packet.wearing_detection.0);
    }
}
