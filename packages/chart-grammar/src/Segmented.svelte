<!-- A row of mutually exclusive options drawn as one inked frame. -->
<script lang="ts" generics="T extends string">
  let {
    label,
    options,
    value,
    onchange,
  }: { label: string; options: { value: T; label: string; key?: string }[]; value: T; onchange: (v: T) => void } = $props();
</script>

<div class="seg" role="group" aria-label={label}>
  {#each options as o (o.value)}
    <button
      type="button"
      aria-pressed={o.value === value}
      aria-keyshortcuts={o.key}
      title={o.key ? `${o.label} (${o.key})` : undefined}
      onclick={() => onchange(o.value)}>{o.label}</button
    >
  {/each}
</div>

<style>
  .seg {
    display: flex;
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
  }
  button {
    height: 36px;
    padding: 0 14px;
    border: 0;
    background: transparent;
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    font-size: 13px;
    cursor: pointer;
  }
  button + button {
    border-left: var(--sse-line) solid var(--sse-ink);
  }
  button[aria-pressed="true"] {
    background: var(--sse-ink);
    color: var(--sse-on-ink);
  }
  button:not([aria-pressed="true"]):hover {
    background: var(--sse-rule);
  }
</style>
