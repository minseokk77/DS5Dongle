use serde::Deserialize;
use std::sync::OnceLock;

static SETTINGS: OnceLock<AppSettings> = OnceLock::new();

#[derive(Clone, Debug, Deserialize)]
pub struct AppSettings {
    pub usb: UsbSettings,
    pub reports: ReportSettings,
    pub config: ConfigSettings,
    pub firmware_update: FirmwareUpdateSettings,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UsbSettings {
    pub sony_vendor_id: u16,
    pub product_ids: Vec<u16>,
    pub config_only_vendor_id: u16,
    pub config_only_product_id: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ReportSettings {
    pub config_read: u8,
    pub command: u8,
    pub firmware_version: u8,
    pub rssi: u8,
    pub battery: u8,
    #[allow(dead_code)]
    pub capabilities: u8,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigSettings {
    pub version: u8,
    #[allow(dead_code)]
    pub body_length: usize,
    pub haptics_gain_min: f32,
    pub haptics_gain_max: f32,
    pub speaker_volume_percent_min: f32,
    pub speaker_volume_percent_max: f32,
    pub inactive_time_min: u8,
    pub inactive_time_max: u8,
    pub haptics_buffer_length_min: u8,
    pub haptics_buffer_length_max: u8,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FirmwareUpdateSettings {
    pub github_owner: String,
    pub github_repo: String,
    pub debug_asset_keyword: String,
}

pub fn settings() -> &'static AppSettings {
    SETTINGS.get_or_init(|| {
        toml::from_str(include_str!("../config/device.toml"))
            .expect("장치 설정 파일을 읽을 수 없습니다.")
    })
}