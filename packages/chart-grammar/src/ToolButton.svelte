<!-- A square tool button with an inline line icon. `pressed` marks the active tool. -->
<script lang="ts">
  let {
    label,
    icon,
    pressed,
    onclick,
    keyhint,
    disabled = false,
  }: { label: string; icon: string; pressed?: boolean; onclick?: () => void; keyhint?: string; disabled?: boolean } = $props();
</script>

<button
  type="button"
  class="tool"
  aria-label={label}
  aria-pressed={pressed === undefined ? undefined : pressed}
  aria-keyshortcuts={keyhint}
  title={keyhint ? `${label} (${keyhint})` : label}
  {onclick}
  {disabled}
>
  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" aria-hidden="true"><path d={icon} /></svg>
</button>

<style>
  .tool {
    width: 44px;
    height: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    cursor: pointer;
  }
  .tool[aria-pressed="true"] {
    background: var(--sse-ink);
    color: var(--sse-on-ink);
  }
  .tool:disabled {
    color: var(--sse-ink-faint);
    border-color: var(--sse-ink-faint);
    cursor: default;
    box-shadow: none;
  }
  .tool:hover:not(:disabled) {
    box-shadow: 2px 2px 0 var(--sse-ink);
  }
</style>
