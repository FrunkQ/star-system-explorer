<!--
  The size comparison stood up as globes (Alex, 24 Nov): the same lineup, the same places and sizes, each body a globe
  turning on its own axis, leaning by its own tilt, its own way round. 3D: shaded in its chart colour with a graticule
  to read the turn by. Wireframe: the graticule alone. True: lit from the side against black space, with the world's
  own surface where the kernel can read it (GM seat, this system's worlds) and its rings. Drawn straight on, so sizes
  are the lineup's whatever the distance. The labels, the limb and the picking stay with the lineup's own drawing.
-->
<script lang="ts" module>
  import type { LiveStar, TrueTexture } from "../../../renderer/src/index.ts";
  // Surfaces read once per world for the page's life: about 3 s each from the kernel.
  const kept = new Map<string, TrueTexture | string>();
  // This system's own stars as the kernel's activity reads them (spin, cells, spots, flares), once per page.
  const keptStars = new Map<string, LiveStar | string>();
  // A giant's winds, read with its face: the bands slide and the storms turn as the globe turns (VP-A1).
  const flows = new Map<string, AirFlow>();
</script>

<script lang="ts">
  import { untrack } from "svelte";
  import { bodySeconds, globeFrame, graticule, liveFace, nearSide, readLiveStar, readTrueFace, ringEdge, spinAt, startGpu, STAR_SECONDS_PER_SECOND, trueLook, type AirFlow, type FaceReader, type Gpu, type GpuItem, type GpuRing, type LineupView, type StarReader, type Vec3 } from "../../../renderer/src/index.ts";
  import { reachKm, type CompareItem, type Placed, type ViewContext } from "../../../ui-contract/src/index.ts";

  let {
    ctx,
    system,
    placed,
    width,
    height,
    base,
    look,
    view,
    pace,
    colour,
    own,
    given = () => ({}),
    onfail,
  }: {
    ctx: ViewContext;
    system: string;
    placed: Placed[];
    width: number;
    height: number;
    /** The baseline's height from the top, px: each globe sits on it. */
    base: number;
    look: "3d" | "true" | "wireframe";
    view: LineupView;
    pace: "same" | "true";
    /** The item's colour as the chart draws it (CSS). */
    colour: (i: CompareItem) => string;
    /** Whether the item is one of this system's own bodies (its surface can be read). */
    own: (i: CompareItem) => boolean;
    /** A star's temperature, mass and radius as the scene holds them, for a star the kernel has no facts for. */
    given?: (i: CompareItem) => { teff_k?: number; mass_sun?: number; radius_sun?: number };
    onfail: () => void;
  } = $props();

  let canvas = $state<HTMLCanvasElement | null>(null);
  let gpu: Gpu | null = null;
  let ready = $state(false);
  $effect(() => {
    const c = canvas;
    if (!c) return;
    let gone = false;
    void startGpu(c).then((g) => {
      if (gone) return g?.dispose();
      if (!g) return onfail();
      gpu = g;
      ready = true;
    });
    return () => {
      gone = true;
      gpu?.dispose();
      gpu = null;
      ready = false;
    };
  });

  // Colours as the GPU takes them: any CSS colour (a token, a var() with its fallback) as the browser works it out on
  // a probe beside the canvas, then flattened to sRGB bytes through a one-pixel canvas (a computed colour may be oklch).
  const paint = new Map<string, string>();
  function rgb(css: string): string {
    const hit = paint.get(css);
    if (hit) return hit;
    const host = canvas?.parentElement;
    if (!host) return "grey";
    const probe = document.createElement("span");
    probe.style.color = css;
    host.appendChild(probe);
    const shown = getComputedStyle(probe).color;
    probe.remove();
    const g = document.createElement("canvas").getContext("2d", { willReadFrequently: true });
    if (!g) return shown;
    g.fillStyle = shown;
    g.fillRect(0, 0, 1, 1);
    const [r, gr, b] = g.getImageData(0, 0, 1, 1).data;
    const out = `rgb(${r}, ${gr}, ${b})`;
    paint.set(css, out);
    return out;
  }
  const token = (name: string, fallback: string) => rgb(`var(${name}, var(${fallback}))`);

  // True: each of this system's worlds' own surface, read in the background on the GM's seat.
  let surfaces = $state(new Map<string, TrueTexture>());
  $effect(() => {
    if (look !== "true" || ctx.session.seat.authority !== "gm") return;
    const worlds = placed.map((p) => p.item).filter((i) => (i.kind === "planet" || i.kind === "moon") && own(i));
    untrack(() =>
      void (async () => {
        for (const i of worlds) {
          const key = `${system}|${i.key}`;
          if (!kept.has(key)) {
            // Ground, then today's clouds and haze; a giant is its cloud decks, with the winds that carry them (true-air.ts).
            const f = await readTrueFace(ctx.session as unknown as FaceReader, system, i.key);
            kept.set(key, typeof f === "string" ? "none" : f.tex);
            if (typeof f !== "string" && f.flow) flows.set(key, f.flow);
          }
          const t = kept.get(key);
          if (t && typeof t !== "string" && !surfaces.has(i.key)) surfaces = new Map(surfaces).set(i.key, t);
        }
      })(),
    );
  });

  // True: this system's own stars live, the same star as the system's True (star-now.ts): churning cells, spots
  // drifting by latitude, the kernel's flares. No glare here: the lineup compares sizes side by side.
  let stars = $state(new Map<string, LiveStar>());
  $effect(() => {
    if (look !== "true" || ctx.session.seat.authority !== "gm") return;
    const own_stars = placed.map((p) => p.item).filter((i) => i.kind === "star" && own(i));
    untrack(() =>
      void (async () => {
        for (const i of own_stars) {
          const key = `${system}|${i.key}`;
          if (!keptStars.has(key)) keptStars.set(key, await readLiveStar(ctx.session as unknown as StarReader, system, i.key, given(i)));
          const s = keptStars.get(key);
          if (s && typeof s !== "string" && !stars.has(i.key)) stars = new Map(stars).set(i.key, s);
        }
      })(),
    );
  });

  // The watch: seconds since the globes appeared, on the page's own clock (presentation only). Still under reduced motion.
  let t = $state(0);
  const still = typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
  $effect(() => {
    if (!ready || still) return;
    let raf = 0;
    const t0 = performance.now() - untrack(() => t) * 1000;
    const tick = (now: number) => {
      t = (now - t0) / 1000;
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  });
  const quickest = $derived(Math.min(...placed.map((p) => Math.abs(p.item.day_h ?? Infinity))));

  $effect(() => {
    if (!ready || !gpu || !canvas) return;
    const dpr = Math.min(2, typeof devicePixelRatio === "number" ? devicePixelRatio : 1);
    const ink = token("--sse-ink-muted", "--sse-ink");
    const mark = token("--sse-accent", "--sse-ink");
    const faint = token("--sse-ink-faint", "--sse-ink-muted");
    const items: GpuItem[] = [];
    const lines: GpuRing[] = [];
    let deepest = 10;
    for (const p of placed) {
      const i = p.item;
      const reach = (reachKm(i) / i.radius_km) * p.r;
      deepest = Math.max(deepest, reach + 10);
      const c: Vec3 = [p.x - width / 2, height / 2 - (base - p.r), 0];
      const f = globeFrame(i.tilt_rad, view, spinAt(i.day_h, t, pace, Number.isFinite(quickest) ? quickest : 24));
      if (look !== "wireframe") {
        // True: the body's face from the one door every True view uses (true-body.ts), so each new look shows here too.
        // A world not read is the plain grey of the system's True; 3D keeps the chart's colour.
        // A live star's face at its own clock (six hours a second, a picture every tenth of a second).
        const live = look === "true" && i.kind === "star" ? stars.get(i.key) : undefined;
        const starFace = live ? liveFace(`${system}|${i.key}`, live, { t: live.age_s + (Math.floor(t * 10) / 10) * STAR_SECONDS_PER_SECOND, clock: STAR_SECONDS_PER_SECOND, span: STAR_SECONDS_PER_SECOND / 10, calm: still, px: 2 * p.r * dpr }).face : undefined;
        const face =
          look === "true"
            ? trueLook({ ...(starFace ? { face: starFace } : {}), key: `${system}|${i.key}`, star: i.kind === "star", colour: i.kind === "star" ? rgb(colour(i)) : faint, radius_m: i.radius_km * 1000, ...(surfaces.get(i.key) ? { surface: surfaces.get(i.key)! } : {}), ...(flows.has(`${system}|${i.key}`) ? { flow: { flow: flows.get(`${system}|${i.key}`)!, dt_s: bodySeconds(i.day_h, t, pace, Number.isFinite(quickest) ? quickest : 24) } } : {}), ...(i.rings ? { rings: i.rings.map((r) => ({ name: "", inner_m: r.inner_km * 1000, outer_m: r.outer_km * 1000, tau: r.tau, albedo: r.albedo })) } : {}), pole: f[2] })
            : { colour: rgb(colour(i)), ...(i.kind === "star" ? { shines: true } : {}) };
        items.push({ id: i.key, rel: c, radius: p.r, frame: f, ...face });
      }
      // Wireframe: every line, see-through. 3D: the near side only, drawn over the shaded globe.
      if (look !== "true" && p.r >= 6)
        for (const g of graticule(f, c, p.r)) {
          const colour = g.kind === "line" ? ink : mark;
          if (look === "wireframe") lines.push({ points: g.points, colour });
          else for (const run of nearSide(g.points, c)) lines.push({ points: run, colour, over: true });
        }
      if (look !== "true") for (const r of i.rings ?? []) for (const e of [r.inner_km, r.outer_km]) lines.push({ points: ringEdge(f, c, (e / i.radius_km) * p.r), colour: faint });
    }
    // Every centre pushed back past the deepest reach, so nothing crosses the near plane; straight on, depth leaves sizes alone.
    for (const it of items) it.rel = [it.rel[0], it.rel[1], -deepest];
    for (const l of lines) l.points = l.points.map((q) => [q[0], q[1], q[2] - deepest]);
    gpu.draw({
      items,
      rings: lines,
      forward: [0, 0, -1],
      up: [0, 1, 0],
      fovY: 0.5,
      width: Math.round(width * dpr),
      height: Math.round(height * dpr),
      ground: token("--sse-chart-ground", "--sse-ground"),
      ...(look === "true" ? { light: { from: [-0.75, 0.45, 0.6] } } : look === "3d" ? { light: { from: [-0.6, 0.5, 0.9], ambient: 0.35, ground: true } } : {}),
      ortho: { width, height, near: 1, far: 2 * deepest + 10 },
    });
  });
</script>

<canvas bind:this={canvas} class="globes" style:width="{width}px" style:height="{height}px" data-lineup-3d={look} data-turning={still ? "still" : "turning"} data-surfaces={surfaces.size} data-live-stars={stars.size} aria-hidden="true"></canvas>

<style>
  .globes {
    position: absolute;
    inset: 0 auto auto 0;
    display: block;
    pointer-events: none;
  }
</style>
