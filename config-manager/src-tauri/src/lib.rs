use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

mod bridge;
mod commands;
mod settings;
mod updater;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if let Some(icon) = app.default_window_icon().cloned() {
                let open_item = MenuItem::with_id(app, "open", "열기", true, None::<&str>)?;
                let quit_item = MenuItem::with_id(app, "quit", "종료", true, None::<&str>)?;
                let tray_menu = Menu::with_items(app, &[&open_item, &quit_item])?;
                let _ = TrayIconBuilder::new()
                    .tooltip("DS5 Dongle Config")
                    .icon(icon)
                    .menu(&tray_menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(|app, event| {
                        match event.id().as_ref() {
                            "open" => {
                                if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                            "quit" => app.exit(0),
                            _ => {}
                        }
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } = event
                        {
                            if let Some(window) = tray.app_handle().get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    })
                    .build(app);
            }

            if let Some(window) = app.get_webview_window("main") {
                let main_window = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = main_window.hide();
                    }
                });
            }

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut last_device_ids: Vec<String> = Vec::new();
                loop {
                    tokio::time::sleep(Duration::from_millis(1500)).await;
                    if let Ok(devices) = bridge::refresh_device_list() {
                        *bridge::DEVICE_CACHE.lock().unwrap() = devices.clone();
                        let mut current_ids: Vec<String> = devices.iter().map(|d| d.id.clone()).collect();
                        current_ids.sort();
                        
                        if current_ids != last_device_ids {
                            // 연결 상태가 흔들리는 순간을 피하기 위해 짧게 한 번 더 확인합니다.
                            tokio::time::sleep(Duration::from_millis(200)).await;
                            if let Ok(double_check) = bridge::refresh_device_list() {
                                *bridge::DEVICE_CACHE.lock().unwrap() = double_check.clone();
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
            commands::apply_config,
            commands::save_config,
            commands::reconnect_usb,
            commands::test_vibration,
            commands::test_adaptive_trigger,
            commands::check_debug_firmware_update,
            commands::flash_latest_debug_firmware,
            commands::recovery_flash_latest_debug_firmware,
            commands::check_app_update,
            commands::install_app_update,
            commands::quit_app
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 애플리케이션 실행 중 복구할 수 없는 오류가 발생했습니다.");
}
