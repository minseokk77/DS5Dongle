use crate::settings::{settings, AppSettings};
use hidapi::{HidApi, HidDevice};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use thiserror::Error;

#[derive(Clone, Debug, Serialize)]
pub struct BridgeDevice {
    pub id: String,
    pub label: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial_number: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BridgeConfig {
    pub config_version: u8,
    pub haptics_gain: f32,
    pub speaker_volume_percent: f32,
    pub inactive_time: u8,
    pub disable_inactive_disconnect: bool,
    pub disable_pico_led: bool,
    pub polling_rate_mode: u8,
    pub haptics_buffer_length: u8,
    pub controller_mode: u8,
}

#[derive(Clone, Debug, Serialize)]
pub struct DeviceInfo {
    pub firmware_version: Option<String>,
    pub rssi: Option<i8>,
    pub firmware_error: Option<String>,
    pub rssi_error: Option<String>,
}

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("HID 초기화에 실패했습니다: {0}")]
    HidInit(String),
    #[error("HID 통신 오류가 발생했습니다: {0}")]
    Hid(#[from] hidapi::HidError),
    #[error("장치 설정 값이 올바르지 않습니다: {0}")]
    InvalidConfig(String),
}

impl serde::Serialize for BridgeError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub fn list_devices() -> Result<Vec<BridgeDevice>, BridgeError> {
    let settings = settings();
    let api = HidApi::new().map_err(|error| BridgeError::HidInit(error.to_string()))?;

    Ok(api
        .device_list()
        .filter(|device| {
            device.vendor_id() == settings.usb.sony_vendor_id
                && settings.usb.product_ids.contains(&device.product_id())
        })
        .map(|device| {
            let product = device.product_string().unwrap_or("Unknown HID");
            let manufacturer = device
                .manufacturer_string()
                .unwrap_or("Unknown manufacturer");
            let serial = device.serial_number().map(ToOwned::to_owned);
            let product_id = device.product_id();
            let serial_label = serial
                .as_ref()
                .filter(|value| !value.is_empty())
                .map(|value| Cow::Owned(format!(" - {value}")))
                .unwrap_or(Cow::Borrowed(""));

            BridgeDevice {
                id: device.path().to_string_lossy().to_string(),
                label: format!("{product} - {manufacturer} - PID {product_id:04x}{serial_label}"),
                vendor_id: device.vendor_id(),
                product_id,
                serial_number: serial,
            }
        })
        .collect())
}

pub fn read_config(device_id: &str) -> Result<BridgeConfig, BridgeError> {
    let settings = settings();
    let device = open_device(device_id)?;
    let mut buffer = [0_u8; 64];
    buffer[0] = settings.reports.config_read;
    let len = device.get_feature_report(&mut buffer)?;

    if len <= 1 {
        return Err(BridgeError::InvalidConfig(
            "장치가 빈 설정 리포트를 반환했습니다.".into(),
        ));
    }

    decode_config(&buffer[1..len], settings)
}

pub fn read_device_info(device_id: &str) -> Result<DeviceInfo, BridgeError> {
    let settings = settings();
    let device = open_device(device_id)?;

    let firmware_result = read_feature_string(&device, settings.reports.firmware_version);
    let rssi_result = read_rssi(&device, settings.reports.rssi);

    Ok(DeviceInfo {
        firmware_version: firmware_result.as_ref().ok().and_then(Clone::clone),
        rssi: rssi_result.as_ref().ok().and_then(|value| *value),
        firmware_error: firmware_result.err().map(|error| error.to_string()),
        rssi_error: rssi_result.err().map(|error| error.to_string()),
    })
}

pub fn apply_config(device_id: &str, config: BridgeConfig) -> Result<(), BridgeError> {
    validate_config(&config, settings())?;
    send_command(device_id, 0x01, Some(&encode_config(&config)))
}

pub fn save_config(device_id: &str) -> Result<(), BridgeError> {
    send_command(device_id, 0x02, None)
}

pub fn reconnect_usb(device_id: &str) -> Result<(), BridgeError> {
    send_command(device_id, 0x03, None)
}

fn open_device(device_id: &str) -> Result<HidDevice, BridgeError> {
    let api = HidApi::new().map_err(|error| BridgeError::HidInit(error.to_string()))?;
    let device_path = std::ffi::CString::new(device_id).map_err(|_| {
        BridgeError::InvalidConfig("장치 경로에 유효하지 않은 문자가 포함되어 있습니다.".into())
    })?;
    api.open_path(&device_path).map_err(BridgeError::from)
}

fn send_command(device_id: &str, command: u8, body: Option<&[u8]>) -> Result<(), BridgeError> {
    let settings = settings();
    let device = open_device(device_id)?;
    let mut report = [0_u8; 64];
    report[0] = settings.reports.command;
    report[1] = command;

    if let Some(body) = body {
        report[2..2 + body.len()].copy_from_slice(body);
    }

    device.send_feature_report(&report)?;
    Ok(())
}

fn decode_config(bytes: &[u8], settings: &AppSettings) -> Result<BridgeConfig, BridgeError> {
    if bytes.len() < settings.config.body_length {
        return Err(BridgeError::InvalidConfig(format!(
            "설정 데이터 길이가 부족합니다. 실제 {}바이트, 필요 {}바이트입니다.",
            bytes.len(),
            settings.config.body_length
        )));
    }

    let speaker_volume_db = f32::from_le_bytes(bytes[5..9].try_into().unwrap());
    let config = BridgeConfig {
        config_version: bytes[0],
        haptics_gain: f32::from_le_bytes(bytes[1..5].try_into().unwrap()),
        speaker_volume_percent: (speaker_volume_db + 100.0).clamp(0.0, 100.0),
        inactive_time: bytes[9],
        disable_inactive_disconnect: bytes[10] != 0,
        disable_pico_led: bytes[11] != 0,
        polling_rate_mode: bytes[12],
        haptics_buffer_length: bytes[13],
        controller_mode: bytes[14],
    };

    validate_config(&config, settings)?;
    Ok(config)
}

fn encode_config(config: &BridgeConfig) -> [u8; 15] {
    let mut out = [0_u8; 15];
    out[0] = settings().config.version;
    out[1..5].copy_from_slice(&config.haptics_gain.to_le_bytes());
    out[5..9]
        .copy_from_slice(&(config.speaker_volume_percent.clamp(0.0, 100.0) - 100.0).to_le_bytes());
    out[9] = config.inactive_time;
    out[10] = u8::from(config.disable_inactive_disconnect);
    out[11] = u8::from(config.disable_pico_led);
    out[12] = config.polling_rate_mode;
    out[13] = config.haptics_buffer_length;
    out[14] = config.controller_mode;
    out
}

fn validate_config(config: &BridgeConfig, settings: &AppSettings) -> Result<(), BridgeError> {
    let rules = &settings.config;

    if config.config_version != rules.version {
        return Err(BridgeError::InvalidConfig(format!(
            "지원하는 설정 버전은 {}인데 장치는 {} 버전을 반환했습니다.",
            rules.version, config.config_version
        )));
    }

    if config.haptics_gain.is_nan()
        || config.haptics_gain < rules.haptics_gain_min
        || config.haptics_gain > rules.haptics_gain_max
    {
        return Err(BridgeError::InvalidConfig(
            format!(
                "햅틱 게인은 {:.2}부터 {:.2} 사이여야 합니다.",
                rules.haptics_gain_min, rules.haptics_gain_max
            ),
        ));
    }

    if config.speaker_volume_percent.is_nan()
        || config.speaker_volume_percent < rules.speaker_volume_percent_min
        || config.speaker_volume_percent > rules.speaker_volume_percent_max
    {
        return Err(BridgeError::InvalidConfig(
            "스피커 볼륨은 0%부터 100% 사이여야 합니다.".into(),
        ));
    }

    if config.inactive_time < rules.inactive_time_min
        || config.inactive_time > rules.inactive_time_max
    {
        return Err(BridgeError::InvalidConfig(
            "비활성 시간은 5분부터 60분 사이여야 합니다.".into(),
        ));
    }

    if config.polling_rate_mode > 2 {
        return Err(BridgeError::InvalidConfig(
            "폴링 레이트 모드는 0, 1, 2 중 하나여야 합니다.".into(),
        ));
    }

    if config.haptics_buffer_length < rules.haptics_buffer_length_min
        || config.haptics_buffer_length > rules.haptics_buffer_length_max
    {
        return Err(BridgeError::InvalidConfig(
            "햅틱 버퍼 길이는 16부터 128 사이여야 합니다.".into(),
        ));
    }

    if config.controller_mode > 2 {
        return Err(BridgeError::InvalidConfig(
            "컨트롤러 모드는 DS5, DSE, Auto 중 하나여야 합니다.".into(),
        ));
    }

    Ok(())
}

fn read_feature_string(device: &HidDevice, report_id: u8) -> Result<Option<String>, BridgeError> {
    let mut buffer = [0_u8; 64];
    buffer[0] = report_id;
    let len = device.get_feature_report(&mut buffer)?;

    if len <= 1 {
        return Ok(None);
    }

    let value = String::from_utf8_lossy(&buffer[1..len])
        .trim_matches(char::from(0))
        .trim()
        .to_string();

    Ok((!value.is_empty()).then_some(value))
}

fn read_rssi(device: &HidDevice, report_id: u8) -> Result<Option<i8>, BridgeError> {
    let mut buffer = [0_u8; 64];
    buffer[0] = report_id;
    let len = device.get_feature_report(&mut buffer)?;
    Ok((len > 1).then_some(buffer[1] as i8))
}
