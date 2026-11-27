<!--
  One star's life (SC1 follow-up, Alex 26 Nov): a starting mass's stages in order, as the life line gives them, the star
  drawn live in True at each age with Earth's orbit at 1 AU to the same scale. Play runs the whole life, a stage every
  few seconds, the star swelling and shrinking as the life line says; the strip below jumps to a stage. The scale is
  the whole life's (its largest size, or Earth's orbit if larger) or Earth's orbit, so a giant can be watched swallowing it.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { roundTo, sayNumber, type Depth } from "../../../ui-contract/src/index.ts";
  import { circleSteps, startGpu, type Gpu } from "../../../renderer/src/index.ts";
  import { AU, frameAt, radiusAt, readLife, sceneScale, type LifeStage, type StarLife } from "./star-life.ts";
  import { drawStage } from "./star-stage.ts";
  import { span } from "./star-gallery.ts";

  let {
    session,
    mass = 1,
    depth = "science",
  }: {
    session: Parameters<typeof readLife>[0];
    /** The starting mass to show first, solar masses (a gallery star followed here). */
    mass?: number;
    depth?: Depth;
  } = $props();
  const full = $derived(depth !== "curious");

  // The mass, on a log slider between the life line's own walls (the lightest that burns deuterium, the heaviest it fits).
  let walls = $state({ low: 0.013, high: 150 });
  let massNow = $state(untrack(() => mass));
  let logMass = $state(untrack(() => Math.log10(mass)));
  $effect(() => {
    if (!session.describeStar) return;
    void session.describeStar.bind(session)(1, 1e6).then((r) => {
      if (r.ok) walls = { low: r.star.walls.deuterium_msun, high: r.star.walls.fits_msun };
    });
  });
  $effect(() => {
    const m = mass;
    untrack(() => ((massNow = m), (logMass = Math.log10(m))));
  });
  const shownMass = (lg: number) => roundTo(10 ** lg, 2);

  // The life, read stage by stage (each one shown as it comes).
  let life = $state<StarLife | null>(null);
  let stagesSoFar = $state<LifeStage[]>([]);
  let problem = $state<string | null>(null);
  let readFor = 0;
  $effect(() => {
    const m = massNow;
    const ask = ++readFor;
    untrack(() => {
      life = null;
      stagesSoFar = [];
      problem = null;
      play = 0;
      playing = false;
    });
    void readLife(session, m, undefined, (s, i) => {
      if (ask === readFor) stagesSoFar = [...stagesSoFar.slice(0, i), s];
    }).then((l) => {
      if (ask !== readFor) return;
      if (typeof l === "string") problem = l;
      // Plays once the whole life is read (drawing it while still reading would only slow the reading).
      else ((life = l), (playing = !calm));
    });
  });
  const stages = $derived(life?.stages ?? stagesSoFar);

  // The playhead: stage index plus how far through it (0..stages). A stage every few seconds of the screen's time.
  const STAGE_ON_SCREEN_S = 4;
  const calm = typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
  let play = $state(0);
  let playing = $state(false);
  const at = $derived(stages.length ? frameAt({ stages }, play) : null);
  const stage = $derived(at ? stages[at.stage]! : null);
  const frame = $derived(at && stage ? stage.frames[at.frame]! : null);

  // The scale: the whole life, or Earth's orbit.
  type Fit = "life" | "earth" | "star";
  let fit = $state<Fit>("life");
  const SCENE_H = 360;
  let sceneW = $state(560);
  const fitPx = $derived(Math.min(sceneW, SCENE_H) * 0.44);
  const radiusM = $derived(stages.length ? radiusAt({ stages }, play) : 0);
  // "The star": the star as it is now fills the view, so its face shows, and Earth's orbit is wherever that puts it.
  const scale = $derived(fit === "star" && radiusM > 0 ? fitPx / radiusM : stages.length ? sceneScale({ stages }, fitPx, fit === "star" ? "life" : fit) : fitPx / AU);
  const radiusPx = $derived(radiusM * scale);
  const SMALLEST_STAR_PX = 1.5;
  const inside = $derived(radiusM > AU);

  // ---- The scene: one GPU canvas ----
  let canvas: HTMLCanvasElement | null = $state(null);
  let host: HTMLElement | null = $state(null);
  let gpu: Gpu | null = null;
  let gpuState = $state<"starting" | "ready" | "none">("starting");
  $effect(() => {
    const c = canvas;
    if (!c) return;
    let gone = false;
    void startGpu(c).then((g) => {
      if (gone) return g?.dispose();
      if (!g) return void (gpuState = "none");
      gpu = g;
      gpuState = "ready";
    });
    return () => {
      gone = true;
      gpu?.dispose();
      gpu = null;
      gpuState = "starting";
    };
  });
  $effect(() => {
    if (!host) return;
    const h = host;
    const ro = new ResizeObserver(() => (sceneW = Math.max(200, h.clientWidth)));
    ro.observe(h);
    return () => ro.disconnect();
  });
  $effect(() => {
    if (gpuState !== "ready" || !canvas) return;
    const cv = canvas;
    let raf = 0;
    const t0 = performance.now();
    let last = t0;
    let drawn = "";
    const draw = (now: number) => {
      raf = requestAnimationFrame(draw);
      const dt = (now - last) / 1000;
      last = now;
      const n = stages.length;
      if (playing && n) {
        // The last stage lasts: the playhead stops at its end.
        const next = play + dt / STAGE_ON_SCREEN_S;
        if (next >= n) ((play = n - 1e-6), (playing = false));
        else play = next;
      }
      if (!gpu || document.hidden) return;
      const dpr = Math.min(2, typeof devicePixelRatio === "number" ? devicePixelRatio : 1);
      const w = sceneW;
      const h = SCENE_H;
      // Drawn again only when something shows a change: playing, the face's tenth-of-a-second tick, the scale or size.
      const t_s = (now - t0) / 1000;
      const sig = `${playing ? play : ""}|${calm ? 0 : Math.floor(t_s * 10)}|${play}|${scale}|${w}x${h}@${dpr}|${frame?.age_yr ?? ""}`;
      if (sig === drawn) return;
      drawn = sig;
      if (cv.width !== Math.round(w * dpr) || cv.height !== Math.round(h * dpr)) ((cv.width = Math.round(w * dpr)), (cv.height = Math.round(h * dpr)));
      const cx = w / 2;
      const cy = h / 2;
      const orbit = AU * scale;
      const steps = circleSteps(orbit);
      // Closed: the last point is the first again.
      const points = Array.from({ length: steps + 1 }, (_, i) => ({ x: cx + orbit * Math.cos((2 * Math.PI * i) / steps), y: cy + orbit * Math.sin((2 * Math.PI * i) / steps) }));
      const f = frame;
      const r = Math.max(SMALLEST_STAR_PX, radiusPx);
      drawStage(gpu, {
        width: w,
        height: h,
        dpr,
        ground: "black",
        t_s,
        calm,
        stars: f?.live ? [{ key: "star-life", live: f.live, x: cx, y: cy, r, clock: f.clock }] : [],
        // Earth: a dot on its orbit, hidden by the star once the star is larger than its orbit.
        bodies: inside ? [] : [{ key: "star-life|earth", x: cx + orbit, y: cy, r: 3, colour: "lightsteelblue" }],
        lines: orbit > 2 && orbit < 4 * Math.max(w, h) ? [{ points, colour: "gray", over: inside }] : [],
      });
    };
    raf = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(raf);
  });

  const years = (yr: number) => (yr >= 1e9 ? `${sayNumber(yr / 1e9, 3)} billion years` : yr >= 1e6 ? `${sayNumber(yr / 1e6, 3)} million years` : `${sayNumber(yr, 3)} years`);
  const lasts = (s: LifeStage) => (s.to_yr === null ? "lasts" : years(s.to_yr - s.from_yr));
  const scaleWords = $derived(
    radiusPx < 1
      ? `At this scale the star is smaller than a pixel (its radius is ${sayNumber(radiusM / AU, 2)} AU), so it is drawn as a point: choose "The star" to see its face.`
      : inside
        ? `Its radius is ${sayNumber(radiusM / AU, 3)} AU, larger than Earth's orbit: Earth's orbit (the grey line) is inside the star.`
        : `Its radius is ${sayNumber(radiusM / AU, 3)} AU; Earth's orbit is 1 AU.`,
  );
</script>

<section class="life" aria-label="One star's life" data-star-life data-gpu={gpuState} data-stages={stages.length} data-read={life ? "done" : problem ? "failed" : "reading"}>
  <div class="bar">
    <label class="mass">Born with
      <input type="range" min={Math.log10(walls.low)} max={Math.log10(walls.high)} step="0.01" bind:value={logMass} onchange={() => (massNow = shownMass(logMass))} aria-valuetext="{sayNumber(10 ** logMass, 2)} times the Sun's mass" data-life-mass />
      <span>{sayNumber(10 ** logMass, 2)} × the Sun's mass</span>
    </label>
    <div class="chips" role="group" aria-label="Scale">
      <button type="button" class="chip" aria-pressed={fit === "life"} data-fit="life" onclick={() => (fit = "life")} title="The scale of the star at its largest, or Earth's orbit if that is larger">Whole life</button>
      <button type="button" class="chip" aria-pressed={fit === "earth"} data-fit="earth" onclick={() => (fit = "earth")} title="Earth's orbit fills the view">Earth's orbit</button>
      <button type="button" class="chip" aria-pressed={fit === "star"} data-fit="star" onclick={() => (fit = "star")} title="The star as it is now fills the view">The star</button>
    </div>
  </div>

  {#if problem}<p class="note">The life line cannot give this star a life: {problem}.</p>{/if}
  {#if gpuState === "none"}<p class="note">This device has no GPU to draw True with, so the star is not drawn; its life is still read.</p>{/if}

  <div class="scene" bind:this={host}>
    <canvas bind:this={canvas} style:width="{sceneW}px" style:height="{SCENE_H}px" aria-label="The star drawn live in True with Earth's orbit to the same scale"></canvas>
    {#if stage && !frame?.live}<p class="over not small" data-not-drawn>{stage.why ? `Not drawn yet: ${stage.why}` : "Nothing to draw"}</p>{/if}
    {#if !stages.length && !problem}<p class="over muted small">Reading its life from the life line…</p>{/if}
  </div>

  <div class="play">
    <button type="button" class="chip" onclick={() => (play >= stages.length - 1e-3 ? ((play = 0), (playing = true)) : (playing = !playing))} disabled={!stages.length} data-life-play aria-pressed={playing}>{playing ? "Pause" : "Play its life"}</button>
    <input type="range" min="0" max={Math.max(0, stages.length - 1e-6)} step="0.001" bind:value={play} oninput={() => (playing = false)} aria-label="Through its life" disabled={!stages.length} />
  </div>

  {#if frame && stage}
    <p data-life-now>
      <strong>{stage.label}</strong>, {years(frame.age_yr)} after its birth: {sayNumber(frame.now.teff_k, 3)} K, {sayNumber(frame.now.radius_rsun, 3)} times the Sun's radius, {sayNumber(frame.now.luminosity_lsun, 3)} times its light{full ? `, ${sayNumber(frame.now.mass_msun, 3)} times its mass` : ""}.
    </p>
    <p class="muted small" data-life-scale>{scaleWords} {AU * scale > Math.max(sceneW, SCENE_H) ? "Earth's orbit, 1 AU out, is beyond the edge of the view at this scale." : "Earth is the blue dot on the grey line, 1 AU out."}</p>
  {/if}

  <ol class="strip" aria-label="Its stages">
    {#each stages as s, i (i)}
      <li>
        <button type="button" class="stage" aria-pressed={at?.stage === i} data-life-stage={s.phase} onclick={() => ((play = i), (playing = false))}>
          <span class="sname">{s.label}</span>
          <span class="muted small">{s.to_yr === null ? `from ${years(s.from_yr)}` : lasts(s)}</span>
          {#if s.why}<span class="not small">Not drawn yet</span>{/if}
        </button>
      </li>
    {/each}
  </ol>
  {#if full && life?.notes.length}<p class="muted small">{life.notes.join(" ")}</p>{/if}
  {#if frame?.live}<p class="muted small">Drawn at its own clock{frame.clock > 1 ? `: about ${span(frame.clock)} of its time a second` : ""}, as it is at each age the life line was read at ({stages[0]?.frames.length ?? 0} ages a stage).</p>{/if}
</section>

<style>
  .life {
    display: grid;
    gap: 10px;
    min-width: 0;
  }
  .bar,
  .play {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .play input {
    flex: 1;
    min-width: 160px;
  }
  .mass {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  p {
    margin: 0;
    line-height: 1.5;
  }
  .muted {
    color: var(--sse-ink-muted);
  }
  .small {
    font-size: 13px;
  }
  .scene {
    position: relative;
    width: 100%;
  }
  .scene canvas {
    display: block;
    max-width: 100%;
    /* True's space: black whatever the theme, as every True view draws it. */
    background: black;
  }
  .over {
    position: absolute;
    left: 10px;
    bottom: 8px;
  }
  .not {
    color: gray;
    font-family: var(--sse-font-mono);
    font-size: 11px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .strip {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 6px;
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .stage {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
    height: 100%;
    padding: 6px 8px;
    border: var(--sse-line) solid var(--sse-rule);
    background: var(--sse-ground);
    color: var(--sse-ink);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .stage[aria-pressed="true"] {
    border-color: var(--sse-ink);
  }
  .sname {
    font-family: var(--sse-font-display);
    font-size: 14px;
  }
</style>
