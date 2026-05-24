use nom::{
    IResult, Parser,
    combinator::map,
    error::{ContextError, ParseError, context},
    number::complete::le_u8,
};
use openscq30_i18n_macros::Translate;
use strum::{Display, EnumIter, EnumString, FromRepr, IntoStaticStr};

use crate::devices::soundcore::common::packet::parsing::take_bool;

#[repr(u8)]
#[derive(
    FromRepr,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Display,
    Default,
    IntoStaticStr,
    EnumIter,
    EnumString,
    Translate,
)]
pub enum AmbientMode {
    #[default]
    NoiseCanceling = 0,
    Transparency = 1,
    Normal = 2,
    Airplane = 3,
}

impl AmbientMode {
    pub fn id(&self) -> u8 {
        *self as u8
    }

    pub fn from_id(id: u8) -> Option<Self> {
        Self::from_repr(id)
    }

    pub fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "a3954 ambient mode",
            map(le_u8, |id| Self::from_id(id).unwrap_or_default()),
        )
        .parse_complete(input)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AmbientSoundControl {
    pub ambient_mode: AmbientMode,
    pub intensity: u8,
    pub airplane_adaptive: bool,
    pub wind_noise_reduction: bool,
    pub initialization_state: u8,
}

impl AmbientSoundControl {
    pub fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "a3954 ambient sound control",
            map(
                (
                    AmbientMode::take,
                    le_u8,
                    take_bool,
                    take_bool,
                    le_u8,
                ),
                |(ambient_mode, intensity, airplane_adaptive, wind_noise_reduction, initialization_state)| {
                    Self {
                        ambient_mode,
                        intensity,
                        airplane_adaptive,
                        wind_noise_reduction,
                        initialization_state,
                    }
                },
            ),
        )
        .parse_complete(input)
    }

    pub fn bytes(&self) -> [u8; 5] {
        [
            self.ambient_mode.id(),
            self.intensity,
            self.airplane_adaptive.into(),
            self.wind_noise_reduction.into(),
            self.initialization_state,
        ]
    }
}
