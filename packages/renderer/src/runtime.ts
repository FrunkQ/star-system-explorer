// The recipe runtime for one globe (R1): picks the tier (probe), plans the
// recipe, draws it, measures every frame, lets the governor walk the degrade
// order, and reports what it did so the view can show the quality badge.
// Presentation only: nothing here reaches the kernel or the state hash.
import type { Globe, Tier } from "../../ui-contract/src/index.ts";
import { badge, Governor, type Badge } from "./governor.ts";
import { DEFAULT_BUDGET_MS, estimateMs, maxLevel, pickRecipe, plan, TIER_ORDER, type PassSpec, type Plan, type RecipeDoc } from "./recipe.ts";
import { drawCanvas } from "./canvas.ts";
import { lightDir, narrate, prepare, type Prepared } from "./surface.ts";
import type { GlobeGpu } from "./gpu-globe.ts";

export interface GlobeRunOptions {
  registry: readonly RecipeDoc[];
  library: readonly PassSpec[];
  recipe: string;
  /** "auto" lets the probe choose; a tier forces it (and the badge says so). */
  tier: Tier | "auto";
  knobs: Record<string, number>;
  /** Frame budget in ms; absent: the recipe's, else the tier default. */
  budget_ms?: number;
  governor: boolean;
  /** Turn slowly (a presentation drift, labelled as such): frames run continuously, so the governor has a steady signal. */
  spin: boolean;
  size_px: number;
  label: string;
}

export interface GlobeStatus {
  tier: Tier;
  backend: "webgpu" | "webgl2" | "canvas2d" | "none";
  recipe: string;
  level: number;
  max: number;
  plan: Plan;
  badge: Badge;
  ewma_ms: number;
  budget_ms: number;
  estimate_ms: number;
  frames: number;
  narration: string;
  /** Why the tier is what it is. */
  tier_note: string;
  /** The view's turn now (spinning moves it). */
  lon: number;
  lat: number;
  /** The last frame's own drawing time, ms (the rest of a frame interval is the page and the browser). */
  work_ms: number;
}

export interface GlobeRun {
  setGlobe(globe: Globe): void;
  setView(lon: number, lat: number): void;
  update(o: Partial<GlobeRunOptions>): void;
  /** Recent frame times in ms, newest last (for tests and the badge). */
  frameTimes(): readonly number[];
  dispose(): void;
}

/** What this device can draw: probed once a page. */
let gpuProbe: Promise<boolean> | null = null;

export function mountGlobe(host: HTMLElement, opts: GlobeRunOptions, onStatus: (s: GlobeStatus) => void): GlobeRun {
  let o = { ...opts };
  let globe: Globe | null = null;
  let prepared: Prepared | null = null;
  let preparedKey = "";
  let lon = 0;
  let lat = 20;
  let canvas: HTMLCanvasElement | null = null;
  let kind: "gpu" | "canvas" | "text" | null = null;
  let gpu: GlobeGpu | null = null;
  let gpuFailed = false;
  const scratch: { c?: HTMLCanvasElement; img?: ImageData } = {};
  let gov: Governor | null = null;
  let govKey = "";
  let tierNote = "";
  let dirty = true;
  let raf = 0;
  let lastFrameAt = -1;
  let frames = 0;
  let spinFrom: { t: number; lon: number } | null = null;
  const times: number[] = [];
  let disposed = false;
  let lastSig = "";
  let lastEmit = -1e9;
  let switching = false;

  const recipe = (tier: Tier) => pickRecipe(o.registry, o.recipe, tier);
  const full = () => Math.round(o.size_px * Math.min(2, typeof devicePixelRatio === "number" ? devicePixelRatio : 1));

  // The probe: can this device draw on the GPU at all? (WebGPU, else WebGL2.)
  async function probeGpu(): Promise<boolean> {
    gpuProbe ??= (async () => {
      const c = document.createElement("canvas");
      const { createGlobeGpu } = await import("./gpu-globe.ts");
      const g = await createGlobeGpu(c);
      g?.dispose();
      return !!g;
    })();
    return gpuProbe;
  }

  let available: Tier[] = ["canvas", "text"];
  let startTier: Tier = "canvas";
  async function chooseTier() {
    const want = o.tier;
    const hasGpu = !gpuFailed && (want === "auto" || want === "gpu-basic") ? await probeGpu() : false;
    available = hasGpu ? ["gpu-basic", "canvas", "text"] : ["canvas", "text"];
    const asked = recipe(want === "auto" ? (hasGpu ? "gpu-basic" : "canvas") : want);
    if (want === "auto") {
      const doc = o.registry.find((r) => r.id === o.recipe) ?? asked?.doc;
      startTier = TIER_ORDER.find((t) => doc?.tiers.includes(t) && available.includes(t)) ?? "text";
      tierNote = hasGpu ? `Chosen for this device: it can draw on the GPU.` : `Chosen for this device: no WebGPU or WebGL2 here, so the globe is drawn on a 2D canvas.`;
    } else if (want === "gpu-basic" && !hasGpu) {
      startTier = "canvas";
      tierNote = "GPU tier asked for, but this device has neither WebGPU nor WebGL2: drawn on a 2D canvas instead.";
    } else {
      startTier = want;
      tierNote = `Drawn on ${want} as set, not by this device's own choice.`;
    }
  }

  function current(): { doc: RecipeDoc; stood_in?: string; plan: Plan; budget: number } | null {
    const r = recipe(startTier);
    if (!r) return null;
    const key = `${r.doc.id}|${startTier}|${o.size_px}|${JSON.stringify(o.knobs)}|${o.budget_ms}`;
    const budget = o.budget_ms ?? r.doc.budget_ms?.[startTier] ?? DEFAULT_BUDGET_MS[startTier];
    if (!gov || govKey !== key) {
      const keep = gov && govKey.split("|")[0] === r.doc.id && govKey.split("|")[1] === startTier ? gov.level : 0;
      gov = new Governor(maxLevel(r.doc, startTier), { budget_ms: budget }, Math.min(keep, maxLevel(r.doc, startTier)));
      govKey = key;
    }
    const level = o.governor ? gov.level : 0;
    return { ...r, plan: plan(r.doc, o.library, startTier, level, o.knobs), budget };
  }

  function ensureSurface(want: "gpu" | "canvas" | "text") {
    if (kind === want && (want === "text" || canvas)) return;
    if (gpu) (gpu.dispose(), (gpu = null));
    canvas?.remove();
    canvas = null;
    kind = want;
    if (want === "text") return;
    canvas = document.createElement("canvas");
    canvas.style.width = canvas.style.height = `${o.size_px}px`;
    canvas.setAttribute("aria-hidden", "true");
    canvas.dataset.surface = want;
    host.appendChild(canvas);
    if (want === "gpu") {
      switching = true;
      const c = canvas;
      void import("./gpu-globe.ts").then(async ({ createGlobeGpu }) => {
        const g = await createGlobeGpu(c);
        switching = false;
        if (disposed || canvas !== c) return g?.dispose();
        if (!g) {
          gpuFailed = true;
          await chooseTier();
          kind = null;
        } else gpu = g;
        dirty = true;
        kick();
      });
    }
  }

  function frame(now: number) {
    raf = 0;
    if (disposed) return;
    const cur = current();
    if (!cur || !globe) return;
    if (o.spin) {
      spinFrom ??= { t: now, lon };
      lon = (((spinFrom.lon + ((now - spinFrom.t) / 1000) * 12) % 360) + 360) % 360;
    }
    const pl = cur.plan;
    const relief = pl.passes.find((x) => x.pass === "relief-globe")?.params ?? {};
    const pkey = `${String(relief.colour ?? "survey")}|${String(relief.exaggeration ?? 12)}`;
    if (!prepared || preparedKey !== pkey || prepared.globe !== globe) ((prepared = prepare(globe, String(relief.colour ?? "survey"), Number(relief.exaggeration ?? 12))), (preparedKey = pkey));
    const want = pl.tier === "gpu-basic" ? "gpu" : pl.tier === "canvas" ? "canvas" : "text";
    ensureSurface(want);
    const t0 = performance.now();
    let drew = false;
    if (want === "canvas" && canvas) {
      if (canvas.style.width !== `${o.size_px}px`) canvas.style.width = canvas.style.height = `${o.size_px}px`;
      drawCanvas(canvas, prepared, pl, lon, lat, full(), scratch);
      drew = true;
    } else if (want === "gpu" && gpu && canvas && !switching) {
      if (canvas.style.width !== `${o.size_px}px`) canvas.style.width = canvas.style.height = `${o.size_px}px`;
      gpu.draw(prepared, pl, lon, lat, full());
      drew = true;
    }
    const work = performance.now() - t0;
    if (drew) {
      frames++;
      // Continuous frames: the interval between them is the frame time; a lone frame: its own work.
      const ms = lastFrameAt >= 0 && now - lastFrameAt < 250 ? Math.max(work, now - lastFrameAt) : work;
      lastFrameAt = now;
      times.push(ms);
      if (times.length > 240) times.shift();
      if (o.governor && gov) {
        const pixels = full() * full();
        const est = estimateMs(pl, o.library, pixels);
        const ratio = est > 0 && Number.isFinite(gov.ewma) ? gov.ewma / est : null;
        const move = gov.sample(ms, (lv) => (ratio === null ? null : estimateMs(plan(cur.doc, o.library, startTier, lv, o.knobs), o.library, pixels) * ratio));
        if (move) dirty = true;
      }
    }
    const c2 = current()!;
    // Tell the view at most four times a second while frames run on (a status redraws the page around the globe,
    // which on a slow phone costs as much as the globe), and at once when anything but the turn changed.
    const sig = `${c2.doc.id}|${c2.plan.tier}|${c2.plan.level}|${gpu?.backend}`;
    const emit = !o.spin || sig !== lastSig || now - lastEmit >= 250 || (!dirty && !o.spin);
    if (emit) {
      lastSig = sig;
      lastEmit = now;
      const narr = c2.plan.passes.find((x) => x.kind === "text");
      const L = lightDir(Number(relief.sun_azimuth_deg ?? 315), Number(relief.sun_elevation_deg ?? 40));
      const narration = narrate(prepared, o.label, lon, lat, String(narr?.params.detail ?? "full"), L);
      onStatus({
      tier: c2.plan.tier,
      backend: c2.plan.tier === "gpu-basic" ? (gpu?.backend ?? "none") : c2.plan.tier === "canvas" ? "canvas2d" : "none",
      recipe: c2.doc.id,
      level: c2.plan.level,
      max: gov?.max ?? 0,
      plan: c2.plan,
      badge: badge({ reductions: c2.plan.reductions, ewma_ms: gov?.ewma ?? NaN, budget_ms: c2.budget, lift: c2.doc.lift, stood_in: c2.stood_in}),
      ewma_ms: gov?.ewma ?? NaN,
      budget_ms: c2.budget,
      estimate_ms: estimateMs(c2.plan, o.library, full() * full()),
      frames,
      narration,
      tier_note: tierNote,
      lon,
      lat,
      work_ms: work,
      });
    }
    if (!drew && want !== "text") dirty = true;
    else if (drew) dirty = false;
    if (o.spin || dirty) kick();
    else lastFrameAt = -1;
  }

  function kick() {
    if (!raf && !disposed) raf = requestAnimationFrame(frame);
  }

  let ready = chooseTier().then(() => ((dirty = true), kick()));

  return {
    setGlobe(g) {
      globe = g;
      dirty = true;
      void ready.then(kick);
    },
    setView(lo, la) {
      lon = lo;
      lat = la;
      spinFrom = null;
      dirty = true;
      kick();
    },
    update(n) {
      const retier = (n.tier !== undefined && n.tier !== o.tier) || (n.recipe !== undefined && n.recipe !== o.recipe);
      o = { ...o, ...n };
      if (!o.spin) spinFrom = null;
      dirty = true;
      if (retier) ready = chooseTier().then(() => ((gov = null), (dirty = true), kick()));
      else kick();
    },
    frameTimes: () => times,
    dispose() {
      disposed = true;
      if (raf) cancelAnimationFrame(raf);
      gpu?.dispose();
      canvas?.remove();
    },
  };
}
