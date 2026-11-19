<!--
  Body view (V6 part 4, handoff 8.2): the selected world as a globe drawn
  from the kernel's S1 surface tiles (six level-0 tiles, one per cube face),
  on a 2D canvas, lit from the upper left. Drag or use the arrow keys to
  turn it. Beside it the same surface in words: how much lies under water
  (or magma, or hydrocarbon), the highest and lowest ground, the main bare
  materials, the regime, the epoch, and what the kernel says is still
  pending. Below it the Cutaway. Nothing here invents terrain: no tiles,
  no globe.
-->
<script module lang="ts">
  import type { SurfaceTile } from "../../../ui-contract/src/index.ts";
  // Tiles take about a second to build; keep them per system and body for the page's life.
  const cache = new Map<string, { tiles: SurfaceTile[]; regime?: string; refused: number }>();
</script>

<script lang="ts">
  import { globeStats, sampleAt, sampleColour, viewToWorld, type Globe, type Json, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import Cutaway from "../../cutaway/src/Cutaway.svelte";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();

  const ws = $derived(ctx.workspace);
  const subject = $derived(scene.subjects.find((s) => s.address === ws.selection));
  const size = $derived(Math.round(Math.min(640, Math.max(160, Number(params.size_px ?? 320)))));

  let got = $state<{ tiles: SurfaceTile[]; regime?: string; refused: number } | null>(null);
  let loading = $state(false);
  let failed = $state<string | null>(null);
  let gen = 0;
  $effect(() => {
    const sub = subject;
    const sys = scene.system;
    const g = ++gen;
    got = null;
    failed = null;
    if (!sub || sub.kind === "star") return;
    const key = `${sys}|${sub.address}`;
    const hit = cache.get(key);
    if (hit) {
      got = hit;
      return;
    }
    loading = true;
    (async () => {
      const r = await ctx.session.surfaceTiles(sys, sub.address);
      if (g !== gen) return;
      loading = false;
      if (!r.ok) {
        failed = r.message;
        return;
      }
      const v = { tiles: r.tiles, regime: r.regime, refused: r.refused.length };
      cache.set(key, v);
      got = v;
    })();
  });

  const globe = $derived<Globe>(got ? [0, 1, 2, 3, 4, 5].map((f) => got!.tiles.find((t) => t.key.face === f && t.key.level === 0)) : []);
  const has = $derived(globe.some((t) => t));
  const stats = $derived(has ? globeStats(globe) : null);
  const epoch = $derived(globe.find((t) => t)?.epoch_myr);
  const pending = $derived([...new Set(globe.flatMap((t) => t?.pending ?? []))]);

  let lon = $state(0);
  let lat = $state(20);
  let canvas = $state<HTMLCanvasElement | undefined>();
  // Light from the upper left, towards the viewer.
  const L = (() => {
    const v = [-0.5, 0.55, 0.67];
    const l = Math.hypot(v[0]!, v[1]!, v[2]!);
    return [v[0]! / l, v[1]! / l, v[2]! / l] as const;
  })();
  $effect(() => {
    const c = canvas;
    if (!c || !has) return;
    const n = size;
    const lo = lon;
    const la = lat;
    const g = globe;
    const cx = c.getContext("2d");
    if (!cx) return;
    const img = cx.createImageData(n, n);
    const R = n / 2;
    for (let py = 0; py < n; py++) {
      const y = (R - py - 0.5) / R;
      for (let px = 0; px < n; px++) {
        const x = (px + 0.5 - R) / R;
        const d = viewToWorld(x, y, lo, la);
        if (!d) continue;
        const s = sampleAt(g, d);
        const o = (py * n + px) * 4;
        const z = Math.sqrt(Math.max(0, 1 - x * x - y * y));
        const light = 0.25 + 0.75 * Math.max(0, x * L[0] + y * L[1] + z * L[2]);
        const rgb = s ? sampleColour(s.tile, s.i, light) : [40, 40, 40];
        img.data[o] = rgb[0]!;
        img.data[o + 1] = rgb[1]!;
        img.data[o + 2] = rgb[2]!;
        img.data[o + 3] = 255;
      }
    }
    cx.putImageData(img, 0, 0);
  });

  let drag: { x: number; y: number; lon: number; lat: number } | null = null;
  function down(e: PointerEvent) {
    drag = { x: e.clientX, y: e.clientY, lon, lat };
    try {
      (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    } catch {
      // Synthetic events have no live pointer; dragging still works without capture.
    }
  }
  function move(e: PointerEvent) {
    if (!drag) return;
    const k = 180 / size;
    lon = (((drag.lon - (e.clientX - drag.x) * k) % 360) + 360) % 360;
    lat = Math.max(-90, Math.min(90, drag.lat + (e.clientY - drag.y) * k));
  }
  function key(e: KeyboardEvent) {
    const step = e.shiftKey ? 45 : 15;
    if (e.key === "ArrowLeft") lon = (lon + 360 - step) % 360;
    else if (e.key === "ArrowRight") lon = (lon + step) % 360;
    else if (e.key === "ArrowUp") lat = Math.min(90, lat + step);
    else if (e.key === "ArrowDown") lat = Math.max(-90, lat - step);
    else return;
    e.preventDefault();
  }

  const pct = (x: number) => (x >= 0.995 ? "all" : x < 0.005 ? "none" : `${Math.round(x * 100)}%`);
  const km = (m: number) => (m < 0 ? "−" : "") + (Math.abs(m) >= 1000 ? `${(Math.abs(m) / 1000).toFixed(1)} km` : `${Math.round(Math.abs(m))} m`);
  const coverWords: Record<string, string> = { water: "under water", magma: "under magma", hydrocarbon: "under hydrocarbon seas", ice: "under ice" };
  const covers = $derived(stats ? Object.entries(stats.covered).filter(([k, v]) => k !== "none" && v > 0).sort((a, b) => b[1] - a[1]) : []);
  const summary = $derived.by(() => {
    if (!stats || !subject) return "";
    const parts = covers.length ? covers.map(([k, v]) => `${pct(v)} ${coverWords[k] ?? k}`) : ["no surface liquid"];
    const mats = stats.materials.slice(0, 3).map(([m, v]) => `${m} ${pct(v)}`);
    return `${subject.label}: ${parts.join(", ")}. Ground from ${km(stats.lowest_m)} to ${km(stats.highest_m)}.${mats.length ? ` Bare ground: ${mats.join(", ")}.` : ""}`;
  });
  const status = $derived(!subject ? "none" : subject.kind === "star" ? "star" : loading ? "loading" : has ? subject.address : "no-surface");
</script>

<div class="body" data-body={status}>
  {#if !subject}
    <p class="note">Pick a world on the chart to see its globe.</p>
  {:else if subject.kind === "star"}
    <p class="note">{subject.label} is a star; the Body view shows worlds.</p>
  {:else}
    <div class="top">
      <div class="globe" style:width="{size}px" style:height="{size}px">
        {#if loading}
          <p class="note">Building the surface of {subject.label}… (about a second)</p>
        {:else if has}
          <canvas
            bind:this={canvas}
            width={size}
            height={size}
            tabindex="0"
            aria-label={`Globe of ${subject.label}. ${summary} Drag or use the arrow keys to turn it.`}
            data-lon={Math.round(lon)}
            data-lat={Math.round(lat)}
            onpointerdown={down}
            onpointermove={move}
            onpointerup={() => (drag = null)}
            onpointercancel={() => (drag = null)}
            onkeydown={key}
          ></canvas>
        {:else}
          <p class="note" data-why>
            No surface for {subject.label} yet.
            {failed ? failed : "The kernel builds surfaces for systems in the campaign: grow or import one with Make, then open it here."}
          </p>
        {/if}
      </div>
      {#if stats}
        <dl class="facts" data-globe-stats>
          <dt>Surface</dt>
          <dd data-covers>{covers.length ? covers.map(([k, v]) => `${pct(v)} ${coverWords[k] ?? k}`).join(", ") : "No surface liquid"}</dd>
          <dt>Highest</dt>
          <dd>{km(stats.highest_m)}</dd>
          <dt>Lowest</dt>
          <dd>{km(stats.lowest_m)}</dd>
          {#if stats.materials.length}
            <dt>Bare ground</dt>
            <dd>{stats.materials.slice(0, 4).map(([m, v]) => `${m} ${pct(v)}`).join(", ")}</dd>
          {/if}
          {#if got?.regime}
            <dt>Regime</dt>
            <dd data-regime>{got.regime}</dd>
          {/if}
          {#if epoch !== undefined}
            <dt>As of</dt>
            <dd>{epoch.toLocaleString("en-GB", { maximumFractionDigits: 1 })} Myr</dd>
          {/if}
          {#if pending.length}
            <dt>Not yet modelled</dt>
            <dd data-pending>{pending.join(", ")}</dd>
          {/if}
          <dt>Detail</dt>
          <dd>{stats.samples.toLocaleString("en-GB")} samples, level 0 (coarse). Turn with drag or arrow keys.</dd>
        </dl>
      {/if}
    </div>
  {/if}
  {#if params.cutaway !== false}
    <div class="cut">
      <Cutaway {ctx} {scene} {params} />
    </div>
  {/if}
</div>

<style>
  .body {
    display: flex;
    flex-direction: column;
    gap: 16px;
    color: var(--sse-ink);
  }
  .top {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    align-items: flex-start;
  }
  .globe {
    max-width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  canvas {
    max-width: 100%;
    border-radius: 50%;
    cursor: grab;
    touch-action: none;
  }
  canvas:focus-visible {
    outline: 2px solid var(--sse-accent);
    outline-offset: 4px;
  }
  .note {
    font-size: 13px;
    color: var(--sse-ink-muted);
    max-width: 36em;
  }
  .facts {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 4px 12px;
    font-size: 13px;
    margin: 0;
    min-width: 200px;
    max-width: 360px;
  }
  dt {
    color: var(--sse-ink-muted);
  }
  dd {
    margin: 0;
  }
</style>
