<!--
  Black holes in True (XO4; exotic objects design 3.1 to 3.4 and 7). The kernel's three holes of design 3.4 (a 10 Sun
  hole fed by a companion, a quasar, the Milky Way's starved centre) drawn by the renderer's light-path stage: the
  shadow, the thin bright ring, the far side of the disc bent up over the top, the side turning towards the viewer
  brighter and bluer, the stars behind pushed out and smeared into rings. Every number is the kernel's
  (`read_light_paths`: the bend table, the disc's edges and feed; `bake`: the disc's shift and glow as a table); the picture reads them per pixel
  and the note says how far it was dimmed. Seen from tips the disc; drag turns it freely, the wheel narrows the lens.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { COMPOSITE, DEFAULT_VIEW, LAYER_COLOURS, sayNumber, viewOptions, type Json, type SystemScene, type Table, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { Card } from "../../../chart-grammar/src/index.ts";
  import { B_CRIT, PICTURE_BANDS, brightest, discOf, discSubject, marksOf, type BendTable, type HoleView, type LightPathsRead, type PictureBand } from "../../../renderer/src/light-path.ts";
  import { galactic, holeSky, type HoleSkyStar } from "../../../renderer/src/hole-sky.ts";
  import type { HoleLayer, HolePicture, HoleSky } from "../../../renderer/src/gpu-light-path.ts";
  import { BLACK_HOLE_CONTROLS } from "./controls.ts";
  import { BACKDROPS, bandShares, GREY, isNatural, layersFor, sci, SEEN_TILT_DEG, tiltWords, whiteLog } from "./picture.ts";

  let { ctx, scene }: { ctx: ViewContext; scene: SystemScene } = $props();

  const ws = $derived(ctx.workspace);
  const opts = $derived(viewOptions(BLACK_HOLE_CONTROLS, ws.view ?? DEFAULT_VIEW));
  const shown = $derived(opts.shown);
  const pickedBand = $derived(ws.view?.band ?? "visible");
  const hud = $derived(shown.hud);
  const long = $derived(shown.exposure !== "eye");

  let paths = $state.raw<LightPathsRead[]>([]);
  let table = $state.raw<BendTable | null>(null);
  let problem = $state("");
  let which = $state(0);
  const hole = $derived(paths[which] ?? null);
  // The disc's shift and glow, baked by the kernel from the physics in force (ER11a, table `sse.light.disc`), sized
  // by this device's tier; the picture only samples it.
  let baked = $state.raw<{ address: string; table: Table } | null>(null);
  const disc = $derived(hole && baked && baked.address === hole.address ? discOf(baked.table) : null);
  $effect(() => {
    const h = hole;
    const subject = h ? discSubject(h) : null;
    if (!h || !subject) return;
    const tier = untrack(() => ctx.styles?.tier());
    void ctx.session.bake({ table: "sse.light.disc", subject: subject as unknown as Json, ...(tier ? { tier } : {}) }).then((r) => {
      if (!r.ok) return void (problem = r.message);
      baked = { address: h.address, table: r.table };
    });
  });

  $effect(() => {
    untrack(() =>
      void ctx.session.readLightPaths({ fixture: "black-holes", table: true }).then((r) => {
        if (!r.ok) return void (problem = r.message);
        paths = r.paths as unknown as LightPathsRead[];
        table = (r.bend_table as unknown as BendTable) ?? null;
      }),
    );
  });

  // The viewer's tilt from the disc's axis: the hole's own viewer to start, then Seen from, then the hands.
  let tiltDeg = $state(80);
  let started = "";
  $effect(() => {
    const h = hole;
    if (!h || started === h.address) return;
    started = h.address;
    tiltDeg = (h.viewer.tilt_to_disc_rad * 180) / Math.PI;
    half = HALF0;
  });
  let framed = -1;
  $effect(() => {
    const f = ws.view?.framed ?? 0;
    const s = shown.seen;
    if (f === framed) return;
    const first = framed === -1;
    framed = f;
    if (first) return;
    const t = SEEN_TILT_DEG[s];
    if (t !== undefined) tiltDeg = t;
    half = HALF0;
  });
  // Drawn as far viewers see it (the kernel's far-viewer rule), from 2,000 r_g, the picture ±HALF0 r_g tall.
  const DISTANCE = 2000;
  const HALF0 = 16;
  let half = $state(HALF0);
  const turning = $derived<1 | -1>(tiltDeg <= 90 ? 1 : -1);
  const view = $derived<HoleView>({ distance: DISTANCE, tilt: ((tiltDeg <= 90 ? tiltDeg : 180 - tiltDeg) * Math.PI) / 180, half, turning });

  // Each band's brightest at this tilt (the CPU trace of the same rays), and the disc's share of each band's light.
  const bright = $derived(table && disc ? brightest(table, disc, view.tilt, turning) : null);
  const shares = $derived(hole ? bandShares(hole) : null);
  const layers = $derived<HoleLayer[]>(layersFor(pickedBand === COMPOSITE ? opts.layers : [{ band: shown.band, colour: shown.band === "visible" ? "natural" : shown.band === "radio" ? LAYER_COLOURS.radio : GREY }], bright, shares));
  const mapped = $derived(layers.some((l) => !isNatural(l)));

  // The sky behind: the kernel's sky from this system's place (the Sun's when it has none), to the same depth as the
  // system's True (VP2). Read once; rays near the shadow are turned by tens of degrees, so the whole sky matters, and the
  // stars round the hole's direction are laid finer.
  const SKY_LIMIT = 6.5;
  let stars = $state.raw<{ list: HoleSkyStar[]; place: string } | null>(null);
  let skyWhy = $state("");
  $effect(() => {
    const sys = scene.system;
    untrack(() =>
      void (async () => {
        const ask = { limit_mag: SKY_LIMIT, band: "V", bands: ["B", "V"] };
        let r = await ctx.session.skyFromHere({ ...ask, observer: sys });
        let place = "this system's place";
        if (!r.ok) ((r = await ctx.session.skyFromHere(ask)), (place = "the Sun's place"));
        if (!r.ok) return void (skyWhy = `The stars behind could not be read: ${r.message}`);
        const list = ((r.sky as { stars?: { dir: number[]; luminosity_lsun: number; distance_pc: number; teff_k?: number; components?: { teff_k: number }[] }[] }).stars ?? []).map(
          (x): HoleSkyStar => ({ dir: [x.dir[0]!, x.dir[1]!, x.dir[2]!], luminosity_lsun: x.luminosity_lsun, distance_pc: x.distance_pc, teff_k: x.teff_k ?? x.components?.[0]?.teff_k ?? 5800 }),
        );
        stars = { list, place };
      })(),
    );
  });
  const sky = $derived.by(() => {
    const h = hole;
    const st = stars;
    const back = h ? BACKDROPS[h.address] : undefined;
    if (!h || !st || !back) return null;
    return holeSky(`${h.address}|${st.place}`, galactic(back.l_deg, back.b_deg), st.list, st.list, { size: 1024, half: Math.tan((back.cone_deg * Math.PI) / 180), wholeW: 1024 });
  });
  $effect(() => {
    const h = hole;
    const back = h ? BACKDROPS[h.address] : undefined;
    if (back && stars) skyWhy = `${back.why} They are the kernel's stars to magnitude ${SKY_LIMIT}, seen from ${stars.place}.`;
  });

  // The picture.
  let host = $state<HTMLDivElement | null>(null);
  let canvas = $state<HTMLCanvasElement | null>(null);
  let pic = $state.raw<HolePicture | null>(null);
  let noGpu = $state(false);
  let cw = $state(640);
  let ch = $state(480);
  $effect(() => {
    const cv = canvas;
    if (!cv) return;
    let live = true;
    let made: HolePicture | null = null;
    void import("../../../renderer/src/gpu-light-path.ts").then(async (m) => {
      made = await m.startHolePicture(cv);
      if (!live) return made?.dispose();
      if (!made) noGpu = true;
      pic = made;
    });
    return () => {
      live = false;
      made?.dispose();
      pic = null;
    };
  });
  // The stars' light as the picture's: at a long exposure lifted until the brightest few fill their texel (VP2's rule,
  // a composite); for the eye, as bright as they really are against the disc's exposure (beside a fed disc, nothing).
  const skyGainFor = () => {
    const nat = layers.find(isNatural);
    if (!sky || !nat) return 0;
    return long ? sky.gain : 10 ** (whiteLog() - nat.top);
  };
  // The time to submit a frame, kept on the canvas for the bench and tests (not shown: a changing line would move the
  // layout, resize the canvas and start the drawing again).
  let drawMs = 0;
  let orders = $state<1 | 2>(2);
  // Drawn when something changes, and again on the next frames for a moment: the first frames after a change of
  // shader can come out black while the device compiles it, and nothing moves on its own here (no disc texture yet).
  $effect(() => {
    const p = pic;
    if (!p || !table || !hole) return;
    // A fed hole waits for its disc table (baked by the kernel), so no frame is drawn without its disc.
    if (hole.feed && !disc) return;
    const dpr = Math.min(2, globalThis.devicePixelRatio || 1);
    const frame = { table, disc, view, layers, sky: sky?.sky ?? null, skyGain: skyGainFor(), orders, width: Math.round(cw * dpr), height: Math.round(ch * dpr) };
    const said = `${hole.address}|${Math.round(view.tilt * 1e4)}|${layers.map((l) => l.band).join("+")}|${sky ? "sky" : "nosky"}`;
    let left = 30;
    let raf = 0;
    const tick = () => {
      const t0 = performance.now();
      p.draw(frame);
      drawMs = performance.now() - t0;
      if (canvas) canvas.dataset.ms = String(Math.round(drawMs));
      if (--left > 0) raf = requestAnimationFrame(tick);
      else if (canvas) {
        // Where the shadow's edge and the arch should be, in CSS pixels up from the middle: what a frame is measured against.
        const m = table && hole ? marksOf(table, disc, view) : null;
        if (m) ((canvas.dataset.shadowPx = String((m.shadow * ch) / 2)), (canvas.dataset.archPx = m.arch === null ? "" : String((m.arch * ch) / 2)));
        canvas.dataset.drawn = said;
      }
    };
    if (canvas) delete canvas.dataset.drawn;
    tick();
    return () => cancelAnimationFrame(raf);
  });

  // The hands: drag up and down tips the disc, the wheel narrows or widens the lens.
  let drag: { y: number; t: number } | null = null;
  const down = (e: PointerEvent) => {
    drag = { y: e.clientY, t: tiltDeg };
    (e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId);
  };
  const move = (e: PointerEvent) => {
    if (!drag) return;
    tiltDeg = Math.min(179, Math.max(1, drag.t + (e.clientY - drag.y) * 0.3));
  };
  const up = () => (drag = null);
  const wheel = (e: WheelEvent) => {
    e.preventDefault();
    half = Math.min(200, Math.max(7, half * Math.exp(e.deltaY * 0.001)));
  };

  // HUD: the shadow's edge and the disc's inner edge as rings, at the sizes the kernel gives.
  const marks = $derived(table && hole ? marksOf(table, disc, view) : null);
  const px = (y: number) => (y * ch) / 2;
  const isco = $derived(hole ? hole.last_stable_orbit_m / hole.gravitational_radius_m : 6);

  const sideRatio = $derived(hole?.image?.side_ratio ?? null);
  const dimmed = $derived(bright?.times_sun_surface ?? null);
  const bandWord: Record<PictureBand, string> = { visible: "visible light", ultraviolet: "ultraviolet", xray: "X-rays", infrared: "infrared", radio: "radio" };
  const hidden = $derived(shares ? layers.filter((l) => (shares[l.band] ?? 0) < 0.01).map((l) => l.band) : []);
</script>

<div class="bh" bind:this={host} data-black-hole={hole?.address ?? ""}>
  {#if problem}
    <p class="problem">The kernel could not give the black holes: {problem}</p>
  {:else if !hole}
    <p class="quiet">Reading the black holes from the kernel…</p>
  {:else}
    <div class="pick" role="group" aria-label="Which black hole">
      {#each paths as p, i (p.address)}
        <button class:on={i === which} aria-pressed={i === which} data-hole-pick={p.address} onclick={() => (which = i)}>{p.name ?? p.address}</button>
      {/each}
    </div>
    <div class="frame" bind:clientWidth={cw} bind:clientHeight={ch}>
      <canvas
        bind:this={canvas}
        data-hole-canvas
        aria-label="{hole.name}: drawn from the bending of light, seen {tiltWords(tiltDeg)}"
        onpointerdown={down}
        onpointermove={move}
        onpointerup={up}
        onpointercancel={up}
        onwheel={wheel}
      ></canvas>
      {#if hud && marks}
        <svg class="hud" viewBox="{-cw / 2} {-ch / 2} {cw} {ch}" aria-hidden="true">
          <circle r={px(marks.shadow)} class="ring" />
          <text x={px(marks.shadow) * 0.72 + 4} y={-px(marks.shadow) * 0.72 - 4}>shadow, √27 r_g</text>
          {#if marks.arch !== null}
            <line x1="-8" x2="8" y1={-px(marks.arch)} y2={-px(marks.arch)} class="tick" />
            <text x="12" y={-px(marks.arch) + 4}>far side's inner edge ({sayNumber(isco, 2)} r_g), bent over the top</text>
          {/if}
        </svg>
      {/if}
      {#if noGpu}
        <p class="nogpu">This device has no GPU for True; the words below say what the picture would show.</p>
      {/if}
      <div class="chip" data-hole-chip>
        {#if mapped}<span>mapped: {layers.map((l) => (isNatural(l) ? "visible actual" : bandWord[l.band])).join(" · ")}</span>{/if}
        {#if layers.some(isNatural) && dimmed !== null}
          <span data-hole-dimmed>{dimmed >= 1 ? `dimmed ${sci(dimmed)} times the Sun's surface brightness` : "not dimmed"}</span>
        {/if}
        {#if sky && layers.some(isNatural)}<span>{long ? "stars at a long exposure" : disc ? "stars as the eye sees them: too faint beside the disc" : "stars as the eye sees them"}</span>{/if}
        {#if orders === 1}<span>thin ring dropped to hold the frame rate</span>{/if}
        {#if (hole.spin?.a ?? 0) > 0}<span>bending drawn for no spin</span>{/if}
      </div>
    </div>
    <Card title={hole.name ?? hole.address} subtitle={hole.about ?? ""}>
      <div class="words" data-hole-words>
        <p>
          Seen {tiltWords(tiltDeg)}. The shadow's edge is {sayNumber(B_CRIT, 3)} gravitational radii from its middle, so it is {sci(hole.gravitational_radius_m * B_CRIT * 2)} m across;
          the disc starts at {sayNumber(isco, 2)} gravitational radii, set by its {hole.feed?.disc.inner_set_by?.replaceAll("_", " ") ?? "inner edge"}.
        </p>
        {#if hole.feed}
          <p>It takes in {sci(hole.feed.rate_kg_s)} kg each second, {sayNumber(hole.feed.eddington_share * 100, 2)}% of the rate at which its own light would push the gas back out.</p>
        {:else}
          <p>Nothing falls onto it: a bare black hole shows only by bending the stars behind it.</p>
        {/if}
        {#if hole.image && disc}
          <p>
            Brightest at {sayNumber(hole.image.brightest.r_rg, 2)} gravitational radii, glowing at {sci(hole.image.brightest.t_emitted_k)} K and seen at {sci(hole.image.brightest.t_seen_k)} K;
            {#if sideRatio !== null} from the kernel's own viewer the side turning towards it is {sayNumber(sideRatio, 2)} times as bright as the side turning away.{/if}
            Its light peaks in {hole.light?.peak_band === "xray" ? "X-rays" : hole.light?.peak_band === "uv" ? "the ultraviolet" : (hole.light?.peak_band ?? "?")}.
          </p>
          {#if dimmed !== null}<p data-hole-note>{dimmed >= 1 ? `In visible light its brightest point is ${sci(dimmed)} times as bright as the Sun's surface: far too bright to look at, so the picture is dimmed until that point reads white.` : `In visible light its brightest point is ${sci(dimmed)} of the Sun's surface brightness, so it is drawn almost dark.`}</p>{/if}
        {/if}
        {#each hidden as b (b)}
          <p class="quiet">In {bandWord[b]} the drawn disc carries {sayNumber((shares?.[b] ?? 0) * 100, 2)}% of this body's light; the rest is the hot flow round it, which the kernel gives as a total only, so it is not drawn yet.</p>
        {/each}
        {#if skyWhy}<p class="quiet">{skyWhy}</p>{/if}
        <details>
          <summary>How it is drawn, and what is left out</summary>
          <p>Each pixel's ray is followed back through the kernel's bend table (the bending of light round a mass that does not spin): where it first crosses the disc inside its edges, that piece of disc is what shows, glowing at its own temperature and shifted by its motion and the climb out (Luminet 1979); a ray that escapes looks up the sky it came from, turned by the table; a ray that is swallowed is the shadow. Visible light is natural colour; other bands are mapped.</p>
          <ul>
            {#each [...new Set([...((hole.marks ?? []) as string[]), ...(hole.feed?.stand_ins ?? [])])] as m (m)}<li>{m}</li>{/each}
            <li>The camera is drawn as a far viewer (the kernel's rule), {DISTANCE} gravitational radii out; turning the view tips the disc, the sky behind stays the one in the hole's direction.</li>
          </ul>
        </details>
      </div>
    </Card>
  {/if}
</div>

<style>
  .bh {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    height: 100%;
    min-height: 0;
  }
  .pick {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
  }
  .pick button {
    font: inherit;
    font-size: 0.85rem;
    padding: 0.25rem 0.6rem;
    border: 1px solid var(--sse-rule);
    border-radius: 999px;
    background: transparent;
    color: inherit;
    cursor: pointer;
  }
  .pick button.on {
    background: var(--sse-ink);
    color: var(--sse-ground);
  }
  .frame {
    position: relative;
    flex: 1 1 60%;
    min-height: 260px;
    background: var(--sse-chart-ground);
    border-radius: 4px;
    overflow: hidden;
  }
  canvas {
    width: 100%;
    height: 100%;
    display: block;
    touch-action: none;
    cursor: grab;
  }
  .hud {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }
  .hud .ring {
    fill: none;
    stroke: var(--sse-accent);
    opacity: 0.7;
    stroke-dasharray: 3 4;
  }
  .hud .tick {
    stroke: var(--sse-accent);
  }
  .hud text {
    fill: var(--sse-accent);
    font-size: 11px;
  }
  .chip {
    position: absolute;
    left: 0.5rem;
    bottom: 0.5rem;
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    font-size: 0.75rem;
  }
  .chip span {
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    opacity: 0.9;
    padding: 0.1rem 0.45rem;
    border-radius: 999px;
  }
  .nogpu {
    position: absolute;
    inset: auto 1rem 50% 1rem;
    color: var(--sse-ink-muted);
    text-align: center;
  }
  .words p {
    margin: 0.3rem 0;
  }
  .quiet {
    opacity: 0.75;
  }
  .problem {
    color: var(--sse-alert);
  }
</style>
