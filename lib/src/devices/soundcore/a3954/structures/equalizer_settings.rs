#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct EqualizerSettings {
    pub preset_selector: u8,
    pub gains: [u8; 8],
    pub hear_id_offset: u8,
    pub preference_test_active: bool,
    pub spatial_audio: bool,
    pub equalizer_type: u8,
    pub preference_test_status: u8,
}

impl EqualizerSettings {
    pub fn new(
        preset_selector: u8,
        gains: [u8; 8],
        hear_id_offset: u8,
        preference_test_active: bool,
        spatial_audio: bool,
        equalizer_type: u8,
        preference_test_status: u8,
    ) -> Self {
        Self {
            preset_selector,
            gains,
            hear_id_offset,
            preference_test_active,
            spatial_audio,
            equalizer_type,
            preference_test_status,
        }
    }
}
