// The recipe runtime for one globe (R1): picks the tier (probe), plans the
// recipe, draws it, measures every frame, lets the governor walk the degrade
// order, and reports what it did so the view can show the quality badge.
// Presentation only: nothing here reaches the kernel or the state hash.
import { constant, cubeDirection, perf, type Globe, type Tier } from "../../ui-contract/src/index.ts";
import { iceAges, iceAgeWords, layersFor, narrateFrame, pickAt, recolour, type LayerSpec, type TimeLapse } from "./timelapse.ts";
import { narrateWeather, weatherAt, withWeather, type WeatherMap } from "./weather.ts";
import { withFace } from "./globe-face.ts";
import { flowFace, type AirFlow } from "./true-air.ts";
import type { TrueTexture } from "./true-look.ts";
import { badge, Governor, type Badge } from "./governor.ts";
import { DEFAULT_BUDGET_MS, estimateMs, maxLevel, pickRecipe, plan, TIER_ORDER, type PassSpec, type Plan, type RecipeDoc } from "./recipe.ts";
import { drawCanvas } from "./canvas.ts";
import { lightDir, narrate, prepare, type Prepared } from "./surface.ts";
import type { GlobeGpu } from "./gpu-globe.ts";
import { air as airOf, type Air, type AirFacts } from "./atmosphere.ts";
import { place, type Placement, type ProbeResult } from "./probe.ts";
import { deviceCaps, deviceSpeed, heat, quality, speedListeners, speedNow } from "./device.ts";
import { REST_FRAMES, RestClock, RestLayer, type RestState } from "./still.ts";

export { deviceReadout, measureAgain, quality, setQuality } from "./device.ts";
import type { AirRender } from "./gpu-air.ts";
import { exposeAurora, type AuroraExposure, type AuroraPlan } from "./aurora.ts";

/** The Sun's light at 1 au, W/m² (Kopp and Lean 2011), when a world's own is not handed in. */
const SOLAR_FLUX_1AU = constant("S_sun");

/** The air in render units (globe radius 1): coefficients × radius, scale height ÷ radius, shell to 10 scale heights. */
export function airRender(a: { air: Air; radius_m: number }, thickness = 1): AirRender {
  const k = Math.max(1, thickness);
  const H = (a.air.scale_height_m / a.radius_m) * k;
  return { beta: a.air.rayleigh_m.map((b) => (b * a.radius_m) / k) as [number, number, number], scale_height: H, top: 1 + 10 * H };
}

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
  /** The world's air facts (absent or null: no air, and the atmosphere pass draws nothing). */
  air?: AirFacts | null;
  /** The world's aurora (aurora.ts `planAurora` from read_field), its radius, and the star's light at it in W/m² (for Eye). */
  aurora?: { plan: AuroraPlan; radius_m: number; flux_w_m2?: number } | null;
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
  /** The air in words (where it came from and what it does), or why there is none. */
  air_note: string;
  /** The aurora in words (what glows where, the exposure), or why none is drawn. */
  aurora_note: string;
  /** The last frame's own drawing time, ms (the rest of a frame interval is the page and the browser). */
  work_ms: number;
  /** Sharpen at rest (VP5): moving, waiting to rest, sharpening, or sharp; and frames averaged so far. */
  rest: RestState;
  rest_frames: number;
}

export interface GlobeRun {
  setGlobe(globe: Globe): void;
  /** Time lapse (the kernel's surface_frames, read): null returns to the present-day surface. */
  setTimeLapse(tl: TimeLapse | null): void;
  /** Which layer colours the globe in a time lapse (an id from layers()). */
  setLayer(id: string): void;
  /** The time shown, Myr from the world's formation (the frames' clock). */
  setTime(t_myr: number): void;
  /** The layers this time lapse can show: id, label, legend, limits, and whether its fields are present or pending. */
  layers(): { id: string; label: string; legend: LayerSpec["legend"]; limits: string; available: boolean; pending: string[] }[];
  /** Keyframe times and dated events, for the scrubber's marks; the ice ages' window (they fall within a few million years of today) for a strip of their own. */
  frames(): { times: number[]; events: { t_myr: number; kind: string; label: string; lon?: number; lat?: number; why?: string[] }[]; iceAges: { from_myr: number; to_myr: number; words: string[] } | null };
  /** What lies under a point on the globe at the time shown: field values, the nearest event and the kernel's why lines. */
  pick(lon_deg: number, lat_deg: number): ReturnType<typeof pickAt> | null;
  /** Today's weather over the globe (the kernel's weather op, read): null takes it off. A time lapse shows its own past instead. */
  setWeather(w: WeatherMap | null): void;
  /**
   * The world's True face (true-air.ts `readTrueFace`) worn in place of the tiles' colours, for a world with no
   * ground to show (a giant's cloud decks), carried by its winds `dt_s` after it was read when a flow comes with it.
   * Null goes back to the tiles. A time lapse shows its own past instead; the face replaces the weather layer.
   */
  setFace(f: { tex: TrueTexture; flow?: AirFlow; radius_m?: number; dt_s?: number } | null): void;
  /** The weather at a point: the cell's layers and rain, and a storm over it (radius sizes the storms). */
  weatherAt(lon_deg: number, lat_deg: number, radius_km: number): ReturnType<typeof weatherAt>;
  setView(lon: number, lat: number): void;
  update(o: Partial<GlobeRunOptions>): void;
  /** Recent frame times in ms, newest last (for tests and the badge). */
  frameTimes(): readonly number[];
  dispose(): void;
}

/** What this device can draw: probed once a page. */
let gpuProbe: Promise<"webgpu" | "webgl2" | null> | null = null;

export function mountGlobe(host: HTMLElement, opts: GlobeRunOptions, onStatus: (s: GlobeStatus) => void): GlobeRun {
  let o = { ...opts };
  let globe: Globe | null = null;
  let prepared: Prepared | null = null;
  let preparedKey = "";
  let lapse: TimeLapse | null = null;
  let lapseLayer = "land";
  let lapseT = 0;
  let wx: WeatherMap | null = null;
  let face: { tex: TrueTexture; flow?: AirFlow; radius_m?: number; dt_s?: number } | null = null;
  let faceN = 0;
  let shown: Prepared | null = null;
  let shownKey = "";
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
  let airPhys: { air: Air; radius_m: number } | null = null;
  const setAir = (f: AirFacts | null | undefined) => {
    const a = f ? airOf(f) : null;
    airPhys = a && f ? { air: a, radius_m: f.radius_m } : null;
  };
  setAir(opts.air);
  let airFactsNow: AirFacts | null | undefined = opts.air;
  const airNote = () => {
    if (!airPhys || !airFactsNow) return "No air drawn: the kernel has no surface pressure and gas mix for this world.";
    const a = airPhys.air;
    const f = airFactsNow;
    const mix = Object.entries(f.composition).sort((x, y) => y[1] - x[1]).slice(0, 3).map(([g, x]) => `${g} ${Math.round(x * 100)}%`).join(", ");
    const th = Number(current()?.plan.passes.find((x) => x.pass === "atmosphere")?.params.thickness ?? 1);
    return `Air from the kernel: ${(f.pressure_pa / 100).toFixed(0)} hPa at ${Math.round(f.temperature_k)} K, ${mix}; scale height ${(a.scale_height_m / 1000).toFixed(1)} km; blue light's path straight up scatters ${Math.round((1 - Math.exp(-a.zenith_depth[2])) * 100)}% (Rayleigh, single scattering).${a.unknown_gases.length ? ` Left out (no data): ${a.unknown_gases.join(", ")}.` : ""}${th > 1 ? ` Drawn ${th}× thicker than true.` : ""}`;
  };
  const auroraNote = (pass: Record<string, unknown> | undefined, d: ReturnType<typeof exposeAurora>) => {
    if (!o.aurora) return "No aurora drawn: the kernel has not read this world's field.";
    if (!pass) return `${o.aurora.plan.note} Drawn in True on the GPU only.`;
    const cav = o.aurora.plan.caveats.length ? ` Not yet: ${o.aurora.plan.caveats.join("; ")}.` : "";
    return `${o.aurora.plan.note}${d ? ` ${d.says}` : ""}${o.aurora.flux_w_m2 ? "" : d && pass.exposure === "eye" ? " The star's light here is taken as the Sun's at Earth." : ""}${cav}`;
  };
  let lastEmit = -1e9;
  // Sharpen at rest (VP5): after half a second still, jittered frames of the recipe's still passes averaged over the globe.
  const rest = new RestClock({ frames: 0 });
  let restLayer: RestLayer | null = null;
  let restTimer: ReturnType<typeof setTimeout> | null = null;
  let restRaf = 0;
  const reduced = typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
  const stopRest = () => {
    if (restTimer) (clearTimeout(restTimer), (restTimer = null));
    if (restRaf) (cancelAnimationFrame(restRaf), (restRaf = 0));
  };
  let lastStatus: GlobeStatus | null = null;
  const tellRest = () => {
    if (lastStatus && (lastStatus.rest !== rest.state || lastStatus.rest_frames !== rest.frames)) onStatus((lastStatus = { ...lastStatus, rest: rest.state, rest_frames: rest.frames }));
  };
  function restStep() {
    restTimer = null;
    restRaf = 0;
    if (disposed || o.spin || !canvas || !globe || !shown || switching) return;
    const now = performance.now();
    const step = rest.next(now);
    if (!step) {
      // A timer can fire a hair early: still waiting, so ask again when the wait is really over.
      if (rest.state === "waiting") return void (restTimer = setTimeout(restStep, Math.max(1, rest.dueIn(performance.now()))));
      if (rest.state === "sharp") restLayer?.show("sharp");
      tellRest();
      return;
    }
    const cur = current();
    if (!cur) return;
    const tier = cur.plan.tier;
    const host0 = canvas.parentElement ?? host;
    if (!restLayer) {
      if (getComputedStyle(host0).position === "static") host0.style.position = "relative";
      restLayer = new RestLayer(host0, { reduced });
    }
    restLayer.cover(canvas);
    const t0 = performance.now();
    const atm = (p: Plan) => p.passes.find((x) => x.pass === "atmosphere")?.params;
    if (kind === "gpu" && gpu) {
      // The still passes where the recipe has them (finer mesh, more air samples), at full size.
      const sr = cur.doc.tiers.includes("still") ? plan(cur.doc, o.library, "still", 0, o.knobs) : { ...cur.plan, render_scale: 1 };
      const a = atm(sr);
      gpu.draw(shown, sr, lon, lat, full(), a && airPhys ? airRender(airPhys, Number(a.thickness ?? 1)) : null, step.jitter);
      restLayer.add(canvas, step.k, canvas.width, canvas.height);
    } else if (kind === "canvas") {
      // One frame at twice the size, scaled down.
      const big = document.createElement("canvas");
      const a = atm(cur.plan);
      drawCanvas(big, shown, { ...cur.plan, render_scale: 1 }, lon, lat, full() * 2, {}, a && airPhys ? airRender(airPhys, Number(a.thickness ?? 1)) : null);
      restLayer.add(big, step.k, full(), full());
    } else return;
    perf.record("render.rest", performance.now() - t0, { detail: tier });
    rest.done(performance.now(), performance.now() - t0);
    if (rest.state === "sharp") restLayer.show("sharp");
    tellRest();
    restRaf = requestAnimationFrame(restStep);
  }
  let switching = false;

  const recipe = (tier: Tier) => pickRecipe(o.registry, o.recipe, tier);
  const full = () => Math.round(o.size_px * Math.min(2, typeof devicePixelRatio === "number" ? devicePixelRatio : 1));

  // The probe: can this device draw on the GPU at all, and is it WebGPU (gpu-high) or WebGL2 (gpu-basic)?
  async function probeGpu(): Promise<"webgpu" | "webgl2" | null> {
    gpuProbe ??= (async () => {
      const c = document.createElement("canvas");
      const { createGlobeGpu } = await import("./gpu-globe.ts");
      const g = await createGlobeGpu(c);
      const b = g?.backend ?? null;
      g?.dispose();
      return b;
    })();
    return gpuProbe;
  }

  let startTier: Tier = "canvas";
  async function chooseTier() {
    const want = o.tier;
    const gpuKind = !gpuFailed && (want === "auto" || want === "gpu-basic" || want === "gpu-high" || want === "gpu-ultra") ? await probeGpu() : null;
    const hasGpu = !!gpuKind;
    if (want === "auto") {
      // Placed by measured speed (VP4), not by which graphics layer the device has: a fast WebGL2 device can earn High.
      const doc = o.registry.find((r) => r.id === o.recipe) ?? recipe(hasGpu ? "gpu-basic" : "canvas")?.doc;
      if (!doc) {
        startTier = "text";
        tierNote = "No recipe to draw with.";
        return;
      }
      const caps = hasGpu ? await deviceCaps().catch(() => null) : null;
      const speed = caps && hasGpu ? await deviceSpeed({ ...caps, backend: gpuKind }) : null;
      const placed: Placement = place({ caps: { backend: gpuKind, compute: gpuKind === "webgpu" && !!caps?.compute }, probe: speed, pixels: full() * full(), recipe: doc, library: o.library, choice: quality(), heat_ceiling: heat.ceiling });
      startTier = placed.tier;
      tierNote = placed.note;
    } else if ((want === "gpu-basic" || want === "gpu-high" || want === "gpu-ultra") && !hasGpu) {
      startTier = "canvas";
      tierNote = "GPU tier asked for, but this device has neither WebGPU nor WebGL2: drawn on a 2D canvas instead.";
    } else {
      startTier = want;
      tierNote = `Drawn on ${want} as set, not by this device's own choice.`;
    }
  }
  // When the background probe returns, place this view again by the measured speed.
  const onSpeed = () => {
    if (disposed || o.tier !== "auto") return;
    ready = chooseTier().then(() => ((gov = null), (dirty = true), kick()));
  };
  speedListeners.add(onSpeed);

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
    stopRest();
    restLayer?.dispose();
    restLayer = null;
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
    // A time lapse recolours the same samples (both tiers draw from them); recoloured once per layer and time.
    if (lapse) {
      const k = `${preparedKey}|${lapseLayer}|${lapseT}`;
      if (!shown || shownKey !== k || shown.globe !== globe) ((shown = recolour(prepared, lapse, lapseLayer, lapseT, cubeDirection)), (shownKey = k));
    } else if (face) {
      const k = `${preparedKey}|face|${faceN}|${face.dt_s ?? 0}`;
      if (!shown || shownKey !== k || shown.globe !== globe) {
        const f = face;
        const tex = f.flow && f.dt_s && f.radius_m && f.radius_m > 0 ? flowFace(f.tex, f.flow, { radius_m: f.radius_m, dt_s: f.dt_s }) : f.tex;
        ((shown = withFace(prepared, tex, cubeDirection)), (shownKey = k));
      }
    } else if (wx) {
      const k = `${preparedKey}|weather|${wx.day}`;
      if (!shown || shownKey !== k || shown.globe !== globe) ((shown = withWeather(prepared, wx, cubeDirection)), (shownKey = k));
    } else shown = prepared;
    const drawn = shown!;
    const want = pl.tier === "gpu-basic" || pl.tier === "gpu-high" || pl.tier === "gpu-ultra" ? "gpu" : pl.tier === "canvas" ? "canvas" : "text";
    const atm = pl.passes.find((x) => x.pass === "atmosphere")?.params;
    const airNow = atm && airPhys ? airRender(airPhys, Number(atm.thickness ?? 1)) : null;
    const auPass = pl.passes.find((x) => x.pass === "aurora")?.params;
    const auNow = auPass && o.aurora ? exposeAurora(o.aurora.plan, o.aurora.radius_m, (auPass.exposure === "eye" ? "eye" : "long") as AuroraExposure, Number(atm?.sun ?? 6), o.aurora.flux_w_m2 ?? SOLAR_FLUX_1AU, Number(auPass.lift_target ?? 0.6)) : null;
    ensureSurface(want);
    const t0 = performance.now();
    let drew = false;
    if (want === "canvas" && canvas) {
      if (canvas.style.width !== `${o.size_px}px`) canvas.style.width = canvas.style.height = `${o.size_px}px`;
      drawCanvas(canvas, drawn, pl, lon, lat, full(), scratch, airNow);
      drew = true;
    } else if (want === "gpu" && gpu && canvas && !switching) {
      if (canvas.style.width !== `${o.size_px}px`) canvas.style.width = canvas.style.height = `${o.size_px}px`;
      gpu.draw(drawn, pl, lon, lat, full(), airNow, undefined, auNow);
      drew = true;
    }
    const work = performance.now() - t0;
    // Any frame drawn means the view changed: drop the sharpened picture at once and wait for rest again.
    if (drew) {
      stopRest();
      restLayer?.hide();
      rest.aim(o.spin ? 0 : REST_FRAMES[pl.tier], now);
      if (!o.spin && rest.state === "waiting") restTimer = setTimeout(restStep, rest.dueIn(performance.now()));
    }
    if (drew) {
      frames++;
      // The performance readout (PF1), when it is on: the globe's own drawing time, by tier, and each pass's share
      // of it (VP4; estimated from the passes' declared costs until per-pass GPU timing lands).
      perf.record("render.globe", work, { detail: pl.tier });
      if (perf.enabled) for (const x of passShares(pl, o.library, full() * full(), speedNow())) perf.record("render.pass", work * x.share, { detail: `${x.pass} (${pl.tier}, estimated share)` });
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
        // Three steps down inside a minute: a hot device. Lower the ceiling for the visit and place again.
        if (move === "down" && o.tier === "auto" && heat.down(now, startTier)) ready = chooseTier().then(() => ((gov = null), (dirty = true), kick()));
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
      const narration = lapse ? narrateFrame(lapse, lapseLayer, lapseT, o.label) : `${narrate(prepared, o.label, lon, lat, String(narr?.params.detail ?? "full"), L)}${wx ? ` ${narrateWeather(wx, "")}` : ""}`;
      onStatus((lastStatus = {
      tier: c2.plan.tier,
      backend: c2.plan.tier === "gpu-basic" || c2.plan.tier === "gpu-high" || c2.plan.tier === "gpu-ultra" ? (gpu?.backend ?? "none") : c2.plan.tier === "canvas" ? "canvas2d" : "none",
      recipe: c2.doc.id,
      level: c2.plan.level,
      max: gov?.max ?? 0,
      plan: c2.plan,
      badge: badge({ reductions: c2.plan.reductions, ewma_ms: gov?.ewma ?? NaN, budget_ms: c2.budget, lift: c2.doc.lift, stood_in: c2.stood_in, at_floor: !!gov && c2.plan.level >= gov.max }),
      ewma_ms: gov?.ewma ?? NaN,
      budget_ms: c2.budget,
      estimate_ms: estimateMs(c2.plan, o.library, full() * full()),
      frames,
      narration,
      tier_note: tierNote,
      lon,
      lat,
      work_ms: work,
      air_note: airNote(),
      rest: rest.state,
      rest_frames: rest.frames,
      aurora_note: auroraNote(auPass, auNow),
      }));
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
    setTimeLapse(tl) {
      lapse = tl;
      shown = null;
      if (tl) lapseT = tl.frames[tl.frames.length - 1]!.t_myr;
      dirty = true;
      kick();
    },
    setLayer(id) {
      lapseLayer = id;
      dirty = true;
      kick();
    },
    setTime(t) {
      lapseT = t;
      dirty = true;
      kick();
    },
    layers() {
      return lapse ? layersFor(lapse).map((l) => ({ id: l.spec.id, label: l.spec.label, legend: l.spec.legend, limits: l.spec.limits, available: l.available, pending: l.pending })) : [];
    },
    frames() {
      if (!lapse) return { times: [], events: [], iceAges: null };
      const ll = (d: readonly number[]) => ({ lon: (Math.atan2(d[1]!, d[0]!) * 180) / Math.PI, lat: (Math.asin(Math.max(-1, Math.min(1, d[2]!))) * 180) / Math.PI });
      const ia = iceAges(lapse);
      const tl = lapse;
      return {
        times: lapse.frames.map((f) => f.t_myr),
        events: lapse.events.map((e) => ({ t_myr: e.t_myr, kind: e.kind, label: e.text, ...(e.dir ? ll(e.dir) : {}), ...(e.why?.length ? { why: e.why } : {}) })),
        iceAges: ia ? { from_myr: ia.from_myr, to_myr: ia.to_myr, words: ia.events.map((e) => iceAgeWords(tl, e)) } : null,
      };
    },
    pick(lo, la) {
      if (!lapse) return null;
      const a = (lo * Math.PI) / 180;
      const b = (la * Math.PI) / 180;
      return pickAt(lapse, [Math.cos(b) * Math.cos(a), Math.cos(b) * Math.sin(a), Math.sin(b)], lapseT);
    },
    setWeather(w) {
      wx = w;
      shown = null;
      dirty = true;
      kick();
    },
    setFace(f) {
      if (f?.tex !== face?.tex) faceN++;
      face = f;
      shown = null;
      dirty = true;
      kick();
    },
    weatherAt(lo, la, radius_km) {
      if (!wx) return null;
      const a = (lo * Math.PI) / 180;
      const b = (la * Math.PI) / 180;
      return weatherAt(wx, [Math.cos(b) * Math.cos(a), Math.cos(b) * Math.sin(a), Math.sin(b)], radius_km);
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
      if ("air" in n) (setAir(n.air), (airFactsNow = n.air));
      if (!o.spin) spinFrom = null;
      dirty = true;
      if (retier) ready = chooseTier().then(() => ((gov = null), (dirty = true), kick()));
      else kick();
    },
    frameTimes: () => times,
    dispose() {
      disposed = true;
      speedListeners.delete(onSpeed);
      if (raf) cancelAnimationFrame(raf);
      stopRest();
      restLayer?.dispose();
      gpu?.dispose();
      canvas?.remove();
    },
  };
}


/** Each pass's share of a frame, from its declared cost (scaled by the probe's factors when measured). */
export function passShares(pl: Plan, library: readonly PassSpec[], pixels: number, speed: Pick<ProbeResult, "factors"> | null): { pass: string; share: number }[] {
  const each = pl.passes.map((p) => ({ pass: p.pass, ms: estimateMs({ ...pl, passes: [p] }, library, pixels, {}, speed?.factors ?? null) }));
  const total = each.reduce((a, x) => a + x.ms, 0);
  return total > 0 ? each.filter((x) => x.ms > 0).map((x) => ({ pass: x.pass, share: x.ms / total })) : [];
}
