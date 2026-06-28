import { invoke } from '@tauri-apps/api/core';

export type PollingRateMode = 0 | 1 | 2;
export type ControllerMode = 0 | 1 | 2;

export interface BridgeDevice {
  id: string;
  label: string;
  vendor_id: number;
  product_id: number;
  serial_number?: string | null;
}

export interface BridgeConfig {
  config_version: number;
  haptics_gain: number;
  speaker_volume_percent: number; // old?
  speaker_volume: number;
  headset_volume: number;
  speaker_gain: number;
  inactive_time: number;
  disable_inactive_disconnect: boolean;
  disable_pico_led: boolean;
  polling_rate_mode: PollingRateMode;
  audio_buffer_length: number;
  controller_mode: ControllerMode;
  disable_mic: boolean;
  disable_speaker: boolean;
  enable_wake: boolean;
  trigger_reduce: number;
  enable_usb_sn: boolean;
  ps_shortcut_enabled: boolean;
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

export const defaultConfig: BridgeConfig = {
  config_version: 5,
  haptics_gain: 1,
  speaker_volume: 50,
  speaker_volume_percent: 50,
  headset_volume: 50,
  speaker_gain: 0,
  inactive_time: 10,
  disable_inactive_disconnect: false,
  disable_pico_led: false,
  polling_rate_mode: 2,
  audio_buffer_length: 64,
  controller_mode: 0,
  enable_usb_sn: false,
  ps_shortcut_enabled: false,
  disable_mic: false,
  disable_speaker: false,
  enable_wake: false,
  trigger_reduce: 0,
  stick_calibration_enabled: false,
  left_stick_center_x: 0,
  left_stick_center_y: 0,
  left_stick_deadzone: 0,
  right_stick_center_x: 0,
  right_stick_center_y: 0,
  right_stick_deadzone: 0,
  left_stick_min_x: -1,
  left_stick_max_x: 1,
  left_stick_min_y: -1,
  left_stick_max_y: 1,
  right_stick_min_x: -1,
  right_stick_max_x: 1,
  right_stick_min_y: -1,
  right_stick_max_y: 1
};

export interface DeviceInfo {
  firmware_version?: string | null;
  rssi?: number | null;
  firmware_error?: string | null;
  rssi_error?: string | null;
  usb_vendor_name: string;
  usb_speed_class: string;
  rssi_strength_label: string;
  battery_level?: number | null;
  is_charging?: boolean | null;
  controller_connected?: boolean;
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

export async function applyConfig(deviceId: string, config: BridgeConfig): Promise<void> {
  try {
    const safeConfig = { ...defaultConfig, ...config };
    // JSON.stringify will drop undefined values, but we want to ensure they are at least the default
    for (const key in safeConfig) {
      if ((safeConfig as any)[key] === undefined || (safeConfig as any)[key] === null) {
        (safeConfig as any)[key] = (defaultConfig as any)[key];
      }
    }
    // Backward compatibility if Svelte only sent speaker_volume_percent
    if (safeConfig.speaker_volume === undefined || safeConfig.speaker_volume === null) {
      safeConfig.speaker_volume = safeConfig.speaker_volume_percent ?? 50;
    }
    await invoke('apply_config', { deviceId, config: safeConfig });
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

export interface AppUpdateInfo {
  version: string;
  asset_name: string;
  download_url: string;
  body: string;
}

export async function checkAppUpdate(currentVersion: string): Promise<AppUpdateInfo | null> {
  try {
    return await invoke<AppUpdateInfo | null>('check_app_update', { currentVersion });
  } catch (error) {
    throw friendlyError(error, '앱 업데이트를 확인하지 못했습니다.');
  }
}

export async function installAppUpdate(downloadUrl: string): Promise<void> {
  try {
    await invoke('install_app_update', { downloadUrl });
  } catch (error) {
    throw friendlyError(error, '앱 업데이트를 설치하지 못했습니다.');
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

export async function recoveryFlashLatestDebugFirmware(deviceId?: string): Promise<FirmwareFlashResult> {
  try {
    return await invoke<FirmwareFlashResult>('recovery_flash_latest_debug_firmware', { deviceId });
  } catch (error) {
    throw friendlyError(error, '복구 펌웨어 업데이트를 완료하지 못했습니다.');
  }
}

function friendlyError(error: unknown, fallback: string): Error {
  const detail = typeof error === 'string' ? error : error instanceof Error ? error.message : '';
  return new Error(detail ? `${fallback} ${detail}` : fallback);
}
