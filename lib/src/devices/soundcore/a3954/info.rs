use async_trait::async_trait;
use openscq30_i18n::Translate;
use strum::{EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

use crate::{
    api::settings::{CategoryId, Setting, SettingId, Value},
    devices::soundcore::common::{
        modules::ModuleCollection,
        settings_manager::{SettingHandler, SettingHandlerError, SettingHandlerResult},
    },
    i18n::fl,
    macros::enum_subset,
};

use super::state::{A3954CaseFirmware, A3954State};

enum_subset!(
    SettingId,
    #[derive(EnumString, EnumIter, IntoStaticStr)]
    enum A3954InfoSetting {
        AmbientSoundMode,
        WindNoiseReduction,
        AirplaneAdaptive,
        AdaptiveMode,
        AdaptiveLeakageCompensation,
        SpatialAudio,
        SpatialAudioProfile,
        HeadTracking,
        HearIdActive,
        ChargingCaseFirmwareVersion,
        PresetEqualizerProfile,
        VolumeAdjustments,
    }
);

pub struct A3954InfoSettingHandler;

#[async_trait]
impl SettingHandler<A3954State> for A3954InfoSettingHandler {
    fn settings(&self) -> Vec<SettingId> {
        A3954InfoSetting::iter().map(Into::into).collect()
    }

    fn get(&self, state: &A3954State, setting_id: &SettingId) -> Option<Setting> {
        let setting: A3954InfoSetting = (*setting_id).try_into().ok()?;
        Some(match setting {
            A3954InfoSetting::AmbientSoundMode => information(
                <&'static str>::from(state.ambient.mode).to_owned(),
                state.ambient.mode.translate(),
            ),
            A3954InfoSetting::WindNoiseReduction => bool_information(state.ambient.wind_noise_reduction),
            A3954InfoSetting::AirplaneAdaptive => bool_information(state.ambient.airplane_adaptive),
            A3954InfoSetting::AdaptiveMode => bool_information(state.adaptive_mode.0),
            A3954InfoSetting::AdaptiveLeakageCompensation => {
                bool_information(state.adaptive_leakage_compensation.0)
            }
            A3954InfoSetting::SpatialAudio => bool_information(state.spatial.enabled),
            A3954InfoSetting::SpatialAudioProfile => match state.spatial.spatial_profile() {
                Some(profile) => information(
                    <&'static str>::from(profile).to_owned(),
                    profile.translate(),
                ),
                None => information(String::new(), fl!("none")),
            },
            A3954InfoSetting::HeadTracking => match state.spatial.head_tracking() {
                Some(true) => information("true".to_owned(), fl!("head-tracking-active")),
                Some(false) => information("false".to_owned(), fl!("fixed")),
                None => information(String::new(), fl!("none")),
            },
            A3954InfoSetting::HearIdActive => match state.spatial.hear_id_active() {
                Some(value) => bool_information(value),
                None => information(String::new(), fl!("none")),
            },
            A3954InfoSetting::ChargingCaseFirmwareVersion => {
                let text = case_firmware_to_string(&state.charging_case_firmware);
                information(text.clone(), text)
            }
            A3954InfoSetting::PresetEqualizerProfile => {
                let value = state.default_preset.0.to_string();
                information(value.clone(), value)
            }
            A3954InfoSetting::VolumeAdjustments => {
                let bands = &state.equalizer_bands.0;
                let text = bands
                    .iter()
                    .map(u8::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                information(text.clone(), text)
            }
        })
    }

    async fn set(
        &self,
        _state: &mut A3954State,
        _setting_id: &SettingId,
        _value: Value,
    ) -> SettingHandlerResult<()> {
        Err(SettingHandlerError::ReadOnly)
    }
}

fn information(value: String, translated_value: String) -> Setting {
    Setting::Information {
        value,
        translated_value,
    }
}

fn bool_information(value: bool) -> Setting {
    Setting::Information {
        value: value.to_string(),
        translated_value: if value {
            fl!("enabled")
        } else {
            fl!("disabled")
        },
    }
}

fn case_firmware_to_string(firmware: &A3954CaseFirmware) -> String {
    firmware.as_str().to_owned()
}

impl ModuleCollection<A3954State> {
    pub fn add_a3954_info(&mut self) {
        self.setting_manager
            .add_handler(CategoryId::DeviceInformation, A3954InfoSettingHandler);
    }
}

