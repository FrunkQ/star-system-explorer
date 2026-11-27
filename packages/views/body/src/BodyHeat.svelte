<!--
  Energy flow of a world (track HF's screen; the melt design's C6 thermal line; Alex: "Heat flow in planets ==
  energy flow", so it is the ship Power card's grammar and words): where a world's inside heat comes from and where
  it goes, at any point of its own life. The flow is the Flow view the ship Power card uses (RK13), fed from the core
  history the kernel publishes in the survey's envelope (the numbers that drive the world); the thermal line under it
  is the same history against time, with the history's marks, and the slider moves along the history's own steps.
  Looking back moves nothing in the campaign. Nothing here works out physics: amounts are the kernel's, words compare
  them. Where the kernel's books are not complete yet, it says so and draws only what is published.

  Desks: Explore reads it (Curious depth: names and two sentences; Science and Engine: figures, the line, the marks).
  Build reads it the same way; it changes when the world's makeup is edited. Run: the GM's seat only (the survey is
  GM only); players never see it.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { HEAT_COLUMNS, heatBooks, heatCaveats, heatColumn, heatMarksBy, heatRank, heatWatts, heatWhen, heatWhy, booksClose, say, surfaceArea, type Depth, type HeatEnvelope, type HeatStep, type ViewSession } from "../../../ui-contract/src/index.ts";
  import { opensFrom } from "../../../chart-grammar/src/index.ts";
  import Flow from "../../flow/src/Flow.svelte";

  let { session, system, address, label, depth = "science" }: { session: ViewSession; system: string; address: string; label: string; depth?: Depth } = $props();

  const gm = untrack(() => session.seat.authority === "gm");
  let open = $state(false);
  let btn = $state<HTMLElement | undefined>();
  let busy = $state(false);
  let failed = $state<string | null>(null);
  let env = $state.raw<HeatEnvelope | null>(null);
  /** The step looked at; null follows the world's present. */
  let picked = $state<number | null>(null);

  $effect(() => {
    void `${system}|${address}`;
    untrack(() => {
      env = null;
      gen++;
      busy = false;
      picked = null;
      failed = null;
      if (open) void load();
    });
  });

  let gen = 0;
  async function load() {
    if (env) return;
    const g = ++gen;
    busy = true;
    failed = null;
    const r = await session.surveyHeat(system, address);
    if (g !== gen) return;
    busy = false;
    if (!r.ok) return void (failed = r.message);
    // The screen's own name for the world (a catalogued world's envelope may carry only its address).
    env = { ...r.envelope, label };
  }
  function toggle() {
    open = !open;
    if (open) void load();
  }

  const h = $derived(env?.core && env.core.steps.length ? env.core : null);
  const n = $derived(h ? h.steps.length : 0);
  const i = $derived(h ? Math.min(picked ?? n - 1, n - 1) : 0);
  const step = $derived(h ? h.steps[i]! : null);
  const books = $derived(env && step ? heatBooks(env, step) : null);
  const when = $derived(h ? heatWhen(h, i, depth) : null);
  const why = $derived(env && h ? heatWhy(env, h, i, depth) : []);
  const marks = $derived(h ? heatMarksBy(h, i) : []);
  const caveats = $derived(env && step ? heatCaveats(env, step) : []);
  const figures = $derived(depth !== "curious");

  // The thermal line: temperatures in one small chart, heat in another (never two scales on one axis).
  const W = 560;
  const H = 96;
  const PAD = [64, 8, 8, 18] as const; // left, right, top, bottom
  const r1 = (v: number) => Math.round(v * 10) / 10;
  const t0 = $derived(h ? h.steps[0]!.t_gyr : 0);
  const t1 = $derived(h ? h.steps[n - 1]!.t_gyr : 1);
  const x = (t: number) => PAD[0] + ((W - PAD[0] - PAD[1]) * (t - t0)) / Math.max(1e-9, t1 - t0);
  const temps = $derived.by(() => {
    if (!h) return null;
    const vals = h.steps.flatMap((s) => [s.t_mantle_k, s.t_cmb_k]);
    const lo = Math.min(...vals);
    const hi = Math.max(...vals);
    const y = (k: number) => PAD[2] + (H - PAD[2] - PAD[3]) * (1 - (k - lo) / Math.max(1, hi - lo));
    const hh = h;
    const path = (f: (s: HeatStep) => number) => hh.steps.map((s, j) => `${j ? "L" : "M"}${r1(x(s.t_gyr))},${r1(y(f(s)))}`).join("");
    return { lo, hi, mantle: path((s) => s.t_mantle_k), core: path((s) => s.t_cmb_k) };
  });
  const lostOf = (s: HeatStep) => (env ? heatBooks(env, s).lost_w : 0);
  const coreOf = (s: HeatStep) => (env && booksClose(s) ? (s.core_heat ?? 0) * surfaceArea(env) : s.q_core_tw * 1e12);
  const heat = $derived.by(() => {
    if (!h) return null;
    const vals = h.steps.flatMap((s) => [lostOf(s), coreOf(s)]).filter((v) => v > 0);
    if (!vals.length) return null;
    const lo = Math.log10(Math.min(...vals));
    const hi = Math.log10(Math.max(...vals));
    const y = (v: number) => PAD[2] + (H - PAD[2] - PAD[3]) * (1 - (Math.log10(Math.max(v, 10 ** lo)) - lo) / Math.max(0.3, hi - lo));
    const hh = h;
    const path = (f: (s: HeatStep) => number) => hh.steps.map((s, j) => `${j ? "L" : "M"}${r1(x(s.t_gyr))},${r1(y(f(s)))}`).join("");
    return { lo: 10 ** lo, hi: 10 ** hi, lost: path(lostOf), core: path(coreOf) };
  });
  const cursor = $derived(step ? x(step.t_gyr) : 0);
  function pickAt(ev: PointerEvent) {
    if (!h) return;
    const el = ev.currentTarget as SVGElement;
    const r = el.getBoundingClientRect();
    const px = ((ev.clientX - r.left) / Math.max(1, r.width)) * W;
    const t = t0 + ((px - PAD[0]) / (W - PAD[0] - PAD[1])) * (t1 - t0);
    let best = 0;
    h.steps.forEach((s, j) => {
      if (Math.abs(s.t_gyr - t) < Math.abs(h.steps[best]!.t_gyr - t)) best = j;
    });
    picked = best === n - 1 ? null : best;
  }
  const temp = (k: number) => say(k, "temperature", undefined, { depth }).text;
</script>

{#if gm}
  <section class="heat" aria-label="Energy flow of {label}" data-heat>
    <button type="button" class="open" bind:this={btn} aria-expanded={open} onclick={toggle} data-heat-toggle>
      <span class="h">Energy flow</span>
      <span class="muted small">where its heat comes from and where it ends up, over its life</span>
    </button>
    {#if open}
      <div class="body" use:opensFrom={() => btn ?? null} data-heat-panel>
        {#if busy}
          <p class="small muted">Reading its thermal history… (about a second)</p>
        {:else if failed}
          <p class="small muted" data-heat-failed>No heat books: {failed}</p>
        {:else if env && !h}
          <p class="small muted" data-heat-none>{label} has no thermal history in the kernel: {env.kind === "cloud-deck" ? "a giant's deep interior is not modelled as a mantle and core" : "it has no solid interior with an iron core"}.</p>
        {:else if h && step && books && when}
          <div class="bar">
            <label class="ctl">
              <span>Looking at</span>
              <input type="range" min="0" max={n - 1} step="1" value={i} oninput={(e) => (picked = Number(e.currentTarget.value) === n - 1 ? null : Number(e.currentTarget.value))} aria-valuetext="{when.since}, {when.ago}" data-heat-slider />
              <output data-heat-when>{when.now ? "Now" : when.ago}<span class="muted"> · {when.since}</span></output>
            </label>
            {#if !when.now}<button type="button" class="link" onclick={() => (picked = null)} data-heat-now>Back to now</button>{/if}
            {#if !when.now}<span class="small muted" data-heat-looking>Looking back: nothing changes in the campaign.</span>{/if}
          </div>
          {#if books.rows.length}
            <Flow rows={books.rows} titles={HEAT_COLUMNS} column={heatColumn(env!)} rank={heatRank} amountWords={(w) => heatWatts(w, depth)} {figures} label="Energy flow of {label}" />
          {/if}
          {#if !books.closes}<p class="small note" data-heat-missing>{books.missing}</p>{/if}
          <div class="why" data-heat-why>
            {#each why as line, k (k)}<p>{line}</p>{/each}
          </div>
          {#if figures && temps}
            <figure class="line" data-heat-line>
              <figcaption class="small muted">Temperature: <span class="key mantle"></span> mantle <span class="key core"></span> top of the core</figcaption>
              <!-- svelte-ignore a11y_no_static_element_interactions (the slider above is the keyboard way to move along the line) -->
              <svg viewBox="0 0 {W} {H}" width="100%" role="img" aria-label="Mantle and core temperature over {label}'s life" onpointerdown={pickAt}>
                <line class="axis" x1={PAD[0]} x2={W - PAD[1]} y1={H - PAD[3]} y2={H - PAD[3]} />
                <text class="tick" x={PAD[0] - 4} y={PAD[2] + 8} text-anchor="end">{temp(temps.hi)}</text>
                <text class="tick" x={PAD[0] - 4} y={H - PAD[3]} text-anchor="end">{temp(temps.lo)}</text>
                <path class="ln mantle" d={temps.mantle} />
                <path class="ln core" d={temps.core} />
                {#each h.events as e, k (k)}<line class="mark" x1={x(e.t_gyr)} x2={x(e.t_gyr)} y1={H - PAD[3]} y2={H - PAD[3] - 6}><title>{e.why}</title></line>{/each}
                <line class="cursor" x1={cursor} x2={cursor} y1={PAD[2]} y2={H - PAD[3]} />
              </svg>
              {#if heat}
                <figcaption class="small muted">Heat: <span class="key lost"></span> lost through the surface <span class="key coreheat"></span> out of the core (log scale)</figcaption>
                <!-- svelte-ignore a11y_no_static_element_interactions (the slider above is the keyboard way to move along the line) -->
                <svg viewBox="0 0 {W} {H}" width="100%" role="img" aria-label="Heat lost and heat out of the core over {label}'s life" onpointerdown={pickAt}>
                  <line class="axis" x1={PAD[0]} x2={W - PAD[1]} y1={H - PAD[3]} y2={H - PAD[3]} />
                  <text class="tick" x={PAD[0] - 4} y={PAD[2] + 8} text-anchor="end">{heatWatts(heat.hi, depth)}</text>
                  <text class="tick" x={PAD[0] - 4} y={H - PAD[3]} text-anchor="end">{heatWatts(heat.lo, depth)}</text>
                  <path class="ln lost" d={heat.lost} />
                  <path class="ln coreheat" d={heat.core} />
                  {#each h.events as e, k (k)}<line class="mark" x1={x(e.t_gyr)} x2={x(e.t_gyr)} y1={H - PAD[3]} y2={H - PAD[3] - 6}><title>{e.why}</title></line>{/each}
                  <line class="cursor" x1={cursor} x2={cursor} y1={PAD[2]} y2={H - PAD[3]} />
                  <text class="tick" x={PAD[0]} y={H - 4}>{heatWhen(h, 0, depth).since}</text>
                  <text class="tick" x={W - PAD[1]} y={H - 4} text-anchor="end">now, {heatWhen(h, n - 1, depth).age}</text>
                </svg>
              {/if}
            </figure>
          {/if}
          {#if figures && marks.length}
            <details class="marks" data-heat-marks>
              <summary class="small">What had happened by then ({marks.length})</summary>
              <ul class="small">
                {#each marks as m, k (k)}<li data-heat-mark={m.kind}>{m.why}</li>{/each}
              </ul>
            </details>
          {/if}
          {#if caveats.length}
            <details class="caveats" data-heat-caveats>
              <summary class="small muted">How far to trust these numbers ({caveats.length})</summary>
              <ul class="small muted">
                {#each caveats as c (c.id)}<li data-heat-caveat={c.id}>{c.words}{#if depth === "engine"} (fixed by {c.fixedBy}){/if}</li>{/each}
              </ul>
            </details>
          {/if}
          {#if depth === "engine"}<p class="small muted" data-heat-source>From the kernel's core history {h.version}, {n} steps on the world's own grid; amounts {books.closes ? "from its heat books (melt design section 7)" : "from q_core_tw and q_surface_tw"}.</p>{/if}
        {/if}
      </div>
    {/if}
  </section>
{/if}

<style>
  .heat {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 0;
    color: var(--sse-ink);
  }
  .open {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 4px 10px;
    font: inherit;
    text-align: left;
    background: transparent;
    color: var(--sse-ink);
    border: 1px solid var(--sse-line);
    border-radius: var(--sse-radius, 0);
    padding: 8px 10px;
    cursor: pointer;
    transition: transform var(--sse-motion-quick, 140ms) var(--sse-ease, ease-out);
  }
  .open:hover,
  .open:focus-visible {
    border-color: var(--sse-ink);
    transform: translateY(-1px);
  }
  .open[aria-expanded="true"] {
    border-color: var(--sse-ink);
  }
  .h {
    font-weight: 600;
  }
  .body {
    display: flex;
    flex-direction: column;
    gap: 10px;
    min-width: 0;
  }
  .small {
    font-size: 12px;
  }
  .muted {
    color: var(--sse-ink-muted);
  }
  .note {
    margin: 0;
    color: var(--sse-ink-muted);
    border-left: 2px solid var(--sse-alert, var(--sse-accent));
    padding-left: 8px;
  }
  .bar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px 14px;
  }
  .ctl {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--sse-ink-muted);
    min-width: 0;
  }
  .ctl output {
    font-family: var(--sse-font-mono);
    color: var(--sse-ink);
  }
  input[type="range"] {
    accent-color: var(--sse-accent);
    width: min(260px, 60vw);
  }
  .link {
    font: inherit;
    font-size: 12px;
    background: none;
    border: none;
    padding: 0;
    color: var(--sse-ink);
    text-decoration: underline;
    cursor: pointer;
  }
  .why p {
    margin: 0 0 4px;
    font-size: 13px;
    max-width: 68ch;
  }
  .line {
    margin: 0;
    min-width: 0;
  }
  .line svg {
    display: block;
    overflow: visible;
    cursor: crosshair;
    touch-action: pan-y;
  }
  .axis {
    stroke: var(--sse-line);
  }
  .tick {
    font-family: var(--sse-font-mono);
    font-size: 10px;
    fill: var(--sse-ink-muted);
  }
  .ln {
    fill: none;
    stroke-width: 2;
    stroke-linejoin: round;
  }
  .ln.mantle,
  .ln.lost {
    stroke: var(--sse-ink);
  }
  .ln.core,
  .ln.coreheat {
    stroke: var(--sse-ink-muted);
    stroke-dasharray: 5 3;
  }
  .mark {
    stroke: var(--sse-ink-muted);
    stroke-width: 2;
  }
  .cursor {
    stroke: var(--sse-accent);
    stroke-width: 1.5;
  }
  .key {
    display: inline-block;
    width: 16px;
    height: 0;
    vertical-align: middle;
    border-top: 2px solid var(--sse-ink);
    margin: 0 2px 0 6px;
  }
  .key.core,
  .key.coreheat {
    border-top: 2px dashed var(--sse-ink-muted);
  }
  .marks ul,
  .caveats ul {
    margin: 4px 0 0;
    padding-left: 18px;
  }
  .marks summary,
  .caveats summary {
    cursor: pointer;
  }
  @media (prefers-reduced-motion: reduce) {
    .open {
      transition: none;
    }
  }
</style>
