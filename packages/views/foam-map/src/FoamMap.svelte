<!--
  The foam map (V9). A square slice of the cosmic web from the kernel's
  web_slice: matter drawn where the Zel'dovich approximation has moved it
  (dots), over the density (shade) or the environment each place belongs to
  (void, sheet, filament, knot). A time slider and Play grow the foam from
  the first billion years to now. Presentation only: the numbers are the
  kernel's.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { foamCells, sci, type Json, type SystemScene, type ViewContext, type WebSlice } from "../../../ui-contract/src/index.ts";

  let { ctx, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();
  const first = untrack(() => ({ extent: String(params.extent_mpc ?? "200"), n: Number(params.samples ?? 96) }));
  const EXTENTS = ["50", "100", "200", "400"] as const;
  const STEPS = [0.6, 1, 1.5, 2, 3, 4, 5, 6, 8, 10, 12, 13.8];
  let extent = $state<string>(EXTENTS.includes(first.extent as (typeof EXTENTS)[number]) ? first.extent : "200");
  const n = Math.max(16, Math.min(160, Math.round(first.n) || 96));
  let step = $state(STEPS.length - 1);
  let show = $state<"density" | "environment">("density");
  let web = $state<WebSlice | null>(null);
  let failed = $state<string | null>(null);
  let playing = $state(false);
  let gen = 0;
  // Slices already grown, by extent and time step (deterministic, so safe to keep).
  const kept = new Map<string, WebSlice>();

  async function slice(e: string, s: number): Promise<WebSlice | string> {
    const key = `${e}:${s}`;
    const k = kept.get(key);
    if (k) return k;
    const t = STEPS[s]!;
    const r = await ctx.session.webSlice({ ...(s === STEPS.length - 1 ? {} : { t_gyr: t }), extent_mpc: Number(e), n });
    if (!r.ok) return r.message;
    kept.set(key, r.web);
    return r.web;
  }

  $effect(() => {
    const e = extent;
    const s = step;
    const g = ++gen;
    void slice(e, s).then((r) => {
      if (g !== gen) return;
      if (typeof r === "string") return void ((failed = r), (web = null));
      failed = null;
      web = r;
    });
  });

  async function play() {
    if (playing) return void (playing = false);
    playing = true;
    if (step === STEPS.length - 1) step = 0;
    while (playing && step < STEPS.length - 1) {
      // Grow the next frame before showing it, then hold it briefly.
      await slice(extent, step + 1);
      if (!playing) break;
      step += 1;
      await new Promise((r) => setTimeout(r, 450));
    }
    playing = false;
  }

  // Shares of space in each environment.
  const shares = $derived.by(() => {
    if (!web) return [];
    const env = web.slice.environment;
    const count = (k: string) => env.filter((x) => x === k).length / Math.max(1, env.length);
    return ENVS.map((e) => ({ ...e, share: count(e.id) }));
  });
  const ENVS = [
    { id: "void", label: "Voids", words: "nearly empty; matter has drained away" },
    { id: "sheet", label: "Sheets", words: "walls between voids" },
    { id: "filament", label: "Filaments", words: "where walls meet; galaxies stream along them" },
    { id: "knot", label: "Knots", words: "where filaments meet; galaxy clusters" },
  ];
  const contrast = $derived.by(() => {
    if (!web) return 0;
    const d = web.slice.density;
    const m = d.reduce((a, b) => a + b, 0) / Math.max(1, d.length);
    return Math.sqrt(d.reduce((a, b) => a + (b - m) ** 2, 0) / Math.max(1, d.length));
  });

  // Drawing: one canvas, colours from the theme's tokens.
  let canvas = $state<HTMLCanvasElement | null>(null);
  let tick = $state(0);
  $effect(() => {
    const o = new MutationObserver(() => (tick += 1));
    o.observe(document.documentElement, { attributes: true, attributeFilter: ["style", "data-theme"] });
    return () => o.disconnect();
  });
  function rgb(el: Element, name: string, fallback: string): [number, number, number] {
    const v = getComputedStyle(el).getPropertyValue(name).trim() || fallback;
    const c = document.createElement("canvas").getContext("2d");
    if (!c) return [0, 0, 0];
    c.fillStyle = v;
    c.fillRect(0, 0, 1, 1);
    const d = c.getImageData(0, 0, 1, 1).data;
    return [d[0]!, d[1]!, d[2]!];
  }
  $effect(() => {
    void tick;
    const cv = canvas;
    const w = web;
    const mode = show;
    if (!cv || !w) return;
    const g = cv.getContext("2d");
    if (!g) return;
    const S = cv.width;
    const ink = rgb(cv, "--sse-ink", "#1b2530");
    const ground = rgb(cv, "--sse-ground", "#f1ece3");
    const accent = rgb(cv, "--sse-accent", "#b5532a");
    const s = w.slice;
    // Background: density or environment, one texel a sample, smoothed up.
    const off = document.createElement("canvas");
    off.width = s.n;
    off.height = s.n;
    const oc = off.getContext("2d");
    if (!oc) return;
    const img = oc.createImageData(s.n, s.n);
    const envShade: Record<string, number> = { void: 0, sheet: 0.18, filament: 0.42, knot: 0.7 };
    for (const c of foamCells(s)) {
      const a = mode === "density" ? c.shade * 0.75 : (envShade[c.env] ?? 0);
      const col = mode === "environment" && c.env === "knot" ? accent : ink;
      // Flip y so +y is up on the chart.
      const p = ((s.n - 1 - c.j) * s.n + c.i) * 4;
      for (let k = 0; k < 3; k++) img.data[p + k] = Math.round(ground[k]! * (1 - a) + col[k]! * a);
      img.data[p + 3] = 255;
    }
    oc.putImageData(img, 0, 0);
    g.imageSmoothingEnabled = true;
    g.drawImage(off, 0, 0, S, S);
    // Matter where it has moved to.
    const half = s.extent / 2;
    g.fillStyle = `rgba(${ink[0]},${ink[1]},${ink[2]},0.55)`;
    const r = Math.max(1, S / s.n / 3.2);
    for (const [x, y] of s.displaced) {
      const px = ((x - s.centre[0] + half) / s.extent) * S;
      const py = S - ((y - s.centre[1] + half) / s.extent) * S;
      if (px < 0 || py < 0 || px > S || py > S) continue;
      g.fillRect(px - r / 2, py - r / 2, r, r);
    }
  });

  const tWords = (t: number) => (t < 1 ? `${Math.round(t * 1000)} million years` : `${t.toFixed(1)} Gyr`);
  // A scale bar of a round length about a fifth of the side.
  const bar = $derived.by(() => {
    const e = Number(extent);
    const want = e / 5;
    const nice = [5, 10, 20, 25, 50, 100, 200].reduce((a, b) => (Math.abs(b - want) < Math.abs(a - want) ? b : a));
    return { mpc: nice, frac: nice / e, ly: nice * 3.2616 };
  });
</script>

<div class="foam" data-web={web ? `${web.slice.extent}:${web.t_gyr.toFixed(2)}` : ""}>
  <div class="bar">
    <span class="seg" role="group" aria-label="Slice size">
      {#each EXTENTS as e (e)}
        <button type="button" class:on={extent === e} data-extent={e} onclick={() => (extent = e)}>{e} Mpc</button>
      {/each}
    </span>
    <span class="seg" role="group" aria-label="Shade by">
      <button type="button" class:on={show === "density"} data-show="density" onclick={() => (show = "density")}>Density</button>
      <button type="button" class:on={show === "environment"} data-show="environment" onclick={() => (show = "environment")}>Voids, sheets, filaments, knots</button>
    </span>
  </div>

  <div class="head">
    <h2>The cosmic web</h2>
    <span class="stamp">GROWN FROM THE SEED</span>
    {#if web}
      <p class="lede" data-lede>
        {web.t_gyr >= web.now_gyr - 0.05 ? "Today" : `${tWords(web.t_gyr)} after the Big Bang`} (redshift {web.z < 0.005 ? "0" : web.z.toFixed(2)}): a slice {web.slice.extent} megaparsecs across, about {sci(web.slice.extent * 3.2616)} million light years.
        {#if shares.length}By where the matter sits: {shares.map((e) => `${Math.round(e.share * 100)}% ${e.label.toLowerCase()}`).join(", ")}.{/if}
      </p>
    {:else if failed}
      <p class="lede" data-failed>{failed}</p>
    {:else}
      <p class="lede">Growing the web…</p>
    {/if}
  </div>

  <div class="split">
    <div class="map">
      <canvas bind:this={canvas} width="640" height="640" data-foam aria-label="Slice of the cosmic web"></canvas>
      <div class="scale" style:width="{bar.frac * 100}%"><span>{bar.mpc} Mpc · {sci(bar.ly)} million ly</span></div>
      <div class="time">
        <button type="button" class="play" data-play onclick={play}>{playing ? "Pause" : "Play"}</button>
        <input type="range" min="0" max={STEPS.length - 1} step="1" bind:value={step} data-time aria-label="Time since the Big Bang" />
        <span class="t" data-t>{tWords(STEPS[step]!)}</span>
      </div>
    </div>
    <aside class="ledger">
      <h3>What this slice holds</h3>
      <dl>
        {#each shares as e (e.id)}
          <div data-env={e.id}><dt>{e.label}<small>{e.words}</small></dt><dd>{Math.round(e.share * 100)}%</dd></div>
        {/each}
        {#if web}
          <div><dt>Spread of density<small>grows as the foam sharpens</small></dt><dd data-contrast>{contrast.toFixed(2)}</dd></div>
          <div><dt>Growth since the start<small>linear growth factor</small></dt><dd>{web.slice.growth.toFixed(3)}</dd></div>
          <div><dt>Size of the universe<small>scale factor, today = 1</small></dt><dd>{web.slice.scale_factor.toFixed(3)}</dd></div>
        {/if}
      </dl>
      <h4>Reading the map</h4>
      <p>Dots are matter where gravity has moved it since the start; the shade is {show === "density" ? "the density (darker is denser, log)" : "the environment: blank voids, light sheets, darker filaments and knots in red"}. Distances are comoving: the slice is drawn at today's size, so the foam grows in contrast, not in size.</p>
      {#if web}
        <h4>The model's limits</h4>
        <ul>
          {#each web.model as m (m)}<li>{m}</li>{/each}
          <li>Linear growth: knots are drawn where matter crowds, but their inner structure (clusters, haloes) is not simulated here.</li>
        </ul>
      {/if}
    </aside>
  </div>
</div>

<style>
  .foam {
    color: var(--sse-ink);
    font-size: 13px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-width: 1180px;
  }
  .bar {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    align-items: center;
  }
  .seg {
    display: inline-flex;
    flex-wrap: wrap;
  }
  .seg button,
  .play {
    font: inherit;
    min-height: 34px;
    padding: 0 12px;
    border: 1px solid var(--sse-ink);
    background: transparent;
    color: var(--sse-ink);
    margin-right: -1px;
    cursor: pointer;
  }
  .seg button.on {
    background: var(--sse-ink);
    color: var(--sse-ground);
  }
  .head {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 12px;
    align-items: baseline;
  }
  h2 {
    font-family: var(--sse-font-display, serif);
    font-size: 24px;
    font-weight: 600;
    margin: 0;
  }
  h3 {
    font-family: var(--sse-font-display, serif);
    font-size: 18px;
    font-weight: 600;
    margin: 0 0 6px;
  }
  h4 {
    font-size: 11px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
    margin: 12px 0 4px;
    font-weight: 500;
  }
  .lede {
    margin: 0;
    flex-basis: 100%;
    font-size: 14px;
  }
  .stamp {
    border: 1.5px solid var(--sse-accent);
    color: var(--sse-accent-ink);
    font-size: 10px;
    letter-spacing: 0.14em;
    padding: 2px 8px;
  }
  .split {
    display: grid;
    grid-template-columns: minmax(0, 560px) minmax(260px, 1fr);
    gap: 20px;
    align-items: start;
  }
  .map {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  canvas {
    width: 100%;
    height: auto;
    aspect-ratio: 1;
    border: 1px solid var(--sse-rule);
    display: block;
  }
  .scale {
    border-bottom: 2px solid var(--sse-ink);
    border-left: 2px solid var(--sse-ink);
    border-right: 2px solid var(--sse-ink);
    height: 6px;
    position: relative;
  }
  .scale span {
    position: absolute;
    top: 8px;
    left: 0;
    white-space: nowrap;
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  .time {
    display: flex;
    gap: 10px;
    align-items: center;
    margin-top: 16px;
  }
  .time input {
    flex: 1;
    accent-color: var(--sse-accent);
  }
  .t {
    font-family: var(--sse-font-mono, monospace);
    min-width: 9em;
  }
  .ledger {
    border: 1px solid var(--sse-ink);
    padding: 14px 18px;
    background: var(--sse-paper, var(--sse-ground));
  }
  dl {
    margin: 0;
  }
  dl div {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    padding: 6px 0;
    border-bottom: 1px dotted var(--sse-rule);
  }
  dt small {
    display: block;
    color: var(--sse-ink-muted);
    font-size: 11px;
  }
  dd {
    margin: 0;
    font-family: var(--sse-font-mono, monospace);
  }
  ul {
    margin: 0;
    padding-left: 18px;
  }
  p {
    margin: 0;
  }
  @media (max-width: 760px) {
    .split {
      grid-template-columns: 1fr;
    }
  }
</style>
