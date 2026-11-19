<!--
  The time ruler: always present, and the system's history. A slider over
  the span with ticks, events below the line, authored edits inked in the
  accent, and the present as a heavy accent stroke. Keyboard: arrows 1 %,
  Page Up/Down 10 %, Home/End.
-->
<script lang="ts">
  import { formatYears, fraction, keyStep, labelRows, ticks, type RulerEvent } from "./ruler.ts";

  let {
    from,
    to,
    now,
    events = [],
    label = "Time",
    onchange,
  }: {
    from: number;
    to: number;
    now: number;
    events?: RulerEvent[];
    label?: string;
    onchange?: (yr: number) => void;
  } = $props();

  let width = $state(0);
  let track: HTMLDivElement | undefined = $state();
  const compact = $derived(width > 0 && width < 520);
  const tk = $derived(ticks(from, to, compact ? 5 : 10));
  const pct = (yr: number) => `${(fraction(yr, from, to) * 100).toFixed(3)}%`;

  // Labels take two rows; a label with no clear room in either keeps its marker and moves to the tooltip.
  const labelled = $derived.by(() => {
    const sorted = [...events].filter((e) => e.tone !== "authored").sort((a, b) => a.at_yr - b.at_yr);
    const rows = labelRows(sorted, from, to, width || 800);
    return sorted.map((e, i) => ({ ...e, row: rows[i]! }));
  });
  const authored = $derived(events.filter((e) => e.tone === "authored"));

  function set(yr: number) {
    if (yr !== now) onchange?.(yr);
  }

  function onkeydown(e: KeyboardEvent) {
    const v = keyStep(e.key, now, from, to);
    if (v === null) return;
    e.preventDefault();
    set(v);
  }

  function at(e: PointerEvent): number {
    const r = track!.getBoundingClientRect();
    const f = Math.min(1, Math.max(0, (e.clientX - r.left) / r.width));
    return Math.round(from + f * (to - from));
  }

  function onpointerdown(e: PointerEvent) {
    track!.setPointerCapture(e.pointerId);
    track!.focus();
    set(at(e));
  }

  function onpointermove(e: PointerEvent) {
    if (track!.hasPointerCapture(e.pointerId)) set(at(e));
  }
</script>

<div class="ruler" class:compact>
  <div class="now" aria-hidden="true">{formatYears(now)}</div>
  <div class="trackwrap" bind:clientWidth={width}>
    <div
      bind:this={track}
      class="track"
      role="slider"
      tabindex="0"
      aria-label={label}
      aria-valuemin={from}
      aria-valuemax={to}
      aria-valuenow={now}
      aria-valuetext={formatYears(now)}
      {onkeydown}
      {onpointerdown}
      {onpointermove}
    >
      <div class="line"></div>
      {#each tk as t (t.at)}
        <div class="tick" class:major={t.major} style="left: {pct(t.at)}"></div>
        {#if t.label && !compact && t.at > from && t.at < to}<div class="tl" style="left: {pct(t.at)}" aria-hidden="true">{t.label}</div>{/if}
      {/each}
      {#each labelled as e (e.id)}
        <div class="ev {e.tone}" style="left: {pct(e.at_yr)}" title="{e.label}, {formatYears(e.at_yr)}">
          <span class="pin"></span>
          {#if !compact && e.row >= 0}<span class="evl" class:row1={e.row === 1}>{e.label}</span>{/if}
        </div>
      {/each}
      {#each authored as e (e.id)}
        <div class="inked" style="left: {pct(e.at_yr)}" data-event-id={e.id} title="{e.label}, {formatYears(e.at_yr)}"></div>
      {/each}
      <div class="present" style="left: {pct(now)}"></div>
    </div>
  </div>
  <div class="end" aria-hidden="true">{formatYears(to)}</div>
</div>

<style>
  .ruler {
    display: flex;
    align-items: center;
    gap: 20px;
    height: 100%;
    padding: 0 24px;
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
  }
  .compact {
    gap: 12px;
    padding: 0 16px;
  }
  .now {
    flex: none;
    min-width: 96px;
    font-family: var(--sse-font-display);
    font-size: 22px;
  }
  .compact .now {
    min-width: 72px;
    font-size: 18px;
  }
  .end {
    flex: none;
    font-family: var(--sse-font-mono);
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .trackwrap {
    flex: 1;
    min-width: 0;
    height: 52px;
  }
  .track {
    position: relative;
    height: 100%;
    cursor: ew-resize;
    touch-action: none;
  }
  .track:focus-visible {
    outline: 2px solid var(--sse-focus);
    outline-offset: 4px;
  }
  .line {
    position: absolute;
    left: 0;
    right: 0;
    top: 20px;
    border-top: var(--sse-line) solid var(--sse-ink);
  }
  .tick {
    position: absolute;
    top: 14px;
    height: 6px;
    border-left: var(--sse-line) solid var(--sse-ink);
  }
  .tick.major {
    top: 10px;
    height: 10px;
  }
  .tl {
    position: absolute;
    top: -4px;
    transform: translateX(-50%);
    font-family: var(--sse-font-mono);
    font-size: 10px;
    color: var(--sse-ink-muted);
    white-space: nowrap;
  }
  .ev {
    position: absolute;
    top: 20px;
  }
  .pin {
    position: absolute;
    left: -0.5px;
    top: 0;
    height: 6px;
    border-left: var(--sse-line) solid var(--sse-ink-muted);
  }
  .evl {
    position: absolute;
    top: 8px;
    left: -4px;
    white-space: nowrap;
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  .evl.row1 {
    top: 20px;
  }
  .ev.future .evl {
    color: var(--sse-accent-ink);
    font-style: italic;
  }
  .inked {
    position: absolute;
    top: 14px;
    width: 8px;
    height: 8px;
    margin-left: -4px;
    transform: rotate(45deg);
    background: var(--sse-accent);
  }
  .present {
    position: absolute;
    top: 2px;
    height: 40px;
    border-left: 2px solid var(--sse-accent);
    pointer-events: none;
  }
</style>
