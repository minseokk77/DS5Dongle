<script lang="ts">
  import Icon from '../Icon.svelte';
  import type { BridgeDevice } from '../api';
  import { getCurrentWindow } from '@tauri-apps/api/window';

  // Svelte 5 Props (Runes) 방식으로 선언
  let {
    devices = [],
    selectedDeviceId = $bindable(),
    isBusy,
    isBridgeConnected = false,
    isControllerConnected = false,
    deviceTitle,
    firmwareLabel,
    rssiLabel,
    rssi = null,
    usbVendorLabel = "",
    usbSpeedLabel = "",
    rssiStatusLabel = "",
    batteryLevel = null,
    isCharging = null,
    text,
    onRefreshDevices,
    onDeviceChanged,
    onOpenInputTester,
    onOpenSettings
  }: {
    devices: BridgeDevice[];
    selectedDeviceId: string;
    isBusy: boolean;
    isBridgeConnected?: boolean;
    isControllerConnected?: boolean;
    deviceTitle: string;
    firmwareLabel: string;
    rssiLabel: string;
    rssi?: number | null;
    usbVendorLabel?: string;
    usbSpeedLabel?: string;
    rssiStatusLabel?: string;
    batteryLevel?: number | null;
    isCharging?: boolean | null;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    text: any;
    onRefreshDevices: () => Promise<void>;
    onDeviceChanged: () => Promise<void>;
    onOpenInputTester?: () => void;
    onOpenSettings?: () => void;
  } = $props();

  let showDiagModal = $state(false);
  let hasNotifiedLowBattery = false;

  // Svelte 5 $derived 룬을 활용해 반응형으로 rssi 수치 계산
  let rssiNum = $derived(
    typeof rssi === 'number'
      ? rssi
      : (typeof rssiLabel === 'string' && rssiLabel.includes('dBm')
          ? parseInt(rssiLabel)
          : null)
  );

  function toggleModal() {
    showDiagModal = !showDiagModal;
  }

  // 배터리 10% 이하 저배터리 알림 트리거 (Svelte 5 $effect 활용)
  $effect(() => {
    if (batteryLevel !== null && batteryLevel !== undefined) {
      if (batteryLevel <= 10 && !isCharging) {
        if (!hasNotifiedLowBattery) {
          triggerLowBatteryNotification(batteryLevel);
          hasNotifiedLowBattery = true;
        }
      } else {
        // 충전 상태가 되거나 10%보다 커지면 알림 플래그 리셋
        hasNotifiedLowBattery = false;
      }
    }
  });

  function triggerLowBatteryNotification(level: number) {
    if (!('Notification' in window)) return;

    if (Notification.permission === 'granted') {
      try {
        new Notification('DS5 Dongle 배터리 부족 경고', {
          body: `DualSense 컨트롤러의 배터리가 ${level}%입니다. 기기가 꺼지기 전에 충전해 주세요.`,
        });
      } catch (e) {
        // 알림 생성 오류 처리 (백그라운드 스레드 제한 등)
      }
    } else if (Notification.permission !== 'denied') {
      Notification.requestPermission().then(permission => {
        if (permission === 'granted') {
          try {
            new Notification('DS5 Dongle 배터리 부족 경고', {
              body: `DualSense 컨트롤러의 배터리가 ${level}%입니다. 기기가 꺼지기 전에 충전해 주세요.`,
            });
          } catch (e) {
            // 알림 생성 오류 처리
          }
        }
      });
    }
  }

  async function minimizeWindow() {
    await getCurrentWindow().minimize();
  }
  async function closeWindow() {
    await getCurrentWindow().hide();
  }
</script>

<section class="device-card" data-tauri-drag-region>
  <div class="device-left" data-tauri-drag-region>
    <div class="square-icon" data-tauri-drag-region><Icon name="cable" size={18} /></div>
    <div style="pointer-events: none;" data-tauri-drag-region>
      <div class="overline">{text.device}</div>
      <div class="device-name">{deviceTitle}</div>
      {#if isBridgeConnected}
      <div class="device-meta" style="display: flex; align-items: center; gap: 14px; flex-wrap: wrap;">
        <span style="display: inline-flex; align-items: center; color: var(--meta-text); font-size: 0.8rem;">
          {text.firmware} {firmwareLabel}
        </span>

        {#if isControllerConnected}
          <span class="rssi-visualizer" style="display: inline-flex; align-items: center; gap: 6px; font-size: 0.8rem;">
            <span class="rssi-bars" style="display: inline-flex; align-items: flex-end; gap: 2.5px; width: 16px; height: 12px;">
              <span style="width: 2.5px; height: 3px; border-radius: 0.5px; background: {rssiNum !== null && rssiNum >= -100 ? (rssiNum < -80 ? '#ffab00' : '#63e2b7') : 'var(--empty-meter)'}; transition: background 0.3s; box-shadow: {rssiNum !== null && rssiNum >= -100 ? (rssiNum < -80 ? '0 0 3px #ffab00' : '0 0 3px #63e2b7') : 'none'};"></span>
              <span style="width: 2.5px; height: 6px; border-radius: 0.5px; background: {rssiNum !== null && rssiNum >= -80 ? '#63e2b7' : 'var(--empty-meter)'}; transition: background 0.3s; box-shadow: {rssiNum !== null && rssiNum >= -80 ? '0 0 3px #63e2b7' : 'none'};"></span>
              <span style="width: 2.5px; height: 9px; border-radius: 0.5px; background: {rssiNum !== null && rssiNum >= -70 ? '#63e2b7' : 'var(--empty-meter)'}; transition: background 0.3s; box-shadow: {rssiNum !== null && rssiNum >= -70 ? '0 0 3px #63e2b7' : 'none'};"></span>
              <span style="width: 2.5px; height: 12px; border-radius: 0.5px; background: {rssiNum !== null && rssiNum >= -60 ? '#63e2b7' : 'var(--empty-meter)'}; transition: background 0.3s; box-shadow: {rssiNum !== null && rssiNum >= -60 ? '0 0 3px #63e2b7' : 'none'};"></span>
            </span>
            <span style="color: var(--meta-text); font-weight: 500;">RSSI {rssiLabel}</span>
          </span>

          {#if batteryLevel !== null && batteryLevel !== undefined}
            <span class="battery-visualizer" style="display: inline-flex; align-items: center; gap: 6px; font-weight: 500; font-size: 0.8rem;">
              <span class="battery-icon-container" style="position: relative; width: 22px; height: 12px; border: 1.5px solid var(--battery-outline); border-radius: 3px; padding: 1px; display: flex; align-items: center; justify-content: flex-start; box-sizing: border-box;">
                <span style="position: absolute; right: -3px; top: 2.5px; width: 1.5px; height: 4px; background: var(--battery-outline); border-radius: 0 1px 1px 0;"></span>
                <span class="battery-bar" class:charging={isCharging} style="width: {batteryLevel}%; height: 100%; border-radius: 1px; background: {isCharging ? '#63e2b7' : (batteryLevel <= 10 ? '#ff4d4f' : batteryLevel <= 40 ? '#ffab00' : '#63e2b7')}; transition: width 0.3s ease, background-color 0.3s ease; box-shadow: 0 0 4px {isCharging ? '#63e2b7' : (batteryLevel <= 10 ? '#ff4d4f' : batteryLevel <= 40 ? '#ffab00' : '#63e2b7')};"></span>
                {#if isCharging}
                  <span class="charging-lightning" style="position: absolute; left: 50%; top: 50%; transform: translate(-50%, -50%); color: #ffffff; display: flex; align-items: center; filter: drop-shadow(0px 0px 1px rgba(0,0,0,0.8));">
                    <Icon name="zap" size={9} />
                  </span>
                {/if}
              </span>
              <span style="color: {isCharging ? '#10b981' : (batteryLevel <= 10 ? '#ff4d4f' : 'var(--meta-text)')}; transition: color 0.3s;">
                {batteryLevel}%
                {#if isCharging}
                  <span style="font-size: 0.72rem; color: #63e2b7; font-weight: 400; margin-left: 2px;">({text.charging})</span>
                {/if}
              </span>
            </span>
          {:else}
            <span style="color: var(--meta-faint); font-size: 0.78rem; display: inline-flex; align-items: center; gap: 4px;">
              <Icon name="battery" size={13} /> {text.batteryUnknown}
            </span>
          {/if}
      {/if}
    </div>
      {/if}
    </div>
  </div>
  <div class="device-right">

    {#if devices.length > 1}
      <select bind:value={selectedDeviceId} disabled={isBusy} aria-label={text.selectDevice} onchange={onDeviceChanged}>
        {#each devices as device}
          <option value={device.id}>{device.label}</option>
        {/each}
      </select>
    {/if}
    
    {#if selectedDeviceId && usbVendorLabel && isBridgeConnected}
      <button class="outline-btn" type="button" onclick={toggleModal} disabled={isBusy} style="margin-right: 8px; background: var(--control-2); border: 1px solid var(--border); color: var(--text); padding: 8px 14px; border-radius: 6px; font-size: 0.85rem; font-weight: 500; cursor: pointer; display: inline-flex; align-items: center; gap: 6px; transition: filter 0.15s, border-color 0.15s;" onmouseenter={(e) => { e.currentTarget.style.filter = 'brightness(1.08)'; }} onmouseleave={(e) => { e.currentTarget.style.filter = 'none'; }}>
        {text.viewDetails}
      </button>

      {#if isControllerConnected}
        <button class="outline-btn" type="button" onclick={onOpenInputTester} disabled={isBusy} style="margin-right: 8px; background: rgba(99, 226, 183, 0.08); border: 1px solid rgba(99, 226, 183, 0.25); color: #63e2b7; padding: 8px 14px; border-radius: 6px; font-size: 0.85rem; font-weight: 500; cursor: pointer; display: inline-flex; align-items: center; gap: 6px; transition: background 0.15s, border-color 0.15s, box-shadow 0.15s;" onmouseenter={(e) => { e.currentTarget.style.background = 'rgba(99, 226, 183, 0.15)'; e.currentTarget.style.borderColor = 'rgba(99, 226, 183, 0.4)'; e.currentTarget.style.boxShadow = '0 0 8px rgba(99, 226, 183, 0.2)'; }} onmouseleave={(e) => { e.currentTarget.style.background = 'rgba(99, 226, 183, 0.08)'; e.currentTarget.style.borderColor = 'rgba(99, 226, 183, 0.25)'; e.currentTarget.style.boxShadow = 'none'; }}>
          <Icon name="gamepad" size={14} />
          {text.openInputTester}
        </button>
      {/if}
    {/if}

    <button class="light-btn" type="button" onclick={onRefreshDevices} disabled={isBusy}>
      <Icon name="cable" size={15} /> {text.connect}
    </button>

    {#if onOpenSettings}
      <button class="outline-btn settings-btn" type="button" onclick={onOpenSettings} aria-label={text.settings} title={text.settings} style="margin-left: 8px; background: transparent; border: 1px solid var(--border); color: var(--text); padding: 8px; border-radius: 6px; cursor: pointer; display: inline-flex; align-items: center; justify-content: center; transition: background 0.15s, color 0.15s;" onmouseenter={(e) => { e.currentTarget.style.background = 'var(--control-hover)'; }} onmouseleave={(e) => { e.currentTarget.style.background = 'transparent'; }}>
        <Icon name="settings" size={16} />
      </button>
    {/if}

    <div class="window-controls compact" style="margin-left: 12px; height: 34px;">
      <button type="button" aria-label="최소화" title="최소화" onclick={minimizeWindow}>−</button>
      <button class="close" type="button" aria-label="닫기" title="닫기" onclick={closeWindow}>×</button>
    </div>
  </div>
</section>

{#if showDiagModal}
  <!-- 모달 오버레이 -->
  <div 
    class="diag-modal-overlay" 
    onclick={toggleModal} 
    onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') toggleModal(); }}
    role="none" 
    style="position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0, 0, 0, 0.6); backdrop-filter: blur(4px); display: flex; align-items: center; justify-content: center; z-index: 9999; animation: fadeIn 0.2s ease-out;"
  >
    <!-- 모달 바디 (오리지널 다크 테마 스타일) -->
    <div 
      class="diag-modal" 
      onclick={(e) => e.stopPropagation()} 
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      style="background: var(--modal-bg); border: 1px solid var(--border); border-radius: 12px; width: 526px; max-width: calc(100vw - 80px); padding: 30px; box-shadow: 0 10px 30px rgba(0, 0, 0, 0.22); color: var(--modal-text); display: flex; flex-direction: column; gap: 18px; position: relative;"
    >
      
      <!-- 헤더 -->
      <div style="display: flex; align-items: center; justify-content: space-between; border-bottom: 1px solid var(--border); padding-bottom: 12px;">
        <h3 style="margin: 0; font-size: 1.05rem; font-weight: 600; color: var(--modal-text); display: flex; align-items: center; gap: 8px;">
          <Icon name="cable" size={16} /> {text.diagTitle}
        </h3>
        <button type="button" onclick={toggleModal} style="background: none; border: none; color: var(--modal-muted); font-size: 1.4rem; cursor: pointer; padding: 0; line-height: 1; transition: color 0.15s;" onmouseenter={(e) => e.currentTarget.style.color = 'var(--modal-text)'} onmouseleave={(e) => e.currentTarget.style.color = 'var(--modal-muted)'}>×</button>
      </div>

      <!-- 콘텐츠 본문 -->
      <div style="display: flex; flex-direction: column; gap: 14px;">
        <div style="display: flex; flex-direction: column; gap: 4px;">
          <span style="color: var(--modal-muted); font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.5px;">{text.diagVendor}</span>
          <span style="color: var(--modal-text); font-size: 0.88rem; font-weight: 500; background: var(--modal-field-bg); padding: 8px 12px; border-radius: 6px; border: 1px solid var(--modal-field-border); line-height: 1.4;">{usbVendorLabel}</span>
        </div>
        
        <div style="display: flex; flex-direction: column; gap: 4px;">
          <span style="color: var(--modal-muted); font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.5px;">{text.diagUsbSpeed}</span>
          <span style="color: var(--modal-text); font-size: 0.88rem; font-weight: 500; background: var(--modal-field-bg); padding: 8px 12px; border-radius: 6px; border: 1px solid var(--modal-field-border); line-height: 1.4;">{usbSpeedLabel}</span>
        </div>

        <div style="display: flex; flex-direction: column; gap: 4px;">
          <span style="color: var(--modal-muted); font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.5px;">{text.diagSignal}</span>
          <span style="color: #63e2b7; font-size: 0.88rem; font-weight: 600; background: rgba(99, 226, 183, 0.06); padding: 8px 12px; border-radius: 6px; border: 1px solid rgba(99, 226, 183, 0.15); line-height: 1.4;">{rssiStatusLabel}</span>
        </div>
      </div>

      <!-- 푸터 -->
      <div style="display: flex; justify-content: flex-end; margin-top: 6px;">
        <button type="button" onclick={toggleModal} style="background: var(--modal-close-bg); border: none; color: var(--modal-text); padding: 8px 16px; border-radius: 6px; font-size: 0.85rem; font-weight: 500; cursor: pointer; transition: background 0.15s;" onmouseenter={(e) => e.currentTarget.style.background = 'var(--modal-close-hover)'} onmouseleave={(e) => e.currentTarget.style.background = 'var(--modal-close-bg)'}>
          {text.close}
        </button>
      </div>

    </div>
  </div>
{/if}

<style>
  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes batteryPulse {
    0% { opacity: 0.5; }
    50% { opacity: 1; }
    100% { opacity: 0.5; }
  }

  :global(.battery-bar.charging) {
    animation: batteryPulse 1.6s infinite ease-in-out;
  }
</style>
