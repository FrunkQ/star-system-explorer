<!--
  A world through time (R6 with the renderer track; U22): the controls for the
  Body globe's time lapse. Layer toggles (one at a time), a scrubber over the
  world's own clock with its dated events marked, Play through the keyframes,
  and a "why is this here?" card for an event: what happened, when, where, the
  values under it then and the kernel's own why lines. Everything comes from
  the kernel's surface_frames through the globe runtime; nothing is computed
  here, and the state hash never moves.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import type { Depth } from "../../../ui-contract/src/index.ts";

  interface LapseEvent {
    t_myr: number;
    kind: string;
    label: string;
    lon?: number;
    lat?: number;
    /** The kernel's why lines for the event (shown when it has no place to pick). */
    why?: string[];
  }
  interface Picked {
    cell: number;
    values: Record<string, number | undefined>;
    sea_level_m: number;
    /** False on a dry world (no sea then). */
    sea?: boolean;
    event: { t_myr: number; kind: string; text: string } | null;
    why: string[];
    /** The kernel's line for the frame shown (land, ice and mean surface then). */
    frame_why?: string[];
  }
  /** The time-lapse API Body.svelte hands over (reactive). */
  export interface LapseApi {
    loaded: boolean;
    loading: boolean;
    error: string | null;
    layer: string;
    t_myr: number;
    layers: { id: string; label: string; legend: { unit: string; stops: readonly { at: number; label: string }[] }; limits: string; available: boolean; pending: string[] }[];
    /** Keyframe times, dated events, and the ice ages' window (within a few million years of today) with a line for each. */
    frames: { times: number[]; events: LapseEvent[]; iceAges?: { from_myr: number; to_myr: number; words: string[] } | null };
    load(): Promise<void>;
    close(): void;
    setLayer(id: string): void;
    setTime(t_myr: number): void;
    pick(lon_deg: number, lat_deg: number): Picked | null;
    view?(lon_deg: number, lat_deg: number): void;
  }

  let { lapse, depth = "science", label = "This world" }: { lapse: LapseApi; depth?: Depth; label?: string } = $props();

  const times = $derived(lapse.frames.times);
  const t0 = $derived(times[0] ?? 0);
  const t1 = $derived(times[times.length - 1] ?? 0);
  const span = $derived(Math.max(t1 - t0, 1e-9));
  const x = (t: number) => ((t - t0) / span) * 100;
  // How long ago, in words that suit the span: thousands of years inside the ice ages' window.
  const ago = (t: number) => {
    const a = t1 - t;
    if (a <= 0.0005) return "today";
    if (a < 1) return `${Math.round(a * 1000).toLocaleString("en-GB")} thousand years ago`;
    if (a < 10) return `${a.toFixed(1)} million years ago`;
    return a >= 1000 ? `${(a / 1000).toFixed(2)} billion years ago` : `${Math.round(a)} million years ago`;
  };
  const age = (t: number) => (t >= 1000 ? `${(t / 1000).toFixed(2)} billion years` : `${Math.round(t)} million years`);
  const spec = $derived(lapse.layers.find((l) => l.id === lapse.layer));

  // Event marks: binned so hundreds of events stay readable; a bin's height says how many, its colour the strongest kind.
  const BINS = 90;
  // Impacts and flood volcanism stand out in accent; plate events (collisions, splits, sutures) are the quiet ink.
  const VIOLENT = /impact|flood|snowball|extinction/i;
  // Ice ages all fall within a few million years of today, inside the last gap between keyframes: on the main track
  // they would be one mark at the end, so they get a strip of their own over the kernel's orbit-history window.
  const ICE = "ice-age";
  const ice = $derived(lapse.frames.iceAges ?? null);
  const iceEvents = $derived(lapse.frames.events.filter((e) => e.kind === ICE));
  const iceSpan = $derived(ice ? Math.max(ice.to_myr - ice.from_myr, 1e-9) : 1);
  const ix = (t: number) => (ice ? ((t - ice.from_myr) / iceSpan) * 100 : 0);
  const iceWindow = $derived(ice ? ago(ice.from_myr).replace(/ ago$/, "") : "");
  const inIce = $derived(!!ice && lapse.t_myr >= ice.from_myr);
  const bins = $derived.by(() => {
    const out: { i: number; events: LapseEvent[]; violent: boolean }[] = [];
    for (const e of lapse.frames.events) {
      if (ice && e.kind === ICE) continue;
      const i = Math.min(BINS - 1, Math.max(0, Math.floor(((e.t_myr - t0) / span) * BINS)));
      let b = out.find((x) => x.i === i);
      if (!b) out.push((b = { i, events: [], violent: false }));
      b.events.push(e);
      if (VIOLENT.test(e.kind) || VIOLENT.test(e.label)) b.violent = true;
    }
    return out.sort((a, b) => a.i - b.i);
  });
  const most = $derived(Math.max(1, ...bins.map((b) => b.events.length)));

  // The why card: the event chosen from the marks, with what lay under it then.
  let chosen = $state<{ e: LapseEvent; at: Picked | null } | null>(null);
  function choose(e: LapseEvent) {
    stop();
    lapse.setTime(e.t_myr);
    if (e.lon !== undefined && e.lat !== undefined) lapse.view?.(e.lon, e.lat);
    chosen = { e, at: e.lon !== undefined && e.lat !== undefined ? lapse.pick(e.lon, e.lat) : null };
  }
  function chooseBin(b: { events: LapseEvent[] }) {
    // The bin's most telling event: a violent one with a place first, then any violent one, else the first.
    const v = (e: LapseEvent) => VIOLENT.test(e.kind) || VIOLENT.test(e.label);
    choose(b.events.find((e) => v(e) && e.lon !== undefined) ?? b.events.find(v) ?? b.events[0]!);
  }
  const nearest = $derived.by(() => {
    let best: LapseEvent | null = null;
    for (const e of lapse.frames.events) if (!best || Math.abs(e.t_myr - lapse.t_myr) < Math.abs(best.t_myr - lapse.t_myr)) best = e;
    return best;
  });

  // Play steps through the keyframes, about two a second, from where the scrubber is (from the start at the end).
  let timer: ReturnType<typeof setInterval> | null = null;
  let playing = $state(false);
  function stop() {
    if (timer) clearInterval(timer);
    timer = null;
    playing = false;
  }
  function play() {
    if (playing) return stop();
    chosen = null;
    let i = times.findIndex((t) => t > lapse.t_myr + 1e-6);
    if (i < 0) i = 0;
    playing = true;
    lapse.setTime(times[i]!);
    timer = setInterval(() => {
      i++;
      if (i >= times.length) return stop();
      lapse.setTime(times[i]!);
    }, 480);
  }
  $effect(() => () => untrack(stop));
  $effect(() => {
    if (!lapse.loaded) untrack(() => (stop(), (chosen = null)));
  });

  const FIELD_WORDS: Record<string, { label: string; fmt: (v: number) => string }> = {
    elevation_m: { label: "Height", fmt: (v) => `${Math.round(v).toLocaleString("en-GB")} m` },
    temp_k: { label: "Surface, yearly mean", fmt: (v) => `${Math.round(v)} K` },
    rain_m_yr: { label: "Rain and snow", fmt: (v) => `${v.toFixed(2)} m a year` },
    ice_m: { label: "Ice", fmt: (v) => `${Math.round(v).toLocaleString("en-GB")} m` },
    runoff_m_yr: { label: "Water running off", fmt: (v) => `${v.toFixed(2)} m a year` },
    vent: { label: "Eruption strength", fmt: (v) => v.toFixed(2) },
    plate: { label: "Plate", fmt: (v) => String(Math.round(v)) },
  };
  const where = (e: LapseEvent) => `${Math.abs(e.lat ?? 0).toFixed(0)}° ${(e.lat ?? 0) >= 0 ? "N" : "S"}, ${Math.abs(e.lon ?? 0).toFixed(0)}° ${(e.lon ?? 0) >= 0 ? "E" : "W"}`;
  const shownValues = (p: Picked) => {
    const keys = depth === "curious" ? ["elevation_m", "temp_k", "ice_m"] : Object.keys(FIELD_WORDS);
    return keys.filter((k) => p.values[k] !== undefined).map((k) => ({ k, ...FIELD_WORDS[k]!, v: p.values[k]! }));
  };
</script>

<div class="timeline" data-lapse-controls data-lapse={lapse.loaded ? "on" : lapse.loading ? "loading" : "off"} data-lapse-layer={lapse.layer} data-lapse-t={lapse.t_myr}>
  {#if !lapse.loaded}
    <button type="button" class="link" disabled={lapse.loading} onclick={() => lapse.load()}>{lapse.loading ? "Reading its history…" : "Through time"}</button>
    {#if lapse.error}<small class="note" data-lapse-error>{lapse.error}</small>{/if}
  {:else}
    <div class="row head">
      <span class="when"><b data-lapse-when>{ago(lapse.t_myr)}</b> <span class="muted">· {label} at {age(lapse.t_myr)}</span></span>
      <button type="button" class="link" onclick={() => (stop(), lapse.close())}>Back to today</button>
    </div>

    <div class="layers" role="radiogroup" aria-label="Layer">
      {#each lapse.layers as l (l.id)}
        <button type="button" role="radio" aria-checked={l.id === lapse.layer} disabled={!l.available} title={l.available ? l.limits : `Waiting on the kernel: ${l.pending.join(", ")}`} data-layer={l.id} onclick={() => lapse.setLayer(l.id)}>{l.label}</button>
      {/each}
    </div>

    <div class="scrub">
      <button type="button" class="play" aria-pressed={playing} onclick={play} title={playing ? "Pause" : "Play through its history"}>
        {#if playing}<svg width="14" height="14" viewBox="0 0 24 24" aria-hidden="true"><path d="M7 5h3v14H7zM14 5h3v14h-3z" fill="currentColor" /></svg>{:else}<svg width="14" height="14" viewBox="0 0 24 24" aria-hidden="true"><path d="M7 4l13 8-13 8z" fill="currentColor" /></svg>{/if}
        <span class="sr">{playing ? "Pause" : "Play"}</span>
      </button>
      <div class="track">
        <div class="marks" role="group" aria-label="Events through its history" data-lapse-marks={lapse.frames.events.length}>
          {#each bins as b (b.i)}
            <button type="button" class="mark" class:violent={b.violent} style:left="{((b.i + 0.5) / BINS) * 100}%" style:height="{6 + 14 * Math.sqrt(b.events.length / most)}px" aria-label="{b.events.length} {b.events.length === 1 ? 'event' : 'events'} around {ago(b.events[0]!.t_myr)}: {b.events[0]!.label}" title={b.events.slice(0, 3).map((e) => e.label).join("\n")} onclick={() => chooseBin(b)}></button>
          {/each}
          <span class="now" style:left="{x(lapse.t_myr)}%" aria-hidden="true"></span>
        </div>
        <label class="range"><span class="sr">When</span><input type="range" aria-label="When" min={t0} max={t1} step="10" value={lapse.t_myr} oninput={(e) => (stop(), (chosen = null), lapse.setTime(Number(e.currentTarget.value)))} /></label>
        <div class="ends" aria-hidden="true"><span>{age(t0)}</span><span>today</span></div>
      </div>
    </div>

    {#if ice && iceEvents.length}
      <div class="scrub ice" data-lapse-ice={iceEvents.length}>
        <span class="tag" title="The kernel dates ice ages from the orbit's swing over this window (Milankovitch)">Ice ages</span>
        <div class="track">
          <div class="marks" role="group" aria-label="Ice ages in the last {iceWindow}">
            {#each iceEvents as e, k (e.t_myr)}
              <button type="button" class="mark cold" style:left="{ix(e.t_myr)}%" aria-label={ice.words[k] ?? e.label} title={ice.words[k] ?? e.label} onclick={() => choose(e)}></button>
            {/each}
            {#if inIce}<span class="now" style:left="{ix(lapse.t_myr)}%" aria-hidden="true"></span>{/if}
          </div>
          <label class="range"><span class="sr">Through the ice ages of the last {iceWindow}</span><input type="range" aria-label="Through the ice ages of the last {iceWindow}" min={ice.from_myr} max={ice.to_myr} step={iceSpan / 500} value={inIce ? lapse.t_myr : ice.from_myr} oninput={(e) => (stop(), (chosen = null), lapse.setTime(Number(e.currentTarget.value)))} /></label>
          <div class="ends" aria-hidden="true"><span>{iceWindow} ago</span><span>today</span></div>
        </div>
      </div>
    {/if}

    {#if spec}
      <div class="legend" data-lapse-legend={spec.id}>
        <span class="muted">{spec.legend.unit}:</span>
        {#each spec.legend.stops as s (s.at)}<span>{s.label}</span>{/each}
      </div>
    {/if}

    {#if chosen}
      <section class="why" aria-label="Why is this here?" data-lapse-why>
        <header><h4>Why is this here?</h4><button type="button" class="link" onclick={() => (chosen = null)} aria-label="Close">×</button></header>
        <p class="said">{chosen.e.kind === ICE ? (ice?.words[iceEvents.indexOf(chosen.e)] ?? chosen.e.label) : chosen.e.label}</p>
        <p class="muted">{ago(chosen.e.t_myr)}, {chosen.e.lon !== undefined ? `at ${where(chosen.e)}` : "across the world"}{depth !== "curious" ? ` · ${chosen.e.kind.replace(/[-_]/g, " ")}` : ""}</p>
        {#if chosen.at}
          <dl>
            {#each shownValues(chosen.at) as v (v.k)}<dt>{v.label}</dt><dd>{v.fmt(v.v)}</dd>{/each}
            {#if depth !== "curious"}<dt>Sea level then</dt><dd>{chosen.at.sea === false ? "no sea" : `${Math.round(chosen.at.sea_level_m)} m`}</dd>{/if}
            {#if depth === "engine"}<dt>Cell</dt><dd>{chosen.at.cell}</dd>{/if}
          </dl>
          {#if chosen.at.why.length}<ul class="because">{#each chosen.at.why as w (w)}<li>{w}</li>{/each}</ul>{/if}
        {:else}
          {#if chosen.e.why?.length}<ul class="because">{#each chosen.e.why as w (w)}<li>{w}</li>{/each}</ul>{/if}
        {/if}
      </section>
    {:else if nearest && !playing && Math.abs(nearest.t_myr - lapse.t_myr) < span / BINS}
      <p class="hint"><button type="button" class="link" onclick={() => choose(nearest!)}>{nearest.label} · why?</button></p>
    {/if}

    {#if spec}<small class="note" data-lapse-limits>{[spec.limits, ...spec.pending].map((s) => s.trim().replace(/([^.])$/, "$1.")).join(" ")}</small>{/if}
  {/if}
</div>

<style>
  .timeline {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin: 8px 0;
    font-size: 13px;
    max-width: 100%;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    justify-content: space-between;
    gap: 6px 12px;
  }
  .when b {
    font-family: var(--sse-font-display);
    font-weight: 400;
    font-size: 18px;
  }
  .muted,
  .note,
  .hint {
    color: var(--sse-ink-muted);
  }
  .note {
    font-size: 12px;
  }
  .hint {
    margin: 0;
  }
  .link {
    border: 0;
    background: none;
    padding: 0;
    font: inherit;
    color: var(--sse-ink);
    text-decoration: underline;
    text-underline-offset: 3px;
    cursor: pointer;
  }
  .link:disabled {
    cursor: progress;
    opacity: 0.6;
  }
  .layers {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .layers button {
    padding: 4px 8px;
    border: 1px solid var(--sse-ink-faint);
    background: var(--sse-ground);
    color: var(--sse-ink);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  .layers button[aria-checked="true"] {
    border-color: var(--sse-ink);
    background: var(--sse-ink);
    color: var(--sse-ground);
  }
  .layers button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .scrub {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 10px;
    align-items: end;
  }
  .play {
    width: 36px;
    height: 36px;
    display: grid;
    place-items: center;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    cursor: pointer;
  }
  .play[aria-pressed="true"] {
    background: var(--sse-ink);
    color: var(--sse-ground);
  }
  .track {
    position: relative;
    min-width: 0;
  }
  .marks {
    position: relative;
    height: 22px;
    border-bottom: 1px solid var(--sse-ink-faint);
  }
  .mark {
    position: absolute;
    bottom: 0;
    width: 4px;
    margin-left: -2px;
    padding: 0;
    border: 0;
    background: var(--sse-ink-muted);
    cursor: pointer;
  }
  .mark.violent {
    background: var(--sse-accent);
  }
  /* Ice ages: their own strip under the main track, each onset a full-height hollow mark (cold, not violent). */
  .scrub.ice {
    align-items: center;
  }
  .scrub.ice .tag {
    width: 36px;
    font-size: 0.7rem;
    line-height: 1.1;
    color: var(--sse-ink-muted);
    text-align: center;
  }
  .mark.cold {
    height: 16px;
    width: 6px;
    margin-left: -3px;
    background: var(--sse-ground-raised);
    border: var(--sse-line) solid var(--sse-ink-muted);
  }
  .mark.cold:hover,
  .mark.cold:focus-visible {
    border-color: var(--sse-ink);
    background: var(--sse-ink-faint);
  }
  .mark:hover,
  .mark:focus-visible {
    background: var(--sse-ink);
  }
  .now {
    position: absolute;
    bottom: -3px;
    top: -2px;
    width: 0;
    border-left: 2px solid var(--sse-accent);
    pointer-events: none;
  }
  .range input {
    width: 100%;
    margin: 4px 0 0;
  }
  .ends {
    display: flex;
    justify-content: space-between;
    font-family: var(--sse-font-mono);
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  .legend {
    display: flex;
    flex-wrap: wrap;
    gap: 2px 10px;
    font-size: 12px;
  }
  .why {
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    box-shadow: var(--sse-card-shadow);
    padding: 10px 12px;
  }
  .why header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }
  .why h4 {
    margin: 0;
    font-family: var(--sse-font-mono);
    font-size: 11px;
    font-weight: 400;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .why .said {
    margin: 6px 0 2px;
    font-size: 15px;
  }
  .why p {
    margin: 0 0 6px;
  }
  .why dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 2px 12px;
    margin: 6px 0;
  }
  .why dt {
    color: var(--sse-ink-muted);
  }
  .why dd {
    margin: 0;
    font-family: var(--sse-font-mono);
  }
  .because {
    margin: 4px 0 0;
    padding-left: 18px;
    color: var(--sse-ink-muted);
  }
  .sr {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0 0 0 0);
    white-space: nowrap;
  }
</style>
