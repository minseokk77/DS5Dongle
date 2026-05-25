<script lang="ts">
  import Icon from '../Icon.svelte';
  import LanguageThemeSelector from './LanguageThemeSelector.svelte';
  import type { Lang } from '../i18n';

  type ThemeMode = 'light' | 'dark' | 'system';
  interface FirmwareCapability {
    key: string;
    label: string;
    supported: boolean;
    reason: string;
  }

  interface DiagnosticLog {
    id: number;
    time: string;
    kind: 'info' | 'error';
    message: string;
  }

  interface DeviceDiagnostics {
    dongleConnected: boolean;
    controllerConnected: boolean;
    gamepadModalOpen: boolean;
    batteryReportAvailable: boolean;
    rssiReportAvailable: boolean;
    configReadable: boolean;
  }

  interface DiagnosticCheck {
    key: string;
    label: string;
    state: 'waiting' | 'running' | 'passed' | 'failed' | 'skipped' | 'warning';
    message: string;
  }

  export let isOpen = false;
  export let lang: Lang;
  export let themeMode: ThemeMode;
  export let isConnected: boolean;
  export let autoFirmwareUpdate: boolean;
  export let appVersion = '';
  export let firmwareVersion = '';
  export let releaseChannel = '';
  export let updateRepository = '';
  export let capabilities: FirmwareCapability[] = [];
  export let logs: DiagnosticLog[] = [];
  export let configVersion = '';
  export let calibrationEnabled = false;
  export let leftCalibrationSummary = '';
  export let rightCalibrationSummary = '';
  export let diagnostics: DeviceDiagnostics = {
    dongleConnected: false,
    controllerConnected: false,
    gamepadModalOpen: false,
    batteryReportAvailable: false,
    rssiReportAvailable: false,
    configReadable: false
  };
  export let diagnosticChecks: DiagnosticCheck[] = [];
  export let diagnosticChecksRunning = false;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  export let text: any;
  export let onClose: () => void = () => {};
  export let onLangChange: (lang: Lang) => void = () => {};
  export let onThemeChange: (themeMode: ThemeMode) => void = () => {};
  export let onAutoFirmwareUpdateChange: (enabled: boolean) => void = () => {};
  export let onResetDefaults: () => void = () => {};
  export let onRecoveryFirmwareUpdate: () => void | Promise<void> = () => {};
  export let onExportLogs: () => void = () => {};
  export let onRunDiagnostics: () => void | Promise<void> = () => {};

  function handleBackdropClick(event: MouseEvent) {
    if (event.currentTarget === event.target) {
      onClose();
    }
  }

  function handleAutoUpdateChange(event: Event) {
    onAutoFirmwareUpdateChange((event.currentTarget as HTMLInputElement).checked);
  }
</script>

{#if isOpen}
  <div
    class="settings-modal-overlay"
    onclick={handleBackdropClick}
    onkeydown={(event) => {
      if (event.key === 'Escape') onClose();
    }}
    role="presentation"
  >
    <div class="settings-modal" role="dialog" aria-modal="true" aria-label={text.settings}>
      <div class="settings-modal-head">
        <h2><Icon name="settings" size={18} /> {text.settings}</h2>
        <button class="icon-close" type="button" onclick={onClose} aria-label={text.close}>
          <Icon name="x" size={18} />
        </button>
      </div>

      <div class="settings-group">
        <div>
          <strong>{text.interfaceSettings}</strong>
          <p>{text.interfaceSettingsDesc}</p>
        </div>
        <LanguageThemeSelector
          {lang}
          {themeMode}
          {isConnected}
          {text}
          onLangChange={onLangChange}
          onThemeChange={onThemeChange}
          showStatus={false}
        />
      </div>

      <div class="settings-group">
        <div>
          <strong>{text.firmwareSettings}</strong>
          <p>{text.autoFirmwareUpdateDesc}</p>
        </div>
        <div class="version-grid">
          <span>{text.appVersion}</span>
          <strong>{appVersion}</strong>
          <span>{text.firmwareVersion}</span>
          <strong>{firmwareVersion}</strong>
          <span>{text.releaseChannel}</span>
          <strong>{releaseChannel}</strong>
          <span>{text.updateRepository}</span>
          <strong>{updateRepository}</strong>
        </div>
        <label class="settings-toggle-row">
          <span>{text.autoFirmwareUpdate}</span>
          <input type="checkbox" checked={autoFirmwareUpdate} onchange={handleAutoUpdateChange} />
        </label>
        <button class="settings-action-btn" type="button" onclick={onRecoveryFirmwareUpdate}>
          <Icon name="download" size={15} /> {text.recoveryFirmwareUpdate}
        </button>
      </div>

      <div class="settings-group">
        <div>
          <strong>{text.firmwareCompatibility}</strong>
          <p>{text.firmwareCompatibilityDesc}</p>
        </div>
        <div class="capability-grid">
          {#each capabilities as capability}
            <div class:supported={capability.supported} class="capability-item">
              <span>{capability.supported ? '✓' : '!'}</span>
              <div>
                <strong>{capability.label}</strong>
                {#if !capability.supported}
                  <p>{capability.reason}</p>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      </div>

      <div class="settings-group">
        <div>
          <strong>{text.deviceDiagnostics}</strong>
          <p>{text.deviceDiagnosticsDesc}</p>
        </div>
        <div class="diagnostic-grid">
          <div class:ok={diagnostics.dongleConnected} class="diagnostic-item">
            <span></span>
            <strong>{text.diagDongle}</strong>
            <p>{diagnostics.dongleConnected ? text.picoConnected : text.picoDisconnected}</p>
          </div>
          <div class:ok={diagnostics.controllerConnected} class="diagnostic-item">
            <span></span>
            <strong>{text.diagController}</strong>
            <p>{diagnostics.controllerConnected ? text.controllerConnected : text.controllerDisconnected}</p>
          </div>
          <div class:ok={diagnostics.configReadable} class="diagnostic-item">
            <span></span>
            <strong>{text.diagConfig}</strong>
            <p>{diagnostics.configReadable ? text.configReadable : text.configUnreadable}</p>
          </div>
          <div class:ok={diagnostics.batteryReportAvailable || diagnostics.rssiReportAvailable} class="diagnostic-item">
            <span></span>
            <strong>{text.diagTelemetry}</strong>
            <p>{diagnostics.batteryReportAvailable || diagnostics.rssiReportAvailable ? text.telemetryReceived : text.telemetryWaiting}</p>
          </div>
        </div>
        <button class="settings-action-btn" type="button" onclick={onRunDiagnostics} disabled={diagnosticChecksRunning}>
          <Icon name="sliders" size={15} /> {diagnosticChecksRunning ? text.diagnosticRunning : text.runDiagnostics}
        </button>
        {#if diagnosticChecks.length > 0}
          <div class="diagnostic-check-list">
            {#each diagnosticChecks as check (check.key)}
              <div class={`diagnostic-check check-${check.state}`}>
                <span></span>
                <strong>{check.label}</strong>
                <p>{check.message || text.diagnosticWaiting}</p>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <div class="settings-group">
        <div>
          <strong>{text.stickCalibrationStatus}</strong>
          <p>{text.stickCalibrationStatusDesc}</p>
        </div>
        <div class="version-grid">
          <span>{text.state}</span>
          <strong>{calibrationEnabled ? text.enabled : text.disabled}</strong>
          <span>{text.leftStick}</span>
          <strong>{leftCalibrationSummary || text.calibrationNoInfo}</strong>
          <span>{text.rightStick}</span>
          <strong>{rightCalibrationSummary || text.calibrationNoInfo}</strong>
          <span>{text.configVersion}</span>
          <strong>{configVersion || text.unknown}</strong>
        </div>
      </div>

      <div class="settings-group">
        <div>
          <strong>{text.configSettings}</strong>
          <p>{text.resetDefaultsDesc}</p>
        </div>
        <button class="settings-action-btn" type="button" onclick={onResetDefaults}>
          <Icon name="rotate-ccw" size={15} /> {text.resetScreenDefaults}
        </button>
      </div>

      <div class="settings-group">
        <div>
          <strong>{text.recentLogs}</strong>
          <p>{text.recentLogsDesc}</p>
        </div>
        <button class="settings-action-btn narrow" type="button" onclick={onExportLogs}>
          <Icon name="download" size={15} /> {text.exportLogs}
        </button>
        <div class="log-list">
          {#if logs.length === 0}
            <div class="empty-log">{text.noLogs}</div>
          {:else}
            {#each logs.slice(0, 8) as log (log.id)}
              <div class:error={log.kind === 'error'} class="log-item">
                <span>{log.time}</span>
                <p>{log.message}</p>
              </div>
            {/each}
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .settings-modal {
    width: min(850px, calc(100vw - 72px));
    max-height: calc(100vh - 36px);
    padding: 22px;
    border: 1px solid var(--border);
    border-radius: 14px;
    background: var(--modal-bg);
    color: var(--modal-text);
    box-shadow: 0 18px 54px rgba(0, 0, 0, 0.34);
    overflow: auto;
    scrollbar-width: none;
    -ms-overflow-style: none;
  }

  .settings-modal-overlay {
    position: fixed;
    inset: 0;
    z-index: 10000;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 18px;
    background: rgba(0, 0, 0, 0.62);
    backdrop-filter: blur(5px);
    overflow: hidden;
    scrollbar-width: none;
    -ms-overflow-style: none;
  }

  .settings-modal-overlay::-webkit-scrollbar,
  .settings-modal::-webkit-scrollbar {
    width: 0;
    height: 0;
    display: none;
  }

  .settings-modal-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    padding-bottom: 14px;
    margin-bottom: 16px;
    border-bottom: 1px solid var(--border);
  }

  .settings-modal-head h2 {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    margin: 0;
    color: var(--modal-text);
    font-size: 1.05rem;
    font-weight: 800;
  }

  .icon-close {
    width: 40px;
    height: 40px;
    display: grid;
    place-items: center;
    border-radius: 10px;
    background: var(--control-2);
    color: var(--modal-muted);
    transition: color 0.15s, filter 0.15s;
  }

  .icon-close:hover {
    color: var(--modal-text);
    filter: brightness(1.08);
  }

  .settings-group {
    display: grid;
    gap: 12px;
    width: 100%;
    padding: 16px;
    margin-top: 16px;
    border: 1px solid var(--border);
    border-radius: 13px;
    background: color-mix(in srgb, var(--modal-bg) 88%, var(--control-2) 12%);
  }

  .settings-group:first-of-type {
    margin-top: 0;
  }

  .settings-group > div:first-child strong {
    display: block;
    color: var(--modal-text);
    font-size: 0.92rem;
    font-weight: 800;
  }

  .settings-group > div:first-child p {
    margin: 4px 0 0;
    color: var(--modal-muted);
    font-size: 0.78rem;
    line-height: 1.35;
  }

  .settings-toggle-row {
    height: 49px;
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 0 14px;
    border-radius: 10px;
    background: var(--control);
    color: var(--text);
    font-size: 0.84rem;
    font-weight: 800;
  }

  .settings-action-btn {
    min-height: 48px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    width: 100%;
    padding: 0 14px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--control);
    color: var(--text);
    font-size: 0.84rem;
    font-weight: 800;
    transition: filter 0.15s, border-color 0.15s;
  }

  .settings-action-btn:hover {
    filter: brightness(1.08);
  }

  .version-grid {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 6px 12px;
    width: 100%;
    padding: 12px 14px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--control);
    box-sizing: border-box;
    font-size: 0.82rem;
  }

  .version-grid span {
    color: var(--muted);
  }

  .version-grid strong {
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .capability-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 8px;
    width: 100%;
  }

  .diagnostic-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 8px;
  }

  .diagnostic-item {
    min-width: 0;
    display: grid;
    grid-template-columns: 10px minmax(0, 1fr);
    gap: 3px 9px;
    align-items: center;
    min-height: 50px;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--control);
  }

  .diagnostic-item > span {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #737780;
  }

  .diagnostic-item.ok > span {
    background: #63e2b7;
    box-shadow: 0 0 8px rgba(99, 226, 183, 0.4);
  }

  .diagnostic-item strong {
    color: var(--text);
    font-size: 0.82rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .diagnostic-item p {
    grid-column: 2;
    margin: 0;
    color: var(--muted);
    font-size: 0.72rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .diagnostic-check-list {
    display: grid;
    gap: 6px;
  }

  .diagnostic-check {
    display: grid;
    grid-template-columns: 10px minmax(120px, 160px) minmax(0, 1fr);
    gap: 9px;
    align-items: start;
    min-height: 34px;
    padding: 8px 10px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--control);
  }

  .diagnostic-check > span {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #737780;
  }

  .diagnostic-check.check-running > span {
    background: #2f6bff;
    box-shadow: 0 0 8px rgba(47, 107, 255, 0.4);
  }

  .diagnostic-check.check-passed > span {
    background: #63e2b7;
    box-shadow: 0 0 8px rgba(99, 226, 183, 0.4);
  }

  .diagnostic-check.check-failed > span {
    background: #ff6b6b;
    box-shadow: 0 0 8px rgba(255, 107, 107, 0.35);
  }

  .diagnostic-check.check-warning > span {
    background: #ffab00;
    box-shadow: 0 0 8px rgba(255, 171, 0, 0.35);
  }

  .diagnostic-check strong {
    color: var(--text);
    font-size: 0.78rem;
    white-space: nowrap;
  }

  .diagnostic-check p {
    margin: 0;
    color: var(--muted);
    font-size: 0.7rem;
    overflow: hidden;
    overflow-wrap: anywhere;
    line-height: 1.35;
  }

  .capability-item {
    display: flex;
    gap: 8px;
    min-width: 0;
    min-height: 46px;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--control);
  }

  .capability-item > span {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    flex: 0 0 auto;
    border-radius: 50%;
    background: rgba(255, 171, 0, 0.14);
    color: #ffab00;
    font-size: 0.68rem;
    font-weight: 700;
  }

  .capability-item.supported > span {
    background: rgba(16, 185, 129, 0.14);
    color: #10b981;
  }

  .capability-item strong {
    display: block;
    font-size: 0.84rem;
    color: var(--text);
  }

  .capability-item p {
    margin: 2px 0 0;
    font-size: 0.72rem;
    color: var(--muted);
    line-height: 1.35;
  }

  .log-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    width: 100%;
    max-height: 150px;
    overflow: hidden;
  }

  .empty-log,
  .log-item {
    padding: 10px 12px;
    border-radius: 8px;
    background: var(--control);
    border: 1px solid var(--border);
  }

  .empty-log {
    color: var(--muted);
    font-size: 0.78rem;
  }

  .log-item {
    display: grid;
    grid-template-columns: 72px minmax(0, 1fr);
    gap: 8px;
    align-items: start;
  }

  .log-item span {
    color: var(--muted);
    font-size: 0.68rem;
    font-family: monospace;
  }

  .log-item p {
    margin: 0;
    color: var(--text);
    font-size: 0.72rem;
    line-height: 1.35;
  }

  .log-item.error p {
    color: #ff8a8a;
  }
</style>
