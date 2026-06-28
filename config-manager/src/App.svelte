<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { getVersion } from '@tauri-apps/api/app';
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
    checkAppUpdate,
    installAppUpdate,
    defaultConfig,
    type AppUpdateInfo,
    type BridgeConfig,
    type BridgeDevice,
    type DeviceInfo
  } from './lib/api';

  type ThemeMode = 'light' | 'dark' | 'system';
  type DiagnosticKind = 'info' | 'error';
  type UpdateStepCode = 'idle' | 'backup' | 'checking' | 'bootloader' | 'copying' | 'waiting' | 'restoring' | 'done' | 'latest' | 'failed';

  interface DiagnosticLog {
    id: number;
    time: string;
    kind: DiagnosticKind;
    message: string;
  }

  interface FirmwareCapability {
    key: string;
    label: string;
    supported: boolean;
    reason: string;
  }

  let appVersion = $state('');
  const releaseChannel = 'debug';
  const updateRepository = 'minseokk77/DS5Dongle';

  let lang: Lang = $state('ko');
  let themeMode: ThemeMode = $state('system');
  let systemTheme: 'light' | 'dark' = $state('dark');
  let devices: BridgeDevice[] = $state([]);
  let selectedDeviceId = $state('');
  let config: BridgeConfig = $state({ ...defaultConfig });
  let originalConfig: BridgeConfig | null = $state(null);
  let deviceInfo: DeviceInfo = $state({
    usb_vendor_name: '',
    usb_speed_class: '',
    rssi_strength_label: ''
  });
  let appUpdate: AppUpdateInfo | null = $state(null);
  let appUpdateRunning = $state(false);
  let statusCode: StatusCode = $state('ready');
  let statusOverride = $state('');
  let toastText = $state('');
  let toastKind: 'info' | 'error' = $state('info');
  let toastTimer: number | undefined = $state(undefined);
  let deviceInfoRefreshTimer: number | undefined = $state(undefined);
  let devicePresenceRefreshTimer: number | undefined = $state(undefined);
  let delayedInfoRefreshTimer: number | undefined = $state(undefined);
  let isBusy = $state(false);
  let errorText = $state('');
  let autoFirmwareUpdate = $state(false);
  let showInputTesterModal = $state(false);
  let showSettingsModal = $state(false);
  let diagnosticLogs: DiagnosticLog[] = $state([]);
  let updateStep: UpdateStepCode = $state('idle');

  let text = $derived(i18n[lang]);
  let effectiveTheme = $derived(themeMode === 'system' ? systemTheme : themeMode);
  let selectedDevice = $derived(devices.find((device) => device.id === selectedDeviceId) ?? null);
  let isBridgeConnected = $derived(Boolean(selectedDeviceId && selectedDevice && statusCode !== 'noDevice'));
  let isControllerConnected = $derived(Boolean(isBridgeConnected && deviceInfo.controller_connected));
  let showControllerUi = $derived(Boolean(isControllerConnected && deviceInfo.battery_level !== undefined && deviceInfo.battery_level !== null));
  let isDirty = $derived(originalConfig ? JSON.stringify(originalConfig) !== JSON.stringify(config) : false);
  let statusText = $derived(statusOverride || text.status[statusCode]);
  let bridgeStatusText = $derived(isBridgeConnected ? text.picoConnected : text.picoDisconnected);
  let actionPanelStatusText = $derived((statusCode === 'idle' || statusCode === 'noDevice' || statusCode === 'connected') && !statusOverride ? bridgeStatusText : statusText);
  let firmwareLabel = $derived(formatFirmwareVersion(deviceInfo.firmware_version));
  let settingsFirmwareVersion = $derived(formatFirmwareVersion(deviceInfo.firmware_version));
  let rssiLabel = $derived(
    deviceInfo.rssi === null || deviceInfo.rssi === undefined ? text.unknown : `${deviceInfo.rssi} dBm`);
  let usbVendorLabel = $derived(deviceInfo.usb_vendor_name || '');
  let usbSpeedLabel = $derived(localizeUsbSpeed(deviceInfo.usb_speed_class || ''));
  let rssiStatusLabel = $derived(localizeSignalStatus(deviceInfo.rssi_strength_label || ''));
  let batteryLevel = $derived(deviceInfo.battery_level !== undefined && deviceInfo.battery_level !== null ? deviceInfo.battery_level : null);
  let isCharging = $derived(deviceInfo.is_charging !== undefined && deviceInfo.is_charging !== null ? deviceInfo.is_charging : null);
  let deviceTitle = $derived(isControllerConnected && selectedDevice
    ? `${selectedDevice.label.split(' - ')[0]} · ${selectedDevice.vendor_id.toString(16).padStart(4, '0').toUpperCase()}:${selectedDevice.product_id.toString(16).padStart(4, '0').toUpperCase()}`
    : text.disconnected);
  let firmwareCapabilities = $derived(buildFirmwareCapabilities());
  let updateStepText = $derived(updateStep === 'idle' ? '' : text.updateSteps[updateStep]);
  let updateProgress = $derived(updateStepProgress(updateStep));
  let calibrationEnabled = $derived(Boolean(config.stick_calibration_enabled));
  let leftCalibrationSummary = $derived(calibrationSummary(
    config.left_stick_center_x,
    config.left_stick_center_y,
    config.left_stick_deadzone
  ));
  let rightCalibrationSummary = $derived(calibrationSummary(
    config.right_stick_center_x,
    config.right_stick_center_y,
    config.right_stick_deadzone
  ));

  async function handleAppUpdate() {
    if (!appUpdate) return;
    appUpdateRunning = true;
    showToast('앱 업데이트 설치 파일을 다운로드하고 있습니다...', 'info');
    try {
      await installAppUpdate(appUpdate.download_url);
    } catch (err) {
      showToast('앱 업데이트 실패: ' + (err instanceof Error ? err.message : String(err)), 'error');
      appUpdateRunning = false;
    }
  }

  onMount(() => {
    getVersion().then(v => {
      let displayVersion = v;
      if (v.startsWith('0.0.') && v.length >= 6) {
        // e.g. 0.0.214 -> 0.0.2.14
        displayVersion = `0.0.${v[4]}.${v.slice(5)}`;
      } else if (v.startsWith('0.0.') && v.length === 5) {
        // e.g. 0.0.14 -> 0.0.1.4
        displayVersion = `0.0.${v[4]}.${v.slice(5)}`;
      }
      appVersion = displayVersion;
      checkAppUpdate(v).then(info => {
        appUpdate = info;
        if (info) {
          addLog(`앱 업데이트 가능: ${info.version}`, 'info');
          showToast(`새로운 앱 버전(${info.version})이 있습니다. 설정에서 업데이트를 설치하세요.`, 'info');
        }
      }).catch(err => {
        addLog(`앱 업데이트 확인 실패: ${err.message}`, 'error');
      });
    });

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

  function addLog(message: string, kind: DiagnosticKind = 'info') {
    diagnosticLogs = [
      {
        id: Date.now(),
        time: new Date().toLocaleTimeString(),
        kind,
        message
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
    return [
      {
        key: 'vibration',
        label: text.capVibration,
        supported: hasBridge,
        reason: text.capRequiresBridge
      },
      {
        key: 'trigger',
        label: text.capAdaptiveTrigger,
        supported: hasBridge,
        reason: text.capRequiresBridge
      },
      {
        key: 'bootloader',
        label: text.capBootloader,
        supported: hasBridge,
        reason: text.capRequiresBridge
      },
      {
        key: 'calibration',
        label: text.stickCalibration,
        supported: hasBridge && config.config_version >= 3,
        reason: hasBridge ? text.calibrationNoInfo : text.capRequiresBridge
      }
    ];
  }

  function updateStepProgress(step: UpdateStepCode) {
    const order: UpdateStepCode[] = ['idle', 'backup', 'checking', 'bootloader', 'copying', 'waiting', 'restoring', 'done', 'latest', 'failed'];
    const index = order.indexOf(step);
    if (step === 'done' || step === 'latest') return 100;
    if (step === 'failed') return 100;
    return Math.max(0, Math.round((Math.max(index, 0) / 7) * 100));
  }

  function calibrationSummary(centerX?: number, centerY?: number, deadzone?: number) {
    if (centerX === undefined || centerY === undefined || deadzone === undefined) return '';
    return `X ${centerX.toFixed(3)} / Y ${centerY.toFixed(3)} · ${deadzone.toFixed(1)}%`;
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
      showError(errorText);
      if (updateStep !== 'idle') {
        updateStep = 'failed';
      }
    } finally {
      if (updateStep !== 'idle') {
        await sleep(1500);
        updateStep = 'idle';
      }
      isBusy = false;
    }
  }

  async function refreshDevices() {
    await runTask(async () => {
      setStatus('reading');
      let latestDevices = await listDevices();
      
      // 듀얼센스 연결/해제 시 USB가 잠시 재연결되며 목록이 비는 현상(flicker) 방지
      if (!latestDevices.length && selectedDeviceId) {
        await sleep(1200);
        latestDevices = await listDevices();
      }

      devices = latestDevices;
      if (!devices.length) {
        selectedDeviceId = '';
        originalConfig = null;
        deviceInfo = {};
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
      let latestDevices = await listDevices();
      
      // 듀얼센스 연결/해제 시 USB가 잠시 재연결되며 목록이 비는 현상(flicker) 방지
      if (!latestDevices.length && selectedDeviceId) {
        await sleep(1200);
        latestDevices = await listDevices();
      }

      devices = latestDevices;
      if (!latestDevices.length) {
        selectedDeviceId = '';
        originalConfig = null;
        deviceInfo = {
          usb_vendor_name: '',
          usb_speed_class: '',
          rssi_strength_label: ''
        };
        setStatus('noDevice');
      } else if (!selectedDeviceId || !latestDevices.some((device) => device.id === selectedDeviceId)) {
        // 이전에 선택된 장치가 사라졌지만 다른 장치가 남아있는 경우 자동 선택
        selectedDeviceId = latestDevices[0].id;
        // 새로 선택된 장치의 정보를 읽어옵니다.
        void readAll().then(() => {
          if (statusCode === 'noDevice') setStatus('connected');
        });
      }
    } catch {
      addLog(text.logDevicePresenceFailed, 'error');
      selectedDeviceId = '';
      devices = [];
      originalConfig = null;
      deviceInfo = {
        usb_vendor_name: '',
        usb_speed_class: '',
        rssi_strength_label: ''
      };
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
      originalConfig = JSON.parse(JSON.stringify(configResult.value));
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
      await applyConfig(selectedDeviceId, config);
      await saveConfig(selectedDeviceId);
      originalConfig = JSON.parse(JSON.stringify(config));
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
    await runTask(() => performFirmwareUpdate({ automatic: false }));
  }

  async function onRecoveryFirmwareUpdate() {
    await runTask(async () => {
      updateStep = 'copying';
      setStatus('updating');
      const result = await recoveryFlashLatestDebugFirmware(selectedDeviceId || undefined);
      updateStep = 'done';
      addLog(`${text.recoveryFirmwareUpdate}: ${result.version} / ${result.asset_name}`);
      showToast(`${text.status.updated}: ${result.version}`);
      await syncDevicePresence();
    });
  }

  async function onCalibrationApply(
    side: 'left' | 'right',
    result: { centerX: number; centerY: number; deadzone: number }
  ) {
    config = {
      ...config,
      config_version: 5,
      stick_calibration_enabled: true,
      ...(side === 'left'
        ? {
            left_stick_center_x: result.centerX,
            left_stick_center_y: result.centerY,
            left_stick_deadzone: result.deadzone
          }
        : {
            right_stick_center_x: result.centerX,
            right_stick_center_y: result.centerY,
            right_stick_deadzone: result.deadzone
          })
    };

    if (selectedDeviceId) {
      await applyConfig(selectedDeviceId, config);
      await saveConfig(selectedDeviceId);
      originalConfig = JSON.parse(JSON.stringify(config));
      addLog(text.calibrationDone);
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

  async function restoreConfigAfterFirmwareUpdate(preservedConfig: BridgeConfig | null, previousDeviceId: string) {
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
        await applyConfig(restoredDeviceId, preservedConfig);
        await saveConfig(restoredDeviceId);
        config = JSON.parse(JSON.stringify(preservedConfig));
        originalConfig = JSON.parse(JSON.stringify(preservedConfig));
        await readAll();
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
    const preservedConfig = selectedDeviceId ? JSON.parse(JSON.stringify(config)) : null;
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
      updateStep = 'idle';
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
    const result = await flashLatestDebugFirmware(selectedDeviceId || undefined);
    updateStep = 'copying';
    addLog(`${text.status.updated}: ${result.version} / ${result.asset_name}`);
    const shouldRestoreSettings = Boolean(preservedConfig && previousDeviceId);
    const settingsRestored = await restoreConfigAfterFirmwareUpdate(preservedConfig, previousDeviceId);
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

  async function minimizeWindow() {
    await getCurrentWindow().minimize();
  }
  async function closeWindow() {
    await getCurrentWindow().hide();
  }
</script>

<main class:theme-light={effectiveTheme === 'light'} class:theme-dark={effectiveTheme === 'dark'} class="app-shell" class:modal-open={showInputTesterModal || showSettingsModal}>

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
    {isBridgeConnected}
    {isControllerConnected}
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
    onOpenSettings={() => (showSettingsModal = true)}
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
      statusText={actionPanelStatusText}
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
    bind:isOpen={showInputTesterModal}
  />
  <SettingsModal
    isOpen={showSettingsModal}
    {lang}
    {themeMode}
    isConnected={isBridgeConnected}
    {autoFirmwareUpdate}
    appVersion={`v${appVersion}`}
    firmwareVersion={settingsFirmwareVersion}
    {releaseChannel}
    {updateRepository}
    {appUpdate}
    {appUpdateRunning}
    onAppUpdate={handleAppUpdate}
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
  />

  {#if updateStep !== 'idle'}
    <div class="update-progress-overlay" role="presentation">
      <div class="update-progress-modal" role="status" aria-label={text.firmwareUpdateProgress}>
        <h2>{text.firmwareUpdateProgress}</h2>
        <p>{updateStepText}</p>
        <div class="progress-track"><span style={`width: ${updateProgress}%`}></span></div>
        <strong>{updateProgress}%</strong>
      </div>
    </div>
  {/if}

  {#if showInputTesterModal || showSettingsModal}
    <!-- 모달이 열린 동안 배경 클릭을 차단합니다. -->
    <div 
      class="global-click-blocker" 
      onclick={(e) => { e.preventDefault(); e.stopPropagation(); }}
      onkeydown={(e) => { e.preventDefault(); e.stopPropagation(); }}
      role="none"
    ></div>
  {/if}
</main>
