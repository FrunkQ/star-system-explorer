<!--
  The time ruler: always present, and the system's history. One ruler from
  the Big Bang to hours (U11): it shows a window of the span that zooms
  (wheel, the − and + buttons, or the Show list) and changes register as it
  goes, from billions of years to calendar dates and hours. Events sit below
  the line, authored edits are inked in the accent, the time being viewed is
  a heavy accent stroke, and the campaign's own "now" is a stamped marker.
  Viewing another time never moves the campaign's now; "Make this now" does.
  While the view is at the campaign's now, stepping and playing move both:
  time passes in the campaign. The track itself (pointer or keyboard) only
  looks: arrows 1 % of the window, Page Up/Down 10 %, Home/End, + and − zoom.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { fraction, keyStep, labelRows, type RulerEvent } from "./ruler.ts";
  import { motionMs } from "./motion.ts";
  import { constant, drawn } from "../../ui-contract/src/index.ts";
  import { around, between, calendarTicks, defaultCalendar, spanWords, steps, timeWords, zoom, type Calendar, type RulerWindow } from "./calendar.ts";

  let {
    from,
    to,
    now,
    campaignNow,
    calendar,
    events = [],
    label = "Time",
    onchange,
    onmakenow,
    note,
    opens,
    onspan,
  }: {
    from: number;
    to: number;
    now: number;
    /** The campaign's own now; without it the ruler is a plain history slider. */
    campaignNow?: number;
    calendar?: Calendar;
    events?: RulerEvent[];
    label?: string;
    onchange?: (yr: number) => void;
    onmakenow?: (yr: number) => void;
    /** A caveat shown while the window is in human time (under 10,000 years). */
    note?: string;
    /**
     * The span the ruler opens on (top-level design 12.1, by desk and level): a length of time around the time in view,
     * one of the Show list's ids ("week"), or null for the whole history. Applied when `key` changes (a desk or level switch), not on every new span, so a
     * zoom is never re-centred. With it, "Whole life ›" shows beside a narrower window.
     */
    opens?: { key: string; span: number | string | null };
    /** The GM changed the span (zoom, Show, Whole life ›): its length, or null for the whole history, to remember. */
    onspan?: (span: number | null) => void;
  } = $props();

  const cal = $derived(calendar ?? defaultCalendar(campaignNow ?? now));
  const bounds = $derived({ from, to });
  let win = $state<RulerWindow | null>(null);
  // The window follows the bounds until the GM zooms; it is clamped if the span shrinks.
  const wt = $derived.by((): RulerWindow => {
    if (!win) return { from, to };
    const span = Math.min(win.to - win.from, to - from);
    const f = Math.min(Math.max(from, win.from), to - span);
    return { from: f, to: f + span };
  });
  // What is drawn: the target window, or a glide towards it after a button,
  // key or Show (the wheel is continuous already). Readout, register and
  // stepping always use the target, so nothing waits on the motion.
  let anim = $state<RulerWindow | null>(null);
  const w = $derived(anim ?? wt);
  const span = $derived(wt.to - wt.from);
  let glideRaf = 0;
  function glide(next: RulerWindow | null, remember = true) {
    const prev = w;
    win = next;
    if (remember) said();
    const target = wt;
    cancelAnimationFrame(glideRaf);
    anim = null;
    const ms = motionMs(track ?? null);
    if (ms <= 0 || (prev.from === target.from && prev.to === target.to)) return;
    const t0 = performance.now();
    const step = (t: number) => {
      const k = Math.min(1, (t - t0) / ms);
      anim = k >= 1 ? null : between(prev, target, 1 - Math.pow(1 - k, 3));
      if (k < 1) glideRaf = requestAnimationFrame(step);
    };
    glideRaf = requestAnimationFrame(step);
  }
  $effect(() => () => cancelAnimationFrame(glideRaf));
  const full = $derived(span >= (to - from) * 0.999);

  let width = $state(0);
  let track: HTMLDivElement | undefined = $state();
  const compact = $derived(width > 0 && width < 520);
  const tk = $derived(calendarTicks(w.from, w.to, cal, compact ? 4 : 8));
  const pct = (yr: number) => `${drawn(fraction(yr, w.from, w.to) * 100, 3)}%`;
  const inView = (yr: number) => yr >= w.from && yr <= w.to;

  const shown = $derived(events.filter((e) => inView(e.at_yr)));
  const labelled = $derived.by(() => {
    const sorted = shown.filter((e) => e.tone !== "authored").sort((a, b) => a.at_yr - b.at_yr);
    const rows = labelRows(sorted, w.from, w.to, width || 800);
    return sorted.map((e, i) => ({ ...e, row: rows[i]! }));
  });
  const authored = $derived(shown.filter((e) => e.tone === "authored"));

  const linked = $derived(campaignNow !== undefined && Math.abs(now - campaignNow) <= Math.max(1e-6, Math.abs(now) * 1e-15));
  function set(yr: number, moveCampaign = false) {
    const v = Math.min(to, Math.max(from, yr));
    if (v !== now) onchange?.(v);
    if (moveCampaign && campaignNow !== undefined && v !== campaignNow) onmakenow?.(v);
    // Keep the viewed time on screen.
    if (win && !inView(v)) win = around(v, span, bounds);
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "+" || e.key === "=") return void (e.preventDefault(), zoomBy(0.5, now));
    if (e.key === "-" || e.key === "_") return void (e.preventDefault(), zoomBy(2, now));
    const v = keyStep(e.key, now, wt.from, wt.to);
    if (v === null) return;
    e.preventDefault();
    // keyStep rounds to whole years; inside a human window keep the fraction.
    const step = span < 1e4 ? { ArrowRight: 0.01, ArrowUp: 0.01, ArrowLeft: -0.01, ArrowDown: -0.01, PageUp: 0.1, PageDown: -0.1 }[e.key] : undefined;
    set(step !== undefined ? now + span * step : v);
  }

  function at(e: PointerEvent | WheelEvent): number {
    const r = track!.getBoundingClientRect();
    const f = Math.min(1, Math.max(0, (e.clientX - r.left) / r.width));
    const t = w.from + f * (w.to - w.from);
    return span > 1e4 ? Math.round(t) : t;
  }
  function onpointerdown(e: PointerEvent) {
    track!.setPointerCapture(e.pointerId);
    track!.focus();
    set(at(e));
  }
  function onpointermove(e: PointerEvent) {
    if (track!.hasPointerCapture(e.pointerId)) set(at(e));
  }
  function zoomBy(factor: number, about: number, smooth = true) {
    const next = zoom(wt, factor, about, bounds);
    if (smooth) glide(next);
    else ((anim = null), (win = next), said());
  }
  /** Tell the shell the span the GM chose, as a length (a difference of times), or null for the whole history. */
  function said() {
    onspan?.(full ? null : span);
  }
  // The opening span, once per key (a desk or level switch).
  let openedKey: string | null = null;
  $effect(() => {
    const o = opens;
    if (!o || o.key === openedKey) return;
    untrack(() => {
      openedKey = o.key;
      const len = typeof o.span === "string" ? SHOWS.find((x) => x.id === o.span)?.span : o.span;
      // At once, not a glide: the desk or level changed the whole screen, and the ruler arrives with it.
      cancelAnimationFrame(glideRaf);
      anim = null;
      win = len === null || len === undefined || !(len > 0) || len === Infinity ? null : around(now, len, bounds);
    });
  });
  // Wheel zoom about the pointer (non-passive, so the page does not scroll).
  $effect(() => {
    const el = track;
    if (!el) return;
    const h = (e: WheelEvent) => {
      if (Math.abs(e.deltaY) < Math.abs(e.deltaX)) return;
      e.preventDefault();
      zoomBy(Math.pow(1.0015, e.deltaY), at(e), false);
    };
    el.addEventListener("wheel", h, { passive: false });
    return () => el.removeEventListener("wheel", h);
  });

  // "Show": windows around the time being viewed.
  const DAY = $derived(cal.day_s / constant("yr_julian"));
  const SHOWS = $derived([
    { id: "all", label: "Whole history", span: Infinity },
    { id: "myr", label: "10 million years", span: 1e7 },
    { id: "kyr", label: "1,000 years", span: 1e3 },
    { id: "decade", label: "10 years", span: (10 * cal.year_days * cal.day_s) / constant("yr_julian") },
    { id: "year", label: "A year", span: (cal.year_days * cal.day_s) / constant("yr_julian") },
    { id: "month", label: "A month", span: 30 * DAY },
    { id: "week", label: "A week", span: 7 * DAY },
    { id: "day", label: "A day", span: DAY },
  ]);
  function show(id: string) {
    const s = SHOWS.find((x) => x.id === id);
    if (!s) return;
    glide(s.span === Infinity ? null : around(now, s.span, bounds));
  }

  // Stepping and playing.
  const STEPS = $derived(steps(cal));
  let stepId = $state("day");
  const stepYr = $derived(STEPS.find((s) => s.id === stepId)?.yr ?? DAY);
  let playing = $state(false);
  /**
   * Play, and make the time passing visible: when the window is far wider than the step (a day a second on the whole
   * of history moves nothing you can see), the ruler first closes in on the smallest "Show" window that holds about
   * thirty steps, so the readout and the marker move. The GM can zoom out again while it plays.
   */
  function play() {
    if (span > stepYr * 600) {
      const fit = SHOWS.filter((x) => x.span !== Infinity && x.span >= stepYr * 30).sort((a, b) => a.span - b.span)[0];
      if (fit && fit.span < span) glide(around(now, fit.span, bounds), false);
    }
    playing = true;
  }
  $effect(() => {
    if (!playing) return;
    let last: number | null = null;
    let sent = 0;
    let t = now;
    const move = linked;
    let raf = 0;
    const frame = (ms: number) => {
      if (last !== null) {
        t += ((ms - last) / 1000) * stepYr;
        if (t >= to) ((t = to), (playing = false));
        // Ten updates a second is plenty for the chart and keeps the kernel calm.
        if (ms - sent >= 100 || !playing) ((sent = ms), set(t, move));
      }
      last = ms;
      if (playing) raf = requestAnimationFrame(frame);
    };
    raf = requestAnimationFrame(frame);
    return () => cancelAnimationFrame(raf);
  });
  const nowWords = $derived(timeWords(now, cal, span));
  const campaignWords = $derived(campaignNow === undefined ? "" : timeWords(campaignNow, cal, Math.min(span, 1e4 - 1)));
</script>

<div class="ruler" class:compact data-span={span} data-linked={linked ? "yes" : "no"}>
  <div class="readout">
    <div class="now" data-viewing={now}>{nowWords}</div>
    {#if campaignNow !== undefined}
      {#if linked}
        <div class="sub"><span class="stamp">Now</span> in the campaign</div>
      {:else}
        <div class="sub" data-campaign-now={campaignNow}>
          Campaign now: {campaignWords}
        </div>
      {/if}
    {/if}
  </div>
  <div class="mid">
    <div class="bar" role="toolbar" aria-label="Time controls">
      <span class="grp">
        <button type="button" onclick={() => zoomBy(2, now)} disabled={full} aria-label="Zoom out" title="Zoom out (−)">−</button>
        <button type="button" onclick={() => zoomBy(0.5, now)} aria-label="Zoom in" title="Zoom in (+)">+</button>
        {#if opens && !full}<button type="button" class="link" data-whole-life onclick={() => glide(null)} title="Show the whole of this place's life, birth to death">Whole life ›</button>{/if}
        <select aria-label="Show" onchange={(e) => (show(e.currentTarget.value), (e.currentTarget.value = ""))}>
          <option value="">{full ? "Whole history" : `Showing ${spanWords(span, cal)}`}</option>
          {#each SHOWS as s (s.id)}<option value={s.id}>{s.label}</option>{/each}
        </select>
      </span>
      {#if campaignNow !== undefined}
        <span class="grp">
          <button type="button" onclick={() => set(now - stepYr, linked)} aria-label="Back one {STEPS.find((s) => s.id === stepId)?.label}" title="Back one step">‹</button>
          <button type="button" class:on={playing} onclick={() => (playing ? (playing = false) : play())} aria-label={playing ? "Pause" : "Play"} aria-pressed={playing} title={linked ? "Play: time passes in the campaign" : "Play: look ahead without moving the campaign's now"}>{playing ? "❚❚" : "▶"}</button>
          <button type="button" onclick={() => set(now + stepYr, linked)} aria-label="Forward one {STEPS.find((s) => s.id === stepId)?.label}" title="Forward one step">›</button>
          <select aria-label="Step" bind:value={stepId}>
            {#each STEPS as s (s.id)}<option value={s.id}>{s.label === "1,000 years" || s.label === "million years" ? `${s.label === "million years" ? "a million years" : "1,000 years"}` : `a ${s.label}`}</option>{/each}
          </select>
          <span class="hint">{playing ? `${spanWords(stepYr, cal)} a second` : ""}</span>
        </span>
        <span class="grp end-grp">
          {#if !linked}
            <button type="button" class="link" onclick={() => (set(campaignNow), win && (win = around(campaignNow, span, bounds)))}>Back to now</button>
            <button type="button" class="link accent" onclick={() => onmakenow?.(now)} title="Move the campaign's now to the time you are viewing; players' screens follow">Make this now</button>
          {/if}
        </span>
      {/if}
      {#if note && span <= 1e4}<span class="note" data-note>{note}</span>{/if}
      <span class="end" aria-hidden="true">{full ? `to ${timeWords(to, cal, span)}` : `to ${timeWords(wt.to, cal, span)}`}</span>
    </div>
    <div class="trackwrap" bind:clientWidth={width}>
      <div
        bind:this={track}
        class="track"
        role="slider"
        tabindex="0"
        aria-label={label}
        aria-valuemin={wt.from}
        aria-valuemax={wt.to}
        aria-valuenow={now}
        aria-valuetext={nowWords}
        {onkeydown}
        {onpointerdown}
        {onpointermove}
      >
        <div class="line"></div>
        {#each tk as t (t.at)}
          <div class="tick" class:major={t.major} style="left: {pct(t.at)}"></div>
          {#if t.label && !compact && t.at > w.from && t.at < w.to}<div class="tl" style="left: {pct(t.at)}" aria-hidden="true">{t.label}</div>{/if}
        {/each}
        {#each labelled as e (e.id)}
          <div class="ev {e.tone}" style="left: {pct(e.at_yr)}" title="{e.label}, {timeWords(e.at_yr, cal, span)}">
            <span class="pin"></span>
            {#if !compact && e.row >= 0}<span class="evl" class:row1={e.row === 1}>{e.label}<span class="evdate">, {timeWords(e.at_yr, cal, span)}</span></span>{/if}
          </div>
        {/each}
        {#each authored as e (e.id)}
          <div class="inked" style="left: {pct(e.at_yr)}" data-event-id={e.id} title="{e.label}, {timeWords(e.at_yr, cal, span)}"></div>
        {/each}
        {#if campaignNow !== undefined && inView(campaignNow)}<div class="cnow" style="left: {pct(campaignNow)}" title="The campaign's now: {campaignWords}"></div>{/if}
        {#if inView(now)}<div class="present" style="left: {pct(now)}"></div>{/if}
      </div>
    </div>
  </div>
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
  .readout {
    flex: none;
    min-width: 132px;
    max-width: 200px;
  }
  .now {
    font-family: var(--sse-font-display);
    font-size: 20px;
    line-height: 1.2;
  }
  .compact .readout {
    min-width: 84px;
  }
  .compact .now {
    font-size: 16px;
  }
  .sub {
    font-size: 11px;
    color: var(--sse-ink-muted);
    margin-top: 2px;
  }
  .stamp {
    display: inline-block;
    border: 1.5px solid var(--sse-accent);
    color: var(--sse-accent-ink);
    font-size: 9px;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    padding: 0 4px;
    transform: rotate(-3deg);
  }
  .mid {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .bar {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 16px;
    align-items: center;
    font-size: 12px;
  }
  .grp {
    display: inline-flex;
    gap: 4px;
    align-items: center;
  }
  .bar button,
  .bar select {
    font: inherit;
    font-size: 12px;
    min-height: 24px;
    background: transparent;
    color: var(--sse-ink);
    border: var(--sse-line) solid var(--sse-ink);
  }
  .bar button {
    min-width: 26px;
    padding: 0 6px;
  }
  .bar button.on {
    background: var(--sse-ink);
    color: var(--sse-ground);
  }
  .bar button:disabled {
    opacity: 0.4;
  }
  .bar button.link {
    border: 0;
    text-decoration: underline;
    padding: 0;
  }
  .bar button.accent {
    color: var(--sse-accent-ink);
  }
  .hint {
    color: var(--sse-ink-muted);
  }
  .note {
    color: var(--sse-ink-muted);
    font-style: italic;
  }
  .end {
    font-family: var(--sse-font-mono);
    font-size: 12px;
    color: var(--sse-ink-muted);
    pointer-events: none;
    white-space: nowrap;
    margin-left: auto;
  }
  .compact .end {
    display: none;
  }
  .trackwrap {
    min-width: 0;
    height: 52px;
    /* Labels near the ends (and mid-glide) never push the page sideways. */
    overflow-x: clip;
    overflow-clip-margin: 8px;
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
  /* Resting on an event lifts it and says when (U12). */
  .ev::before {
    content: "";
    position: absolute;
    left: -6px;
    top: -4px;
    width: 12px;
    height: 16px;
  }
  .pin {
    transition:
      height var(--sse-motion-quick) var(--sse-ease),
      border-color var(--sse-motion-quick) var(--sse-ease);
  }
  .ev:hover .pin {
    height: 10px;
    border-color: var(--sse-ink);
  }
  .ev:hover .evl {
    color: var(--sse-ink);
    z-index: 1;
    background: var(--sse-ground-raised);
  }
  .evdate {
    display: none;
  }
  .ev:hover .evdate {
    display: inline;
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
  .cnow {
    position: absolute;
    top: 21px;
    margin-left: -5px;
    width: 0;
    height: 0;
    border-left: 5px solid transparent;
    border-right: 5px solid transparent;
    border-bottom: 8px solid var(--sse-ink);
    pointer-events: none;
  }
  .present {
    position: absolute;
    top: 2px;
    height: 40px;
    border-left: 2px solid var(--sse-accent);
    pointer-events: none;
  }
  /* Phones: the readout goes on its own line above the controls and track. */
  @media (max-width: 600px) {
    .ruler {
      flex-wrap: wrap;
      gap: 4px 12px;
      padding: 8px 16px;
    }
    .readout {
      flex: 1 1 100%;
      max-width: none;
      display: flex;
      align-items: baseline;
      gap: 10px;
      flex-wrap: wrap;
    }
    .mid {
      flex: 1 1 100%;
    }
  }
</style>
