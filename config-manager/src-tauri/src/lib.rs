use std::time::Duration;
use tauri::Emitter;

mod bridge;
mod commands;
mod settings;
mod updater;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut last_device_ids: Vec<String> = Vec::new();
                loop {
                    tokio::time::sleep(Duration::from_millis(1500)).await;
                    if let Ok(devices) = bridge::list_devices() {
                        let mut current_ids: Vec<String> = devices.iter().map(|d| d.id.clone()).collect();
                        current_ids.sort();
                        
                        if current_ids != last_device_ids {
                            // 연결 상태가 흔들리는 순간을 피하기 위해 짧게 한 번 더 확인합니다.
                            tokio::time::sleep(Duration::from_millis(200)).await;
                            if let Ok(double_check) = bridge::list_devices() {
                                let mut final_ids: Vec<String> = double_check.iter().map(|d| d.id.clone()).collect();
                                final_ids.sort();
                                
                                if final_ids == current_ids {
                                    last_device_ids = final_ids;
                                    let _ = app_handle.emit("device-list-changed", &last_device_ids);
                                }
                            }
                        }
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_devices,
            commands::read_config,
            commands::read_device_info,
            commands::read_capabilities,
            commands::apply_config,
            commands::save_config,
            commands::reconnect_usb,
            commands::test_vibration,
            commands::test_adaptive_trigger,
            commands::check_debug_firmware_update,
            commands::flash_latest_debug_firmware,
            commands::recovery_flash_latest_debug_firmware
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 애플리케이션 실행 중 복구할 수 없는 오류가 발생했습니다.");
}
