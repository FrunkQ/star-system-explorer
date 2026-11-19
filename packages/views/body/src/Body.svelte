<!--
  Body view (V6 part 4, handoff 8.2): the selected world as a globe drawn
  from the kernel's S1 surface tiles (six level-0 tiles, one per cube face),
  by a render recipe (R1, packages/renderer): on the GPU, a 2D canvas or in
  words, hill-shaded from the elevation layer. The quality badge says how it
  was drawn and what the governor reduced. Drag or use the arrow keys to
  turn it. Beside it the same surface in words: how much lies under water
  (or magma, or hydrocarbon), the highest and lowest ground, the main bare
  materials, the regime, the epoch, and what the kernel says is still
  pending. Below it the Cutaway. Nothing here invents terrain: no tiles,
  no globe.
-->
<script module lang="ts">
  import type { SurfaceTile } from "../../../ui-contract/src/index.ts";
  import type { AirFacts } from "../../../renderer/src/index.ts";
  // Tiles take about a second to build; keep them per system and body for the page's life.
  const cache = new Map<string, { tiles: SurfaceTile[]; regime?: string; refused: number; air: AirFacts | null }>();
</script>

<script lang="ts">
  import { globeStats, type Globe, type Json, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { untrack } from "svelte";
  import { airFactsOf, keepPhoto, listPhotos, LIBRARY, mountGlobe, photograph, removePhoto, type GlobeRun, type MediaItem, type GlobeRunOptions, type GlobeStatus } from "../../../renderer/src/index.ts";
  import { SHIPPED_RECIPES } from "../../../renderer/src/recipes.ts";
  import Cutaway from "../../cutaway/src/Cutaway.svelte";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();

  const ws = $derived(ctx.workspace);
  const subject = $derived(scene.subjects.find((s) => s.address === ws.selection));
  const size = $derived(Math.round(Math.min(640, Math.max(160, Number(params.size_px ?? 320)))));

  let got = $state<{ tiles: SurfaceTile[]; regime?: string; refused: number; air: AirFacts | null } | null>(null);
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
      if (!r.ok) {
        loading = false;
        failed = r.message;
        return;
      }
      // The air the atmosphere pass draws: the kernel's surface pressure, temperature, gravity and gas mix.
      const rs = await ctx.session.resolvedSystem(sys);
      if (g !== gen) return;
      const surf = rs.ok ? (rs.result.surfaces.find((x) => x.address === sub.address) as Parameters<typeof airFactsOf>[1]) : undefined;
      const v = { tiles: r.tiles, regime: r.regime, refused: r.refused.length, air: rs.ok ? airFactsOf(rs.result.facts[sub.address], surf) : null };
      cache.set(key, v);
      got = v;
      loading = false;
    })();
  });

  const globe = $derived<Globe>(got ? [0, 1, 2, 3, 4, 5].map((f) => got!.tiles.find((t) => t.key.face === f && t.key.level === 0)) : []);
  const has = $derived(globe.some((t) => t));
  const stats = $derived(has ? globeStats(globe) : null);
  const epoch = $derived(globe.find((t) => t)?.epoch_myr);
  const pending = $derived([...new Set(globe.flatMap((t) => t?.pending ?? []))]);

  let lon = $state(0);
  let lat = $state(20);
  let host = $state<HTMLElement | undefined>();
  let run: GlobeRun | null = null;
  let gs = $state<GlobeStatus | null>(null);
  let badgeOpen = $state(false);
  // The user's own choices on this device, over the view's settings (the probe can be overridden: rendering report 6.2).
  let ovRecipe = $state<string | null>(null);
  let ovTier = $state<string | null>(null);
  let ovSpin = $state<boolean | null>(null);
  const runOpts = $derived({
    registry: SHIPPED_RECIPES,
    library: LIBRARY,
    recipe: ovRecipe ?? String(params.recipe ?? "survey-relief"),
    tier: (ovTier ?? String(params.tier ?? "auto")) as GlobeRunOptions["tier"],
    knobs: { relief: Number(params.relief ?? 0.5), detail: Number(params.detail ?? 0.5) },
    budget_ms: Number(params.budget_ms ?? 33.3),
    governor: params.governor !== false,
    spin: ovSpin ?? params.spin === true,
    size_px: size,
    label: subject?.label ?? "",
    air: got?.air ?? null,
  });
  // One runtime per globe element; settings and turns are passed to it.
  $effect(() => {
    const h = host;
    if (!h) return;
    const r = mountGlobe(h, untrack(() => runOpts), (st) => {
      gs = st;
      if (st.lon !== lon) lon = st.lon;
    });
    run = r;
    return () => {
      r.dispose();
      run = null;
      gs = null;
    };
  });
  $effect(() => {
    const o = runOpts;
    untrack(() => run)?.update(o);
  });
  $effect(() => {
    const g = globe;
    if (has) untrack(() => run)?.setGlobe(g);
  });
  $effect(() => {
    const lo = lon;
    const la = lat;
    const st = untrack(() => gs);
    if (st && st.lon === lo && st.lat === la) return;
    untrack(() => run)?.setView(lo, la);
  });
  // R3: take a photograph (the still tier, off screen), keep it on this device, save it as a PNG.
  let shooting = $state<string | null>(null);
  let shotNote = $state<string | null>(null);
  let photos = $state<{ item: MediaItem; url: string }[]>([]);
  $effect(() => {
    const a = subject?.address;
    if (!a) return;
    let live = true;
    void listPhotos(a).then((l) => {
      if (!live) return;
      for (const p of untrack(() => photos)) URL.revokeObjectURL(p.url);
      photos = l.map((item) => ({ item, url: URL.createObjectURL(item.png) }));
    });
    return () => {
      live = false;
    };
  });
  const stamp = (d: Date) => d.toISOString().slice(0, 16).replace("T", " ");
  async function takePhoto() {
    const s = subject;
    if (!s || !has || shooting) return;
    shooting = "Starting…";
    shotNote = null;
    try {
      const o = runOpts;
      const shot = await photograph({ registry: o.registry, library: o.library, recipe: o.recipe, knobs: o.knobs, globe, air: o.air, label: s.label, lon, lat, size: 1024, frames: 16, onProgress: (d, n) => (shooting = `Refining ${d} of ${n}…`) });
      const png = await new Promise<Blob>((res, rej) => shot.canvas.toBlob((b) => (b ? res(b) : rej(new Error("the browser would not encode the picture"))), "image/png"));
      const now = new Date();
      const caption = `${s.label}, centred on ${lat.toFixed(0)}° ${lon.toFixed(0)}°. ${shot.note} Made in ${(shot.ms / 1000).toFixed(1)} s.`;
      const item: MediaItem = { id: `${s.address}@${now.getTime()}`, title: `${s.label} ${stamp(now)}`, subject: s.address, caption, created: now.toISOString(), png, width: shot.canvas.width, height: shot.canvas.height };
      const where = await keepPhoto(item);
      photos = [{ item, url: URL.createObjectURL(png) }, ...photos];
      shotNote = `${caption} ${where === "device" ? "Kept on this device." : "This browser will not keep it after the page closes: save it."}`;
    } catch (e) {
      shotNote = `No photograph: ${e instanceof Error ? e.message : String(e)}`;
    } finally {
      shooting = null;
    }
  }
  async function dropPhoto(id: string) {
    await removePhoto(id);
    const gone = photos.find((p) => p.item.id === id);
    if (gone) URL.revokeObjectURL(gone.url);
    photos = photos.filter((p) => p.item.id !== id);
  }
  const fileName = (m: MediaItem) => `${m.title.replace(/[^A-Za-z0-9 _-]+/g, "").replace(/ +/g, "-")}.png`;
  const tierChoices: [string, string][] = [["auto", "This device's choice"], ["gpu-high", "GPU, high"], ["gpu-basic", "GPU"], ["canvas", "2D canvas"], ["text", "Words only"]];
  const tierWords: Record<string, string> = { "gpu-high": "GPU high", "gpu-basic": "GPU", canvas: "Canvas", text: "Text" };

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
  const coverWords: Record<string, string> = { water: "under water", magma: "under magma", hydrocarbon: "under hydrocarbon seas" };
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
          <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions (a turnable globe: keys and drag, named by its label) -->
          <div
            class="surface"
            bind:this={host}
            role="application"
            aria-roledescription="globe"
            tabindex="0"
            aria-label={`Globe of ${subject.label}. ${summary} ${gs?.narration ?? ""} Drag or use the arrow keys to turn it.`}
            data-lon={Math.round(lon)}
            data-lat={Math.round(lat)}
            data-tier={gs?.tier ?? ""}
            data-backend={gs?.backend ?? ""}
            data-level={gs?.level ?? 0}
            data-max={gs?.max ?? 0}
            data-recipe={gs?.recipe ?? ""}
            data-frame-ms={gs && Number.isFinite(gs.ewma_ms) ? gs.ewma_ms.toFixed(1) : ""}
            data-budget-ms={gs?.budget_ms ?? ""}
            data-frames={gs?.frames ?? 0}
            data-work-ms={gs ? gs.work_ms.toFixed(1) : ""}
            onpointerdown={down}
            onpointermove={move}
            onpointerup={() => (drag = null)}
            onpointercancel={() => (drag = null)}
            onkeydown={key}
          >
            {#if gs?.tier === "text"}
              <p class="words" data-narration>{gs.narration}</p>
            {/if}
          </div>
        {:else}
          <p class="note" data-why>
            No surface for {subject.label} yet.
            {failed ? failed : "The kernel builds surfaces for systems in the campaign: grow or import one with Make, then open it here."}
          </p>
        {/if}
      </div>
      {#if stats}
        <dl class="facts" data-globe-stats>
          <dt>Drawn</dt>
          <dd>
            <button class="badge" class:reduced={gs?.badge.reduced} data-badge={gs?.badge.reduced ? "reduced" : "full"} aria-expanded={badgeOpen} onclick={() => (badgeOpen = !badgeOpen)}>
              {gs ? `${tierWords[gs.tier] ?? gs.tier}${gs.backend === "webgpu" ? " (WebGPU)" : gs.backend === "webgl2" ? " (WebGL2)" : ""} · ${gs.badge.short}` : "Starting…"}
            </button>
            {#if badgeOpen && gs}
              <div class="badge-detail" data-badge-detail>
                <p>{gs.tier_note}</p>
                {#if gs.plan.passes.some((x) => x.pass === "atmosphere")}<p data-air-note>{gs.air_note}</p>{/if}
                {#each gs.badge.detail as line (line)}<p>{line}</p>{/each}
                {#if gs.badge.reduced}<p>{gs.badge.lift}</p>{/if}
                <p>Recipe {gs.recipe}, level {gs.level} of {gs.max}. {Number.isFinite(gs.ewma_ms) ? `Frames about ${Math.round(gs.ewma_ms)} ms; budget ${Math.round(gs.budget_ms)} ms.` : ""}</p>
                <div class="drawing">
                  <label>Recipe <select value={runOpts.recipe} onchange={(e) => (ovRecipe = e.currentTarget.value)}>{#each SHIPPED_RECIPES as r (r.id)}<option value={r.id}>{r.name}</option>{/each}</select></label>
                  <label>Draw on <select value={runOpts.tier} onchange={(e) => (ovTier = e.currentTarget.value)}>{#each tierChoices as [v, w] (v)}<option value={v}>{w}</option>{/each}</select></label>
                  <label><input type="checkbox" checked={runOpts.spin} onchange={(e) => (ovSpin = e.currentTarget.checked)} /> Spin</label>
                </div>
              </div>
            {/if}
          </dd>
          <dt>Photograph</dt>
          <dd data-photo>
            <button class="badge" data-take-photo disabled={!!shooting} onclick={takePhoto}>{shooting ?? "Take photograph"}</button>
            {#if shotNote}<p class="shot-note" data-photo-note>{shotNote}</p>{/if}
            {#if photos.length}
              <ul class="photos" data-photos>
                {#each photos.slice(0, 6) as p (p.item.id)}
                  <li>
                    <img src={p.url} alt={p.item.caption} width="48" height="48" />
                    <a href={p.url} download={fileName(p.item)}>Save</a>
                    <button class="link" onclick={() => dropPhoto(p.item.id)} aria-label={`Remove ${p.item.title}`}>Remove</button>
                  </li>
                {/each}
              </ul>
            {/if}
          </dd>
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
  .surface {
    max-width: 100%;
    cursor: grab;
    touch-action: none;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .surface :global(canvas) {
    max-width: 100%;
    border-radius: 50%;
    display: block;
  }
  .words {
    font-size: 13px;
    max-width: 30em;
    margin: 0;
  }
  .badge {
    font: inherit;
    font-size: 12px;
    color: var(--sse-ink);
    background: none;
    border: 1px solid var(--sse-line);
    border-radius: 3px;
    padding: 1px 6px;
    cursor: pointer;
    text-align: left;
  }
  .badge.reduced {
    border-color: var(--sse-accent);
  }
  .badge-detail {
    font-size: 12px;
    color: var(--sse-ink-muted);
    margin-top: 4px;
  }
  .drawing {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 12px;
    margin-top: 4px;
    color: var(--sse-ink);
  }
  .drawing select {
    font: inherit;
  }
  .badge-detail p {
    margin: 2px 0;
  }
  .surface:focus-visible {
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
  .shot-note {
    font-size: 12px;
    color: var(--sse-ink-muted);
    margin: 4px 0 0;
  }
  .photos {
    list-style: none;
    padding: 0;
    margin: 6px 0 0;
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .photos li {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    font-size: 11px;
  }
  .photos img {
    border-radius: 50%;
    background: var(--sse-line);
  }
  .photos a,
  .link {
    font: inherit;
    color: var(--sse-accent);
    background: none;
    border: 0;
    padding: 0;
    cursor: pointer;
  }
</style>
