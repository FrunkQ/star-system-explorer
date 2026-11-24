<!--
  A world's drawn pictures for the body card (Alex, 24 Nov: "same as a ship"): its surface as a flat map (2D) or a
  turnable globe (3D), both from the same level-0 tiles the Body view draws (surface.ts), in true colour. The surface
  takes about a second to build, so a click on a world never builds it: a world already built shows at once, any
  other waits for a tap ("no heavy work for a click"). A star shows its disc (StarDisc).
-->
<script lang="ts">
  import { untrack } from "svelte";
  import type { Globe, SystemScene, ViewContext } from "../../../ui-contract/src/index.ts";
  import { LIBRARY, mountGlobe, trueSurface, type GlobeRun } from "../../../renderer/src/index.ts";
  import { SHIPPED_RECIPES } from "../../../renderer/src/recipes.ts";
  import { loadSurface, surfaceKept, type Surface } from "./surface.ts";
  import StarDisc from "./StarDisc.svelte";

  let { ctx, scene, address, label, kind, show, size = 220 }: { ctx: ViewContext; scene: SystemScene; address: string; label: string; kind: string; show: "map" | "globe" | "disc" | "x-ray"; size?: number } = $props();

  let got = $state<Surface | null>(null);
  let busy = $state(false);
  let failed = $state<string | null>(null);
  let gen = 0;
  $effect(() => {
    void address;
    got = surfaceKept(scene.system, address) ?? null;
    failed = null;
    busy = false;
    gen++;
  });
  async function build() {
    const g = gen;
    busy = true;
    const v = await loadSurface(ctx, scene.system, address);
    if (g !== gen) return;
    busy = false;
    if (typeof v === "string") failed = v;
    else got = v;
  }

  const globe = $derived<Globe>(got ? [0, 1, 2, 3, 4, 5].map((f) => got!.tiles.find((t) => t.key.face === f && t.key.level === 0)) : []);
  const has = $derived(globe.some((t) => t));

  // The flat map: the true-colour ground, equirectangular, north up (the texture's row 0 is the south pole).
  let mapCanvas = $state<HTMLCanvasElement | undefined>();
  $effect(() => {
    const c = mapCanvas;
    const tex = got && show === "map" ? trueSurface(got.tiles, 256, 128) : null;
    if (!c || !tex) return;
    c.width = tex.width;
    c.height = tex.height;
    const img = new ImageData(tex.width, tex.height);
    for (let j = 0; j < tex.height; j++) img.data.set(tex.data.subarray((tex.height - 1 - j) * tex.width * 4, (tex.height - j) * tex.width * 4), j * tex.width * 4);
    c.getContext("2d")?.putImageData(img, 0, 0);
  });

  // The globe: one runtime per element, turned by drag or the arrow keys.
  let host = $state<HTMLElement | undefined>();
  let run: GlobeRun | null = null;
  let lon = $state(0);
  let lat = $state(20);
  $effect(() => {
    const h = host;
    if (!h || !has) return;
    const r = mountGlobe(h, { registry: SHIPPED_RECIPES, library: LIBRARY, recipe: "true-colour", tier: "auto", knobs: { relief: 0.5, detail: 0.5 }, budget_ms: 33.3, governor: true, spin: false, size_px: size, label, air: untrack(() => got?.air ?? null) }, () => {});
    r.setGlobe(untrack(() => globe));
    run = r;
    return () => {
      r.dispose();
      run = null;
    };
  });
  $effect(() => {
    const lo = lon;
    const la = lat;
    run?.setView(lo, la);
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
    e.stopPropagation();
  }
</script>

<div class="drawn" data-body-picture={show} data-built={has ? "yes" : "no"}>
  {#if kind === "star"}
    <StarDisc {ctx} {scene} {address} {label} {size} bare light={show === "x-ray" ? "xray" : "visible"} />
  {:else if failed}
    <p class="note">No surface to draw: {failed}</p>
  {:else if !got}
    <button type="button" class="draw" class:round={show !== "map"} style:width="{show === 'map' ? size * 1.5 : size}px" style:height="{show === 'map' ? size * 0.75 : size}px" disabled={busy} onclick={build} data-draw-surface>
      {busy ? `Building the surface of ${label}…` : `Draw ${label}'s surface`}<span class="muted">{busy ? "" : "about a second"}</span>
    </button>
  {:else if !has}
    <p class="note" data-no-surface>No surface for {label} yet: only the GM's own pictures.</p>
  {:else if show === "map"}
    <div role="img" aria-label={`Map of ${label}, its whole surface laid flat, north up`}><canvas class="map" bind:this={mapCanvas} style:width="{size * 1.5}px" style:height="{size * 0.75}px"></canvas></div>
  {:else}
    <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions (a turnable globe: keys and drag, named by its label) -->
    <div
      class="globe"
      bind:this={host}
      role="application"
      aria-roledescription="globe"
      tabindex="0"
      aria-label={`Globe of ${label}. Drag or use the arrow keys to turn it.`}
      style:width="{size}px"
      style:height="{size}px"
      onpointerdown={down}
      onpointermove={move}
      onpointerup={() => (drag = null)}
      onpointercancel={() => (drag = null)}
      onkeydown={key}
    ></div>
  {/if}
</div>

<style>
  .drawn { display: flex; justify-content: center; max-width: 100%; }
  .map { max-width: 100%; height: auto !important; aspect-ratio: 2 / 1; image-rendering: auto; border: 1px solid var(--sse-rule); }
  .globe { max-width: 100%; touch-action: none; cursor: grab; }
  .globe :global(canvas) { display: block; max-width: 100%; }
  .draw { font: inherit; font-size: 13px; max-width: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 4px; border: 1px dashed var(--sse-rule); background: transparent; color: var(--sse-ink); cursor: pointer; }
  .draw.round { border-radius: 50%; }
  .draw[data-draw-surface]:disabled { cursor: progress; }
  .muted, .note { color: var(--sse-ink-muted); font-size: 12px; }
  .note { margin: 0; }
</style>
