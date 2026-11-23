<!--
  The stars beyond (Y5 part 1, renderer thread): the night sky from this
  system, drawn from the kernel's sky_from_here (the stars that reach the
  limit) and sky_glow (everyone else's light, as the Milky Way's glow). It is
  drawn in galactic coordinates in its own panel: the system's own orientation
  in the galaxy is not published yet, so it does not sit behind the Window's
  planets until it is. Presentation only: nothing here changes the record.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { drawSky, narrateSky, pickStar, readSky, skyCamera, SKY_POINTS_MAX, type Sky, type SkyStar } from "../../../renderer/src/sky.ts";
  import { DEFAULT_VIEW, viewOptions, type Json, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { WINDOW_CONTROLS } from "./controls.ts";

  let { ctx, scene }: { ctx: ViewContext; scene: SystemScene } = $props();

  // The kernel's sky_from_here walk is slow toward the naked-eye limit for now; magnitude 5 keeps a first look quick.
  const LIMIT = 5;
  const NSIDE = 16;
  // The sky's bands, read once: the view controls' Band (one set of controls) picks which is drawn.
  const BANDS = [
    { id: "V", label: "Visible" },
    { id: "Ks", label: "Near infrared" },
  ] as const;
  const band = $derived(viewOptions(WINDOW_CONTROLS, ctx.workspace.view ?? DEFAULT_VIEW).shown.band === "infrared" ? "Ks" : "V");
  let open = $state(false);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let from = $state<string>("");
  let sky = $state<Sky | null>(null);
  const cache = new Map<string, Sky>();
  let l0 = $state(0);
  let b0 = $state(0);
  let field = $state(120);
  let picked = $state<SkyStar | null>(null);
  let leftOut = $state(0);
  let drawn: ReturnType<typeof drawSky>["drawn"] = [];
  let canvas = $state<HTMLCanvasElement | null>(null);
  let gen = 0;

  // Stars are chosen once, in V; switching band redraws the same stars and the glow in that band (no new reads).
  async function load() {
    const key = scene.system;
    const hit = cache.get(key);
    if (hit) return void (sky = hit);
    const g = ++gen;
    loading = true;
    error = null;
    const ask = { limit_mag: LIMIT, band: "V", bands: BANDS.map((b) => b.id) };
    // From this system's own place when the kernel knows it; otherwise from the Sun's.
    let st = await ctx.session.skyFromHere({ ...ask, observer: scene.system });
    let place = "this system";
    if (!st.ok) ((st = await ctx.session.skyFromHere(ask)), (place = "the Sun's place (this system has no place in the galaxy yet)"));
    if (g !== gen) return;
    if (!st.ok) return void ((loading = false), (error = st.message));
    const xyz = (st.sky as { observer?: { xyz_pc?: [number, number, number] } }).observer?.xyz_pc;
    const gl = await ctx.session.skyGlow({ ...ask, nside: NSIDE, ...(xyz ? { position_pc: xyz } : {}) });
    if (g !== gen) return;
    loading = false;
    try {
      const s = readSky(st.sky as never, gl.ok ? (gl.glow as never) : null);
      cache.set(key, s);
      sky = s;
      from = place;
      if (!gl.ok) error = `The glow is missing: ${gl.message}`;
    } catch (e) {
      error = String(e);
    }
  }

  $effect(() => {
    if (open) untrack(() => void load());
  });

  $effect(() => {
    const cv = canvas;
    const s = sky;
    const bd = band;
    const c = skyCamera(l0, b0, field, cv?.width ?? 0);
    if (!cv || !s) return;
    const g = cv.getContext("2d");
    if (!g) return;
    const img = g.createImageData(cv.width, cv.height);
    const r = drawSky(img, s, c, { maxPoints: SKY_POINTS_MAX.canvas, glowStep: 3, band: bd });
    g.putImageData(img, 0, 0);
    drawn = r.drawn;
    leftOut = r.leftOut;
  });

  let drag: { x: number; y: number; l: number; b: number } | null = null;
  function down(e: PointerEvent) {
    drag = { x: e.clientX, y: e.clientY, l: l0, b: b0 };
    (e.currentTarget as Element).setPointerCapture?.(e.pointerId);
  }
  function move(e: PointerEvent) {
    if (!drag || !canvas) return;
    const k = field / canvas.getBoundingClientRect().width;
    l0 = (((drag.l + (e.clientX - drag.x) * k) % 360) + 360) % 360;
    b0 = Math.max(-89, Math.min(89, drag.b + (e.clientY - drag.y) * k));
  }
  function up(e: PointerEvent) {
    const d = drag;
    drag = null;
    if (!d || !canvas || !sky) return;
    if (Math.hypot(e.clientX - d.x, e.clientY - d.y) > 4) return;
    const box = canvas.getBoundingClientRect();
    const s = pickStar(sky, drawn, ((e.clientX - box.left) * canvas.width) / box.width, ((e.clientY - box.top) * canvas.height) / box.height);
    // Shown here, not made the workspace's selection: a background star is not one of this system's subjects.
    picked = s;
  }
  const words = $derived(sky ? narrateSky(sky, leftOut, band) : "");
  const fmt = (x: number, d = 1) => x.toLocaleString("en-GB", { maximumFractionDigits: d });
</script>

<section class="beyond" data-sky-beyond={sky ? "on" : loading ? "loading" : "off"} data-sky-band={band}>
  <h3>The stars beyond</h3>
  {#if !open}
    <button type="button" class="link" onclick={() => (open = true)}>Show the night sky from here</button>
  {:else}
    <div class="bar">
      <span class="muted" data-sky-band-words>{BANDS.find((b) => b.id === band)?.label} (Band, at the top)</span>
      <label>Field <input type="range" min="20" max="200" step="5" bind:value={field} /></label>
      <button type="button" class="link" onclick={() => ((l0 = 0), (b0 = 0))}>Toward the galaxy's centre</button>
    </div>
    <canvas bind:this={canvas} width="480" height="480" data-sky-canvas aria-label={words || "The night sky"} onpointerdown={down} onpointermove={move} onpointerup={up}></canvas>
    {#if loading}<p class="note">Reading the sky…</p>{/if}
    {#if error}<p class="note" data-sky-error>{error}</p>{/if}
    {#if sky}
      <p class="words" data-sky-words>{words}</p>
      <p class="note">
        {band === "V" ? "Actual colour." : `Mapped: ${band} light shown in grey, with the exposure a camera for that band would use.`} Seen from {from}, in galactic coordinates (the Milky Way's plane runs across the middle; the centre is at the middle when the view is reset). Stars to magnitude {LIMIT}; the rest of their light is the glow. Not yet lined up with the planets above: the kernel does not yet say how this system is turned in the galaxy.
      </p>
    {/if}
    {#if picked}
      <p class="picked" data-sky-picked={picked.address}>
        Picked: a star of magnitude {fmt(picked.mags[band] ?? picked.mag)} in {band}, {fmt(picked.light_time_yr, 0)} light years away, about {Math.round(picked.teff_k).toLocaleString("en-GB")} K and {fmt(picked.luminosity_lsun, picked.luminosity_lsun < 10 ? 2 : 0)} times the Sun's light{picked.double ? ", a double" : ""}; at galactic l {fmt(picked.l_deg)}°, b {fmt(picked.b_deg)}°.
      </p>
    {/if}
  {/if}
</section>

<style>
  .beyond {
    margin-top: 1.5rem;
  }
  h3 {
    font: inherit;
    font-weight: 600;
    margin: 0 0 0.5rem;
  }
  .bar {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    align-items: center;
    margin-bottom: 0.5rem;
  }
  .muted {
    color: var(--sse-ink-muted);
  }
  canvas {
    display: block;
    width: 100%;
    max-width: 480px;
    aspect-ratio: 1;
    border-radius: 4px;
    touch-action: none;
    cursor: grab;
    /* The sky paints every pixel itself; no backdrop colour here. */
  }
  .note,
  .words,
  .picked {
    max-width: 60ch;
    font-size: 0.9em;
  }
  .note {
    color: var(--sse-ink-muted);
  }
  .link {
    background: none;
    border: 0;
    padding: 0;
    color: var(--sse-accent, currentColor);
    cursor: pointer;
    text-decoration: underline;
  }
</style>
