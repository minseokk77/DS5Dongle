#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    ds5_bridge_config_tauri_lib::run();
}
