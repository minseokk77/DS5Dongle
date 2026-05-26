import { invoke } from '@tauri-apps/api/core';

export type PollingRateMode = 0 | 1 | 2;
export type ControllerMode = 0 | 1 | 2;

export interface BridgeDevice {
  id: string;
  label: string;
  vendor_id: number;
  product_id: number;
  serial_number?: string | null;
  config_only: boolean;
}

export interface BridgeConfig {
  config_version: number;
  haptics_gain: number;
  speaker_volume_percent: number;
  inactive_time: number;
  disable_inactive_disconnect: boolean;
  disable_pico_led: boolean;
  polling_rate_mode: PollingRateMode;
  haptics_buffer_length: number;
  controller_mode: ControllerMode;
  stick_calibration_enabled?: boolean;
  left_stick_center_x?: number;
  left_stick_center_y?: number;
  left_stick_deadzone?: number;
  right_stick_center_x?: number;
  right_stick_center_y?: number;
  right_stick_deadzone?: number;
  left_stick_min_x?: number;
  left_stick_max_x?: number;
  left_stick_min_y?: number;
  left_stick_max_y?: number;
  right_stick_min_x?: number;
  right_stick_max_x?: number;
  right_stick_min_y?: number;
  right_stick_max_y?: number;
}

export interface DeviceInfo {
  firmware_version?: string | null;
  rssi?: number | null;
  capabilities?: FirmwareCapabilities | null;
  firmware_error?: string | null;
  rssi_error?: string | null;
  capabilities_error?: string | null;
  usb_vendor_name: string;
  usb_speed_class: string;
  rssi_strength_label: string;
  battery_level?: number | null;
  is_charging?: boolean | null;
  dongle_connected?: boolean;
  controller_connected?: boolean;
  battery_report_available?: boolean;
  rssi_report_available?: boolean;
  config_readable?: boolean;
}

export interface FirmwareCapabilities {
  protocol_version: number;
  config_version: number;
  config_body_length: number;
  build_channel?: string | null;
  controller_connected?: boolean | null;
  feature_flags: number;
  supports_battery: boolean;
  supports_rssi: boolean;
  supports_vibration_test: boolean;
  supports_adaptive_trigger: boolean;
  supports_bootloader_command: boolean;
  supports_stick_calibration: boolean;
  supports_directional_stick_calibration: boolean;
}

export interface FirmwareUpdateInfo {
  version: string;
  asset_name: string;
  download_url: string;
}

export interface FirmwareFlashResult {
  version: string;
  asset_name: string;
  drive: string;
  copied_bytes: number;
  expected_bytes: number;
  drive_disappeared: boolean;
  reconnected: boolean;
  restored_settings: boolean;
}

export async function listDevices(): Promise<BridgeDevice[]> {
  try {
    return await invoke<BridgeDevice[]>('list_devices');
  } catch (error) {
    throw friendlyError(error, 'USB 장치 목록을 불러오지 못했습니다.');
  }
}

export async function readConfig(deviceId: string): Promise<BridgeConfig> {
  try {
    return await invoke<BridgeConfig>('read_config', { deviceId });
  } catch (error) {
    throw friendlyError(error, '장치 설정을 읽지 못했습니다.');
  }
}

export async function readDeviceInfo(deviceId: string): Promise<DeviceInfo> {
  try {
    return await invoke<DeviceInfo>('read_device_info', { deviceId });
  } catch (error) {
    throw friendlyError(error, '장치 정보를 읽지 못했습니다.');
  }
}

export async function readCapabilities(deviceId: string): Promise<FirmwareCapabilities> {
  try {
    return await invoke<FirmwareCapabilities>('read_capabilities', { deviceId });
  } catch (error) {
    throw friendlyError(error, '펌웨어 기능 리포트를 읽지 못했습니다.');
  }
}

export async function applyConfig(deviceId: string, config: BridgeConfig): Promise<void> {
  try {
    await invoke('apply_config', { deviceId, config });
  } catch (error) {
    throw friendlyError(error, '설정을 장치에 적용하지 못했습니다.');
  }
}

export async function saveConfig(deviceId: string): Promise<void> {
  try {
    await invoke('save_config', { deviceId });
  } catch (error) {
    throw friendlyError(error, '설정을 플래시에 저장하지 못했습니다.');
  }
}

export async function reconnectUsb(deviceId: string): Promise<void> {
  try {
    await invoke('reconnect_usb', { deviceId });
  } catch (error) {
    throw friendlyError(error, 'USB 재연결 명령을 보내지 못했습니다.');
  }
}

export async function testVibration(
  deviceId: string,
  weakMagnitude: number,
  strongMagnitude: number,
  durationMs: number
): Promise<void> {
  try {
    await invoke('test_vibration', {
      deviceId,
      weakMagnitude,
      strongMagnitude,
      durationMs
    });
  } catch (error) {
    throw friendlyError(error, '진동 테스트 명령을 보내지 못했습니다.');
  }
}

export async function testAdaptiveTrigger(
  deviceId: string,
  side: 'left' | 'right',
  startPosition: number,
  strength: number,
  durationMs: number
): Promise<void> {
  try {
    await invoke('test_adaptive_trigger', {
      deviceId,
      side,
      startPosition,
      strength,
      durationMs
    });
  } catch (error) {
    throw friendlyError(error, '적응형 트리거 테스트 명령을 보내지 못했습니다.');
  }
}

export async function checkDebugFirmwareUpdate(): Promise<FirmwareUpdateInfo> {
  try {
    return await invoke<FirmwareUpdateInfo>('check_debug_firmware_update');
  } catch (error) {
    throw friendlyError(error, '펌웨어 업데이트를 확인하지 못했습니다.');
  }
}

export async function flashLatestDebugFirmware(deviceId?: string): Promise<FirmwareFlashResult> {
  try {
    return await invoke<FirmwareFlashResult>('flash_latest_debug_firmware', { deviceId });
  } catch (error) {
    throw friendlyError(error, '펌웨어를 업데이트하지 못했습니다.');
  }
}

export async function recoveryFlashLatestDebugFirmware(): Promise<FirmwareFlashResult> {
  try {
    return await invoke<FirmwareFlashResult>('recovery_flash_latest_debug_firmware');
  } catch (error) {
    throw friendlyError(error, '복구 펌웨어 업데이트를 완료하지 못했습니다.');
  }
}

function friendlyError(error: unknown, fallback: string): Error {
  const detail = typeof error === 'string' ? error : error instanceof Error ? error.message : '';
  return new Error(detail ? `${fallback} ${detail}` : fallback);
}
