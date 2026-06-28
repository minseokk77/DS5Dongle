use crate::bridge::{self, BridgeConfig, BridgeDevice, BridgeError, DeviceInfo};
use crate::updater::{self, FirmwareFlashResult, FirmwareUpdateInfo, UpdateError, AppUpdateInfo};

#[tauri::command]
pub async fn list_devices() -> Result<Vec<BridgeDevice>, BridgeError> {
    tauri::async_runtime::spawn_blocking(|| bridge::list_devices()).await.unwrap()
}

#[tauri::command]
pub async fn read_config(device_id: String) -> Result<BridgeConfig, BridgeError> {
    tauri::async_runtime::spawn_blocking(move || bridge::read_config(&device_id)).await.unwrap()
}

#[tauri::command]
pub async fn read_device_info(device_id: String) -> Result<DeviceInfo, BridgeError> {
    tauri::async_runtime::spawn_blocking(move || bridge::read_device_info(&device_id)).await.unwrap()
}

#[tauri::command]
pub async fn apply_config(device_id: String, config: BridgeConfig) -> Result<(), BridgeError> {
    tauri::async_runtime::spawn_blocking(move || bridge::apply_config(&device_id, config)).await.unwrap()
}

#[tauri::command]
pub async fn save_config(device_id: String) -> Result<(), BridgeError> {
    tauri::async_runtime::spawn_blocking(move || bridge::save_config(&device_id)).await.unwrap()
}

#[tauri::command]
pub async fn reconnect_usb(device_id: String) -> Result<(), BridgeError> {
    tauri::async_runtime::spawn_blocking(move || bridge::reconnect_usb(&device_id)).await.unwrap()
}

#[tauri::command]
pub async fn test_vibration(
    device_id: String,
    weak_magnitude: f32,
    strong_magnitude: f32,
    duration_ms: u16,
) -> Result<(), BridgeError> {
    tauri::async_runtime::spawn_blocking(move || bridge::test_vibration(&device_id, weak_magnitude, strong_magnitude, duration_ms)).await.unwrap()
}

#[tauri::command]
pub async fn test_adaptive_trigger(
    device_id: String,
    side: String,
    start_position: f32,
    strength: f32,
    duration_ms: u16,
) -> Result<(), BridgeError> {
    tauri::async_runtime::spawn_blocking(move || bridge::test_adaptive_trigger(&device_id, &side, start_position, strength, duration_ms)).await.unwrap()
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
pub async fn recovery_flash_latest_debug_firmware(device_id: Option<String>) -> Result<FirmwareFlashResult, UpdateError> {
    updater::recovery_flash_latest_debug_firmware(device_id).await
}

#[tauri::command]
pub async fn check_app_update(current_version: String) -> Result<Option<AppUpdateInfo>, UpdateError> {
    updater::check_app_update(&current_version).await
}

#[tauri::command]
pub async fn install_app_update(download_url: String) -> Result<(), UpdateError> {
    updater::install_app_update(&download_url).await
}
