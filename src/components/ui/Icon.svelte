<script>
  /**
   * Tiny dependency-free icon set.
   * Usage: <Icon name="play" size={16} class="text-zinc-400" />
   */
  let { name, size = 16, class: className = '' } = $props();
</script>

{#if name === 'play'}
  <svg viewBox="0 0 24 24" width={size} height={size} class={className} fill="currentColor" aria-hidden="true">
    <path d="M7.5 5.4c0-1 1.1-1.7 2-1.1l10.7 6.6c.9.5.9 1.8 0 2.3L9.5 19.8c-.9.6-2-.1-2-1.1V5.4Z" />
  </svg>
{:else if name === 'pause'}
  <svg viewBox="0 0 24 24" width={size} height={size} class={className} fill="currentColor" aria-hidden="true">
    <rect x="6" y="4.5" width="4.2" height="15" rx="1.1" />
    <rect x="13.8" y="4.5" width="4.2" height="15" rx="1.1" />
  </svg>
{:else if name === 'shuffle'}
  <svg viewBox="0 0 24 24" width={size} height={size} class={className} fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M2.5 6.5h3.4c1.3 0 2.5.7 3.2 1.8l4.8 7.4c.7 1.1 1.9 1.8 3.2 1.8h3.4" />
    <path d="M17.5 3.5l4 3.5-4 3.5" />
    <path d="M2.5 17.5h3.4c1.3 0 2.5-.7 3.2-1.8l.6-.9" />
    <path d="M13.9 9.2l.6-.9c.7-1.1 1.9-1.8 3.2-1.8h3.4" />
    <path d="M17.5 20.5l4-3.5-4-3.5" />
  </svg>
{:else if name === 'more-horizontal'}
  <svg viewBox="0 0 24 24" width={size} height={size} class={className} fill="currentColor" aria-hidden="true">
    <circle cx="4.5" cy="12" r="2" />
    <circle cx="12" cy="12" r="2" />
    <circle cx="19.5" cy="12" r="2" />
  </svg>
{:else if name === 'clock'}
  <svg viewBox="0 0 24 24" width={size} height={size} class={className} fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <circle cx="12" cy="12" r="9" />
    <path d="M12 7.5V12l3.2 1.9" />
  </svg>
{:else if name === 'chevron-up'}
  <svg viewBox="0 0 24 24" width={size} height={size} class={className} fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M5 14.5 12 7.5l7 7" />
  </svg>
{:else if name === 'chevron-down'}
  <svg viewBox="0 0 24 24" width={size} height={size} class={className} fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M5 9.5 12 16.5l7-7" />
  </svg>
{:else if name === 'chevron-right'}
  <svg viewBox="0 0 24 24" width={size} height={size} class={className} fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M9 5l7 7-7 7" />
  </svg>
{:else if name === 'heart'}
  <svg viewBox="0 0 24 24" width={size} height={size} class={className} fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M12 20.2s-7.4-4.5-9.9-9.1C.7 7.9 2.1 4.6 5.4 3.8c2-.5 3.9.3 5.1 1.9 1.1-1.6 3-2.4 5.1-1.9 3.2.8 4.6 4.1 2.9 7.3-2.5 4.6-5.9 9.1-5.9 9.1Z" />
  </svg>
{:else if name === 'heart-filled'}
  <svg viewBox="0 0 24 24" width={size} height={size} class={className} fill="currentColor" aria-hidden="true">
    <path d="M12 20.2s-7.4-4.5-9.9-9.1C.7 7.9 2.1 4.6 5.4 3.8c2-.5 3.9.3 5.1 1.9 1.1-1.6 3-2.4 5.1-1.9 3.2.8 4.6 4.1 2.9 7.3-2.5 4.6-5.9 9.1-5.9 9.1Z" />
  </svg>
{:else if name === 'list-plus'}
  <svg viewBox="0 0 24 24" width={size} height={size} class={className} fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M3.5 6.5h11M3.5 12h7.5M3.5 17.5h7.5" />
    <path d="M17.8 13.5v6M14.8 16.5h6" />
  </svg>
{:else if name === 'skip-forward'}
  <svg viewBox="0 0 24 24" width={size} height={size} class={className} fill="currentColor" aria-hidden="true">
    <path d="M5.5 5.8c0-1 1-1.6 1.9-1.1l8.4 5.2c.8.5.8 1.7 0 2.2l-8.4 5.2c-.9.5-1.9-.1-1.9-1.1V5.8Z" />
    <rect x="17" y="5" width="2.3" height="14" rx="0.6" />
  </svg>
{:else if name === 'plus'}
  <svg viewBox="0 0 24 24" width={size} height={size} class={className} fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M12 4.5v15M4.5 12h15" />
  </svg>
{:else if name === 'info'}
  <svg viewBox="0 0 24 24" width={size} height={size} class={className} fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <circle cx="12" cy="12" r="9" />
    <path d="M12 11v5.2M12 7.8v.01" />
  </svg>
{:else if name === 'user'}
  <svg viewBox="0 0 24 24" width={size} height={size} class={className} fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <circle cx="12" cy="8.2" r="3.6" />
    <path d="M4.3 20c1-3.6 4.1-5.7 7.7-5.7s6.7 2.1 7.7 5.7" />
  </svg>
{:else if name === 'disc'}
  <svg viewBox="0 0 24 24" width={size} height={size} class={className} fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <circle cx="12" cy="12" r="9" />
    <circle cx="12" cy="12" r="2.4" />
  </svg>
{:else if name === 'x'}
  <svg viewBox="0 0 24 24" width={size} height={size} class={className} fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M6 6l12 12M18 6 6 18" />
  </svg>
{:else if name === 'sliders'}
  <svg viewBox="0 0 24 24" width={size} height={size} class={className} fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M4 6.5h8M16.2 6.5H20M4 12h2.2M10.2 12H20M4 17.5h12.2M19.8 17.5H20" />
    <circle cx="13.2" cy="6.5" r="1.8" fill="currentColor" stroke="none" />
    <circle cx="8.2" cy="12" r="1.8" fill="currentColor" stroke="none" />
    <circle cx="18.2" cy="17.5" r="1.8" fill="currentColor" stroke="none" />
  </svg>
{:else if name === 'trash'}
  <svg viewBox="0 0 24 24" width={size} height={size} class={className} fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M5 7h14M9.5 7V5.2c0-.7.5-1.2 1.2-1.2h4.6c.7 0 1.2.5 1.2 1.2V7m-8.7 0 1 12.3c.1.8.7 1.4 1.5 1.4h6.4c.8 0 1.4-.6 1.5-1.4L17.7 7" />
    <path d="M10.3 11v6M13.7 11v6" />
  </svg>
{:else if name === 'edit'}
  <svg viewBox="0 0 24 24" width={size} height={size} class={className} fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M4.5 19.5l.8-3.4L15.6 5.8l2.6 2.6L8 18.7l-3.5.8Z" />
  </svg>
{/if}
