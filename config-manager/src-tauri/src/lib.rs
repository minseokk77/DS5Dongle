mod bridge;
mod commands;
mod settings;
mod updater;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::list_devices,
            commands::read_config,
            commands::read_device_info,
            commands::apply_config,
            commands::save_config,
            commands::reconnect_usb,
            commands::check_debug_firmware_update,
            commands::flash_latest_debug_firmware
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 애플리케이션 실행 중 복구할 수 없는 오류가 발생했습니다.");
}
