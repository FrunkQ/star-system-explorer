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
  import type { ClimateSummary, SurfaceTile } from "../../../ui-contract/src/index.ts";
  import type { AirFacts } from "../../../renderer/src/index.ts";
  // Tiles take about a second to build; keep them per system and body for the page's life.
  const cache = new Map<string, { tiles: SurfaceTile[]; regime?: string; refused: number; air: AirFacts | null }>();
  // The climate comes only when asked for (a quick survey, about half a second); kept the same way.
  const climates = new Map<string, ClimateSummary>();
</script>

<script lang="ts">
  import { globeStats, regimeWords, viewOptions, DEFAULT_VIEW, type Globe, type Json, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { BODY_CONTROLS, GLOBE_LAT } from "./controls.ts";
  import { untrack } from "svelte";
  import { DAY_S } from "../../../chart-grammar/src/calendar.ts";
  import { airFactsOf, airFromOptics, airRequest, loadAurora, type AuroraPlan, applyEdits, editableSettings, keepPhoto, listPhotos, LIBRARY, mountGlobe, photograph, readFrames, readWeather, removePhoto, type GlobeRun, type RawFrames, type WeatherSystem, type WeatherTile, type MediaItem, type RecipeDoc, type RecipeEdit, type GlobeRunOptions, type GlobeStatus } from "../../../renderer/src/index.ts";
  import { SHIPPED_RECIPES } from "../../../renderer/src/recipes.ts";
  import Cutaway from "../../cutaway/src/Cutaway.svelte";
  import StarDisc from "./StarDisc.svelte";
  import ClimateWhy from "./ClimateWhy.svelte";
  import BodyTimeline from "./BodyTimeline.svelte";
  import BodyMag from "./BodyMag.svelte";

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
      const facts = rs.ok ? airFactsOf(rs.result.facts[sub.address], surf) : null;
      // The optics themselves come from the kernel (light track, read_air_optics).
      const req = facts ? airRequest(sub.address, facts) : null;
      if (facts && req) {
        const ao = await ctx.session.readAirOptics([req.air]);
        if (g !== gen) return;
        facts.optics = ao.ok && ao.airs[0] ? airFromOptics(ao.airs[0] as Parameters<typeof airFromOptics>[0], req) : null;
      }
      const v = { tiles: r.tiles, regime: r.regime, refused: r.refused.length, air: facts };
      cache.set(key, v);
      got = v;
      loading = false;
    })();
  });

  // Aurorae (VP-AU5): where they glow from the kernel's field read (read_field's aurora), what colours and how high
  // from the air. The Solar System's own worlds read the measured fixture by name; any other adopted system is read
  // world by world from its facts. Asked once per world, only when True is shown.
  const auroraCache = new Map<string, { plan: AuroraPlan; radius_m: number } | null>();
  let aurora = $state.raw<{ plan: AuroraPlan; radius_m: number } | null>(null);
  $effect(() => {
    const sub = subject;
    const sys = scene.system;
    const air = got?.air;
    const isTrue = shownView.style === "true";
    aurora = null;
    if (!sub || sub.kind === "star" || !got || !isTrue) return;
    const key = `${sys}|${sub.address}`;
    if (auroraCache.has(key)) return void (aurora = auroraCache.get(key) ?? null);
    let live = true;
    void (async () => {
      const v = await loadAurora(ctx.session, sys, sub, { radius_m: air?.radius_m ?? 0, scale_height_m: air?.optics?.scale_height_m, composition: air?.composition });
      if (!live) return;
      auroraCache.set(key, v);
      aurora = v;
    })();
    return () => {
      live = false;
    };
  });

  // "Why?" for the climate (W7a): asked for, never fetched on a click of the world alone.
  let climate = $state<ClimateSummary | null>(null);
  let climateOpen = $state(false);
  let climateBusy = $state(false);
  let climateFailed = $state<string | null>(null);
  let climateBtn = $state<HTMLElement | undefined>();
  $effect(() => {
    const key = `${scene.system}|${subject?.address}`;
    untrack(() => {
      climate = climates.get(key) ?? null;
      climateOpen = false;
      climateBusy = false;
      climateFailed = null;
    });
  });
  async function toggleClimate() {
    if (climateOpen) return void (climateOpen = false);
    climateOpen = true;
    const sub = subject;
    if (climate || !sub) return;
    const key = `${scene.system}|${sub.address}`;
    climateBusy = true;
    climateFailed = null;
    const r = await ctx.session.surveyClimate(scene.system, sub.address);
    if (ws.selection !== sub.address) return;
    climateBusy = false;
    if (!r.ok) {
      climateFailed = r.message;
      return;
    }
    climates.set(key, r.climate);
    climate = r.climate;
  }

  const globe = $derived<Globe>(got ? [0, 1, 2, 3, 4, 5].map((f) => got!.tiles.find((t) => t.key.face === f && t.key.level === 0)) : []);
  const has = $derived(globe.some((t) => t));
  const stats = $derived(has ? globeStats(globe) : null);
  const epoch = $derived(globe.find((t) => t)?.epoch_myr);
  const pending = $derived([...new Set(globe.flatMap((t) => t?.pending ?? []))]);

  let lon = $state(0);
  let lat = $state(20);
  // The standard view controls (rule 12): the style picks the recipe, "seen from" the latitude looked down from.
  const shownView = $derived(viewOptions(BODY_CONTROLS, ws.view ?? DEFAULT_VIEW).shown);
  const styleRecipe = $derived(shownView.style === "true" ? "true-colour" : String(params.recipe ?? "survey-relief"));
  // Picking a fixed "Seen from" frames the globe afresh, even the one already shown.
  $effect(() => {
    const s = shownView.seen;
    void shownView.framed;
    if (s !== "free") ((lat = GLOBE_LAT[s]), (lon = 0));
  });
  let host = $state<HTMLElement | undefined>();
  let run: GlobeRun | null = null;
  let gs = $state<GlobeStatus | null>(null);
  let badgeOpen = $state(false);
  // The user's own choices on this device, over the view's settings (the probe can be overridden: rendering report 6.2).
  let ovRecipe = $state<string | null>(null);
  let ovTier = $state<string | null>(null);
  let ovSpin = $state<boolean | null>(null);
  // R4: recipes edited on this device. Kept in this browser (a per-viewer look), never in the campaign or the kernel.
  const MINE = "sse-recipes-mine";
  const loadMine = (): RecipeDoc[] => {
    try {
      const x = JSON.parse(localStorage.getItem(MINE) ?? "[]");
      return Array.isArray(x) ? x : [];
    } catch {
      return [];
    }
  };
  let mine = $state<RecipeDoc[]>(loadMine());
  let editing = $state(false);
  let edits = $state<RecipeEdit[]>([]);
  let editName = $state("");
  const registry = $derived<RecipeDoc[]>([...SHIPPED_RECIPES, ...mine]);
  const baseRecipe = $derived(registry.find((r) => r.id === (ovRecipe ?? styleRecipe)) ?? SHIPPED_RECIPES[0]!);
  const draft = $derived(editing && edits.length ? applyEdits(baseRecipe, editName.trim() && editName.trim() !== baseRecipe.name ? [...edits, { op: "name", name: editName.trim() }] : edits, { library: LIBRARY, pixels: size * size, budget_ms: Number(params.budget_ms ?? 33.3), id: baseRecipe.derived_from ? baseRecipe.id : `${baseRecipe.id}-mine-${mine.length + 1}` }) : null);
  function setEdit(use: string, param: string, value: number | string) {
    edits = [...edits.filter((e) => !(e.op === "set" && e.use === use && e.param === param)), { op: "set", use, param, value }];
  }
  function keepDraft() {
    const d = draft;
    if (!d?.ok) return;
    mine = [...mine.filter((r) => r.id !== d.doc.id), d.doc];
    try {
      localStorage.setItem(MINE, JSON.stringify(mine));
    } catch {
      /* kept for this page only */
    }
    ovRecipe = d.doc.id;
    editing = false;
    edits = [];
  }
  function forget(id: string) {
    mine = mine.filter((r) => r.id !== id);
    try {
      localStorage.setItem(MINE, JSON.stringify(mine));
    } catch {
      /* ignore */
    }
    if (ovRecipe === id) ovRecipe = null;
  }

  const runOpts = $derived({
    registry: draft?.ok ? [...registry.filter((r) => r.id !== draft.doc.id), draft.doc] : registry,
    library: LIBRARY,
    recipe: draft?.ok ? draft.doc.id : (ovRecipe ?? styleRecipe),
    tier: (ovTier ?? String(params.tier ?? "auto")) as GlobeRunOptions["tier"],
    knobs: { relief: Number(params.relief ?? 0.5), detail: Number(params.detail ?? 0.5) },
    budget_ms: Number(params.budget_ms ?? 33.3),
    governor: params.governor !== false,
    spin: ovSpin ?? params.spin === true,
    size_px: size,
    label: subject?.label ?? "",
    air: got?.air ?? null,
    aurora,
  });
  // Time lapse (R6): the kernel's surface_frames for this world, played on the same globe. `lapse` is the API the
  // chart room's timeline (BodyTimeline) drives; it is reactive, so a scrubber follows changes made anywhere.
  type Lapse = {
    loaded: boolean;
    loading: boolean;
    error: string | null;
    layer: string;
    t_myr: number;
    layers: ReturnType<GlobeRun["layers"]>;
    frames: ReturnType<GlobeRun["frames"]>;
    load(): Promise<void>;
    close(): void;
    setLayer(id: string): void;
    setTime(t_myr: number): void;
    pick(lon_deg: number, lat_deg: number): ReturnType<GlobeRun["pick"]>;
    /** Turn the globe to face a point (a clicked event mark brings its place into view). */
    view(lon_deg: number, lat_deg: number): void;
  };
  let lapseData: RawFrames | null = null;
  let lapseKey = "";
  const lapse: Lapse = $state({
    loaded: false,
    loading: false,
    error: null,
    layer: "land",
    t_myr: 0,
    layers: [],
    frames: { times: [], events: [], iceAges: null },
    async load() {
      const sub = subject;
      if (!sub || lapse.loading) return;
      const key = `${scene.system}|${sub.address}`;
      lapse.loading = true;
      lapse.error = null;
      const r = await ctx.session.surfaceFrames({ system: scene.system, body: sub.address, detail: "orbit" });
      lapse.loading = false;
      if (key !== `${scene.system}|${subject?.address}`) return;
      if (!r.ok) return void (lapse.error = r.message);
      lapseData = r.frames as unknown as RawFrames;
      lapseKey = key;
      attachLapse();
    },
    close() {
      lapseData = null;
      lapseKey = "";
      lapse.loaded = false;
      run?.setTimeLapse(null);
    },
    setLayer(id: string) {
      lapse.layer = id;
      run?.setLayer(id);
    },
    setTime(t: number) {
      lapse.t_myr = t;
      run?.setTime(t);
    },
    pick: (lo: number, la: number) => run?.pick(lo, la) ?? null,
    view(lo: number, la: number) {
      lon = ((lo % 360) + 360) % 360;
      lat = Math.max(-90, Math.min(90, la));
    },
  });
  /** Hand the loaded frames to the current globe runtime (again after the globe remounts). */
  function attachLapse() {
    const r = run;
    if (!r || !lapseData) return;
    r.setTimeLapse(readFrames(lapseData));
    lapse.layers = r.layers();
    lapse.frames = r.frames();
    const times = lapse.frames.times;
    if (!lapse.loaded) lapse.t_myr = times[times.length - 1] ?? 0;
    r.setLayer(lapse.layer);
    r.setTime(lapse.t_myr);
    lapse.loaded = true;
  }
  // A different world closes the time lapse.
  $effect(() => {
    const key = `${scene.system}|${subject?.address}`;
    if (untrack(() => lapseKey) && key !== untrack(() => lapseKey)) untrack(() => lapse.close());
  });

  // Weather (R2b): today's clouds and haze from the kernel's weather op, on the campaign's day. `weather` is reactive,
  // like `lapse`, so the chart room's controls can drive it; a time lapse shows its own past instead.
  const WX_N = 32;
  type Weather = { on: boolean; loading: boolean; error: string | null; day: number | null; show(): Promise<void>; hide(): void };
  let wxKey = "";
  let wxData: ReturnType<typeof readWeather> | null = null;
  const weather: Weather = $state({
    on: false,
    loading: false,
    error: null,
    day: null,
    async show() {
      const sub = subject;
      if (!sub || weather.loading) return;
      const key = `${scene.system}|${sub.address}`;
      weather.loading = true;
      weather.error = null;
      const now = await ctx.session.campaignNow();
      const at_s = now.ok && now.at_s !== undefined ? Number(now.at_s) : undefined;
      const day = at_s !== undefined ? at_s / DAY_S : 0;
      const where = { system: scene.system, body: sub.address, ...(at_s !== undefined ? { at_s } : {}), day };
      const tiles: WeatherTile[] = [];
      let fail: string | null = null;
      let meta = { block: "", evidence: "" };
      for (const face of [0, 1, 2, 3, 4, 5]) {
        const r = await ctx.session.weather({ want: "field", ...where, tile: [face, 0, 0, 0], n: WX_N });
        if (!r.ok) {
          fail = r.message;
          break;
        }
        tiles.push(r.data as unknown as WeatherTile);
        const pv = r.provenance as { block?: { id?: string }; evidence?: string };
        meta = { block: pv.block?.id ?? "", evidence: pv.evidence ?? "" };
      }
      const sy = fail ? null : await ctx.session.weather({ want: "systems", ...where });
      weather.loading = false;
      if (key !== `${scene.system}|${subject?.address}`) return;
      if (fail) return void (weather.error = fail);
      wxData = readWeather(tiles, sy?.ok ? (sy.data as unknown as WeatherSystem[]) : [], { day, ...meta });
      wxKey = key;
      weather.day = day;
      weather.on = true;
      run?.setWeather(wxData);
    },
    hide() {
      wxData = null;
      wxKey = "";
      weather.on = false;
      weather.day = null;
      run?.setWeather(null);
    },
  });
  $effect(() => {
    const key = `${scene.system}|${subject?.address}`;
    if (untrack(() => wxKey) && key !== untrack(() => wxKey)) untrack(() => weather.hide());
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
    untrack(() => attachLapse());
    if (wxData) r.setWeather(wxData);
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
      const shot = await photograph({ registry: o.registry, library: o.library, recipe: o.recipe, knobs: o.knobs, globe, air: o.air, aurora: o.aurora, label: s.label, lon, lat, size: 1024, frames: 16, onProgress: (d, n) => (shooting = `Refining ${d} of ${n}…`) });
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
  const tierWords: Record<string, string> = { still: "Photograph", "gpu-high": "GPU high", "gpu-basic": "GPU", canvas: "Canvas", text: "Text" };

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
    if (Math.abs(e.clientY - drag.y) + Math.abs(e.clientX - drag.x) > 3 && shownView.seen !== "free") ws.turned?.();
    const k = 180 / size;
    lon = (((drag.lon - (e.clientX - drag.x) * k) % 360) + 360) % 360;
    lat = Math.max(-90, Math.min(90, drag.lat + (e.clientY - drag.y) * k));
  }
  function key(e: KeyboardEvent) {
    const step = e.shiftKey ? 45 : 15;
    if (e.key === "ArrowLeft") lon = (lon + 360 - step) % 360;
    else if (e.key === "ArrowRight") lon = (lon + step) % 360;
    else if (e.key === "ArrowUp") (ws.turned?.(), (lat = Math.min(90, lat + step)));
    else if (e.key === "ArrowDown") (ws.turned?.(), (lat = Math.max(-90, lat - step)));
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
    <StarDisc {ctx} {scene} address={subject.address} label={subject.label} {size} />
  {:else}
    <div class="top">
      <div class="globe" class:hidden={shownView.band === "mag"} style:width="{size}px" style:height="{size}px">
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
      {#if has && shownView.band !== "mag" && aurora?.plan.stand_in && gs?.plan.passes.some((x) => x.pass === "aurora")}
        <p class="note" data-aurora-stand-in>Aurora colours are a stand-in: oxygen's green and red lines at heights scaled from Earth's, until the kernel gives this air's own lines.</p>
      {/if}
      {#if has && shownView.band !== "mag"}
        <!-- The time lapse's controls (U22): layer toggles, scrubber with event marks, Play, and the why card. -->
        <BodyTimeline {lapse} depth={ws.depth ?? "science"} label={subject.label} />
        <!-- Interim weather control; the chart room's controls replace it. A time lapse shows its own past, so this hides then. -->
        {#if !lapse.loaded}<div class="weather" data-weather-controls data-weather={weather.on ? "on" : weather.loading ? "loading" : "off"}>
          {#if weather.on}<button type="button" class="link" onclick={() => weather.hide()}>Clear the weather</button>{:else}<button type="button" class="link" disabled={weather.loading} onclick={() => weather.show()}>{weather.loading ? "Reading the weather…" : "Today's weather"}</button>{/if}
          {#if weather.error}<small data-weather-error>{weather.error}</small>{/if}
        </div>{/if}
      {/if}
      <!-- The Mag layer (MF14b): the field's Chart cut, its words and a why for every part, when Band is Mag. -->
      <BodyMag session={ctx.session} system={scene.system} name={subject.label} tier={gs?.tier ?? "gpu-basic"} depth={ws.depth ?? "science"} size={size} on={shownView.band === "mag"} />
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
                {#if gs.plan.passes.some((x) => x.pass === "aurora")}<p data-aurora-note>{gs.aurora_note}</p>{/if}
                {#each gs.badge.detail as line (line)}<p>{line}</p>{/each}
                {#if gs.badge.reduced}<p>{gs.badge.lift}</p>{/if}
                <p>Recipe {gs.recipe}, level {gs.level} of {gs.max}. {Number.isFinite(gs.ewma_ms) ? `Frames about ${Math.round(gs.ewma_ms)} ms; budget ${Math.round(gs.budget_ms)} ms.` : ""}</p>
                <div class="drawing">
                  <label>Recipe <select value={runOpts.recipe} onchange={(e) => (ovRecipe = e.currentTarget.value)}>{#each registry as r (r.id)}<option value={r.id}>{r.name}</option>{/each}</select></label>
                  <label>Draw on <select value={runOpts.tier} onchange={(e) => (ovTier = e.currentTarget.value)}>{#each tierChoices as [v, w] (v)}<option value={v}>{w}</option>{/each}</select></label>
                  <label><input type="checkbox" checked={runOpts.spin} onchange={(e) => (ovSpin = e.currentTarget.checked)} /> Spin</label>
                </div>
                <div class="drawing">
                  <button class="link" data-edit-recipe aria-expanded={editing} onclick={() => ((editing = !editing), (edits = []), (editName = baseRecipe.derived_from ? baseRecipe.name : `My ${baseRecipe.name.toLowerCase()}`))}>{editing ? "Stop editing" : "Edit this recipe"}</button>
                  {#if baseRecipe.derived_from}<button class="link" onclick={() => forget(baseRecipe.id)}>Forget {baseRecipe.name}</button>{/if}
                </div>
                {#if editing}
                  <div class="editor" data-recipe-editor>
                    <label>Name <input value={editName} oninput={(e) => (editName = e.currentTarget.value)} /></label>
                    {#each editableSettings(baseRecipe, LIBRARY).filter((u) => u.tiers.includes(gs?.tier ?? "canvas")) as u (u.use)}
                      <fieldset>
                        <legend>{u.name}</legend>
                        {#each u.settings.filter((x) => x.type === "number" || x.type === "integer" || x.type === "enum") as st (st.name)}
                          {@const now = (edits.find((e) => e.op === "set" && e.use === u.use && e.param === st.name) as { value?: unknown } | undefined)?.value ?? st.value}
                          <label title={st.meaning}>
                            {st.name.replace(/_/g, " ")}{st.unit ? ` (${st.unit})` : ""}
                            {#if st.type === "enum"}
                              <select value={String(now)} onchange={(e) => setEdit(u.use, st.name, e.currentTarget.value)}>{#each st.options ?? [] as o (o)}<option value={o}>{o}</option>{/each}</select>
                            {:else}
                              <input type="number" value={Number(now)} min={st.range?.[0]} max={st.range?.[1]} step={st.type === "integer" ? 1 : "any"} disabled={!!st.knob} data-setting={`${u.use}.${st.name}`} onchange={(e) => setEdit(u.use, st.name, st.type === "integer" ? Math.round(Number(e.currentTarget.value)) : Number(e.currentTarget.value))} />
                            {/if}
                            {#if st.knob}<small>(the {st.knob} knob sets this)</small>{/if}
                          </label>
                        {/each}
                      </fieldset>
                    {/each}
                    {#if draft}
                      {#if draft.ok}
                        <ul data-edit-changes>{#each draft.changes as c (c)}<li>{c}</li>{/each}</ul>
                        <ul data-edit-cost>
                          {#each draft.cost.filter((c) => c.tier !== "text") as c (c.tier)}
                            <li class:over={!c.fits}>{tierWords[c.tier] ?? c.tier}: about {c.before_ms?.toFixed(1)} ms, now {c.after_ms?.toFixed(1)} ms a frame of {Math.round(c.budget_ms)} ms{c.fits ? "" : " (over: the governor will reduce it here)"}</li>
                          {/each}
                        </ul>
                        <p>The globe above is drawing the edited recipe now.</p>
                        <button class="badge" data-keep-recipe onclick={keepDraft}>Keep as my recipe</button>
                      {:else}
                        <p class="over" data-edit-errors>{draft.errors.join(" ")}</p>
                      {/if}
                    {/if}
                  </div>
                {/if}
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
          <dt>Climate</dt>
          <dd data-climate>
            {#if climate}<span data-climate-label>{regimeWords(climate.regime)}</span>{/if}
            <button class="link" bind:this={climateBtn} aria-expanded={climateOpen} data-climate-toggle onclick={toggleClimate}>{climateBusy ? "Reading the climate…" : "Why?"}</button>
          </dd>
          {#if climateOpen && climate}
            <div class="span">
              <ClimateWhy {climate} label={subject.label} depth={ws.depth ?? "science"} from={() => climateBtn} onclose={() => (climateOpen = false)} />
            </div>
          {:else if climateOpen && climateFailed}
            <p class="note span" data-climate-failed>No climate: {climateFailed}</p>
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
  /* Mag replaces the picture (the globe stays built underneath, so Visible comes straight back). */
  .globe.hidden {
    display: none;
  }
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
  .weather {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 12px;
    align-items: center;
    font-size: 13px;
    margin: 8px 0;
  }
  .weather small {
    flex-basis: 100%;
    color: var(--sse-ink-muted);
    font-size: 12px;
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
  .facts .span {
    grid-column: 1 / -1;
  }
  dt {
    color: var(--sse-ink-muted);
  }
  dd {
    margin: 0;
  }
  .editor {
    margin-top: 6px;
    color: var(--sse-ink);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .editor fieldset {
    border: 1px solid var(--sse-line);
    border-radius: 3px;
    padding: 2px 6px 4px;
    display: flex;
    flex-wrap: wrap;
    gap: 2px 10px;
  }
  .editor input,
  .editor select {
    font: inherit;
    width: 6em;
  }
  .editor ul {
    margin: 2px 0;
    padding-left: 16px;
  }
  .over {
    color: var(--sse-accent);
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
