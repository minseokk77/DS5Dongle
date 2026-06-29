<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from '../Icon.svelte';
  import type { BridgeConfig } from '../api';

  export let config: BridgeConfig;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  export let text: any;
  export let showToast: (message: string, kind?: 'info' | 'error') => void;
  export let onLog: (message: string, kind?: 'info' | 'error') => void = () => {};

  // Preset model.
  interface Preset {
    id: string;
    name: string;
    isBuiltIn: boolean;
    config: BridgeConfig;
  }

  // Local preset state.
  let customPresets: Preset[] = [];
  let selectedPresetId = '';
  let selectedPresetSnapshot = '';
  let isSavingMode = false;
  let newPresetName = '';
  let importInput: HTMLInputElement | null = null;

  // Built-in presets.
  $: builtInPresets = [
    {
      id: 'racing',
      name: text.presetRacing,
      isBuiltIn: true,
      config: {
        config_version: 1,
        haptics_gain: 2.0,
        speaker_volume_percent: 50,
        audio_buffer_length: 64,
        inactive_time: 10,
        disable_inactive_disconnect: false,
        disable_pico_led: false,
        polling_rate_mode: 2,
        controller_mode: 0,
        stick_calibration_enabled: false,
        left_stick_center_x: 0,
        left_stick_center_y: 0,
        left_stick_deadzone: 0,
        right_stick_center_x: 0,
        right_stick_center_y: 0,
        right_stick_deadzone: 0,
        left_stick_min_x: 0,
        left_stick_max_x: 0,
        left_stick_min_y: 0,
        left_stick_max_y: 0,
        right_stick_min_x: 0,
        right_stick_max_x: 0,
        right_stick_min_y: 0,
        right_stick_max_y: 0
      }
    },
    {
      id: 'fps',
      name: text.presetFps,
      isBuiltIn: true,
      config: {
        config_version: 1,
        haptics_gain: 0.6,
        speaker_volume_percent: 50,
        audio_buffer_length: 64,
        inactive_time: 10,
        disable_inactive_disconnect: false,
        disable_pico_led: false,
        polling_rate_mode: 2,
        controller_mode: 0,
        stick_calibration_enabled: false,
        left_stick_center_x: 0,
        left_stick_center_y: 0,
        left_stick_deadzone: 0,
        right_stick_center_x: 0,
        right_stick_center_y: 0,
        right_stick_deadzone: 0,
        left_stick_min_x: 0,
        left_stick_max_x: 0,
        left_stick_min_y: 0,
        left_stick_max_y: 0,
        right_stick_min_x: 0,
        right_stick_max_x: 0,
        right_stick_min_y: 0,
        right_stick_max_y: 0
      }
    },
    {
      id: 'silent',
      name: text.presetSilent,
      isBuiltIn: true,
      config: {
        config_version: 1,
        haptics_gain: 0.3,
        speaker_volume_percent: 50,
        audio_buffer_length: 64,
        inactive_time: 10,
        disable_inactive_disconnect: false,
        disable_pico_led: true,
        polling_rate_mode: 0,
        controller_mode: 0,
        stick_calibration_enabled: false,
        left_stick_center_x: 0,
        left_stick_center_y: 0,
        left_stick_deadzone: 0,
        right_stick_center_x: 0,
        right_stick_center_y: 0,
        right_stick_deadzone: 0,
        left_stick_min_x: 0,
        left_stick_max_x: 0,
        left_stick_min_y: 0,
        left_stick_max_y: 0,
        right_stick_min_x: 0,
        right_stick_max_x: 0,
        right_stick_min_y: 0,
        right_stick_max_y: 0
      }
    }
  ] as Preset[];

  // Combined preset list.
  $: allPresets = [...builtInPresets, ...customPresets];
  $: selectedPreset = allPresets.find((preset) => preset.id === selectedPresetId) ?? null;
  $: selectedPresetModified = Boolean(selectedPreset && selectedPresetSnapshot && JSON.stringify(config) !== selectedPresetSnapshot);

  onMount(() => {
    loadCustomPresets();
  });

  // Load custom presets from local storage.
  function loadCustomPresets() {
    try {
      const stored = localStorage.getItem('ds5:custom_presets');
      if (stored) {
        customPresets = JSON.parse(stored);
      }
    } catch (e) {
      // Ignore invalid local storage data.
    }
  }

  function handlePresetChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    const presetId = target.value;
    if (!presetId) return;

    const preset = allPresets.find(p => p.id === presetId);
    if (preset) {
      // Apply preset to the visible form.
      config = { ...preset.config };
      selectedPresetId = preset.id;
      selectedPresetSnapshot = JSON.stringify(preset.config);
      showToast(`${text.presetLoaded} (${preset.isBuiltIn ? text.presets : text.presetName})`, 'info');
      onLog(`${text.presetLoaded}: ${preset.name}`);
    }
  }

  function saveCurrentConfigAsPreset() {
    const name = newPresetName.trim();
    if (!name) {
      showToast(text.enterPresetName, 'error');
      return;
    }

    try {
      const newPreset: Preset = {
        id: `custom_${Date.now()}`,
        name,
        isBuiltIn: false,
        config: { ...config }
      };

      customPresets = [...customPresets, newPreset];
      localStorage.setItem('ds5:custom_presets', JSON.stringify(customPresets));
      selectedPresetId = newPreset.id;
      selectedPresetSnapshot = JSON.stringify(newPreset.config);
      showToast(text.presetSaved, 'info');
      onLog(`${text.presetSaved}: ${name}`);

      // Reset save form.
      newPresetName = '';
      isSavingMode = false;
    } catch (error) {
      showToast(text.errorUnknown, 'error');
    }
  }

  // Delete a custom preset.
  function deletePreset(presetId: string, event: MouseEvent) {
    event.stopPropagation();
    try {
      customPresets = customPresets.filter(p => p.id !== presetId);
      if (selectedPresetId === presetId) {
        selectedPresetId = '';
      }
      localStorage.setItem('ds5:custom_presets', JSON.stringify(customPresets));
      showToast(text.presetDeleted, 'info');
      onLog(text.presetDeleted);
    } catch (e) {
      showToast(text.errorUnknown, 'error');
      onLog(text.errorUnknown, 'error');
    }
  }

  function isBridgeConfig(value: unknown): value is BridgeConfig {
    if (!value || typeof value !== 'object') return false;
    const candidate = value as BridgeConfig;
    return (
      typeof candidate.config_version === 'number' &&
      typeof candidate.haptics_gain === 'number' &&
      typeof candidate.speaker_volume === 'number' &&
      typeof candidate.headset_volume === 'number' &&
      typeof candidate.speaker_gain === 'number' &&
      typeof candidate.inactive_time === 'number' &&
      typeof candidate.disable_pico_led === 'boolean' &&
      typeof candidate.polling_rate_mode === 'number' &&
      typeof candidate.audio_buffer_length === 'number' &&
      typeof candidate.controller_mode === 'number' &&
      typeof candidate.enable_usb_sn === 'boolean' &&
      typeof candidate.ps_shortcut_enabled === 'boolean' &&
      typeof candidate.disable_mic === 'boolean' &&
      typeof candidate.disable_speaker === 'boolean' &&
      typeof candidate.enable_wake === 'boolean' &&
      typeof candidate.trigger_reduce === 'number' &&
      typeof candidate.stick_calibration_enabled === 'boolean'
    );
  }

  function exportConfig() {
    const blob = new Blob([JSON.stringify(config, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `ds5-dongle-config-${new Date().toISOString().slice(0, 10)}.json`;
    link.click();
    URL.revokeObjectURL(url);
    showToast(text.configExported, 'info');
    onLog(text.configExported);
  }

  function requestImportConfig() {
    importInput?.click();
  }

  async function importConfig(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = '';
    if (!file) return;

    try {
      const parsed = JSON.parse(await file.text());
      if (!isBridgeConfig(parsed)) {
        throw new Error(text.configImportInvalid);
      }

      config = { ...parsed };
      selectedPresetId = '';
      selectedPresetSnapshot = '';
      showToast(text.configImported, 'info');
      onLog(text.configImported);
    } catch (error) {
      const message = error instanceof Error ? error.message : text.configImportInvalid;
      showToast(message, 'error');
      onLog(message, 'error');
    }
  }
</script>

<section class="config-panel panel-dark">
  <div class="section-title" style="display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 12px; margin-bottom: 20px;">
    <div style="display: flex; align-items: center; gap: 8px;">
      <span><Icon name="sliders" size={18} /></span>
      <h2 style="margin: 0;">{text.configuration}</h2>
    </div>

    <!-- 프리셋 관리 영역 -->
    <div class="preset-manager" style="display: flex; align-items: center; gap: 8px;">
      {#if !isSavingMode}
        <div style="display: flex; align-items: center; gap: 6px;">
          <select 
            bind:value={selectedPresetId} 
            onchange={handlePresetChange}
            aria-label={text.presets}
            style="background: var(--control-2); border: 1px solid var(--border); color: var(--text); padding: 6px 12px; border-radius: 6px; font-size: 0.8rem; outline: none; cursor: pointer; min-width: 160px; height: 32px;"
          >
            <option value="" style="background: var(--control-2); color: var(--muted);">{text.selectPreset}</option>
            
            <optgroup label={text.officialPresets} style="background: var(--control-2); color: #10b981; font-weight: 500;">
              {#each builtInPresets as preset}
                <option value={preset.id} style="background: var(--control-2); color: var(--text);">{preset.name}</option>
              {/each}
            </optgroup>

            {#if customPresets.length > 0}
              <optgroup label={text.customPresets} style="background: var(--control-2); color: #d97706; font-weight: 500;">
                {#each customPresets as preset}
                  <option value={preset.id} style="background: var(--control-2); color: var(--text);">{preset.name}</option>
                {/each}
              </optgroup>
            {/if}
          </select>

          {#if selectedPreset && selectedPresetModified}
            <span class="preset-state modified">
              {text.modified}
            </span>
          {/if}

          <button 
            type="button" 
            onclick={() => (isSavingMode = true)}
            style="background: rgba(99, 226, 183, 0.1); border: 1px solid rgba(99, 226, 183, 0.25); color: #63e2b7; padding: 6px 12px; border-radius: 6px; font-size: 0.8rem; font-weight: 500; cursor: pointer; height: 32px; display: inline-flex; align-items: center; gap: 4px; transition: background 0.15s;"
            onmouseenter={(e) => e.currentTarget.style.background = 'rgba(99, 226, 183, 0.2)'}
            onmouseleave={(e) => e.currentTarget.style.background = 'rgba(99, 226, 183, 0.1)'}
          >
            <Icon name="save" size={13} /> {text.savePreset}
          </button>
          <button
            type="button"
            onclick={exportConfig}
            class="preset-tool-btn"
          >
            <Icon name="download" size={13} /> {text.exportConfig}
          </button>
          <button
            type="button"
            onclick={requestImportConfig}
            class="preset-tool-btn"
          >
            <Icon name="upload" size={13} /> {text.importConfig}
          </button>
          <input
            bind:this={importInput}
            type="file"
            accept="application/json,.json"
            onchange={importConfig}
            style="display: none;"
          />
        </div>
      {:else}
        <!-- 프리셋 저장 입력 -->
        <div style="display: flex; align-items: center; gap: 6px; animation: fadeIn 0.15s ease-out;">
          <input 
            type="text" 
            placeholder={text.presetName}
            bind:value={newPresetName}
            style="background: var(--control-2); border: 1px solid var(--border); color: var(--text); padding: 6px 12px; border-radius: 6px; font-size: 0.8rem; outline: none; width: 140px; height: 32px;"
          />
          <button 
            type="button" 
            onclick={saveCurrentConfigAsPreset}
            style="background: #63e2b7; border: none; color: #121214; padding: 6px 12px; border-radius: 6px; font-size: 0.8rem; font-weight: 600; cursor: pointer; height: 32px; transition: opacity 0.15s;"
            onmouseenter={(e) => e.currentTarget.style.opacity = '0.9'}
            onmouseleave={(e) => e.currentTarget.style.opacity = '1'}
          >
            {text.save}
          </button>
          <button 
            type="button" 
            onclick={() => { isSavingMode = false; newPresetName = ''; }}
            style="background: var(--control-2); border: 1px solid var(--border); color: var(--text); padding: 6px 12px; border-radius: 6px; font-size: 0.8rem; font-weight: 500; cursor: pointer; height: 32px; transition: filter 0.15s;"
            onmouseenter={(e) => e.currentTarget.style.filter = 'brightness(1.08)'}
            onmouseleave={(e) => e.currentTarget.style.filter = 'none'}
          >
            {text.close}
          </button>
        </div>
      {/if}
    </div>
  </div>

  <div class="cards-grid">
    <!-- Group 1: Power & Sleep -->
    <section class="config-card" style="padding-bottom: 6px;">
      <div class="card-head">
        <span><Icon name="zap" size={17} /></span>
        <div>
          <h3>{text.powerTitle}</h3>
        </div>
      </div>
      <label class="control-row">
        <strong>{text.inactiveTime}</strong>
        <input type="range" min="5" max="60" step="1" bind:value={config.inactive_time} />
        <input type="number" min="5" max="60" step="1" bind:value={config.inactive_time} />
      </label>
      <label class="switch-row">
        <strong>{text.enableWake}</strong>
        <input type="checkbox" bind:checked={config.enable_wake} />
      </label>
      <label class="switch-row">
        <strong>{text.disableLed}</strong>
        <input type="checkbox" bind:checked={config.disable_pico_led} />
      </label>
    </section>

    <!-- Group 2: Haptics & Trigger -->
    <section class="config-card" style="padding-bottom: 6px;">
      <div class="card-head">
        <span><Icon name="gamepad" size={17} /></span>
        <div>
          <h3>{text.hapticsTriggerTitle}</h3>
        </div>
      </div>
      <label class="control-row">
        <strong>{text.hapticsGain}</strong>
        <input type="range" min="0.1" max="2" step="0.01" bind:value={config.haptics_gain} />
        <input type="number" min="0.1" max="2" step="0.01" bind:value={config.haptics_gain} />
      </label>
      <label class="control-row">
        <strong>{text.triggerReduce || 'Trigger Reduce'}</strong>
        <input type="range" min="0" max="10" step="1" bind:value={config.trigger_reduce} />
        <input type="number" min="0" max="10" step="1" bind:value={config.trigger_reduce} />
      </label>
      <label class="control-row">
        <strong>{text.hapticsBuffer}</strong>
        <input type="range" min="16" max="128" step="1" bind:value={config.audio_buffer_length} />
        <input type="number" min="16" max="128" step="1" bind:value={config.audio_buffer_length} />
      </label>
    </section>

    <!-- Group 3: Audio & Mic -->
    <section class="config-card" style="padding-bottom: 6px;">
      <div class="card-head">
        <span><Icon name="volume" size={17} /></span>
        <div>
          <h3>{text.audioTitle}</h3>
        </div>
      </div>
      <label class="switch-row">
        <strong>{text.disableSpeaker || 'Disable Speaker'}</strong>
        <input type="checkbox" bind:checked={config.disable_speaker} />
      </label>
      <label class="switch-row">
        <strong>{text.disableMic || 'Disable Mic'}</strong>
        <input type="checkbox" bind:checked={config.disable_mic} />
      </label>
      <label class="control-row">
        <strong>{text.speakerVolume}</strong>
        <input type="range" min="0" max="127" step="1" bind:value={config.speaker_volume} />
        <input type="number" min="0" max="127" step="1" bind:value={config.speaker_volume} />
      </label>
      <label class="control-row">
        <strong>{text.speakerGain || 'Speaker Gain'}</strong>
        <input type="range" min="0" max="7" step="1" bind:value={config.speaker_gain} />
        <input type="number" min="0" max="7" step="1" bind:value={config.speaker_gain} />
      </label>
      <label class="control-row">
        <strong>{text.headsetVolume || 'Headset Volume'}</strong>
        <input type="range" min="0" max="127" step="1" bind:value={config.headset_volume} />
        <input type="number" min="0" max="127" step="1" bind:value={config.headset_volume} />
      </label>
    </section>

    <!-- Group 4: Advanced & System -->
    <section class="config-card compact" style="padding-bottom: 6px;">
      <div class="card-head">
        <span><Icon name="settings" size={17} /></span>
        <div>
          <h3>{text.advancedTitle}</h3>
        </div>
      </div>
      <strong class="field-label" style="margin-top: 2px; margin-bottom: 3px;">{text.pollingMode}</strong>
      <div class="seg wide">
        <button class:active={config.polling_rate_mode === 0} type="button" onclick={() => (config.polling_rate_mode = 0)}>250 Hz</button>
        <button class:active={config.polling_rate_mode === 1} type="button" onclick={() => (config.polling_rate_mode = 1)}>500 Hz</button>
        <button class:active={config.polling_rate_mode === 2} type="button" onclick={() => (config.polling_rate_mode = 2)}>{text.realTime}</button>
      </div>

      <strong class="field-label" style="margin-top: 14px; margin-bottom: 3px;">{text.controllerMode}</strong>
      <div class="seg wide">
        <button class:active={config.controller_mode === 0} type="button" onclick={() => (config.controller_mode = 0)}>DS5</button>
        <button class:active={config.controller_mode === 1} type="button" onclick={() => (config.controller_mode = 1)}>DSE</button>
        <button class:active={config.controller_mode === 2} type="button" onclick={() => (config.controller_mode = 2)}>Auto</button>
      </div>

      <label class="switch-row" style="margin-top: 12px;">
        <strong>{text.enableUsbSn || 'Enable USB SN'}</strong>
        <input type="checkbox" bind:checked={config.enable_usb_sn} />
      </label>
      <label class="switch-row" style="margin-top: 10px;">
        <strong>{text.psShortcutEnabled || 'PS Shortcut'}</strong>
        <input type="checkbox" bind:checked={config.ps_shortcut_enabled} />
      </label>
    </section>
  </div>
</section>

<style>
  .preset-state {
    max-width: 180px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--muted);
    font-size: 0.76rem;
  }

  .preset-state.modified {
    color: #ffab00;
  }

  .preset-tool-btn {
    background: var(--control-2);
    border: 1px solid var(--border);
    color: var(--text);
    padding: 6px 10px;
    border-radius: 6px;
    font-size: 0.78rem;
    font-weight: 500;
    cursor: pointer;
    height: 32px;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    transition: filter 0.15s;
  }

  .preset-tool-btn:hover {
    filter: brightness(1.08);
  }
</style>
