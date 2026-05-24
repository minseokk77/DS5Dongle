<script lang="ts">
  import Icon from '../Icon.svelte';
  import type { Lang } from '../i18n';

  type ThemeMode = 'light' | 'dark' | 'system';

  export let lang: Lang;
  export let themeMode: ThemeMode;
  export let isConnected = false;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  export let text: any;
  export let showStatus = true;
  export let onLangChange: (lang: Lang) => void = () => {};
  export let onThemeChange: (themeMode: ThemeMode) => void = () => {};

  function switchLanguage(nextLang: Lang) {
    onLangChange(nextLang);
  }

  function switchTheme(nextThemeMode: ThemeMode) {
    onThemeChange(nextThemeMode);
  }
</script>

<div class="toolbar compact-toolbar">
  <span class="translate" aria-hidden="true"><Icon name="languages" size={16} /></span>
  <div class="seg small" aria-label={text.selectLanguage}>
    <button class:active={lang === 'ko'} type="button" onclick={() => switchLanguage('ko')}>
      {text.langKo}
    </button>
    <button class:active={lang === 'en'} type="button" onclick={() => switchLanguage('en')}>
      {text.langEn}
    </button>
    <button class:active={lang === 'zh'} type="button" onclick={() => switchLanguage('zh')}>
      {text.langZh}
    </button>
  </div>
  <div class="seg icon-group" aria-label={text.selectTheme}>
    <button
      class:active={themeMode === 'light'}
      type="button"
      onclick={() => switchTheme('light')}
      title={text.lightTheme}
    >
      <Icon name="sun" size={15} />
    </button>
    <button
      class:active={themeMode === 'dark'}
      type="button"
      onclick={() => switchTheme('dark')}
      title={text.darkTheme}
    >
      <Icon name="moon" size={15} />
    </button>
    <button
      class:active={themeMode === 'system'}
      type="button"
      onclick={() => switchTheme('system')}
      title={text.systemTheme}
    >
      <Icon name="monitor" size={15} />
    </button>
  </div>
  {#if showStatus}
    <div class:connected={isConnected} class="status-pill">
      <span><Icon name="check" size={10} /></span>
      {isConnected ? text.controllerConnected : text.controllerDisconnected}
    </div>
  {/if}
</div>
