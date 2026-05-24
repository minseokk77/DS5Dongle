<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import appIcon from './assets/app-icon.svg';
  import Icon from './lib/Icon.svelte';
  import { i18n, type Lang, type StatusCode } from './lib/i18n';
  import DeviceCard from './lib/components/DeviceCard.svelte';
  import ConfigPanel from './lib/components/ConfigPanel.svelte';
  import ActionPanel from './lib/components/ActionPanel.svelte';
  import InputTester from './lib/components/InputTester.svelte';
  import SettingsModal from './lib/components/SettingsModal.svelte';
  import {
    applyConfig,
    checkDebugFirmwareUpdate,
    flashLatestDebugFirmware,
    listDevices,
    readConfig,
    readDeviceInfo,
    reconnectUsb,
    recoveryFlashLatestDebugFirmware,
    saveConfig,
    type BridgeConfig,
    type BridgeDevice,
    type DeviceInfo,
    type FirmwareCapabilities
  } from './lib/api';

  type ThemeMode = 'light' | 'dark' | 'system';
  type DiagnosticKind = 'info' | 'error';
  type UpdateStepCode = 'idle' | 'backup' | 'checking' | 'bootloader' | 'copying' | 'waiting' | 'restoring' | 'done' | 'latest' | 'failed';

  interface DiagnosticLog {
    id: number;
    time: string;
    kind: DiagnosticKind;
    message: string;
    data?: unknown;
  }

  interface ConfigMismatch {
    field: keyof BridgeConfig;
    expected: unknown;
    actual: unknown;
    tolerance?: number;
  }

  interface FirmwareCapability {
    key: string;
    label: string;
    supported: boolean;
    reason: string;
  }

  const defaultConfig: BridgeConfig = {
    config_version: 3,
    haptics_gain: 1,
    speaker_volume_percent: 0,
    inactive_time: 10,
    disable_inactive_disconnect: false,
    disable_pico_led: false,
    polling_rate_mode: 2,
    haptics_buffer_length: 64,
    controller_mode: 0,
    stick_calibration_enabled: false,
    left_stick_center_x: 0,
    left_stick_center_y: 0,
    left_stick_deadzone: 1,
    right_stick_center_x: 0,
    right_stick_center_y: 0,
    right_stick_deadzone: 1,
    left_stick_min_x: -1,
    left_stick_max_x: 1,
    left_stick_min_y: -1,
    left_stick_max_y: 1,
    right_stick_min_x: -1,
    right_stick_max_x: 1,
    right_stick_min_y: -1,
    right_stick_max_y: 1
  };
  const appVersion = '0.0.1.6';
  const releaseChannel = 'debug';
  const updateRepository = 'minseokk77/DS5Dongle';

  const emptyDeviceInfo = (): DeviceInfo => ({
    usb_vendor_name: '',
    usb_speed_class: '',
    rssi_strength_label: '',
    dongle_connected: false,
    controller_connected: false,
    battery_report_available: false,
    rssi_report_available: false,
    config_readable: false
  });

  let lang: Lang = 'ko';
  let themeMode: ThemeMode = 'system';
  let systemTheme: 'light' | 'dark' = 'dark';
  let devices: BridgeDevice[] = [];
  let selectedDeviceId = '';
  let config: BridgeConfig = { ...defaultConfig };
  let originalConfig: BridgeConfig | null = null;
  let deviceInfo: DeviceInfo = emptyDeviceInfo();
  let statusCode: StatusCode = 'ready';
  let statusOverride = '';
  let toastText = '';
  let toastKind: 'info' | 'error' = 'info';
  let toastTimer: number | undefined;
  let deviceInfoRefreshTimer: number | undefined;
  let devicePresenceRefreshTimer: number | undefined;
  let delayedInfoRefreshTimer: number | undefined;
  let isBusy = false;
  let errorText = '';
  let autoFirmwareUpdate = false;
  let showInputTesterModal = false;
  let showSettingsModal = false;
  let diagnosticLogs: DiagnosticLog[] = [];
  let updateStep: UpdateStepCode = 'idle';
  let showUpdateProgressModal = false;
  let lastUpdateError = '';

  $: text = i18n[lang];
  $: effectiveTheme = themeMode === 'system' ? systemTheme : themeMode;
  $: selectedDevice = devices.find((device) => device.id === selectedDeviceId) ?? null;
  $: isBridgeConnected = Boolean(selectedDeviceId && selectedDevice && deviceInfo.dongle_connected !== false && statusCode !== 'noDevice');
  $: isControllerConnected = Boolean(isBridgeConnected && deviceInfo.controller_connected);
  $: showControllerUi = isControllerConnected;
  $: isDirty = originalConfig ? JSON.stringify(originalConfig) !== JSON.stringify(config) : false;
  $: statusText = statusOverride || text.status[statusCode];
  $: bridgeStatusText = isBridgeConnected ? text.picoConnected : text.picoDisconnected;
  $: firmwareLabel = formatFirmwareVersion(deviceInfo.firmware_version);
  $: settingsFirmwareVersion = formatFirmwareVersion(deviceInfo.firmware_version);
  $: rssiLabel =
    deviceInfo.rssi === null || deviceInfo.rssi === undefined ? text.unknown : `${deviceInfo.rssi} dBm`;
  $: usbVendorLabel = deviceInfo.usb_vendor_name || '';
  $: usbSpeedLabel = localizeUsbSpeed(deviceInfo.usb_speed_class || '');
  $: rssiStatusLabel = localizeSignalStatus(deviceInfo.rssi_strength_label || '');
  $: batteryLevel = deviceInfo.battery_level !== undefined && deviceInfo.battery_level !== null ? deviceInfo.battery_level : null;
  $: isCharging = deviceInfo.is_charging !== undefined && deviceInfo.is_charging !== null ? deviceInfo.is_charging : null;
  $: deviceTitle = showControllerUi && selectedDevice
    ? `${selectedDevice.label.split(' - ')[0]} · ${selectedDevice.vendor_id.toString(16).padStart(4, '0').toUpperCase()}:${selectedDevice.product_id.toString(16).padStart(4, '0').toUpperCase()}`
    : text.controllerDisconnected;
  $: firmwareCapabilities = buildFirmwareCapabilities();
  $: updateStepText = updateStep === 'idle' ? '' : text.updateSteps[updateStep];
  $: updateProgress = updateStepProgress(updateStep);
  $: updateStepItems = buildUpdateStepItems();
  $: calibrationEnabled = Boolean(config.stick_calibration_enabled);
  $: leftCalibrationSummary = calibrationSummary(
    config.left_stick_center_x,
    config.left_stick_center_y,
    config.left_stick_deadzone
  );
  $: rightCalibrationSummary = calibrationSummary(
    config.right_stick_center_x,
    config.right_stick_center_y,
    config.right_stick_deadzone
  );

  onMount(() => {
    // 저장된 언어와 테마 설정을 먼저 반영합니다.
    const savedLang = localStorage.getItem('ds5:lang');
    if (savedLang === 'ko' || savedLang === 'en' || savedLang === 'zh') {
      lang = savedLang;
    }

    const savedTheme = localStorage.getItem('ds5:themeMode');
    if (savedTheme === 'light' || savedTheme === 'dark' || savedTheme === 'system') {
      themeMode = savedTheme;
    }

    const mediaQuery = window.matchMedia('(prefers-color-scheme: light)');
    const syncSystemTheme = () => {
      systemTheme = mediaQuery.matches ? 'light' : 'dark';
    };

    syncSystemTheme();
    mediaQuery.addEventListener('change', syncSystemTheme);
    autoFirmwareUpdate = localStorage.getItem('ds5:autoFirmwareUpdate') === 'true';
    
    // 장치 상태를 주기적으로 갱신합니다.
    void refreshDevices();
    deviceInfoRefreshTimer = window.setInterval(() => {
      void refreshDeviceInfoOnly();
    }, 2500);
    devicePresenceRefreshTimer = window.setInterval(() => {
      void syncDevicePresence();
    }, 3000);

    let unlisten: (() => void) | undefined;

    // USB 연결 변화 이벤트를 받아 장치 목록을 다시 읽습니다.
    const setupListener = async () => {
      try {
        unlisten = await listen<string[]>('device-list-changed', async () => {
          if (isBusy || statusCode === 'updating' || statusCode === 'updateChecking') {
            return;
          }
          await refreshDevices();
        });
      } catch (e) {
        // 백그라운드 리스너 등록 실패는 로그만 남깁니다.
      }
    };

    void setupListener();

    return () => {
      mediaQuery.removeEventListener('change', syncSystemTheme);
      if (deviceInfoRefreshTimer !== undefined) {
        window.clearInterval(deviceInfoRefreshTimer);
      }
      if (devicePresenceRefreshTimer !== undefined) {
        window.clearInterval(devicePresenceRefreshTimer);
      }
      if (delayedInfoRefreshTimer !== undefined) {
        window.clearTimeout(delayedInfoRefreshTimer);
      }
      if (unlisten) unlisten();
    };
  });

  function setStatus(nextStatus: StatusCode) {
    statusCode = nextStatus;
    statusOverride = '';
  }

  function showToast(message: string, kind: 'info' | 'error' = 'info') {
    toastText = message;
    toastKind = kind;
    if (toastTimer !== undefined) {
      window.clearTimeout(toastTimer);
    }
    toastTimer = window.setTimeout(() => {
      toastText = '';
      toastTimer = undefined;
    }, 4200);
  }

  function addLog(message: string, kind: DiagnosticKind = 'info', data?: unknown) {
    diagnosticLogs = [
      {
        id: Date.now(),
        time: new Date().toLocaleTimeString(),
        kind,
        message,
        ...(data === undefined ? {} : { data })
      },
      ...diagnosticLogs
    ].slice(0, 40);
  }

  function showError(message: string) {
    addLog(message, 'error');
    showToast(message, 'error');
  }

  // 사용자 선택값을 로컬 스토리지에 동기화합니다.
  function handleLangChange(nextLang: Lang) {
    lang = nextLang;
    localStorage.setItem('ds5:lang', nextLang);
  }

  function handleThemeChange(nextTheme: ThemeMode) {
    themeMode = nextTheme;
    localStorage.setItem('ds5:themeMode', nextTheme);
  }

  function setAutoFirmwareUpdate(enabled: boolean) {
    autoFirmwareUpdate = enabled;
    localStorage.setItem('ds5:autoFirmwareUpdate', String(enabled));
  }

  function formatFirmwareVersion(rawVersion?: string | null) {
    const normalized = normalizeFirmwareVersion(rawVersion);
    return normalized || text.unknown;
  }

  function normalizeFirmwareVersion(rawVersion?: string | null) {
    const raw = rawVersion?.trim();
    if (!raw) {
      return '';
    }

    const primary = raw.split(';')[0].trim();
    const semanticVersion = primary.match(/^v?\d+(?:\.\d+)+(?:[-+][A-Za-z0-9._-]+)?/i);
    return semanticVersion ? semanticVersion[0] : primary;
  }

  function isCurrentFirmware(updateVersion: string) {
    const current = normalizeFirmwareVersion(deviceInfo.firmware_version).toLowerCase();
    const latest = normalizeFirmwareVersion(updateVersion).toLowerCase();
    if (!current || !latest) {
      return false;
    }

    return current === latest || current.startsWith(`${latest}-`);
  }

  function buildFirmwareCapabilities(): FirmwareCapability[] {
    const hasBridge = Boolean(selectedDeviceId) || devices.length > 0 || statusCode !== 'noDevice';
    const caps = deviceInfo.capabilities;
    const legacyReason = hasBridge ? text.legacyFirmwareCapabilityUnknown : text.capRequiresBridge;
    const legacySupported = new Set(['vibration', 'trigger', 'bootloader']);
    const capability = (key: string, label: string, supported?: boolean): FirmwareCapability => ({
      key,
      label,
      supported: hasBridge && (caps ? Boolean(supported) : Boolean(supported) || legacySupported.has(key)),
      reason: caps ? text.capUnsupportedByFirmware : legacyReason
    });

    return [
      ...(caps?.supports_battery || deviceInfo.battery_report_available
        ? [capability('battery', text.capBattery, caps?.supports_battery ?? true)]
        : []),
      ...(caps?.supports_rssi || deviceInfo.rssi_report_available
        ? [capability('rssi', text.capRssi, caps?.supports_rssi ?? true)]
        : []),
      capability('vibration', text.capVibration, caps?.supports_vibration_test),
      capability('trigger', text.capAdaptiveTrigger, caps?.supports_adaptive_trigger),
      capability('bootloader', text.capBootloader, caps?.supports_bootloader_command),
      capability('calibration', text.stickCalibration, caps?.supports_stick_calibration ?? config.config_version >= 3),
      ...(caps?.supports_directional_stick_calibration
        ? [capability('directionalCalibration', text.capDirectionalCalibration, true)]
        : [])
    ];
  }

  function updateStepProgress(step: UpdateStepCode) {
    const order: UpdateStepCode[] = ['idle', 'backup', 'checking', 'bootloader', 'copying', 'waiting', 'restoring', 'done', 'latest', 'failed'];
    const index = order.indexOf(step);
    if (step === 'done' || step === 'latest') return 100;
    if (step === 'failed') return 100;
    return Math.max(0, Math.round((Math.max(index, 0) / 7) * 100));
  }

  function buildUpdateStepItems() {
    const order: UpdateStepCode[] = ['backup', 'checking', 'bootloader', 'copying', 'waiting', 'restoring', 'done'];
    const activeIndex = order.indexOf(updateStep);

    return order.map((step, index) => ({
      step,
      label: text.updateSteps[step],
      state:
        updateStep === 'failed' || updateStep === 'latest'
          ? 'waiting'
          : index < activeIndex || updateStep === 'done'
            ? 'done'
            : index === activeIndex
              ? 'active'
              : 'waiting'
    }));
  }

  function finishUpdateProgressModal() {
    if (updateStep !== 'done' && updateStep !== 'latest') {
      return;
    }

    window.setTimeout(() => {
      showUpdateProgressModal = false;
      updateStep = 'idle';
    }, 1400);
  }

  function calibrationSummary(centerX?: number, centerY?: number, deadzone?: number) {
    if (centerX === undefined || centerY === undefined || deadzone === undefined) return '';
    return `X ${centerX.toFixed(3)} / Y ${centerY.toFixed(3)} · ${deadzone.toFixed(1)}%`;
  }

  function configMismatchDetails(verified: BridgeConfig, expected: BridgeConfig): ConfigMismatch[] {
    const details: ConfigMismatch[] = [];
    const stickTolerance = 1 / 127 + 0.001;
    const percentTolerance = 0.051;
    const floatTolerance = 0.001;

    const checkExact = (field: keyof BridgeConfig) => {
      if (verified[field] !== expected[field]) {
        details.push({ field, expected: expected[field], actual: verified[field] });
      }
    };

    const checkBool = (field: keyof BridgeConfig) => {
      if (Boolean(verified[field]) !== Boolean(expected[field])) {
        details.push({ field, expected: Boolean(expected[field]), actual: Boolean(verified[field]) });
      }
    };

    const checkNumber = (field: keyof BridgeConfig, tolerance: number) => {
      const actual = Number(verified[field] ?? 0);
      const expectedValue = Number(expected[field] ?? 0);
      if (Math.abs(actual - expectedValue) > tolerance) {
        details.push({ field, expected: expectedValue, actual, tolerance });
      }
    };

    checkExact('config_version');
    checkNumber('haptics_gain', floatTolerance);
    checkNumber('speaker_volume_percent', floatTolerance);
    checkExact('inactive_time');
    checkExact('disable_inactive_disconnect');
    checkExact('disable_pico_led');
    checkExact('polling_rate_mode');
    checkExact('haptics_buffer_length');
    checkExact('controller_mode');
    checkBool('stick_calibration_enabled');
    checkNumber('left_stick_center_x', stickTolerance);
    checkNumber('left_stick_center_y', stickTolerance);
    checkNumber('left_stick_deadzone', percentTolerance);
    checkNumber('right_stick_center_x', stickTolerance);
    checkNumber('right_stick_center_y', stickTolerance);
    checkNumber('right_stick_deadzone', percentTolerance);
    checkNumber('left_stick_min_x', stickTolerance);
    checkNumber('left_stick_max_x', stickTolerance);
    checkNumber('left_stick_min_y', stickTolerance);
    checkNumber('left_stick_max_y', stickTolerance);
    checkNumber('right_stick_min_x', stickTolerance);
    checkNumber('right_stick_max_x', stickTolerance);
    checkNumber('right_stick_min_y', stickTolerance);
    checkNumber('right_stick_max_y', stickTolerance);

    return details;
  }

  async function saveAndVerify(deviceId: string, nextConfig: BridgeConfig) {
    await applyConfig(deviceId, nextConfig);
    await saveConfig(deviceId);
    const verified = await readConfig(deviceId);
    const mismatchDetails = configMismatchDetails(verified, nextConfig);
    if (mismatchDetails.length > 0) {
      addLog(text.settingsVerifyMismatch, 'error', { mismatches: mismatchDetails });
      mismatchDetails.forEach((detail) =>
        addLog(
          `${text.settingsVerifyMismatch}: ${String(detail.field)} expected=${String(detail.expected)}, actual=${String(detail.actual)}`,
          'error',
          detail
        )
      );
      showToast(text.settingsVerifyMismatch, 'error');
    }
    config = verified;
    originalConfig = structuredClone(verified);
  }

  function exportDiagnosticLogs() {
    const payload = {
      exported_at: new Date().toISOString(),
      app_version: appVersion,
      firmware_version: settingsFirmwareVersion,
      device_state: {
        dongle_connected: isBridgeConnected,
        controller_connected: isControllerConnected,
        gamepad_modal_open: showInputTesterModal,
        battery_report_available: Boolean(deviceInfo.battery_report_available),
        rssi_report_available: Boolean(deviceInfo.rssi_report_available),
        config_readable: Boolean(deviceInfo.config_readable)
      },
      last_update_step: updateStep,
      last_update_error: lastUpdateError,
      hid_errors: {
        firmware_error: deviceInfo.firmware_error ?? null,
        rssi_error: deviceInfo.rssi_error ?? null,
        capabilities_error: deviceInfo.capabilities_error ?? null
      },
      capabilities: deviceInfo.capabilities ?? null,
      logs: diagnosticLogs
    };
    const blob = new Blob([JSON.stringify(payload, null, 2)], { type: 'application/json;charset=utf-8' });
    const link = document.createElement('a');
    link.href = URL.createObjectURL(blob);
    link.download = `ds5-dongle-logs-${new Date().toISOString().replace(/[:.]/g, '-')}.json`;
    link.click();
    URL.revokeObjectURL(link.href);
  }

  function localizeUsbSpeed(raw: string) {
    if (!raw) return '';
    if (raw.includes('Full-Speed')) return text.usbFullSpeedStable;
    if (raw.includes('High-Speed')) return text.usbHighSpeed;
    return raw;
  }

  function localizeSignalStatus(raw: string) {
    if (!raw) return text.signalUnknown;
    if (raw.includes('우수') || raw.toLowerCase().includes('excellent')) return text.signalExcellent;
    if (raw.includes('보통') || raw.toLowerCase().includes('normal')) return text.signalNormal;
    return raw;
  }

  function sleep(ms: number) {
    return new Promise((resolve) => window.setTimeout(resolve, ms));
  }

  async function runTask(task: () => Promise<void>) {
    isBusy = true;
    errorText = '';
    try {
      await task();
    } catch (error) {
      errorText = error instanceof Error ? error.message : text.errorUnknown;
      lastUpdateError = errorText;
      showError(errorText);
    } finally {
      isBusy = false;
    }
  }

  async function refreshDevices() {
    await runTask(async () => {
      devices = await listDevices();
      if (!devices.length) {
        selectedDeviceId = '';
        originalConfig = null;
        deviceInfo = emptyDeviceInfo();
        setStatus('noDevice');
        return;
      }

      if (!devices.some((device) => device.id === selectedDeviceId)) {
        selectedDeviceId = devices[0].id;
      }

      await readAll();
      if (autoFirmwareUpdate) {
        await performFirmwareUpdate({ automatic: true });
      } else {
        setStatus('connected');
      }
    });
  }

  async function onDeviceChanged() {
    await runTask(readAll);
  }

  async function refreshDeviceInfoOnly() {
    if (!selectedDeviceId || isBusy || statusCode === 'updating' || statusCode === 'updateChecking') {
      return;
    }

    const deviceId = selectedDeviceId;
    try {
      const nextInfo = await readDeviceInfo(deviceId);
      if (selectedDeviceId === deviceId) {
        deviceInfo = nextInfo;
      }
    } catch (error) {
      addLog(`${text.logDeviceInfoFailed}: ${error instanceof Error ? error.message : text.errorUnknown}`, 'error');
      await syncDevicePresence();
    }
  }

  async function syncDevicePresence() {
    if (isBusy || statusCode === 'updating' || statusCode === 'updateChecking') {
      return;
    }

    try {
      const latestDevices = await listDevices();
      devices = latestDevices;
      if (!latestDevices.length || !latestDevices.some((device) => device.id === selectedDeviceId)) {
        selectedDeviceId = '';
        originalConfig = null;
        deviceInfo = emptyDeviceInfo();
        setStatus('noDevice');
      }
    } catch {
      addLog(text.logDevicePresenceFailed, 'error');
      selectedDeviceId = '';
      devices = [];
      originalConfig = null;
      deviceInfo = emptyDeviceInfo();
      setStatus('noDevice');
    }
  }

  function scheduleDeviceInfoRefresh() {
    if (delayedInfoRefreshTimer !== undefined) {
      window.clearTimeout(delayedInfoRefreshTimer);
    }
    delayedInfoRefreshTimer = window.setTimeout(() => {
      delayedInfoRefreshTimer = undefined;
      void refreshDeviceInfoOnly();
    }, 1200);
  }

  async function readAll() {
    if (!selectedDeviceId) return;
    setStatus('reading');

    const [infoResult, configResult] = await Promise.allSettled([
      readDeviceInfo(selectedDeviceId),
      readConfig(selectedDeviceId)
    ]);

    if (infoResult.status === 'fulfilled') {
      deviceInfo = infoResult.value;
    } else {
      addLog(`${text.logDeviceInfoFailed}: ${infoResult.reason}`, 'error');
    }
    if (configResult.status === 'fulfilled') {
      config = configResult.value;
      originalConfig = structuredClone(configResult.value);
    } else {
      throw configResult.reason;
    }
    setStatus('connected');
    scheduleDeviceInfoRefresh();
  }

  async function onRead() {
    await runTask(readAll);
  }

  async function onSave() {
    if (!selectedDeviceId) return;
    await runTask(async () => {
      await saveAndVerify(selectedDeviceId, config);
      setStatus('saved');
    });
  }

  async function onReconnect() {
    if (!selectedDeviceId) return;
    await runTask(async () => {
      await reconnectUsb(selectedDeviceId);
      setStatus('reconnectSent');
      selectedDeviceId = '';
    });
  }

  async function onFirmwareUpdate() {
    showUpdateProgressModal = true;
    updateStep = 'checking';
    await runTask(async () => {
      try {
        await performFirmwareUpdate({ automatic: false });
      } catch (error) {
        updateStep = 'failed';
        throw error;
      }
    });
    finishUpdateProgressModal();
  }

  async function onRecoveryFirmwareUpdate() {
    showUpdateProgressModal = true;
    await runTask(async () => {
      updateStep = 'copying';
      setStatus('updating');
      let result;
      try {
        result = await recoveryFlashLatestDebugFirmware();
      } catch (error) {
        updateStep = 'failed';
        throw error;
      }
      updateStep = 'done';
      addLog(`${text.recoveryFirmwareUpdate}: ${result.version} / ${result.asset_name}`);
      showToast(`${text.status.updated}: ${result.version}`);
      await syncDevicePresence();
    });
    finishUpdateProgressModal();
  }

  async function onCalibrationApply(
    side: 'left' | 'right',
    result: { centerX: number; centerY: number; deadzone: number; minX: number; maxX: number; minY: number; maxY: number }
  ) {
    const nextConfig = {
      ...config,
      config_version: 3,
      stick_calibration_enabled: true,
      ...(side === 'left'
        ? {
            left_stick_center_x: result.centerX,
            left_stick_center_y: result.centerY,
            left_stick_deadzone: result.deadzone,
            left_stick_min_x: result.minX,
            left_stick_max_x: result.maxX,
            left_stick_min_y: result.minY,
            left_stick_max_y: result.maxY
          }
        : {
            right_stick_center_x: result.centerX,
            right_stick_center_y: result.centerY,
            right_stick_deadzone: result.deadzone,
            right_stick_min_x: result.minX,
            right_stick_max_x: result.maxX,
            right_stick_min_y: result.minY,
            right_stick_max_y: result.maxY
          })
    };
    config = nextConfig;

    if (selectedDeviceId) {
      await saveAndVerify(selectedDeviceId, nextConfig);
      addLog(text.calibrationDone);
    }
  }

  async function onCalibrationClear(side: 'left' | 'right') {
    const nextConfig = {
      ...config,
      stick_calibration_enabled: side === 'left'
        ? Boolean(config.right_stick_center_x || config.right_stick_center_y || (config.right_stick_deadzone ?? 1) > 1)
        : Boolean(config.left_stick_center_x || config.left_stick_center_y || (config.left_stick_deadzone ?? 1) > 1),
      ...(side === 'left'
        ? {
            left_stick_center_x: 0,
            left_stick_center_y: 0,
            left_stick_deadzone: 1,
            left_stick_min_x: -1,
            left_stick_max_x: 1,
            left_stick_min_y: -1,
            left_stick_max_y: 1
          }
        : {
            right_stick_center_x: 0,
            right_stick_center_y: 0,
            right_stick_deadzone: 1,
            right_stick_min_x: -1,
            right_stick_max_x: 1,
            right_stick_min_y: -1,
            right_stick_max_y: 1
          })
    };
    config = nextConfig;
    if (selectedDeviceId) {
      await saveAndVerify(selectedDeviceId, nextConfig);
      addLog(text.calibrationCleared);
    }
  }

  async function waitForBridgeReconnect(preferredDeviceId: string, timeoutMs = 35000) {
    const deadline = Date.now() + timeoutMs;

    while (Date.now() < deadline) {
      await sleep(750);

      try {
        const latestDevices = await listDevices();
        devices = latestDevices;

        const preferredDevice = latestDevices.find((device) => device.id === preferredDeviceId);
        const nextDevice = preferredDevice ?? latestDevices[0];
        if (nextDevice) {
          selectedDeviceId = nextDevice.id;
          return nextDevice.id;
        }
      } catch {
        // 부트로더로 전환되는 짧은 구간에서는 HID 조회가 실패할 수 있습니다.
      }
    }

    return '';
  }

  async function verifyFirmwareAfterUpdate(deviceId: string, expectedVersion: string) {
    if (!deviceId) {
      return;
    }

    const summary: {
      expected_version: string;
      firmware_version?: string | null;
      config_version?: number;
      capabilities?: FirmwareCapabilities | null;
      errors: string[];
    } = {
      expected_version: expectedVersion,
      errors: []
    };

    try {
      const info = await readDeviceInfo(deviceId);
      deviceInfo = info;
      summary.firmware_version = info.firmware_version;
      summary.capabilities = info.capabilities ?? null;
      if (!isCurrentFirmware(expectedVersion)) {
        summary.errors.push(`firmware_version=${info.firmware_version ?? text.unknown}`);
      }
      if (info.capabilities_error) {
        summary.errors.push(`capabilities=${info.capabilities_error}`);
      }
    } catch (error) {
      summary.errors.push(`device_info=${error instanceof Error ? error.message : text.errorUnknown}`);
    }

    try {
      const verifiedConfig = await readConfig(deviceId);
      config = verifiedConfig;
      originalConfig = structuredClone(verifiedConfig);
      summary.config_version = verifiedConfig.config_version;
    } catch (error) {
      summary.errors.push(`config=${error instanceof Error ? error.message : text.errorUnknown}`);
    }

    if (summary.errors.length > 0) {
      addLog(`${text.status.updated}: ${text.viewLogs}`, 'error', summary);
    } else {
      addLog(`${text.status.updated}: ${expectedVersion}`, 'info', summary);
    }
  }

  async function restoreConfigAfterFirmwareUpdate(preservedConfig: BridgeConfig | null, previousDeviceId: string, expectedVersion: string) {
    if (!preservedConfig || !previousDeviceId) {
      return false;
    }

    updateStep = 'waiting';
    statusOverride = text.settingsRestoreWaiting;
    const restoredDeviceId = await waitForBridgeReconnect(previousDeviceId);
    if (!restoredDeviceId) {
      statusOverride = '';
      updateStep = 'failed';
      showError(text.settingsRestoreReconnectFailed);
      return false;
    }

    updateStep = 'restoring';
    statusOverride = text.settingsRestoring;
    const deadline = Date.now() + 12000;
    while (Date.now() < deadline) {
      try {
        await saveAndVerify(restoredDeviceId, preservedConfig);
        await readAll();
        await verifyFirmwareAfterUpdate(restoredDeviceId, expectedVersion);
        statusOverride = '';
        addLog(text.settingsRestored);
        return true;
      } catch (error) {
        addLog(`${text.settingsRestoreFailed}: ${error instanceof Error ? error.message : text.errorUnknown}`, 'error');
        await sleep(800);
      }
    }

    statusOverride = '';
    updateStep = 'failed';
    showError(text.settingsRestoreFailed);
    return false;
  }

  async function performFirmwareUpdate(options: { automatic: boolean }) {
    const previousStatus = statusCode;
    const previousOverride = statusOverride;
    const previousDeviceId = selectedDeviceId;
    const preservedConfig = selectedDeviceId ? structuredClone(config) : null;
    updateStep = preservedConfig ? 'backup' : 'checking';
    if (!options.automatic) {
      setStatus('updateChecking');
    }

    if (preservedConfig) {
      addLog(text.updateBackupComplete);
    }

    updateStep = 'checking';
    const update = await checkDebugFirmwareUpdate();
    if (isCurrentFirmware(update.version)) {
      updateStep = 'latest';
      if (options.automatic) {
        statusCode = previousStatus;
        statusOverride = previousOverride;
        updateStep = 'idle';
        return;
      }

      setStatus('upToDate');
      showToast(`${text.status.upToDate}: ${update.version}`);
      addLog(`${text.status.upToDate}: ${update.version}`);
      return;
    }

    if (!options.automatic) {
      showToast(`${text.status.updateReady}: ${update.version} / ${update.asset_name}`);
      setStatus('updating');
    }

    updateStep = selectedDeviceId ? 'bootloader' : 'copying';
    statusOverride = text.updateSteps[updateStep];
    updateStep = 'copying';
    const result = await flashLatestDebugFirmware(selectedDeviceId || undefined);
    if (result.expected_bytes !== result.copied_bytes || !result.drive_disappeared) {
      addLog(
        `${text.updateCopyVerifyWarning}: ${result.copied_bytes}/${result.expected_bytes}, driveGone=${result.drive_disappeared}`,
        'error'
      );
    }
    addLog(`${text.status.updated}: ${result.version} / ${result.asset_name}`);
    const shouldRestoreSettings = Boolean(preservedConfig && previousDeviceId);
    const settingsRestored = await restoreConfigAfterFirmwareUpdate(preservedConfig, previousDeviceId, result.version);
    if (!shouldRestoreSettings && selectedDeviceId) {
      await verifyFirmwareAfterUpdate(selectedDeviceId, result.version);
    }
    if (shouldRestoreSettings && !settingsRestored) {
      if (options.automatic) {
        statusCode = previousStatus;
        statusOverride = previousOverride;
        return;
      }

      setStatus('updated');
      return;
    }

    updateStep = 'done';
    if (options.automatic) {
      statusCode = previousStatus;
      statusOverride = previousOverride;
      showToast(
        settingsRestored
          ? `${text.status.updated}: ${result.version} / ${text.settingsRestored}`
          : `${text.status.updated}: ${result.version} / ${result.asset_name}`
      );
      return;
    }

    setStatus('updated');
    showToast(
      settingsRestored
        ? `${text.status.updated}: ${result.version} / ${text.settingsRestored}`
        : `${text.status.updated}: ${result.version} / ${result.asset_name}`
    );
  }

  function resetToDefaults() {
    config = { ...defaultConfig };
    setStatus('defaults');
    addLog(text.resetDefaults);
  }
</script>

<main class:theme-light={effectiveTheme === 'light'} class:theme-dark={effectiveTheme === 'dark'} class="app-shell" class:modal-open={showInputTesterModal || showSettingsModal || showUpdateProgressModal}>
  <header class="topbar">
    <div class="brand">
      <div class="brand-icon">
        <img src={appIcon} alt="" />
      </div>
      <h1>DS5 Dongle Config</h1>
    </div>
    
    <div class="toolbar compact-toolbar">
      <div class:connected={showControllerUi} class="status-pill">
        <span><Icon name="check" size={10} /></span>
        {showControllerUi ? text.controllerConnected : text.controllerDisconnected}
      </div>
      <button class="settings-btn" type="button" onclick={() => (showSettingsModal = true)} aria-label={text.settings} title={text.settings}>
        <Icon name="settings" size={17} />
      </button>
    </div>
  </header>

  {#if toastText}
    <div class:error={toastKind === 'error'} class="toast" role="status">
      <span>{toastText}</span>
      <button type="button" onclick={() => (toastText = '')}>×</button>
    </div>
  {/if}

  <!-- 장치 상태와 연결 카드 -->
  <DeviceCard
    {devices}
    bind:selectedDeviceId
    {isBusy}
    isControllerConnected={showControllerUi}
    {deviceTitle}
    {firmwareLabel}
    {rssiLabel}
    rssi={deviceInfo.rssi}
    {usbVendorLabel}
    {usbSpeedLabel}
    {rssiStatusLabel}
    {batteryLevel}
    {isCharging}
    {text}
    onRefreshDevices={refreshDevices}
    onDeviceChanged={onDeviceChanged}
    onOpenInputTester={() => (showInputTesterModal = true)}
  />

  <div class="main-grid">
    <!-- 설정 보드 -->
    <ConfigPanel
      bind:config
      {text}
      {showToast}
      onLog={addLog}
    />

    <!-- 작업 패널 -->
      <ActionPanel
      isConnected={isBridgeConnected}
      {isBusy}
      {isDirty}
      statusText={bridgeStatusText}
      {updateStepText}
      {text}
      onRead={onRead}
      onSave={onSave}
      onReconnect={onReconnect}
      onFirmwareUpdate={onFirmwareUpdate}
    />
  </div>

  <!-- 실시간 입력 테스트 보드 -->
  <InputTester
    {text}
    deviceId={selectedDeviceId}
    capabilities={firmwareCapabilities}
    onLog={addLog}
    onCalibrationApply={onCalibrationApply}
    onCalibrationClear={onCalibrationClear}
    bind:isOpen={showInputTesterModal}
  />
  <SettingsModal
    isOpen={showSettingsModal}
    {lang}
    {themeMode}
    isConnected={showControllerUi}
    {autoFirmwareUpdate}
    {appVersion}
    firmwareVersion={settingsFirmwareVersion}
    {releaseChannel}
    {updateRepository}
    configVersion={`v${config.config_version}`}
    {calibrationEnabled}
    {leftCalibrationSummary}
    {rightCalibrationSummary}
    capabilities={firmwareCapabilities}
    logs={diagnosticLogs}
    {text}
    onClose={() => (showSettingsModal = false)}
    onLangChange={handleLangChange}
    onThemeChange={handleThemeChange}
    onAutoFirmwareUpdateChange={setAutoFirmwareUpdate}
    onResetDefaults={resetToDefaults}
    onRecoveryFirmwareUpdate={onRecoveryFirmwareUpdate}
    onExportLogs={exportDiagnosticLogs}
  />

  {#if showUpdateProgressModal}
    <div class="update-progress-overlay" role="presentation">
      <div class="update-progress-modal" role="status" aria-label={text.firmwareUpdateProgress}>
        <div class="update-progress-head">
          <h2><Icon name="download" size={18} /> {text.firmwareUpdateProgress}</h2>
          <p>{text.firmwareUpdateProgressDesc}</p>
        </div>
        <div class="update-progress-current">
          <strong>{updateStepText || text.updateSteps.checking}</strong>
          <span>{updateProgress}%</span>
        </div>
        <div class="progress-track"><span style={`width: ${updateProgress}%`}></span></div>
        <div class="update-progress-asset">
          {settingsFirmwareVersion} / ds5-bridge-debug-{settingsFirmwareVersion}.uf2
        </div>
        <div class="update-step-grid">
          {#each updateStepItems as item}
            <div class={`update-step-card ${item.state}`}>
              <span></span>
              <strong>{item.label}</strong>
            </div>
          {/each}
        </div>
        {#if updateStep === 'failed'}
          <div class="update-progress-actions">
            <button type="button" onclick={onFirmwareUpdate}>{text.retryUpdate}</button>
            <button type="button" onclick={onRecoveryFirmwareUpdate}>{text.recoveryFirmwareUpdate}</button>
            <button type="button" onclick={() => (showSettingsModal = true)}>{text.viewLogs}</button>
          </div>
        {/if}
        <p class="update-progress-footnote">{text.firmwareUpdateProgressFootnote}</p>
      </div>
    </div>
  {/if}

  {#if showInputTesterModal || showSettingsModal || showUpdateProgressModal}
    <!-- 모달이 열린 동안 배경 클릭을 차단합니다. -->
    <div 
      class="global-click-blocker" 
      onclick={(e) => { e.preventDefault(); e.stopPropagation(); }}
      onkeydown={(e) => { e.preventDefault(); e.stopPropagation(); }}
      role="none"
    ></div>
  {/if}
</main>
