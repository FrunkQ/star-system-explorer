<!--
  The cluster map (V9). One halo cell of the universe from the kernel's
  halo_cell (G2): each galaxy's dark-matter halo at its place (seen from
  above, x across, y up), sized by the cube root of its mass, over the cosmic
  web through the cell's middle (web_slice). Pick one to read its history;
  "Chart this galaxy" opens it in the galaxy chart. Neighbouring cells are a
  step away. Presentation only: the numbers are the kernel's.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { cellHalo, foamCells, haloRadius, overdensityWords, projectHalo, sci, type CellHalo, type HaloCell, type Json, type SystemScene, type ViewContext, type WebSlice } from "../../../ui-contract/src/index.ts";

  let { ctx, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();
  const first = untrack(() => ({ min: String(params.min_mass ?? "1e10"), web: params.web !== false }));
  const MINS = ["1e10", "1e11", "1e12"] as const;
  let min = $state<string>(MINS.includes(first.min as (typeof MINS)[number]) ? first.min : "1e10");
  let showWeb = $state(first.web);
  const fromSel = untrack(() => cellHalo(ctx.workspace.selection));
  let idx = $state<[number, number, number]>(fromSel ? fromSel.cell : [0, 0, 0]);
  let cell = $state<HaloCell | null>(null);
  let web = $state<WebSlice | null>(null);
  let failed = $state<string | null>(null);
  let busy = $state(false);
  let picked = $state<string | null>(untrack(() => (fromSel ? ctx.workspace.selection : null)));
  let gen = 0;

  $effect(() => {
    const i = idx;
    const m = Number(min);
    const g = ++gen;
    busy = true;
    ctx.session.haloCell({ cell: i, min_mass_msun: m }).then((r) => {
      if (g !== gen) return;
      busy = false;
      if (!r.ok) return void ((failed = r.message), (cell = null));
      failed = null;
      cell = r.cell;
      if (picked && !r.cell.haloes.some((h) => h.address === picked)) picked = null;
      if (!picked && r.cell.haloes[0]) picked = r.cell.haloes[0].address;
    });
  });
  // The web through the cell's middle, for the background (one slice per cell).
  let wgen = 0;
  $effect(() => {
    const c = cell;
    const on = showWeb;
    if (!c || !on) return void (web = null);
    const s = c.side_mpc;
    const centre: [number, number, number] = [c.origin_mpc[0] + s / 2, c.origin_mpc[1] + s / 2, c.origin_mpc[2] + s / 2];
    const g = ++wgen;
    ctx.session.webSlice({ centre, extent_mpc: s, n: 48 }).then((r) => {
      if (g === wgen) web = r.ok ? r.web : null;
    });
  });

  function move(axis: 0 | 1 | 2, by: number) {
    const n: [number, number, number] = [...idx];
    n[axis] += by;
    idx = n;
  }
  const S = 520;
  const dots = $derived.by(() => {
    if (!cell) return [];
    const c = cell;
    // Smallest last so the big ones never hide them.
    return c.haloes.map((h) => ({ h, ...projectHalo(h.position_mpc, c.origin_mpc, c.side_mpc, "xy", S), r: haloRadius(h.mass_msun) })).reverse();
  });
  const links = $derived.by(() => {
    if (!cell) return [];
    const at = new Map(dots.map((d) => [d.h.address, d]));
    return dots.flatMap((d) => {
      const n = d.h.neighbour;
      const o = n?.bound ? at.get(n.address) : undefined;
      return o && d.h.address < o.h.address ? [{ a: d, b: o }] : [];
    });
  });
  const one = $derived<CellHalo | null>(cell?.haloes.find((h) => h.address === picked) ?? null);
  const biggest = $derived(cell?.haloes[0] ?? null);
  function pick(h: CellHalo) {
    picked = h.address;
  }
  let root = $state<HTMLDivElement | null>(null);
  function chart(h: CellHalo) {
    ctx.workspace.select(h.address);
    root?.dispatchEvent(new CustomEvent("sse:explore", { bubbles: true, detail: { section: "galaxy" } }));
  }
  const nameOf = (a: string) => a.split("/").at(-1)?.replace("halo:", "Halo ") ?? a;
  const gyr = (t: number) => `${t.toFixed(t < 1 ? 2 : 1)} Gyr`;
  // The mass function: expected and drawn per bin, over the bins that matter.
  const bins = $derived(cell ? cell.bins.filter((b) => b.expected >= 0.05 || b.drawn > 0).sort((a, b) => a.log10_mass - b.log10_mass) : []);
  const binTop = $derived(Math.max(1, ...bins.map((b) => Math.max(b.expected, b.drawn))));

  // Drawing the web: one canvas under the dots, colours from the theme's tokens.
  let canvas = $state<HTMLCanvasElement | null>(null);
  let tick = $state(0);
  $effect(() => {
    const o = new MutationObserver(() => (tick += 1));
    o.observe(document.documentElement, { attributes: true, attributeFilter: ["style"] });
    return () => o.disconnect();
  });
  function rgb(el: Element, name: string): [number, number, number] {
    const v = getComputedStyle(el).getPropertyValue(name).trim() || "currentColor";
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
    if (!cv) return;
    const g = cv.getContext("2d");
    if (!g) return;
    const ground = rgb(cv, "--sse-ground");
    g.fillStyle = `rgb(${ground.join(",")})`;
    g.fillRect(0, 0, cv.width, cv.height);
    if (!w) return;
    const ink = rgb(cv, "--sse-ink");
    const s = w.slice;
    const off = document.createElement("canvas");
    off.width = s.n;
    off.height = s.n;
    const oc = off.getContext("2d");
    if (!oc) return;
    const img = oc.createImageData(s.n, s.n);
    for (const c of foamCells(s)) {
      const a = c.shade * 0.35;
      const p = ((s.n - 1 - c.j) * s.n + c.i) * 4;
      for (let k = 0; k < 3; k++) img.data[p + k] = Math.round(ground[k]! * (1 - a) + ink[k]! * a);
      img.data[p + 3] = 255;
    }
    oc.putImageData(img, 0, 0);
    g.imageSmoothingEnabled = true;
    const pad = (12 * cv.width) / S;
    g.drawImage(off, pad, pad, cv.width - 2 * pad, cv.height - 2 * pad);
  });
</script>

<div class="clu" data-cell={cell ? cell.index.join(".") : ""} bind:this={root}>
  <div class="bar">
    <span class="lbl">Cell {idx.join(" · ")}</span>
    <div class="seg" role="group" aria-label="Go to a neighbouring cell">
      <button type="button" onclick={() => move(0, -1)} data-move="x-" aria-label="West, minus x">← x</button>
      <button type="button" onclick={() => move(0, 1)} data-move="x+" aria-label="East, plus x">x →</button>
      <button type="button" onclick={() => move(1, 1)} data-move="y+" aria-label="North, plus y">↑ y</button>
      <button type="button" onclick={() => move(1, -1)} data-move="y-" aria-label="South, minus y">↓ y</button>
      <button type="button" onclick={() => move(2, 1)} data-move="z+" aria-label="Up, plus z">z +</button>
      <button type="button" onclick={() => move(2, -1)} data-move="z-" aria-label="Down, minus z">z −</button>
    </div>
    <label>Smallest
      <select bind:value={min} data-min>
        {#each MINS as m (m)}<option value={m}>{sci(Number(m), 1)} suns</option>{/each}
      </select>
    </label>
    <label class="tick"><input type="checkbox" bind:checked={showWeb} data-web-toggle /> the cosmic web behind</label>
    {#if busy}<span class="hint">drawing the haloes…</span>{/if}
  </div>

  {#if failed}
    <p class="note" role="alert">Could not chart the cell: {failed}</p>
  {:else if cell}
    <header class="head">
      <h2>Galaxies nearby</h2>
      <span class="stamp">grown, not prebaked</span>
      <p class="lede" data-lede>
        A cube {cell.side_mpc} megaparsecs (about {Math.round(cell.side_mpc * 3.26)} million light years) a side, {overdensityWords(cell.delta_l)}. It holds {cell.total} galaxies' haloes of 10 billion suns or more{cell.above_min !== cell.total ? `, ${cell.above_min} of them above ${sci(cell.min_mass_msun, 1)}` : ""}{biggest ? `; the biggest weighs ${sci(biggest.mass_msun)} suns` : ""}.
      </p>
    </header>
    <div class="split">
      <figure class="map">
        <div class="frame">
          <canvas bind:this={canvas} width={S * 2} height={S * 2} aria-hidden="true"></canvas>
          <svg viewBox="0 0 {S} {S}" role="img" aria-label="Haloes in the cell, seen from above">
            <rect class="box" x="12" y="12" width={S - 24} height={S - 24} />
            {#each links as l (l.a.h.address)}<line class="bound" x1={l.a.x} y1={l.a.y} x2={l.b.x} y2={l.b.y} />{/each}
            {#each dots as d (d.h.address)}
              <circle
                class="halo"
                class:big={d.h === biggest}
                class:on={d.h.address === picked}
                cx={d.x}
                cy={d.y}
                r={d.r}
                role="button"
                tabindex="0"
                aria-label="{nameOf(d.h.address)}, {sci(d.h.mass_msun)} suns"
                onclick={() => pick(d.h)}
                onkeydown={(e) => (e.key === "Enter" || e.key === " ") && pick(d.h)}
                data-halo-dot={d.h.key}
              ><title>{nameOf(d.h.address)}: {sci(d.h.mass_msun)} suns</title></circle>
            {/each}
            <text class="axis" x={S - 14} y={S - 2} text-anchor="end">x, {cell.side_mpc} Mpc →</text>
            <text class="axis" x="14" y="10">↑ y</text>
          </svg>
        </div>
        <figcaption class="hint">Seen from above (x across, y up), every halo in the cube whatever its height. Dots grow with the cube root of mass; a line joins two galaxies bound to fall together. {showWeb ? "Shading: the cosmic web through the cube's middle, today." : ""}</figcaption>
      </figure>

      <aside class="ledger" data-halo-ledger={one?.key ?? ""}>
        {#if one}
          <h3>{nameOf(one.address)}</h3>
          <p class="mass"><strong>{sci(one.mass_msun)}</strong> suns of matter, mostly dark</p>
          <dl>
            <dt>Half its mass in place by</dt><dd>{gyr(one.half_mass_gyr)} after the Big Bang</dd>
            <dt>Mergers since redshift 6</dt><dd>{one.mergers}, {one.major_mergers} of them major{one.last_major_gyr !== null ? `, the last at ${gyr(one.last_major_gyr)}` : ""}</dd>
            <dt>Spin</dt><dd>{one.spin.toFixed(3)} (a typical halo spins at 0.035)</dd>
            <dt>Where</dt><dd>{one.position_mpc.map((v) => v.toFixed(1)).join(", ")} Mpc</dd>
            {#if one.neighbour}
              <dt>Nearest big neighbour</dt>
              <dd>{nameOf(one.neighbour.address)}, {one.neighbour.distance_mpc.toFixed(2)} Mpc away, {one.neighbour.bound ? `bound: they first meet in about ${gyr(one.neighbour.approach_gyr ?? 0)}` : "not bound"}</dd>
            {/if}
          </dl>
          <button type="button" class="go" onclick={() => chart(one!)} data-chart-galaxy>Chart this galaxy</button>
          <p class="hint">Opens it in the galaxy chart, grown from its own history.</p>
        {:else}
          <p class="hint">Pick a halo on the map.</p>
        {/if}
        <h4>How many, of each size</h4>
        <svg class="mf" viewBox="0 0 300 110" role="img" aria-label="Haloes per mass bin: expected and drawn" data-mass-function>
          {#each bins as b, i (b.log10_mass)}
            {@const w = 300 / Math.max(1, bins.length)}
            {@const he = (b.expected / binTop) * 80}
            {@const hd = (b.drawn / binTop) * 80}
            <rect class="drawn" x={i * w + 2} y={90 - hd} width={Math.max(1, w - 4)} height={hd} />
            <line class="expected" x1={i * w + 1} x2={i * w + w - 1} y1={90 - he} y2={90 - he} />
            {#if i % 2 === 0}<text class="axis" x={i * w + w / 2} y="104" text-anchor="middle">{b.log10_mass.toFixed(1)}</text>{/if}
          {/each}
        </svg>
        <p class="hint">Bars: haloes drawn in this cube, by log of mass. Ticks: how many the mass function expects for a cube this dense. The draw is a Poisson count, so a few bins differ.</p>
        <h4>How it is made</h4>
        <ul class="model">{#each cell.model as m, i (i)}<li>{m}</li>{/each}</ul>
      </aside>
    </div>
  {/if}
</div>

<style>
  .clu {
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
    gap: 10px 12px;
    align-items: center;
  }
  .lbl {
    font-family: var(--sse-font-mono, monospace);
  }
  .seg {
    display: inline-flex;
    flex-wrap: wrap;
  }
  .seg button,
  .go {
    font: inherit;
    min-height: 34px;
    padding: 0 10px;
    border: 1px solid var(--sse-ink);
    background: transparent;
    color: var(--sse-ink);
    margin-right: -1px;
    cursor: pointer;
  }
  .go {
    margin: 10px 0 0;
    background: var(--sse-ink);
    color: var(--sse-ground);
  }
  .tick {
    display: inline-flex;
    gap: 6px;
    align-items: center;
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
    margin: 0 0 4px;
  }
  h4 {
    font-size: 11px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
    margin: 14px 0 4px;
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
    text-transform: uppercase;
    padding: 2px 8px;
  }
  .split {
    display: grid;
    grid-template-columns: minmax(0, 560px) minmax(260px, 1fr);
    gap: 20px;
    align-items: start;
  }
  .map {
    margin: 0;
  }
  .frame {
    position: relative;
    width: 100%;
    aspect-ratio: 1;
    border: 1px solid var(--sse-ink);
  }
  .frame canvas,
  .frame svg {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }
  .box {
    fill: none;
    stroke: var(--sse-rule);
    stroke-dasharray: 3 3;
  }
  .halo {
    fill: var(--sse-ink);
    fill-opacity: 0.8;
    stroke: var(--sse-ground);
    stroke-width: 1;
    cursor: pointer;
  }
  .halo.big {
    fill: var(--sse-accent);
    fill-opacity: 1;
  }
  .halo.on {
    stroke: var(--sse-accent);
    stroke-width: 3;
  }
  .bound {
    stroke: var(--sse-ink-muted);
    stroke-width: 1;
    stroke-dasharray: 2 2;
  }
  .axis {
    fill: var(--sse-ink-muted);
    font-size: 10px;
  }
  .ledger {
    border: 1px solid var(--sse-ink);
    padding: 14px 18px;
  }
  .mass {
    margin: 0 0 8px;
  }
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 4px 12px;
    margin: 0;
  }
  dt {
    color: var(--sse-ink-muted);
  }
  dd {
    margin: 0;
  }
  .mf {
    width: 100%;
    max-width: 320px;
  }
  .mf .drawn {
    fill: var(--sse-ink);
    fill-opacity: 0.7;
  }
  .mf .expected {
    stroke: var(--sse-accent);
    stroke-width: 2;
  }
  .model {
    margin: 0;
    padding-left: 18px;
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
  .hint {
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
  .note {
    border-left: 3px solid var(--sse-accent);
    padding: 6px 10px;
  }
  @media (max-width: 760px) {
    .split {
      grid-template-columns: 1fr;
    }
  }
</style>
