use crate::settings::{settings, AppSettings};
use hidapi::{HidApi, HidDevice};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::time::Duration;
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
    #[serde(default)]
    pub stick_calibration_enabled: bool,
    #[serde(default)]
    pub left_stick_center_x: f32,
    #[serde(default)]
    pub left_stick_center_y: f32,
    #[serde(default = "default_stick_deadzone")]
    pub left_stick_deadzone: f32,
    #[serde(default)]
    pub right_stick_center_x: f32,
    #[serde(default)]
    pub right_stick_center_y: f32,
    #[serde(default = "default_stick_deadzone")]
    pub right_stick_deadzone: f32,
    #[serde(default = "default_stick_min")]
    pub left_stick_min_x: f32,
    #[serde(default = "default_stick_max")]
    pub left_stick_max_x: f32,
    #[serde(default = "default_stick_min")]
    pub left_stick_min_y: f32,
    #[serde(default = "default_stick_max")]
    pub left_stick_max_y: f32,
    #[serde(default = "default_stick_min")]
    pub right_stick_min_x: f32,
    #[serde(default = "default_stick_max")]
    pub right_stick_max_x: f32,
    #[serde(default = "default_stick_min")]
    pub right_stick_min_y: f32,
    #[serde(default = "default_stick_max")]
    pub right_stick_max_y: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct DeviceInfo {
    pub firmware_version: Option<String>,
    pub rssi: Option<i8>,
    pub capabilities: Option<FirmwareCapabilities>,
    pub firmware_error: Option<String>,
    pub rssi_error: Option<String>,
    pub capabilities_error: Option<String>,
    pub usb_vendor_name: String,
    pub usb_speed_class: String,
    pub rssi_strength_label: String,
    pub battery_level: Option<u8>,
    pub is_charging: Option<bool>,
    pub dongle_connected: bool,
    pub controller_connected: bool,
    pub battery_report_available: bool,
    pub rssi_report_available: bool,
    pub config_readable: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct FirmwareCapabilities {
    pub protocol_version: u8,
    pub config_version: u8,
    pub config_body_length: u8,
    pub feature_flags: u32,
    pub supports_battery: bool,
    pub supports_rssi: bool,
    pub supports_vibration_test: bool,
    pub supports_adaptive_trigger: bool,
    pub supports_bootloader_command: bool,
    pub supports_stick_calibration: bool,
    pub supports_directional_stick_calibration: bool,
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
            let manufacturer = device.manufacturer_string().unwrap_or("Unknown manufacturer");
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
    let capabilities_result = read_capabilities_from_device(&device, settings.reports.capabilities);
    let rssi_result = read_rssi(&device, settings.reports.rssi);
    let (battery_level, is_charging) = read_battery(&device, settings.reports.battery);
    let rssi_val = rssi_result.as_ref().ok().and_then(|v| *v);
    let battery_report_available = battery_level.is_some();
    let rssi_report_available = rssi_val.is_some();
    // RSSI 리포트는 Pico만 연결된 상태에서도 0 dBm처럼 보일 수 있습니다.
    // DualSense 연결 UI는 실제 컨트롤러 상태를 나타내야 하므로 배터리 리포트 수신 여부로 판정합니다.
    let controller_connected = battery_report_available;
    let config_readable = read_config(device_id).is_ok();

    let manufacturer = device
        .get_manufacturer_string()
        .unwrap_or_else(|_| Some("Sony Interactive Entertainment".to_string()))
        .unwrap_or_else(|| "Sony Interactive Entertainment".to_string());
    let product_name = device
        .get_product_string()
        .unwrap_or_else(|_| Some("Wireless Controller / Dongle".to_string()))
        .unwrap_or_else(|| "Wireless Controller / Dongle".to_string());

    let rssi_strength_label = match rssi_val {
        Some(r) if r >= -60 => "우수 (최상의 입력 레이턴시)".to_string(),
        Some(r) if r >= -80 => "보통 (안정적인 무선 연결)".to_string(),
        Some(r) if r < -80 => "신호 약함 (장애물 확인 권장)".to_string(),
        _ => "측정 불가 또는 유선 연결 상태".to_string(),
    };
    let usb_speed_class = if product_name.to_lowercase().contains("high") {
        "USB 2.0 High-Speed"
    } else {
        "USB 1.1 / 2.0 Full-Speed (안정적 게이밍 입력 대역폭)"
    }
    .to_string();

    Ok(DeviceInfo {
        firmware_version: firmware_result.as_ref().ok().and_then(Clone::clone),
        rssi: rssi_val,
        capabilities: capabilities_result.as_ref().ok().cloned(),
        firmware_error: firmware_result.err().map(|error| error.to_string()),
        rssi_error: rssi_result.err().map(|error| error.to_string()),
        capabilities_error: capabilities_result.err().map(|error| error.to_string()),
        usb_vendor_name: format!("{manufacturer} ({product_name})"),
        usb_speed_class,
        rssi_strength_label,
        battery_level,
        is_charging,
        dongle_connected: true,
        controller_connected,
        battery_report_available,
        rssi_report_available,
        config_readable,
    })
}

pub fn read_capabilities(device_id: &str) -> Result<FirmwareCapabilities, BridgeError> {
    let settings = settings();
    let device = open_device(device_id)?;
    read_capabilities_from_device(&device, settings.reports.capabilities)
}

pub fn apply_config(device_id: &str, config: BridgeConfig) -> Result<(), BridgeError> {
    validate_config(&config, settings())?;
    let body = encode_config(&config);
    send_command(device_id, 0x01, Some(&body))
}

pub fn save_config(device_id: &str) -> Result<(), BridgeError> {
    send_command(device_id, 0x02, None)
}

pub fn reconnect_usb(device_id: &str) -> Result<(), BridgeError> {
    send_command(device_id, 0x03, None)
}

pub fn enter_bootloader(device_id: &str) -> Result<(), BridgeError> {
    send_command(device_id, 0x04, None)
}

pub fn test_vibration(
    device_id: &str,
    weak_magnitude: f32,
    strong_magnitude: f32,
    duration_ms: u16,
) -> Result<(), BridgeError> {
    if weak_magnitude.is_nan()
        || strong_magnitude.is_nan()
        || !(0.0..=1.0).contains(&weak_magnitude)
        || !(0.0..=1.0).contains(&strong_magnitude)
    {
        return Err(BridgeError::InvalidConfig(
            "진동 강도는 0.0부터 1.0 사이여야 합니다.".into(),
        ));
    }

    let duration_ms = duration_ms.clamp(50, 3000);
    let weak = (weak_magnitude * 255.0).round() as u8;
    let strong = (strong_magnitude * 255.0).round() as u8;

    write_vibration_output(device_id, weak, strong)?;
    std::thread::sleep(Duration::from_millis(u64::from(duration_ms)));
    write_vibration_output(device_id, 0, 0)
}

pub fn test_adaptive_trigger(
    device_id: &str,
    side: &str,
    start_position: f32,
    strength: f32,
    duration_ms: u16,
) -> Result<(), BridgeError> {
    if start_position.is_nan()
        || strength.is_nan()
        || !(0.0..=1.0).contains(&start_position)
        || !(0.0..=1.0).contains(&strength)
    {
        return Err(BridgeError::InvalidConfig(
            "적응형 트리거 시작 위치와 강도는 0.0부터 1.0 사이여야 합니다.".into(),
        ));
    }

    let duration_ms = duration_ms.clamp(50, 3000);
    let position = (start_position * 9.0).round() as u8;
    let strength = (strength * 8.0).round().clamp(0.0, 8.0) as u8;

    write_adaptive_trigger_output(device_id, side, position, strength)?;
    std::thread::sleep(Duration::from_millis(u64::from(duration_ms)));
    write_adaptive_trigger_output(device_id, side, 0, 0)
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
        let end = 2 + body.len().min(report.len() - 2);
        report[2..end].copy_from_slice(&body[..end - 2]);
    }

    device.send_feature_report(&report)?;
    Ok(())
}

fn write_vibration_output(device_id: &str, weak: u8, strong: u8) -> Result<(), BridgeError> {
    let device = open_device(device_id)?;
    let mut report = [0_u8; 64];

    report[0] = 0x02;
    report[1] = 0x03;
    report[3] = weak;
    report[4] = strong;

    if let Err(ds_error) = device.write(&report[..48]) {
        device.write(&report).map_err(|dse_error| {
            BridgeError::Hid(hidapi::HidError::HidApiError {
                message: format!("DS 출력 리포트 실패: {ds_error}; DSE 출력 리포트 실패: {dse_error}"),
            })
        })?;
    }
    Ok(())
}

fn write_adaptive_trigger_output(
    device_id: &str,
    side: &str,
    position: u8,
    strength: u8,
) -> Result<(), BridgeError> {
    let device = open_device(device_id)?;
    let mut report = [0_u8; 64];

    report[0] = 0x02;
    let trigger_offset = match side {
        "right" => {
            report[1] = 1 << 2;
            11
        }
        "left" => {
            report[1] = 1 << 3;
            22
        }
        _ => {
            return Err(BridgeError::InvalidConfig(
                "적응형 트리거 대상은 left 또는 right여야 합니다.".into(),
            ));
        }
    };

    if strength == 0 {
        report[trigger_offset] = 0x05;
    } else {
        let force_value = (strength - 1) & 0x07;
        let mut active_zones = 0_u16;
        let mut force_zones = 0_u32;

        for zone in position.min(9)..10 {
            active_zones |= 1 << zone;
            force_zones |= u32::from(force_value) << (3 * zone);
        }

        report[trigger_offset] = 0x21;
        report[trigger_offset + 1] = (active_zones & 0xff) as u8;
        report[trigger_offset + 2] = ((active_zones >> 8) & 0xff) as u8;
        report[trigger_offset + 3] = (force_zones & 0xff) as u8;
        report[trigger_offset + 4] = ((force_zones >> 8) & 0xff) as u8;
        report[trigger_offset + 5] = ((force_zones >> 16) & 0xff) as u8;
        report[trigger_offset + 6] = ((force_zones >> 24) & 0xff) as u8;
    }

    if let Err(ds_error) = device.write(&report[..48]) {
        device.write(&report).map_err(|dse_error| {
            BridgeError::Hid(hidapi::HidError::HidApiError {
                message: format!("DS 적응형 트리거 출력 실패: {ds_error}; DSE 출력 실패: {dse_error}"),
            })
        })?;
    }
    Ok(())
}

fn decode_config(bytes: &[u8], settings: &AppSettings) -> Result<BridgeConfig, BridgeError> {
    if bytes.len() < 15 {
        return Err(BridgeError::InvalidConfig(format!(
            "설정 데이터 길이가 부족합니다. 실제 {}바이트, 최소 15바이트입니다.",
            bytes.len()
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
        stick_calibration_enabled: bytes.get(15).copied().unwrap_or(0) != 0,
        left_stick_center_x: read_i8_scaled(bytes, 16, 127.0).unwrap_or(0.0),
        left_stick_center_y: read_i8_scaled(bytes, 17, 127.0).unwrap_or(0.0),
        left_stick_deadzone: bytes.get(18).map(|v| f32::from(*v) / 10.0).unwrap_or(1.0),
        right_stick_center_x: read_i8_scaled(bytes, 19, 127.0).unwrap_or(0.0),
        right_stick_center_y: read_i8_scaled(bytes, 20, 127.0).unwrap_or(0.0),
        right_stick_deadzone: bytes.get(21).map(|v| f32::from(*v) / 10.0).unwrap_or(1.0),
        left_stick_min_x: read_i8_scaled(bytes, 22, 127.0).unwrap_or(-1.0),
        left_stick_max_x: read_i8_scaled(bytes, 23, 127.0).unwrap_or(1.0),
        left_stick_min_y: read_i8_scaled(bytes, 24, 127.0).unwrap_or(-1.0),
        left_stick_max_y: read_i8_scaled(bytes, 25, 127.0).unwrap_or(1.0),
        right_stick_min_x: read_i8_scaled(bytes, 26, 127.0).unwrap_or(-1.0),
        right_stick_max_x: read_i8_scaled(bytes, 27, 127.0).unwrap_or(1.0),
        right_stick_min_y: read_i8_scaled(bytes, 28, 127.0).unwrap_or(-1.0),
        right_stick_max_y: read_i8_scaled(bytes, 29, 127.0).unwrap_or(1.0),
    };

    validate_config(&config, settings)?;
    Ok(config)
}

fn encode_config(config: &BridgeConfig) -> Vec<u8> {
    let mut out = vec![0_u8; settings().config.body_length.max(15)];
    out[0] = settings().config.version;
    out[1..5].copy_from_slice(&config.haptics_gain.to_le_bytes());
    out[5..9].copy_from_slice(
        &(config.speaker_volume_percent.clamp(0.0, 100.0) - 100.0).to_le_bytes(),
    );
    out[9] = config.inactive_time;
    out[10] = u8::from(config.disable_inactive_disconnect);
    out[11] = u8::from(config.disable_pico_led);
    out[12] = config.polling_rate_mode;
    out[13] = config.haptics_buffer_length;
    out[14] = config.controller_mode;

    if out.len() >= 22 {
        out[15] = u8::from(config.stick_calibration_enabled);
        write_i8_scaled(&mut out, 16, config.left_stick_center_x, 127.0);
        write_i8_scaled(&mut out, 17, config.left_stick_center_y, 127.0);
        out[18] = (config.left_stick_deadzone.clamp(0.0, 25.5) * 10.0).round() as u8;
        write_i8_scaled(&mut out, 19, config.right_stick_center_x, 127.0);
        write_i8_scaled(&mut out, 20, config.right_stick_center_y, 127.0);
        out[21] = (config.right_stick_deadzone.clamp(0.0, 25.5) * 10.0).round() as u8;
    }

    if out.len() >= 30 {
        write_i8_scaled(&mut out, 22, config.left_stick_min_x, 127.0);
        write_i8_scaled(&mut out, 23, config.left_stick_max_x, 127.0);
        write_i8_scaled(&mut out, 24, config.left_stick_min_y, 127.0);
        write_i8_scaled(&mut out, 25, config.left_stick_max_y, 127.0);
        write_i8_scaled(&mut out, 26, config.right_stick_min_x, 127.0);
        write_i8_scaled(&mut out, 27, config.right_stick_max_x, 127.0);
        write_i8_scaled(&mut out, 28, config.right_stick_min_y, 127.0);
        write_i8_scaled(&mut out, 29, config.right_stick_max_y, 127.0);
    }

    out
}

fn validate_config(config: &BridgeConfig, settings: &AppSettings) -> Result<(), BridgeError> {
    let rules = &settings.config;

    if config.config_version > rules.version {
        return Err(BridgeError::InvalidConfig(format!(
            "지원하는 설정 버전은 {}인데 장치는 {} 버전을 반환했습니다.",
            rules.version, config.config_version
        )));
    }

    if config.haptics_gain.is_nan()
        || config.haptics_gain < rules.haptics_gain_min
        || config.haptics_gain > rules.haptics_gain_max
    {
        return Err(BridgeError::InvalidConfig(format!(
            "햅틱 강도는 {:.2}부터 {:.2} 사이여야 합니다.",
            rules.haptics_gain_min, rules.haptics_gain_max
        )));
    }

    if config.speaker_volume_percent.is_nan()
        || config.speaker_volume_percent < rules.speaker_volume_percent_min
        || config.speaker_volume_percent > rules.speaker_volume_percent_max
    {
        return Err(BridgeError::InvalidConfig(
            "스피커 음량은 0%부터 100% 사이여야 합니다.".into(),
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
            "폴링 속도 모드는 0, 1, 2 중 하나여야 합니다.".into(),
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

fn read_battery(device: &HidDevice, report_id: u8) -> (Option<u8>, Option<bool>) {
    let mut buffer = [0_u8; 64];
    buffer[0] = report_id;
    if let Ok(len) = device.get_feature_report(&mut buffer) {
        if len >= 3 {
            let level = buffer[1].min(100);
            let power_state = buffer[2] & 0x0f;
            return (Some(level), Some(power_state == 0x01));
        }
    }
    (None, None)
}

fn read_capabilities_from_device(
    device: &HidDevice,
    report_id: u8,
) -> Result<FirmwareCapabilities, BridgeError> {
    let mut buffer = [0_u8; 64];
    buffer[0] = report_id;
    let len = device.get_feature_report(&mut buffer)?;
    if len <= 1 {
        return Err(BridgeError::InvalidConfig(
            "펌웨어 기능 리포트가 비어 있습니다.".into(),
        ));
    }
    let payload = &buffer[1..len];

    if payload.len() < 12 || &payload[0..4] != b"D5CP" {
        return Err(BridgeError::InvalidConfig(
            "펌웨어 기능 리포트 형식이 올바르지 않습니다.".into(),
        ));
    }

    let feature_flags = u32::from_le_bytes(payload[8..12].try_into().unwrap());
    let protocol_version = payload[4];
    let config_version = payload[5];
    let config_body_length = payload[6];
    if protocol_version == 0 {
        return Err(BridgeError::InvalidConfig(
            "펌웨어 기능 리포트 프로토콜 버전이 올바르지 않습니다.".into(),
        ));
    }
    if config_body_length < 15 {
        return Err(BridgeError::InvalidConfig(format!(
            "펌웨어 기능 리포트의 설정 길이가 너무 짧습니다: {}바이트",
            config_body_length
        )));
    }

    Ok(FirmwareCapabilities {
        protocol_version,
        config_version,
        config_body_length,
        feature_flags,
        supports_battery: feature_flags & (1 << 0) != 0,
        supports_rssi: feature_flags & (1 << 1) != 0,
        supports_vibration_test: feature_flags & (1 << 2) != 0,
        supports_adaptive_trigger: feature_flags & (1 << 3) != 0,
        supports_bootloader_command: feature_flags & (1 << 4) != 0,
        supports_stick_calibration: feature_flags & (1 << 5) != 0,
        supports_directional_stick_calibration: feature_flags & (1 << 6) != 0,
    })
}

fn read_i8_scaled(bytes: &[u8], offset: usize, scale: f32) -> Option<f32> {
    bytes.get(offset).map(|value| (*value as i8) as f32 / scale)
}

fn write_i8_scaled(out: &mut [u8], offset: usize, value: f32, scale: f32) {
    if let Some(slot) = out.get_mut(offset) {
        let scaled = (value.clamp(-1.0, 1.0) * scale)
            .round()
            .clamp(i8::MIN as f32, i8::MAX as f32) as i8;
        *slot = scaled as u8;
    }
}

fn default_stick_deadzone() -> f32 {
    1.0
}

fn default_stick_min() -> f32 {
    -1.0
}

fn default_stick_max() -> f32 {
    1.0
}
