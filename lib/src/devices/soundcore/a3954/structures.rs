use nom::{
    IResult, Parser,
    combinator::map,
    error::{ContextError, ParseError, context},
    number::complete::le_u8,
};
use openscq30_i18n::Translate;
use strum::{EnumIter, EnumString, FromRepr, IntoStaticStr};

use crate::{devices::soundcore::common::packet::parsing::take_bool, i18n::fl};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
    FromRepr,
    EnumIter,
    EnumString,
    IntoStaticStr,
)]
#[repr(u8)]
pub enum A3954AmbientMode {
    NoiseCanceling = 0,
    Transparency = 1,
    #[default]
    Normal = 2,
    Airplane = 3,
}

impl Translate for A3954AmbientMode {
    fn translate(&self) -> String {
        match self {
            Self::NoiseCanceling => fl!("noise-canceling"),
            Self::Transparency => fl!("transparency"),
            Self::Normal => fl!("normal"),
            Self::Airplane => fl!("airplane"),
        }
    }
}

impl A3954AmbientMode {
    pub fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "a3954 ambient mode",
            map(le_u8, |value| Self::from_repr(value).unwrap_or_default()),
        )
        .parse_complete(input)
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
    FromRepr,
    EnumIter,
    EnumString,
    IntoStaticStr,
)]
#[repr(u8)]
pub enum A3954SpatialProfile {
    #[default]
    Music = 0,
    Podcast = 1,
    Movie = 2,
    Gaming = 3,
}

impl Translate for A3954SpatialProfile {
    fn translate(&self) -> String {
        match self {
            Self::Music => fl!("spatial-music"),
            Self::Podcast => fl!("spatial-podcast"),
            Self::Movie => fl!("spatial-movie"),
            Self::Gaming => fl!("spatial-gaming"),
        }
    }
}

impl A3954SpatialProfile {
    pub fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "a3954 spatial profile",
            map(le_u8, |value| Self::from_repr(value).unwrap_or_default()),
        )
        .parse_complete(input)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct A3954AmbientState {
    pub mode: A3954AmbientMode,
    pub intensity: u8,
    pub airplane_adaptive: bool,
    pub wind_noise_reduction: bool,
}

impl A3954AmbientState {
    pub fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "a3954 ambient state",
            map(
                (
                    A3954AmbientMode::take,
                    le_u8,
                    take_bool,
                    take_bool,
                ),
                |(mode, intensity, airplane_adaptive, wind_noise_reduction)| Self {
                    mode,
                    intensity,
                    airplane_adaptive,
                    wind_noise_reduction,
                },
            ),
        )
        .parse_complete(input)
    }

    pub fn bytes(&self) -> [u8; 4] {
        [
            self.mode as u8,
            self.intensity,
            self.airplane_adaptive as u8,
            self.wind_noise_reduction as u8,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct A3954SpatialState {
    pub enabled: bool,
    /// When `enabled` is false, encodes EQ category (1 = HearID, 2 = Default Preset).
    /// When `enabled` is true, encodes head tracking mode (1 = fixed, 2 = head tracking).
    pub eq_or_head_tracking: u8,
    /// When `enabled` is false, encodes preference test profile (0 inactive, 2 active).
    /// When `enabled` is true, encodes the active spatial profile (0..3).
    pub submode: u8,
}

impl A3954SpatialState {
    pub fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "a3954 spatial state",
            map(
                (take_bool, le_u8, le_u8),
                |(enabled, eq_or_head_tracking, submode)| Self {
                    enabled,
                    eq_or_head_tracking,
                    submode,
                },
            ),
        )
        .parse_complete(input)
    }

    pub fn bytes(&self) -> [u8; 3] {
        [self.enabled as u8, self.eq_or_head_tracking, self.submode]
    }

    pub fn head_tracking(&self) -> Option<bool> {
        if self.enabled {
            Some(self.eq_or_head_tracking == 2)
        } else {
            None
        }
    }

    pub fn hear_id_active(&self) -> Option<bool> {
        if !self.enabled {
            Some(self.eq_or_head_tracking == 1)
        } else {
            None
        }
    }

    pub fn spatial_profile(&self) -> Option<A3954SpatialProfile> {
        if self.enabled {
            A3954SpatialProfile::from_repr(self.submode)
        } else {
            None
        }
    }
}
