<!-- A row of mutually exclusive options drawn as one inked frame. With `slide`, the ink slides to the chosen option (U14:
     a switch that reads as a switch, not as links). Motion is transform only and stops under reduced motion.
     At rest the chosen button paints its own ink, so a measurement taken before a web font arrived (or on a scaled
     display) can never leave the ink over the next option's first letter; the sliding bar exists only while it moves. -->
<script lang="ts" generics="T extends string">
  import { motionMs } from "./motion.ts";
  let {
    label,
    options,
    value,
    onchange,
    slide = false,
  }: { label: string; options: { value: T; label: string; key?: string; hint?: string }[]; value: T; onchange: (v: T) => void; slide?: boolean } = $props();

  let frame = $state<HTMLDivElement | null>(null);
  let ink = $state<{ x: number; w: number } | null>(null);
  // True only while the ink travels from one option to another.
  let moving = $state(false);
  let shown: T | undefined;
  let timer: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    const v = value;
    const ms = shown !== undefined && shown !== v && slide ? motionMs(frame) : 0;
    if (ms > 0) {
      moving = true;
      clearTimeout(timer);
      timer = setTimeout(() => (moving = false), ms + 40);
    }
    shown = v;
    return () => clearTimeout(timer);
  });
  // The ink is measured from the chosen button's box, and measured again whenever any button changes size (a web font
  // arriving, a label changing, the bar narrowing), so it never covers the next option's first letter.
  $effect(() => {
    void value;
    void options.length;
    const el = frame;
    if (!slide || !el) return;
    const measure = () => {
      const b = el.querySelector<HTMLButtonElement>('button[aria-pressed="true"]');
      if (!b) return void (ink = null);
      const f = el.getBoundingClientRect();
      const r = b.getBoundingClientRect();
      ink = { x: r.left - f.left - el.clientLeft, w: r.width };
    };
    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(el);
    for (const b of el.querySelectorAll("button")) ro.observe(b);
    const fonts = typeof document !== "undefined" ? document.fonts : undefined;
    fonts?.ready.then(measure);
    fonts?.addEventListener("loadingdone", measure);
    return () => {
      ro.disconnect();
      fonts?.removeEventListener("loadingdone", measure);
    };
  });
</script>

<div class="seg" class:slide class:moving role="group" aria-label={label} bind:this={frame}>
  {#if slide && ink}<span class="ink" data-ink aria-hidden="true" style="transform: translateX({ink.x}px) scaleX({ink.w})"></span>{/if}
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
  .slide .ink {
    opacity: 0;
  }
  .slide.moving .ink {
    opacity: 1;
  }
  .slide.moving button[aria-pressed="true"] {
    background: transparent;
  }
  .slide button[aria-pressed="true"] {
    transition: color var(--sse-motion) var(--sse-ease);
  }
</style>
