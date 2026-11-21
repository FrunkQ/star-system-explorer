<!-- A row of mutually exclusive options drawn as one inked frame. With `slide`, the ink slides to the chosen option (U14:
     a switch that reads as a switch, not as links). Motion is transform only and stops under reduced motion. -->
<script lang="ts" generics="T extends string">
  let {
    label,
    options,
    value,
    onchange,
    slide = false,
  }: { label: string; options: { value: T; label: string; key?: string; hint?: string }[]; value: T; onchange: (v: T) => void; slide?: boolean } = $props();

  let frame = $state<HTMLDivElement | null>(null);
  let ink = $state<{ x: number; w: number } | null>(null);
  $effect(() => {
    void value;
    void options.length;
    const el = frame;
    if (!slide || !el) return;
    const measure = () => {
      const b = el.querySelector<HTMLButtonElement>('button[aria-pressed="true"]');
      ink = b ? { x: b.offsetLeft, w: b.offsetWidth } : null;
    };
    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(el);
    return () => ro.disconnect();
  });
</script>

<div class="seg" class:slide role="group" aria-label={label} bind:this={frame}>
  {#if slide && ink}<span class="ink" aria-hidden="true" style="transform: translateX({ink.x}px) scaleX({ink.w})"></span>{/if}
  {#each options as o (o.value)}
    <button
      type="button"
      aria-pressed={o.value === value}
      aria-keyshortcuts={o.key}
      title={o.hint ?? (o.key ? `${o.label} (${o.key})` : undefined)}
      onclick={() => onchange(o.value)}>{o.label}</button
    >
  {/each}
</div>

<style>
  .seg {
    position: relative;
    display: flex;
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
  }
  button {
    position: relative;
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
  /* Sliding ink: one bar behind the buttons, moved and stretched by transform only. */
  .ink {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 1px;
    background: var(--sse-ink);
    transform-origin: left center;
    transition: transform var(--sse-motion) var(--sse-ease);
  }
  .slide button + button {
    border-left-color: transparent;
  }
  .slide button[aria-pressed="true"] {
    background: transparent;
    transition: color var(--sse-motion) var(--sse-ease);
  }
</style>
