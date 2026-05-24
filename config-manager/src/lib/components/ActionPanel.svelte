<script lang="ts">
  import Icon from '../Icon.svelte';

  export let isConnected: boolean;
  export let isBusy: boolean;
  export let isDirty: boolean;
  export let statusText: string;
  export let updateStepText = '';
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  export let text: any;

  export let onRead: () => Promise<void>;
  export let onSave: () => Promise<void>;
  export let onReconnect: () => Promise<void>;
  export let onFirmwareUpdate: () => Promise<void>;
</script>

<aside class="actions-panel panel-dark">
  <div class="section-title">
    <span><Icon name="download" size={18} /></span>
    <h2>{text.actions}</h2>
  </div>
  <div class="action-stack">
    <button type="button" onclick={onRead} disabled={!isConnected || isBusy}>
      <Icon name="rotate-cw" size={15} /> {text.read}
    </button>
    <button class="primary" type="button" onclick={onSave} disabled={!isConnected || isBusy}>
      <Icon name="save" size={15} /> {text.saveToFlash}
    </button>
    <button type="button" onclick={onReconnect} disabled={!isConnected || isBusy}>
      <Icon name="power" size={15} /> {text.reconnectUsb}
    </button>
    <button type="button" onclick={onFirmwareUpdate} disabled={isBusy}>
      <Icon name="download" size={15} /> {text.firmwareUpdate}
    </button>
  </div>
  <div class="state-card">
    <div class="overline">{text.state}</div>
    <strong>{statusText}</strong>
    {#if updateStepText}
      <p>{updateStepText}</p>
    {/if}
    {#if isDirty}
      <p>{text.dirty}</p>
    {/if}
  </div>
</aside>
