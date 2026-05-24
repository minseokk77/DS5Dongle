<script lang="ts">
  export let name:
    | 'check'
    | 'cable'
    | 'battery'
    | 'download'
    | 'gamepad'
    | 'gauge'
    | 'languages'
    | 'monitor'
    | 'moon'
    | 'power'
    | 'rotate-ccw'
    | 'rotate-cw'
    | 'save'
    | 'settings'
    | 'sliders'
    | 'sun'
    | 'upload'
    | 'volume'
    | 'wifi'
    | 'x'
    | 'zap';
  export let size = 18;
</script>

<svg
  aria-hidden="true"
  class="icon"
  fill="none"
  height={size}
  stroke="currentColor"
  stroke-linecap="round"
  stroke-linejoin="round"
  stroke-width="2"
  viewBox="0 0 24 24"
  width={size}
>
  {#if name === 'battery'}
    <rect x="2" y="7" width="18" height="10" rx="2" />
    <path d="M22 11v2" />
  {:else if name === 'check'}
    <path d="M20 6 9 17l-5-5" />
  {:else if name === 'cable'}
    <path d="M17 7 7 17" />
    <path d="M10 7 7 4 4 7l3 3" />
    <path d="m14 17 3 3 3-3-3-3" />
    <path d="m16 8 2-2" />
    <path d="m8 16-2 2" />
  {:else if name === 'download'}
    <path d="M12 3v12" />
    <path d="m7 10 5 5 5-5" />
    <path d="M5 21h14" />
  {:else if name === 'gamepad'}
    <line x1="6" x2="10" y1="12" y2="12" />
    <line x1="8" x2="8" y1="10" y2="14" />
    <line x1="15" x2="15.01" y1="13" y2="13" />
    <line x1="18" x2="18.01" y1="11" y2="11" />
    <path d="M17.3 6H6.7A4.7 4.7 0 0 0 2 10.7v3.6A3.7 3.7 0 0 0 5.7 18c1.2 0 2.2-.6 2.9-1.5h6.8A3.5 3.5 0 0 0 18.3 18 3.7 3.7 0 0 0 22 14.3v-3.6A4.7 4.7 0 0 0 17.3 6Z" />
  {:else if name === 'gauge'}
    <path d="M4 14a8 8 0 0 1 16 0" />
    <path d="M12 14 16 9" />
  {:else if name === 'languages'}
    <path d="m5 8 6 6" />
    <path d="m4 14 6-6 2-3" />
    <path d="M2 5h12" />
    <path d="M7 2h1" />
    <path d="m22 22-5-10-5 10" />
    <path d="M14 18h6" />
  {:else if name === 'monitor'}
    <rect height="11" rx="2" width="16" x="4" y="5" />
    <path d="M8 21h8" />
    <path d="M12 16v5" />
  {:else if name === 'moon'}
    <path d="M12 3a6 6 0 0 0 9 7.2A9 9 0 1 1 12 3Z" />
  {:else if name === 'power'}
    <path d="M12 2v10" />
    <path d="M18.4 6.6a9 9 0 1 1-12.8 0" />
  {:else if name === 'rotate-ccw'}
    <path d="M3 12a9 9 0 1 0 3-6.7L3 8" />
    <path d="M3 3v5h5" />
  {:else if name === 'rotate-cw'}
    <path d="M21 12a9 9 0 1 1-3-6.7L21 8" />
    <path d="M21 3v5h-5" />
  {:else if name === 'save'}
    <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2Z" />
    <path d="M17 21v-8H7v8" />
    <path d="M7 3v5h8" />
  {:else if name === 'settings'}
    <path d="M12.2 2h-.4a2 2 0 0 0-2 1.8l-.1.8a2 2 0 0 1-2.8 1.4l-.7-.3a2 2 0 0 0-2.5.9l-.2.3a2 2 0 0 0 .4 2.6l.6.5a2 2 0 0 1 0 3.1l-.6.5a2 2 0 0 0-.4 2.6l.2.3a2 2 0 0 0 2.5.9l.7-.3a2 2 0 0 1 2.8 1.4l.1.8a2 2 0 0 0 2 1.8h.4a2 2 0 0 0 2-1.8l.1-.8a2 2 0 0 1 2.8-1.4l.7.3a2 2 0 0 0 2.5-.9l.2-.3a2 2 0 0 0-.4-2.6l-.6-.5a2 2 0 0 1 0-3.1l.6-.5a2 2 0 0 0 .4-2.6l-.2-.3a2 2 0 0 0-2.5-.9l-.7.3a2 2 0 0 1-2.8-1.4l-.1-.8a2 2 0 0 0-2-1.8Z" />
    <circle cx="12" cy="12" r="3" />
  {:else if name === 'sliders'}
    <line x1="4" x2="21" y1="4" y2="4" />
    <line x1="4" x2="21" y1="12" y2="12" />
    <line x1="4" x2="21" y1="20" y2="20" />
    <line x1="9" x2="9" y1="2" y2="6" />
    <line x1="15" x2="15" y1="10" y2="14" />
    <line x1="12" x2="12" y1="18" y2="22" />
  {:else if name === 'sun'}
    <circle cx="12" cy="12" r="4" />
    <path d="M12 2v2" />
    <path d="M12 20v2" />
    <path d="m4.9 4.9 1.4 1.4" />
    <path d="m17.7 17.7 1.4 1.4" />
    <path d="M2 12h2" />
    <path d="M20 12h2" />
    <path d="m6.3 17.7-1.4 1.4" />
    <path d="m19.1 4.9-1.4 1.4" />
  {:else if name === 'volume'}
    <path d="M11 5 6 9H3v6h3l5 4V5Z" />
    <path d="M16 9.5a5 5 0 0 1 0 5" />
  {:else if name === 'wifi'}
    <path d="M5 12.5a10 10 0 0 1 14 0" />
    <path d="M8.5 16a5 5 0 0 1 7 0" />
    <path d="M12 20h.01" />
  {:else if name === 'zap'}
    <path d="M13 2 3 14h8l-1 8 11-14h-8l1-6Z" />
  {:else if name === 'upload'}
    <path d="M12 21V9" />
    <path d="m7 14 5-5 5 5" />
    <path d="M5 3h14" />
  {:else if name === 'x'}
    <path d="M18 6 6 18" />
    <path d="m6 6 12 12" />
  {/if}
</svg>
