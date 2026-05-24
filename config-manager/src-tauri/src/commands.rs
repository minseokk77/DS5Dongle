use crate::bridge::{self, BridgeConfig, BridgeDevice, BridgeError, DeviceInfo};
use crate::updater::{self, FirmwareFlashResult, FirmwareUpdateInfo, UpdateError};

#[tauri::command]
pub fn list_devices() -> Result<Vec<BridgeDevice>, BridgeError> {
    bridge::list_devices()
}

#[tauri::command]
pub fn read_config(device_id: String) -> Result<BridgeConfig, BridgeError> {
    bridge::read_config(&device_id)
}

#[tauri::command]
pub fn read_device_info(device_id: String) -> Result<DeviceInfo, BridgeError> {
    bridge::read_device_info(&device_id)
}

#[tauri::command]
pub fn apply_config(device_id: String, config: BridgeConfig) -> Result<(), BridgeError> {
    bridge::apply_config(&device_id, config)
}

#[tauri::command]
pub fn save_config(device_id: String) -> Result<(), BridgeError> {
    bridge::save_config(&device_id)
}

#[tauri::command]
pub fn reconnect_usb(device_id: String) -> Result<(), BridgeError> {
    bridge::reconnect_usb(&device_id)
}

#[tauri::command]
pub fn test_vibration(
    device_id: String,
    weak_magnitude: f32,
    strong_magnitude: f32,
    duration_ms: u16,
) -> Result<(), BridgeError> {
    bridge::test_vibration(&device_id, weak_magnitude, strong_magnitude, duration_ms)
}

#[tauri::command]
pub fn test_adaptive_trigger(
    device_id: String,
    side: String,
    start_position: f32,
    strength: f32,
    duration_ms: u16,
) -> Result<(), BridgeError> {
    bridge::test_adaptive_trigger(&device_id, &side, start_position, strength, duration_ms)
}

#[tauri::command]
pub async fn check_debug_firmware_update() -> Result<FirmwareUpdateInfo, UpdateError> {
    updater::check_debug_firmware_update().await
}

#[tauri::command]
pub async fn flash_latest_debug_firmware(device_id: Option<String>) -> Result<FirmwareFlashResult, UpdateError> {
    updater::flash_latest_debug_firmware(device_id).await
}

#[tauri::command]
pub async fn recovery_flash_latest_debug_firmware() -> Result<FirmwareFlashResult, UpdateError> {
    updater::recovery_flash_latest_debug_firmware().await
}
