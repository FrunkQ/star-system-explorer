<!--
  The count (SH3, design section 4.5): people in space through time as one stepped line, from the pack's own step list
  (or the roster's), across the source's whole span. The day in view is the clock's accent; days someone died are small
  hollow marks in ink, not red and with no symbol. Hover reads a date and a count; pressing goes there.
  Drawn in the Roll call panel rather than along the ruler: the ruler's row is a fixed height in the shared shell (open
  seam, handover/packages/SH3.md).
-->
<script lang="ts">
  import { countAt, sayNumber } from "../../../ui-contract/src/index.ts";

  let { steps, from_s, to_s, at_s, losses, dateText, ongo }: { steps: readonly (readonly [number, number])[]; from_s: number; to_s: number; at_s: number; losses: readonly number[]; dateText: (s: number) => string; ongo: (s: number) => void } = $props();

  const W = 1000;
  const H = 40;
  const span = $derived(Math.max(1, to_s - from_s));
  const peak = $derived(Math.max(1, ...steps.map(([, n]) => n)));
  const x = (s: number) => ((s - from_s) / span) * W;
  const y = (n: number) => H - 2 - (n / peak) * (H - 6);
  // One stepped path: across at each count, then up or down at the next step.
  const path = $derived.by(() => {
    let d = `M0 ${y(countAt(steps, from_s))}`;
    for (const [t, n] of steps) {
      if (t <= from_s || t >= to_s) continue;
      d += ` H${x(t)} V${y(n)}`;
    }
    return `${d} H${W}`;
  });
  let hover = $state<number | null>(null);
  const atPointer = (e: PointerEvent) => {
    const r = (e.currentTarget as SVGElement).getBoundingClientRect();
    return from_s + ((e.clientX - r.left) / Math.max(1, r.width)) * span;
  };
  const words = (s: number) => {
    const n = countAt(steps, s);
    return `${dateText(s)}: ${sayNumber(n, 6)} ${n === 1 ? "person" : "people"} in space`;
  };
</script>

<figure class="strip" data-rollcall-strip>
  <svg
    viewBox="0 0 {W} {H}"
    preserveAspectRatio="none"
    role="slider"
    tabindex="0"
    aria-label="People in space through time; press to go to a day"
    aria-valuemin={from_s}
    aria-valuemax={to_s}
    aria-valuenow={at_s}
    aria-valuetext={words(at_s)}
    onpointermove={(e) => (hover = atPointer(e))}
    onpointerleave={() => (hover = null)}
    onclick={(e) => ongo(atPointer(e as unknown as PointerEvent))}
    onkeydown={(e) => {
      const step = span / 100;
      if (e.key === "ArrowRight") (e.preventDefault(), ongo(Math.min(to_s, at_s + step)));
      if (e.key === "ArrowLeft") (e.preventDefault(), ongo(Math.max(from_s, at_s - step)));
    }}
  >
    <path class="area" d="{path} V{H} H0 Z" />
    <path class="count" d={path} vector-effect="non-scaling-stroke" />
    {#if at_s >= from_s && at_s <= to_s}<line class="now" x1={x(at_s)} x2={x(at_s)} y1="0" y2={H} vector-effect="non-scaling-stroke" />{/if}
    {#if hover !== null}<line class="hover" x1={x(hover)} x2={x(hover)} y1="0" y2={H} vector-effect="non-scaling-stroke" />{/if}
  </svg>
  <div class="losses" aria-hidden="true">
    {#each losses as l, i (i)}
      {#if l >= from_s && l <= to_s}<button class="loss" style="left: {(x(l) / W) * 100}%" tabindex="-1" title={dateText(l)} onclick={() => ongo(l)} data-rollcall-strip-loss></button>{/if}
    {/each}
  </div>
  <figcaption data-rollcall-strip-read>{hover !== null ? words(hover) : `People in space, ${dateText(from_s)} to ${dateText(to_s)}`}</figcaption>
</figure>

<style>
  .strip {
    margin: 0 0 12px;
  }
  svg {
    display: block;
    width: 100%;
    height: 40px;
    cursor: pointer;
    outline-offset: 2px;
  }
  .count {
    fill: none;
    stroke: var(--sse-ink);
    stroke-width: 1;
  }
  .area {
    fill: var(--sse-rule);
    opacity: 0.6;
  }
  .losses {
    position: relative;
    height: 9px;
  }
  .loss {
    position: absolute;
    top: 2px;
    width: 6px;
    height: 6px;
    margin-left: -3px;
    padding: 0;
    border: 1px solid var(--sse-ink);
    border-radius: 50%;
    background: var(--sse-ground-raised);
    cursor: pointer;
  }
  .now {
    stroke: var(--sse-accent);
    stroke-width: 1.5;
  }
  .hover {
    stroke: var(--sse-ink-muted);
    stroke-width: 1;
    stroke-dasharray: 2 2;
  }
  figcaption {
    margin-top: 2px;
    font-size: 0.75rem;
    color: var(--sse-ink-muted);
    font-variant-numeric: tabular-nums;
  }
</style>
