<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from './lib/Icon.svelte';
  import {
    applyConfig,
    checkDebugFirmwareUpdate,
    flashLatestDebugFirmware,
    listDevices,
    readConfig,
    readDeviceInfo,
    reconnectUsb,
    saveConfig,
    type BridgeConfig,
    type BridgeDevice,
    type DeviceInfo
  } from './lib/api';

  type Lang = 'ko' | 'en' | 'zh';
  type ThemeMode = 'light' | 'dark' | 'system';
  type StatusCode =
    | 'ready'
    | 'noDevice'
    | 'connected'
    | 'applied'
    | 'saved'
    | 'reconnectSent'
    | 'defaults'
    | 'updateReady'
    | 'updated'
    | 'upToDate';

  const defaultConfig: BridgeConfig = {
    config_version: 1,
    haptics_gain: 1,
    speaker_volume_percent: 0,
    inactive_time: 10,
    disable_inactive_disconnect: false,
    disable_pico_led: false,
    polling_rate_mode: 2,
    haptics_buffer_length: 64,
    controller_mode: 0
  };

  const i18n = {
    ko: {
      langKo: '한국어',
      langEn: 'EN',
      langZh: '中文',
      connected: '연결됨',
      disconnected: '연결 끊김',
      device: '장치',
      noCompatibleDevice: '호환 장치 없음',
      unknown: '알 수 없음',
      firmware: '펌웨어',
      selectDevice: '장치 선택',
      connect: '연결',
      configuration: '설정',
      feedbackTitle: '피드백 출력',
      feedbackDesc: '컨트롤러 햅틱, 스피커 음량, 버퍼 크기를 조정합니다.',
      hapticsGain: '햅틱 강도',
      speakerVolume: '스피커 음량 (%)',
      hapticsBuffer: '햅틱 버퍼 길이',
      powerTitle: '전원 및 표시등',
      powerDesc: '비활성 연결 해제와 Pico LED 동작을 제어합니다.',
      inactiveTime: '비활성 시간 (분)',
      disableInactive: '비활성 연결 해제 끄기',
      disableLed: 'Pico LED 끄기',
      performanceTitle: '성능',
      performanceDesc: 'HID 리포트 폴링 주기를 선택합니다.',
      pollingMode: '폴링 속도 모드',
      realTime: '실시간',
      compatibilityTitle: '호환성',
      compatibilityDesc: '컨트롤러 인식 모드를 전환합니다.',
      controllerMode: '컨트롤러 모드',
      actions: '작업',
      read: '읽기',
      saveToFlash: '플래시에 저장',
      reconnectUsb: 'USB 재연결',
      firmwareUpdate: 'debug 펌웨어 업데이트',
      autoFirmwareUpdate: '자동 업데이트',
      resetDefaults: '기본값 복원',
      state: '상태',
      dirty: '변경 사항이 있습니다.',
      errorUnknown: '알 수 없는 오류가 발생했습니다.',
      status: {
        ready: '준비됨',
        noDevice: 'DS5 Bridge 장치를 찾지 못했습니다.',
        connected: '연결됨',
        applied: '장치에 적용됨',
        saved: '플래시에 저장됨',
        reconnectSent: 'USB 재연결 명령을 보냈습니다.',
        defaults: '기본값을 불러왔습니다.',
        updateReady: '공식 debug 펌웨어 업데이트 가능',
        updated: 'debug 펌웨어 업데이트 파일을 복사했습니다.',
        upToDate: '최신 debug 펌웨어입니다.'
      }
    },
    en: {
      langKo: 'KO',
      langEn: 'EN',
      langZh: '中文',
      connected: 'Connected',
      disconnected: 'Disconnected',
      device: 'Device',
      noCompatibleDevice: 'No compatible device',
      unknown: 'Unknown',
      firmware: 'Firmware',
      selectDevice: 'Select device',
      connect: 'Connect',
      configuration: 'Configuration',
      feedbackTitle: 'Feedback output',
      feedbackDesc: 'Tune controller haptics, speaker level, and buffer size.',
      hapticsGain: 'Haptics gain',
      speakerVolume: 'Speaker volume (%)',
      hapticsBuffer: 'Haptics buffer length',
      powerTitle: 'Power & indicators',
      powerDesc: 'Control inactive disconnect and Pico LED behavior.',
      inactiveTime: 'Inactive time (minutes)',
      disableInactive: 'Disable inactive disconnect',
      disableLed: 'Disable Pico LED',
      performanceTitle: 'Performance',
      performanceDesc: 'Choose the HID report polling cadence.',
      pollingMode: 'Polling rate mode',
      realTime: 'Real-Time',
      compatibilityTitle: 'Compatibility',
      compatibilityDesc: 'Switch the controller identification mode.',
      controllerMode: 'Controller mode',
      actions: 'Actions',
      read: 'Read',
      saveToFlash: 'Save to Flash',
      reconnectUsb: 'Reconnect USB',
      firmwareUpdate: 'Update debug firmware',
      autoFirmwareUpdate: 'Auto update',
      resetDefaults: 'Reset to Defaults',
      state: 'State',
      dirty: 'There are unsaved changes.',
      errorUnknown: 'An unknown error occurred.',
      status: {
        ready: 'Ready',
        noDevice: 'DS5 Bridge device was not found.',
        connected: 'Connected',
        applied: 'Applied to device',
        saved: 'Saved to flash',
        reconnectSent: 'USB reconnect command was sent.',
        defaults: 'Defaults loaded.',
        updateReady: 'Official debug firmware update is available',
        updated: 'Debug firmware file was copied.',
        upToDate: 'Debug firmware is up to date.'
      }
    },
    zh: {
      langKo: '한국어',
      langEn: 'EN',
      langZh: '中文',
      connected: '已连接',
      disconnected: '未连接',
      device: '设备',
      noCompatibleDevice: '没有兼容设备',
      unknown: '未知',
      firmware: '固件',
      selectDevice: '选择设备',
      connect: '连接',
      configuration: '配置',
      feedbackTitle: '反馈输出',
      feedbackDesc: '调整控制器触觉、扬声器音量和缓冲区大小。',
      hapticsGain: '触觉强度',
      speakerVolume: '扬声器音量 (%)',
      hapticsBuffer: '触觉缓冲长度',
      powerTitle: '电源与指示灯',
      powerDesc: '控制空闲断开和 Pico LED 行为。',
      inactiveTime: '空闲时间 (分钟)',
      disableInactive: '禁用空闲断开',
      disableLed: '关闭 Pico LED',
      performanceTitle: '性能',
      performanceDesc: '选择 HID 报告轮询频率。',
      pollingMode: '轮询频率模式',
      realTime: '实时',
      compatibilityTitle: '兼容性',
      compatibilityDesc: '切换控制器识别模式。',
      controllerMode: '控制器模式',
      actions: '操作',
      read: '读取',
      saveToFlash: '保存到闪存',
      reconnectUsb: '重新连接 USB',
      firmwareUpdate: '更新 debug 固件',
      autoFirmwareUpdate: '自动更新',
      resetDefaults: '恢复默认值',
      state: '状态',
      dirty: '有未保存的更改。',
      errorUnknown: '发生未知错误。',
      status: {
        ready: '就绪',
        noDevice: '未找到 DS5 Bridge 设备。',
        connected: '已连接',
        applied: '已应用到设备',
        saved: '已保存到闪存',
        reconnectSent: '已发送 USB 重新连接命令。',
        defaults: '已加载默认值。',
        updateReady: '可更新官方 debug 固件',
        updated: '已复制 debug 固件文件。',
        upToDate: 'debug 固件已是最新。'
      }
    }
  } satisfies Record<Lang, Record<string, unknown> & { status: Record<StatusCode, string> }>;

  let lang: Lang = 'ko';
  let themeMode: ThemeMode = 'system';
  let systemTheme: 'light' | 'dark' = 'dark';
  let devices: BridgeDevice[] = [];
  let selectedDeviceId = '';
  let config: BridgeConfig = { ...defaultConfig };
  let originalConfig: BridgeConfig | null = null;
  let deviceInfo: DeviceInfo = {};
  let statusCode: StatusCode = 'ready';
  let statusOverride = '';
  let toastText = '';
  let toastKind: 'info' | 'error' = 'info';
  let toastTimer: number | undefined;
  let isBusy = false;
  let errorText = '';
  let autoFirmwareUpdate = false;

  $: text = i18n[lang];
  $: effectiveTheme = themeMode === 'system' ? systemTheme : themeMode;
  $: selectedDevice = devices.find((device) => device.id === selectedDeviceId) ?? null;
  $: isConnected = Boolean(selectedDevice);
  $: isDirty = originalConfig ? JSON.stringify(originalConfig) !== JSON.stringify(config) : false;
  $: statusText = statusOverride || text.status[statusCode];
  $: firmwareLabel = deviceInfo.firmware_version || text.unknown;
  $: rssiLabel =
    deviceInfo.rssi === null || deviceInfo.rssi === undefined ? text.unknown : `${deviceInfo.rssi} dBm`;
  $: deviceTitle = selectedDevice
    ? `${selectedDevice.label.split(' - ')[0]} · ${selectedDevice.vendor_id.toString(16).padStart(4, '0').toUpperCase()}:${selectedDevice.product_id.toString(16).padStart(4, '0').toUpperCase()}`
    : text.noCompatibleDevice;

  onMount(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: light)');
    const syncSystemTheme = () => {
      systemTheme = mediaQuery.matches ? 'light' : 'dark';
    };

    syncSystemTheme();
    mediaQuery.addEventListener('change', syncSystemTheme);
    autoFirmwareUpdate = localStorage.getItem('ds5:autoFirmwareUpdate') === 'true';
    void refreshDevices();

    return () => {
      mediaQuery.removeEventListener('change', syncSystemTheme);
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

  function switchLanguage(nextLang: Lang) {
    lang = nextLang;
  }

  function switchTheme(nextThemeMode: ThemeMode) {
    themeMode = nextThemeMode;
  }

  function setAutoFirmwareUpdate(enabled: boolean) {
    autoFirmwareUpdate = enabled;
    localStorage.setItem('ds5:autoFirmwareUpdate', String(enabled));
  }

  async function runTask(task: () => Promise<void>) {
    isBusy = true;
    errorText = '';
    try {
      await task();
    } catch (error) {
      errorText = error instanceof Error ? error.message : text.errorUnknown;
      showToast(errorText, 'error');
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
        deviceInfo = {};
        setStatus('noDevice');
        return;
      }

      if (!devices.some((device) => device.id === selectedDeviceId)) {
        selectedDeviceId = devices[0].id;
      }

      await readAll();
      if (autoFirmwareUpdate) {
        await performFirmwareUpdate(true);
      } else {
        setStatus('connected');
      }
    });
  }

  async function refreshInfoOnly() {
    if (!selectedDeviceId) return;
    deviceInfo = await readDeviceInfo(selectedDeviceId);
  }

  async function readAll() {
    if (!selectedDeviceId) return;

    const infoPromise = readDeviceInfo(selectedDeviceId).then((info) => {
      deviceInfo = info;
    });

    const configPromise = readConfig(selectedDeviceId).then((nextConfig) => {
      config = nextConfig;
      originalConfig = structuredClone(nextConfig);
    });

    await Promise.allSettled([infoPromise, configPromise]);
    setStatus('connected');
  }

  async function onRead() {
    await runTask(readAll);
  }

  async function onApply() {
    if (!selectedDeviceId) return;
    await runTask(async () => {
      await applyConfig(selectedDeviceId, config);
      originalConfig = structuredClone(config);
      setStatus('applied');
    });
  }

  async function onSave() {
    if (!selectedDeviceId) return;
    await runTask(async () => {
      await applyConfig(selectedDeviceId, config);
      await saveConfig(selectedDeviceId);
      originalConfig = structuredClone(config);
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
    await runTask(() => performFirmwareUpdate(false));
  }

  async function performFirmwareUpdate(skipIfCurrent: boolean) {
    const update = await checkDebugFirmwareUpdate();
    if (skipIfCurrent && deviceInfo.firmware_version === update.version) {
      setStatus('upToDate');
      return;
    }

    showToast(`${text.status.updateReady}: ${update.version} / ${update.asset_name}`);
    const result = await flashLatestDebugFirmware(selectedDeviceId || undefined);
    setStatus('updated');
    showToast(`${text.status.updated}: ${result.version} / ${result.asset_name}`);
  }

  function resetToDefaults() {
    config = { ...defaultConfig };
    setStatus('defaults');
  }
</script>

<main class:theme-light={effectiveTheme === 'light'} class:theme-dark={effectiveTheme === 'dark'} class="app-shell">
  <header class="topbar">
    <div class="brand">
      <div class="brand-icon">
        <span class="wifi"><Icon name="wifi" size={13} /></span>
        <span class="pad"><Icon name="gamepad" size={30} /></span>
      </div>
      <h1>DS5 Bridge Config</h1>
    </div>
    <div class="toolbar">
      <span class="translate"><Icon name="languages" size={16} /></span>
      <div class="seg small" aria-label="언어 선택">
        <button class:active={lang === 'ko'} type="button" on:click={() => switchLanguage('ko')}>{text.langKo}</button>
        <button class:active={lang === 'en'} type="button" on:click={() => switchLanguage('en')}>{text.langEn}</button>
        <button class:active={lang === 'zh'} type="button" on:click={() => switchLanguage('zh')}>{text.langZh}</button>
      </div>
      <div class="seg icon-group" aria-label="테마 선택">
        <button class:active={themeMode === 'light'} type="button" on:click={() => switchTheme('light')} title="라이트 테마"><Icon name="sun" size={15} /></button>
        <button class:active={themeMode === 'dark'} type="button" on:click={() => switchTheme('dark')} title="다크 테마"><Icon name="moon" size={15} /></button>
        <button class:active={themeMode === 'system'} type="button" on:click={() => switchTheme('system')} title="시스템 테마"><Icon name="monitor" size={15} /></button>
      </div>
      <div class:connected={isConnected} class="status-pill"><span><Icon name="check" size={10} /></span>{isConnected ? text.connected : text.disconnected}</div>
    </div>
  </header>

  {#if toastText}
    <div class:error={toastKind === 'error'} class="toast" role="status">
      <span>{toastText}</span>
      <button type="button" on:click={() => (toastText = '')}>×</button>
    </div>
  {/if}

  <section class="device-card">
    <div class="device-left">
      <div class="square-icon"><Icon name="cable" size={18} /></div>
      <div>
        <div class="overline">{text.device}</div>
        <div class="device-name">{deviceTitle}</div>
        <div class="device-meta"><span>{text.firmware} {firmwareLabel}</span><span>RSSI {rssiLabel}</span></div>
      </div>
    </div>
    <div class="device-right">
      {#if devices.length > 1}
        <select bind:value={selectedDeviceId} disabled={isBusy} aria-label={text.selectDevice}>
          {#each devices as device}
            <option value={device.id}>{device.label}</option>
          {/each}
        </select>
      {/if}
      <button class="light-btn" type="button" on:click={() => refreshDevices()} disabled={isBusy}><Icon name="cable" size={15} /> {text.connect}</button>
    </div>
  </section>

  <div class="main-grid">
    <section class="config-panel panel-dark">
      <div class="section-title"><span><Icon name="sliders" size={18} /></span><h2>{text.configuration}</h2></div>
      <div class="cards-grid">
        <section class="config-card">
          <div class="card-head"><span><Icon name="volume" size={17} /></span><div><h3>{text.feedbackTitle}</h3><p>{text.feedbackDesc}</p></div></div>
          <label class="control-row"><strong>{text.hapticsGain}</strong><input type="range" min="0.25" max="2" step="0.01" bind:value={config.haptics_gain} /><input type="number" min="0.25" max="2" step="0.01" bind:value={config.haptics_gain} /></label>
          <label class="control-row"><strong>{text.speakerVolume}</strong><input type="range" min="0" max="100" step="1" bind:value={config.speaker_volume_percent} /><input type="number" min="0" max="100" step="1" bind:value={config.speaker_volume_percent} /></label>
          <label class="control-row"><strong>{text.hapticsBuffer}</strong><input type="range" min="16" max="128" step="1" bind:value={config.haptics_buffer_length} /><input type="number" min="16" max="128" step="1" bind:value={config.haptics_buffer_length} /></label>
        </section>

        <section class="config-card">
          <div class="card-head"><span><Icon name="zap" size={17} /></span><div><h3>{text.powerTitle}</h3><p>{text.powerDesc}</p></div></div>
          <label class="control-row"><strong>{text.inactiveTime}</strong><input type="range" min="5" max="60" step="1" bind:value={config.inactive_time} /><input type="number" min="5" max="60" step="1" bind:value={config.inactive_time} /></label>
          <label class="switch-row"><strong>{text.disableInactive}</strong><input type="checkbox" bind:checked={config.disable_inactive_disconnect} /></label>
          <label class="switch-row"><strong>{text.disableLed}</strong><input type="checkbox" bind:checked={config.disable_pico_led} /></label>
        </section>

        <section class="config-card compact">
          <div class="card-head"><span><Icon name="gauge" size={17} /></span><div><h3>{text.performanceTitle}</h3><p>{text.performanceDesc}</p></div></div>
          <strong class="field-label">{text.pollingMode}</strong>
          <div class="seg wide">
            <button class:active={config.polling_rate_mode === 0} type="button" on:click={() => (config.polling_rate_mode = 0)}>250 Hz</button>
            <button class:active={config.polling_rate_mode === 1} type="button" on:click={() => (config.polling_rate_mode = 1)}>500 Hz</button>
            <button class:active={config.polling_rate_mode === 2} type="button" on:click={() => (config.polling_rate_mode = 2)}>{text.realTime}</button>
          </div>
        </section>

        <section class="config-card compact">
          <div class="card-head"><span><Icon name="gamepad" size={17} /></span><div><h3>{text.compatibilityTitle}</h3><p>{text.compatibilityDesc}</p></div></div>
          <strong class="field-label">{text.controllerMode}</strong>
          <div class="seg wide">
            <button class:active={config.controller_mode === 0} type="button" on:click={() => (config.controller_mode = 0)}>DS5</button>
            <button class:active={config.controller_mode === 1} type="button" on:click={() => (config.controller_mode = 1)}>DSE</button>
            <button class:active={config.controller_mode === 2} type="button" on:click={() => (config.controller_mode = 2)}>Auto</button>
          </div>
        </section>
      </div>
    </section>

    <aside class="actions-panel panel-dark">
      <div class="section-title"><span><Icon name="download" size={18} /></span><h2>{text.actions}</h2></div>
      <div class="action-stack">
        <button type="button" on:click={onRead} disabled={!isConnected || isBusy}><Icon name="rotate-cw" size={15} /> {text.read}</button>
        <button class="primary" type="button" on:click={onSave} disabled={!isConnected || isBusy}><Icon name="save" size={15} /> {text.saveToFlash}</button>
        <button type="button" on:click={onReconnect} disabled={!isConnected || isBusy}><Icon name="power" size={15} /> {text.reconnectUsb}</button>
        <button type="button" on:click={onFirmwareUpdate} disabled={isBusy}><Icon name="download" size={15} /> {text.firmwareUpdate}</button>
        <button class="ghost" type="button" on:click={resetToDefaults} disabled={isBusy}><Icon name="rotate-ccw" size={15} /> {text.resetDefaults}</button>
        <label class="auto-update-row">
          <span>{text.autoFirmwareUpdate}</span>
          <input type="checkbox" checked={autoFirmwareUpdate} on:change={(event) => setAutoFirmwareUpdate(event.currentTarget.checked)} />
        </label>
      </div>
      <div class="state-card"><div class="overline">{text.state}</div><strong>{statusText}</strong>{#if isDirty}<p>{text.dirty}</p>{/if}</div>
    </aside>
  </div>
</main>
