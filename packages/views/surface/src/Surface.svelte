<!--
  Surface view (SL1; surfaces design 4.1). Three moments, one view:
  - Choose: the world's map (from its six level-0 tiles) with its landmarks. Press a landmark, or anywhere; the
    kernel judges the site and flies the chosen lander down to it (plan_descent), and the card shows the verdict,
    every danger and why. Nothing is refused: a wet or rough site is offered with its dangers (danger is drama).
  - Flying: Land plays the kernel's descent as a flown camera over the ground it lands on, with the kernel's own
    clock, height, speed and events beside it. It can be skipped.
  - Down: the ground at eye height over the few kilometres the kernel knows to its fact floor, under the world's
    own sky. Walk, or fly a low drone; going further means taking off and choosing again.
  The ground is the kernel's tiles; finer than the fact floor it is decoration, and the view says so.
  On a player's screen (`follow`) the same view follows the landing the GM sent the table (landing.ts LandingShow):
  the same site and lander, the descent planned on the seat's own kernel, the GM's pose on the ground. Players watch;
  they never pick the site or steer (seats rule).
-->
<script module lang="ts">
  import type { Landmark, SurfaceTile } from "../../../ui-contract/src/index.ts";
  // The world's map tiles (a second or so) and its landmarks, kept per world for the page's life.
  const globes = new Map<string, SurfaceTile[]>();
  const marks = new Map<string, Landmark[]>();
</script>

<script lang="ts">
  import { onDestroy, untrack } from "svelte";
  import { constant, DEFAULT_VIEW, LAMP_SPILL, LANDER_HULL_ALBEDO, landerHeightOf, lampRadiance, DESCENT_VERDICT_WORDS, LANDER_WORDS, landingSeaOf, landmarkOrder, partyLander, roundPose, sampleAt, sampleColour, STOCK_LANDERS, suggestLander, thermalColour, viewOptions, type DescentPlan, type Json, type LandingGround, type LandingSea, type LandingShow, type LandingWorld, type PlayerContext, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { airFactsOf, airFromOptics, caveCut, caveOpenings, nearestOpening, openingLine, type CaveOpening, readSeaOptics, seaWords, surfacePressureOf, airRequest, decoration, descentTrack, floorFromRefusal, playbackKeys, groundHeightAt, groundMesh, horizonDistance, sunAt, adaptedExposure, starsOverSite, auroraSky, loadAurora, type AuroraPlan, type SiteSkyFrame, type SiteSkyStar, levelFor, patchKeys, patchPoint, readPatch, startSurface, sunDirection, type Air, type Patch, type SurfaceGpu, LIBRARY, TrueRun } from "../../../renderer/src/index.ts";
  import { LAND_TRUE } from "../../../renderer/src/recipes.ts";
  import { SURFACE_CONTROLS } from "./controls.ts";

  let { ctx, scene, params, follow }: { ctx?: ViewContext; scene?: SystemScene; params: Record<string, Json>; follow?: PlayerContext } = $props();

  // A player's screen follows the landing the table watches; the GM's reads its own selection.
  // Fixed for the view's life: a screen is the GM's or a seat's from mounting.
  const watching = untrack(() => follow !== undefined);
  const show = $derived<LandingShow | null>(follow?.landing?.() ?? null);
  // Keyed by plain strings, so a new pose (a new show object) never reads as a new world.
  const followKey = $derived(show ? `${show.system}\n${show.body}\n${show.label}` : "");
  const followWorld = $derived(show ? JSON.stringify(show.world) : "");
  const system = $derived(watching ? followKey.split("\n")[0] ?? "" : scene!.system);
  const subject = $derived(watching ? (followKey ? { address: followKey.split("\n")[1]!, label: followKey.split("\n")[2]!, kind: "planet" } : undefined) : scene!.subjects.find((s) => s.address === ctx!.workspace.selection));
  const reads: LandingGround | undefined = untrack(() => (watching ? follow!.ground : ctx!.session));
  const shown = $derived(viewOptions(SURFACE_CONTROLS, (watching ? null : ctx!.workspace.view) ?? DEFAULT_VIEW).shown);
  // On the ground Chart draws as 3D: the chart is the map you choose on. A seat draws as the GM's screen does.
  const look = $derived<"true" | "3d" | "wireframe">(watching ? (show?.look ?? "true") : shown.style === "true" ? "true" : shown.style === "wireframe" ? "wireframe" : "3d");
  const hud = $derived(watching ? true : shown.hud);
  // The party's ship (party.ts) is offered first as the lander when it is at this world.
  const party = $derived(watching ? null : (ctx!.workspace.party ?? null));
  const partyHere = $derived(party && subject && party.at === subject.address ? party : null);
  const flightS = $derived(Math.min(120, Math.max(6, Number((watching ? show?.flight_s : undefined) ?? params.flight_s ?? 24))));
  const eyeM = $derived(Math.min(3, Math.max(0.3, Number(params.eye_m ?? 1.7))));

  // ── The world: map tiles, landmarks, air, star ─────────────────────────────────────────────
  /** The world as drawn, with (on the GM's screen) what its clock needs: its spin, its year and what goes round the star. */
  type World = LandingWorld & { air: Air | null; spin_s?: number; orbit_s?: number; orbiter?: string; host_lsun?: number; orbit_m?: number };
  let tiles = $state<SurfaceTile[] | null>(null);
  let landmarks = $state<Landmark[]>([]);
  let world = $state<World | null>(null);
  let failed = $state<string | null>(null);
  let gen = 0;
  $effect(() => {
    const sub = subject;
    const sys = system;
    const fw = followWorld;
    const g = ++gen;
    untrack(() => reset());
    tiles = null;
    landmarks = [];
    world = null;
    failed = null;
    if (!sub || sub.kind === "star") return;
    if (!reads) return void (failed = "This screen cannot read the ground (it runs no kernel of its own).");
    const key = `${sys}|${sub.address}`;
    (async () => {
      const hit = globes.get(key);
      const t = hit ? { ok: true as const, tiles: hit } : await reads.surfaceTiles(sys, sub.address);
      if (g !== gen) return;
      if (!t.ok) return void (failed = t.message);
      if (!t.tiles.length) return void (failed = `${sub.label} has no surface the kernel has built`);
      globes.set(key, t.tiles);
      tiles = t.tiles;
      // A seat takes the world as the GM's reads found it; it never reads the world's record (nor its landmarks).
      if (watching) return void (world = fw ? (JSON.parse(fw) as World) : null);
      const rs = await ctx!.session.resolvedSystem(sys);
      if (g !== gen || !rs.ok) return;
      const facts = rs.result.facts as Record<string, Record<string, Json>>;
      const f = facts[sub.address] ?? {};
      const surf = rs.result.surfaces.find((x) => x.address === sub.address) as { gravity_m_s2?: number; t_surface_now_k?: number; reference_radius_m?: number } | undefined;
      const radius = Number(f["body.radius"] ?? surf?.reference_radius_m ?? 0) || (t.tiles[0]!.spacing_m * t.tiles[0]!.samples * 2) / Math.PI;
      // The star: up the chain of hosts to the one with a temperature; its disc's size from its radius and distance.
      const hostOf = (x: Record<string, Json>) => (x["parent"] ?? x["orbit.host"]) as string | undefined;
      let host = hostOf(f);
      let dist = Number(f["orbit.semi_major_axis"] ?? 0);
      let star_k = 0;
      let star_r = 0;
      let star_m = 0;
      let star_l = 0;
      // The body going round the star: the world itself, or the planet a moon circles (its year sets the seasons).
      let orbiter = sub.address;
      for (let i = 0; host && i < 6; i++) {
        const h = facts[host] ?? {};
        if (typeof h["star.effective_temperature"] === "number") {
          star_k = h["star.effective_temperature"] as number;
          star_r = Number(h["body.radius"] ?? 0);
          star_m = Number(h["body.mass"] ?? 0);
          // Its light: the kernel's luminosity, else Stefan and Boltzmann's from its size and temperature.
          star_l = typeof h["star.luminosity"] === "number" ? (h["star.luminosity"] as number) : 4 * Math.PI * star_r * star_r * constant("sigma_SB") * star_k ** 4;
          break;
        }
        dist = Number(h["orbit.semi_major_axis"] ?? dist);
        orbiter = host;
        host = hostOf(h);
      }
      const spin = Number(f["body.rotation_period"] ?? 0);
      // The tilt: the kernel's and V3's axial tilt, else the catalogue's obliquity (both radians, the catalogue's
      // degrees turned to SI at the door; above 90 degrees the spin runs backwards, as Venus's does).
      const tiltRad = typeof f["body.axial_tilt"] === "number" ? (f["body.axial_tilt"] as number) : Number(f["body.obliquity"] ?? 0);
      const tiltDeg = (tiltRad * 180) / Math.PI;
      const orbitA = Number((facts[orbiter] ?? {})["orbit.semi_major_axis"] ?? 0);
      const gm = star_m > 0 ? (constant("GM_sun") * star_m) / constant("M_sun") : 0;
      const af = airFactsOf(f, surf);
      let air: Air | null = null;
      const req = af ? airRequest(sub.address, af) : null;
      if (af && req) {
        const ao = await ctx!.session.readAirOptics([req.air]);
        if (g !== gen) return;
        air = ao.ok && ao.airs[0] ? airFromOptics(ao.airs[0] as Parameters<typeof airFromOptics>[0], req) : null;
      }
      world = {
        radius_m: radius,
        pressure_bar: surfacePressureOf(f) / 1e5,
        t_k: surf?.t_surface_now_k ?? 0,
        ...(surf?.gravity_m_s2 ? { gravity_m_s2: surf.gravity_m_s2 } : {}),
        air,
        star_k: star_k || constant("T_sun"),
        sun_radius: star_r > 0 && dist > 0 ? star_r / dist : 0.0047,
        tilt_deg: tiltDeg,
        ...(spin ? { spin_s: tiltDeg > 90 ? -Math.abs(spin) : spin } : {}),
        ...(gm > 0 && orbitA > 0 ? { orbit_s: 2 * Math.PI * Math.sqrt((orbitA * orbitA * orbitA) / gm) } : {}),
        orbiter,
        ...(star_l > 0 ? { host_lsun: star_l / constant("L_sun") } : {}),
        ...(orbitA > 0 ? { orbit_m: orbitA } : {}),
      };
      landerPick = untrack(() => partyHere) ? "party" : suggestLander(world.pressure_bar, world.t_k);
      // The aurora over the site (VP-AU7): the same plan as the Body globe's, from read_field and this air.
      auroraPlan = null;
      void loadAurora(ctx!.session, sys, sub, { radius_m: radius, scale_height_m: air?.scale_height_m, composition: af?.composition }).then((v) => {
        if (g === gen) auroraPlan = v?.plan ?? null;
      });
      const lm: Landmark[] = marks.get(key) ?? (await ctx!.session.surfaceLandmarks(sys, sub.address).then((r) => (r.ok ? r.landmarks : [])));
      if (g !== gen) return;
      marks.set(key, lm);
      landmarks = landmarkOrder(lm);
    })();
  });

  // A site a link asked for (Voyages' "Land here"): chosen as if pressed on the map once the world is read, whether
  // the screen was open already or not, then the ask is spent.
  $effect(() => {
    const asked = watching ? null : ctx!.workspace.landing_site;
    const sub = subject;
    if (!asked || !sub || !world || asked.body !== sub.address) return;
    untrack(() => {
      if (phase !== "choose") takeOff();
      const near = landmarks.find((l) => Math.abs(l.lat_deg - asked.lat_deg) < 0.05 && Math.abs(l.lon_deg - asked.lon_deg) < 0.05);
      site = { lat: asked.lat_deg, lon: asked.lon_deg, label: near ? (near.name ?? near.original_name) : "" };
      // The link's hour and lander, when it carries them (a bookmark or a Playbook preset of a whole landing, LR5).
      if (asked.lander === "party" ? partyHere : (STOCK_LANDERS as readonly string[]).includes(asked.lander ?? "")) landerPick = asked.lander!;
      askedTime = asked.time ?? null;
      ctx!.workspace.landing_site = null;
    });
  });
  // The hour a link asked for is set once the world's clock at the site is known.
  let askedTime = $state<string | null>(null);
  $effect(() => {
    const t = TIMES.find((x) => x.id === askedTime);
    if (!t || !clock) return;
    untrack(() => ((askedTime = null), setLocal(t.local)));
  });

  // ── The map (Chart) ───────────────────────────────────────────────────────────────────────
  const MAP_W = 720;
  const MAP_H = 360;
  let mapCanvas = $state<HTMLCanvasElement | undefined>();
  $effect(() => {
    const t = tiles;
    const c = mapCanvas;
    if (!t || !c) return;
    const globe = [0, 1, 2, 3, 4, 5].map((f) => t.find((x) => x.key.face === f && x.key.level === 0));
    c.width = MAP_W;
    c.height = MAP_H;
    const g = c.getContext("2d")!;
    const img = g.createImageData(MAP_W, MAP_H);
    for (let y = 0; y < MAP_H; y++) {
      const lat = ((90 - (180 * (y + 0.5)) / MAP_H) * Math.PI) / 180;
      for (let x = 0; x < MAP_W; x++) {
        const lon = ((-180 + (360 * (x + 0.5)) / MAP_W) * Math.PI) / 180;
        const s = sampleAt(globe, [Math.cos(lat) * Math.cos(lon), Math.cos(lat) * Math.sin(lon), Math.sin(lat)]);
        const k = (y * MAP_W + x) * 4;
        if (!s) continue;
        const col = sampleColour(s.tile, s.i);
        img.data[k] = col[0];
        img.data[k + 1] = col[1];
        img.data[k + 2] = col[2];
        img.data[k + 3] = 255;
      }
    }
    g.putImageData(img, 0, 0);
  });
  const toMap = (lat: number, lon: number) => ({ x: (((((lon + 180) % 360) + 360) % 360) / 360) * 100, y: ((90 - lat) / 180) * 100 });
  const pins = $derived(landmarks.slice(0, 24));

  // ── The site and its descent ──────────────────────────────────────────────────────────────
  let site = $state<{ lat: number; lon: number; label: string } | null>(null);
  // The lander picked on the card: a stock one, or "party" for the party's ship. A seat flies the GM's.
  let landerPick = $state<string>("capsule");
  const landerKey = $derived(watching ? JSON.stringify(show?.lander ?? "capsule") : landerPick === "party" && partyHere ? JSON.stringify(partyLander(partyHere)) : JSON.stringify(landerPick === "party" ? "capsule" : landerPick));
  let plan = $state<DescentPlan | null>(null);
  let planning = $state(false);
  let planFailed = $state<string | null>(null);
  let pgen = 0;
  $effect(() => {
    const s = site;
    const l = JSON.parse(landerKey) as string | Record<string, unknown>;
    const sub = subject;
    const sys = system;
    const g = ++pgen;
    plan = null;
    planFailed = null;
    if (!s || !sub || !reads) return;
    planning = true;
    void reads.planDescent({ system: sys, body: sub.address, lat_deg: s.lat, lon_deg: s.lon, lander: l }).then((r) => {
      if (g !== pgen) return;
      planning = false;
      if (r.ok) plan = r.plan;
      else planFailed = r.message;
    });
  });
  function pickMap(e: MouseEvent) {
    if (watching) return;
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const lon = -180 + (360 * (e.clientX - r.left)) / r.width;
    const lat = 90 - (180 * (e.clientY - r.top)) / r.height;
    site = { lat: Math.round(lat * 100) / 100, lon: Math.round(lon * 100) / 100, label: "" };
  }
  const pickLandmark = (l: Landmark) => void (!watching && (site = { lat: l.lat_deg, lon: l.lon_deg, label: l.name ?? l.original_name }));
  const worst = $derived(plan ? (plan.hazards.some((h) => h.level === "danger") ? "danger" : plan.hazards.some((h) => h.level === "caution") ? "caution" : "ok") : "");
  const fmtLat = (v: number) => `${Math.abs(v).toFixed(2)}° ${v >= 0 ? "N" : "S"}`;
  const fmtLon = (v: number) => `${Math.abs(v).toFixed(2)}° ${v >= 0 ? "E" : "W"}`;
  const fmtT = (s: number) => (s >= 3600 ? `${Math.floor(s / 3600)} h ${Math.round((s % 3600) / 60)} min` : s >= 90 ? `${Math.floor(s / 60)} min ${Math.round(s % 60)} s` : `${Math.round(s)} s`);
  const fmtM = (m: number) => (Math.abs(m) >= 10000 ? `${(m / 1000).toFixed(0)} km` : Math.abs(m) >= 1000 ? `${(m / 1000).toFixed(1)} km` : `${Math.round(m)} m`);

  // ── The sea at the site (V10k): the kernel's weather report for the wind and waves there, and the tide now ──────
  let seaRead = $state<LandingSea | null>(null);
  let seaGen = 0;
  $effect(() => {
    const s = site;
    const sub = subject;
    const sys = system;
    const g = ++seaGen;
    seaRead = null;
    if (watching || !s || !sub) return;
    const w = ctx!.workspace;
    const t_s = w.now_yr * constant("yr_julian");
    const day = (w.now_yr - (w.campaign_epoch_yr ?? w.now_yr)) * 365.25;
    void ctx!.session.weather({ want: "report", system: sys, body: sub.address, lat: s.lat, lon: s.lon, day }).then((r) => {
      if (g === seaGen && r.ok) seaRead = landingSeaOf(r.data, t_s);
    });
  });
  const seaNow = $derived<LandingSea | null>(watching ? (show?.sea ?? null) : seaRead);
  // The liquid's own numbers (W-sea2 to W-sea4), read once per sea; the words say what the kernel has not sent yet.
  const seaOptics = $derived(seaNow?.optics ? readSeaOptics(seaNow.optics) : null);

  // ── The ground round the site ─────────────────────────────────────────────────────────────
  let phase = $state<"choose" | "loading" | "flying" | "down">("choose");
  let near = $state<Patch | null>(null);
  let far = $state<Patch | null>(null);
  let groundFailed = $state<string | null>(null);
  // Lava tube openings near the site (TRUE6): the kernel's cave survey, its fallen roofs cut into the walk patch.
  let openings = $state<CaveOpening[]>([]);
  const NEAR_HALF = 6000;
  const DETAIL_HALF = 2000;
  const FAR_HALF = 120000;
  const WALK_HALF = DETAIL_HALF - 100;
  /** The ground over ±half about the site at a level (or the kernel's floor, when it names a coarser one), on a grid of `nFor(spacing)` points a side. */
  async function readGround(level: number, half: number, nFor: (spacing_m: number) => number): Promise<Patch | string> {
    const sub = subject!;
    const w = world!;
    let L = level;
    for (let tries = 0; tries < 3; tries++) {
      const keys = patchKeys(site!.lat, site!.lon, half, L, w.radius_m, tiles![0]!.samples);
      const r = await reads!.surfaceTilesAt(system, sub.address, keys.slice(0, 24));
      if (!r.ok) return r.message;
      const floor = r.refused.map((x) => floorFromRefusal(x.why)).find((x) => x !== null);
      if (floor !== undefined && floor !== null && floor < L) {
        L = floor;
        continue;
      }
      if (!r.tiles.length) return r.refused[0]?.why ?? "no tiles";
      return readPatch(r.tiles, site!.lat, site!.lon, w.radius_m, half, nFor(r.tiles[0]!.spacing_m));
    }
    return "the kernel would not give tiles at any level asked";
  }
  async function land() {
    if (!site || !world || !tiles) return;
    phase = "loading";
    groundFailed = null;
    const spacing0 = tiles[0]!.spacing_m;
    const fine = levelFor(spacing0, 100);
    // The near ground on a grid at the kernel's own spacing; the far ground four levels coarser.
    const n2 = await readGround(fine, NEAR_HALF, (sp) => Math.min(321, Math.round((2 * NEAR_HALF) / sp) + 1));
    if (typeof n2 === "string") return void ((groundFailed = n2), (phase = "choose"));
    const b = await readGround(Math.max(0, n2.level - 4), FAR_HALF, () => 161);
    if (typeof b === "string") return void ((groundFailed = b), (phase = "choose"));
    const cv = reads?.surveyCaves ? await reads.surveyCaves({ system, body: subject!.address, lat_deg: site.lat, lon_deg: site.lon, radius_km: 25 }) : null;
    openings = cv && cv.ok ? caveOpenings(cv.caves) : [];
    near = n2;
    far = b;
    built = false;
    walk = { x: 0, y: 0, yaw: Math.PI / 2, pitch: -0.05 };
    t0 = 0;
    phase = plan && track.length ? "flying" : "down";
  }
  function skip() {
    toGround();
  }
  /**
   * Down: a few of its own heights west of the lander, facing it, so the ship that came down is in frame (LR6); or
   * where the GM is standing.
   */
  const landerH = $derived(plan ? landerHeightOf(plan.lander.mass_kg) : 0);
  function toGround(pose?: LandingShow["pose"]) {
    const back = Math.max(12, 3 * landerH);
    walk = pose ? { x: pose.x_m, y: pose.y_m, yaw: pose.yaw, pitch: pose.pitch } : { x: -back, y: 0, yaw: 0, pitch: Math.atan2(landerH * 0.4 - eyeM, back) };
    phase = "down";
  }
  function takeOff() {
    phase = "choose";
    openings = [];
    near = null;
    far = null;
    built = false;
  }
  function reset() {
    phase = "choose";
    openings = [];
    site = null;
    plan = null;
    near = null;
    far = null;
    built = false;
  }
  const siteZ = $derived(near ? groundHeightAt(near, 0, 0) : 0);
  const roughness = $derived(plan?.site.roughness_m ?? 1);
  /** The ground at a point: the fine grid, with decoration below the kernel's spacing inside the walk patch. */
  const groundAt = (x: number, y: number) => {
    const p = near!;
    const g = patchPoint(p, x, y);
    g.kernel_z = g.z;
    if (g.cover === 0 && Math.abs(x) < DETAIL_HALF && Math.abs(y) < DETAIL_HALF) g.z += decoration(x, y, roughness, p.spacing_m) - caveCut(openings, x, y);
    return g;
  };

  // ── Drawing ──────────────────────────────────────────────────────────────────────────────
  let canvas = $state<HTMLCanvasElement | undefined>();
  let line = $state<HTMLCanvasElement | undefined>();
  let host = $state<HTMLElement | undefined>();
  let w = $state(0);
  let h = $state(0);
  let gpu = $state<SurfaceGpu | null>(null);
  let using = $state<"" | "gpu" | "canvas">("");
  let reason = $state("");
  let built = false;
  // The lander drawn on the ground (LR6): what was last set, and for the page's marks whether its lamps are lit.
  let landerSet = "";
  let landerShown = $state<"" | "lit" | "dark">("");
  /** A floodlamp's colour temperature (a warm white LED): a picture choice, fudge register "The lander on the ground". */
  const LAMP_K = 4000;
  // The lander's lamps (LR6): the GM's switch on the walk pad; lit only once the sun is near the horizon.
  let lamps = $state(false);
  let hullLight = 0;
  /** The lit hull's brightness after the eye adapts to it: a mid grey (fudge register, "The lander on the ground"). */
  const HULL_GREY = 0.35;
  $effect(() => {
    if (!canvas || using) return;
    if (String(params.tier ?? "auto") === "canvas") return void ((using = "canvas"), (reason = "canvas tier chosen"));
    void startSurface(canvas).then((g) => {
      if (g) ((gpu = g), (using = "gpu"), (reason = g.backend === "webgpu" ? "WebGPU" : "WebGL2 (no WebGPU here)"));
      else ((using = "canvas"), (reason = "no GPU path on this device"));
    });
  });
  onDestroy(() => {
    cancelAnimationFrame(raf);
    gpu?.dispose();
  });

  // Quality in True (VP4 part 2b, VP5): placed by the device's measured speed on the land-true recipe, stepped down on
  // slow frames, and sharpened while the view holds still (no moving sea, nobody walking): jittered frames averaged in
  // the finish. Once sharp the picture is left on screen and nothing is drawn until something changes.
  let run: TrueRun | null = null;
  let quality = $state("");
  let heldKey = "";
  let lastDrawn = -1;
  const landPixels = () => {
    const dpr = typeof devicePixelRatio === "number" ? devicePixelRatio : 1;
    return Math.max(1, Math.round(w * h * dpr * dpr));
  };
  $effect(() => {
    if (!gpu) return;
    const r = new TrueRun({ recipe: LAND_TRUE, library: LIBRARY, pixels: untrack(landPixels), backend: gpu.backend, source: "render.land" });
    run = r;
    void r.place();
    return () => (r.dispose(), (run = null));
  });
  $effect(() => {
    const px = landPixels();
    untrack(() => run?.update({ pixels: px }));
  });
  function buildMeshes() {
    if (!gpu || !near || !far || built) return;
    built = true;
    const R = world!.radius_m;
    const fine = near.spacing_m / 4;
    const nDetail = Math.min(401, Math.round((2 * DETAIL_HALF) / fine) + 1);
    const nf = far;
    gpu.setMeshes([
      groundMesh("far", nf.n, nf.half_m, NEAR_HALF - 2 * nf.step_m, R, (x, y) => patchPoint(nf, x, y)),
      groundMesh("near", near.n, near.half_m, DETAIL_HALF - 2 * near.step_m, R, (x, y) => patchPoint(near!, x, y), 1),
      groundMesh("detail", nDetail, DETAIL_HALF, 0, R, groundAt, 2),
    ]);
  }

  // The flown descent: the kernel's track laid toward the site from the west (prograde), drawn from where the far
  // ground is in sight; the kernel's clock runs at the true time beside it.
  const track = $derived(plan && near ? descentTrack(plan.descent.telemetry, siteZ, world?.radius_m ?? 1, 90) : []);
  const shownFrom = $derived(Math.max(0, track.findIndex((p) => p.to_go_km * 1000 < FAR_HALF * 0.85)));
  const keys = $derived(track.length ? playbackKeys(track.slice(shownFrom), siteZ) : []);
  let t0 = 0;
  let flyT = $state(0);
  let flyAt = $state(0);

  // Walking: position, facing, and the keys held.
  let walk = $state({ x: 0, y: 0, yaw: Math.PI / 2, pitch: -0.05 });
  let drone = $state(false);
  let hour = $state(-30);
  // The world's own clock (Land part 2): its star's hour and declination at the time being viewed, from its spin and
  // its place on its orbit; the GM can shift the hour from there. Without a known spin, the hour is the GM's choice.
  // `shift` is in the world's own local hours (a 24th of its solar day, 15° of its turn), so 6 is a quarter turn.
  let shift = $state(0);
  const clock = $derived.by(() => {
    if (watching || !world || !site) return null;
    const orb = world.orbiter ? scene!.subjects.find((x) => x.address === world!.orbiter) : undefined;
    return sunAt({ tilt_deg: world.tilt_deg, orbit_phase_deg: orb?.orbit?.phase_deg ?? 0, ...(world.spin_s ? { spin_s: world.spin_s } : {}), ...(world.orbit_s ? { orbit_s: world.orbit_s } : {}), t_s: ctx!.workspace.now_yr * constant("yr_julian"), lon_deg: site.lon });
  });
  // The stars over the site at night (True): the kernel's sky from this system, turned onto the site's sky round the
  // same pole as the sun (night-sky.ts), read once per system. The eye adapts to the light about it, so they show
  // when the sky is dark.
  const STAR_LIMIT = 6.5;
  const SQUARE: SiteSkyFrame = { to_galactic: [[1, 0, 0], [0, 1, 0], [0, 0, 1]] };
  let skyRead = $state<{ system: string; stars: SiteSkyStar[]; frame: SiteSkyFrame } | null>(null);
  $effect(() => {
    const sys = scene?.system;
    if (!sys || !ctx || look !== "true" || skyRead?.system === sys) return;
    untrack(() =>
      void (async () => {
        // From this system's own place when the kernel knows it; otherwise the Sun's sky, laid square on the system's
        // frame (its turn against the galaxy is not on record: fudge register, "Stars over the site at night").
        const ask = { limit_mag: STAR_LIMIT, band: "V", bands: ["B", "V"] };
        let r = await ctx!.session.skyFromHere({ ...ask, observer: sys });
        if (!r.ok) r = await ctx!.session.skyFromHere(ask);
        const sky = r.ok ? (r.sky as { frame?: unknown; stars?: { dir: number[]; luminosity_lsun: number; distance_pc: number; teff_k?: number; components?: { teff_k: number }[] }[] }) : null;
        if (!sky?.stars) return;
        const frame = sky.frame && typeof sky.frame === "object" && Array.isArray((sky.frame as SiteSkyFrame).to_galactic) ? (sky.frame as SiteSkyFrame) : SQUARE;
        skyRead = { system: sys, frame, stars: sky.stars.map((x) => ({ dir: [x.dir[0]!, x.dir[1]!, x.dir[2]!], luminosity_lsun: x.luminosity_lsun, distance_pc: x.distance_pc, teff_k: x.teff_k ?? x.components?.[0]?.teff_k ?? 5800 })) };
      })(),
    );
  });
  const siteStars = $derived.by(() => {
    if (!skyRead || !world || !site || !world.host_lsun || !world.orbit_m) return null;
    const orb = world.orbiter ? scene!.subjects.find((x) => x.address === world!.orbiter) : undefined;
    const list = starsOverSite({ stars: skyRead.stars, frame: skyRead.frame, tilt_deg: world.tilt_deg, orbit_phase_deg: orb?.orbit?.phase_deg ?? 0, sun_hour_deg: sunHour, lat_deg: site.lat, retrograde: (world.spin_s ?? 0) < 0, host_lsun: world.host_lsun, orbit_m: world.orbit_m });
    const dirs = new Float32Array(list.length * 3);
    const light = new Float32Array(list.length * 3);
    list.forEach((x, i) => {
      const c = linear(thermalColour(x.teff_k));
      dirs.set(x.dir, i * 3);
      light.set([c[0] * x.share, c[1] * x.share, c[2] * x.share], i * 3);
    });
    return { key: `${skyRead.system}|${site.lat}|${site.lon}|${sunHour}|${sunDecl}`, dirs, light };
  });
  // The exposure last drawn at, for the page's marks (tests read the eye opening up at night).
  let seenExposure = $state(6);
  // The aurora's plan for this world (aurora.ts), and its words as last drawn. Long exposure, the default in every True
  // view (Alex, 24 Nov), until VP1's shared Exposure control carries the choice here.
  let auroraPlan = $state.raw<AuroraPlan | null>(null);
  let auroraSays = $state("");
  const wrapDeg = (d: number) => ((((d + 180) % 360) + 360) % 360) - 180;
  const sunHour = $derived(watching ? (show?.hour_deg ?? hour) : clock ? wrapDeg(clock.hour_deg + shift * 15) : hour);
  const sunDecl = $derived(watching ? (show?.decl_deg ?? 0) : (clock?.declination_deg ?? 0));
  /** Local solar time as a clock face (12:00 is noon), for the note. */
  const localTime = $derived(`${String(Math.floor((((sunHour / 15 + 12) % 24) + 24) % 24)).padStart(2, "0")}:${String(Math.floor((((((sunHour / 15 + 12) % 24) + 24) % 24) % 1) * 60)).padStart(2, "0")}`);
  /** Local solar hour now, 0 to 24 (12 is noon). */
  const localHour = $derived((((sunHour / 15 + 12) % 24) + 24) % 24);
  // Quick choices of the hour (LR2): the GM can set a landing in daylight with a press. Each moves the world's own clock
  // to the next such hour (or the hand-set hour without a known spin); nothing else about the world changes.
  const TIMES = [
    { id: "dawn", label: "Dawn", local: 6.5 },
    { id: "midday", label: "Midday", local: 12 },
    { id: "dusk", label: "Dusk", local: 17.5 },
    { id: "night", label: "Night", local: 0 },
  ] as const;
  function setLocal(local: number) {
    const target = (local - 12) * 15;
    if (clock) {
      const base = wrapDeg(clock.hour_deg);
      // Forward from now when that fits on the slider, else back.
      let h = ((((target - base) / 15) % 24) + 24) % 24;
      if (h > 12) h -= 24;
      shift = Math.round(h * 4) / 4;
    } else hour = Math.max(-100, Math.min(100, target));
  }
  const nearTime = $derived(TIMES.find((t) => Math.min(Math.abs(localHour - t.local), 24 - Math.abs(localHour - t.local)) < 0.5)?.id ?? null);
  let flyU = 0;

  // ── The table's view of this landing (the landing run-through) ──────────────────────────────
  // The GM's screen keeps the shell's copy up to date, a few times a second at most; nothing reaches the table until
  // the GM sends it ("Send this view"). A seat follows what arrives.
  let sentAt = 0;
  let sentKey = "";
  function landingNow(): LandingShow | null {
    if (!site || !subject || !world) return null;
    const p = phase === "loading" ? "choose" : phase;
    return {
      system,
      body: subject.address,
      label: subject.label,
      site: { lat_deg: site.lat, lon_deg: site.lon, label: site.label },
      lander: JSON.parse(landerKey) as string | Record<string, unknown>,
      phase: p,
      flight_s: flightS,
      ...(p === "flying" ? { fly_u: Math.round(flyU * 1000) / 1000 } : {}),
      ...(p === "down" ? { pose: roundPose({ x_m: walk.x, y_m: walk.y, yaw: walk.yaw, pitch: walk.pitch, drone, lamps }) } : {}),
      sea: seaRead,
      hour_deg: Math.round(sunHour * 100) / 100,
      decl_deg: Math.round(sunDecl * 100) / 100,
      look,
      world: { ...world, air: world.air ? { ...world.air, unknown_gases: [...world.air.unknown_gases] } : null },
    };
  }
  function publish(now: number, force = false) {
    const send = ctx?.workspace.sendLanding;
    if (!send) return;
    const l = landingNow();
    if (!l) return;
    // The flight's progress alone moves every frame; it is worth sending only every second or so.
    const { fly_u, ...rest } = l;
    const key = JSON.stringify(rest);
    if (!force && key === sentKey && (fly_u === undefined || now - sentAt < 1000)) return;
    sentKey = key;
    sentAt = now;
    // Plain data (no reactive proxies), as the wire and structuredClone want it.
    send(JSON.parse(JSON.stringify(l)) as Json);
  }
  // The choosing moments (a site, a lander, landing, taking off) go at once; the pose goes from the frame loop.
  $effect(() => {
    if (watching) return;
    void [site, landerKey, phase, world, sunHour, sunDecl, drone, look, seaRead];
    untrack(() => publish(performance.now(), true));
  });

  // A seat: follow the GM's site, lander, moment and pose.
  $effect(() => {
    const l = show;
    if (!watching || !l) return;
    untrack(() => {
      if (!site || site.lat !== l.site.lat_deg || site.lon !== l.site.lon_deg) {
        site = { lat: l.site.lat_deg, lon: l.site.lon_deg, label: l.site.label };
        if (phase !== "choose") takeOff();
      }
      if (l.pose) {
        walk = { x: l.pose.x_m, y: l.pose.y_m, yaw: l.pose.yaw, pitch: l.pose.pitch };
        drone = l.pose.drone;
        lamps = !!l.pose.lamps;
      }
      if (l.phase === "choose" && (phase === "flying" || phase === "down")) takeOff();
      // Keep pace with the GM's flight: a second or more apart, jump to where the GM is.
      if (l.phase === "flying" && phase === "flying" && l.fly_u !== undefined && t0 && Math.abs(flyU - l.fly_u) * flightS > 1) t0 = performance.now() - l.fly_u * flightS * 1000;
      if (l.phase === "down" && phase === "flying") toGround(l.pose);
    });
  });
  // Land once the seat's own descent is planned (it plans from the same site and lander as the GM's).
  $effect(() => {
    if (!watching || !show || !plan || !world || !tiles) return;
    const want = show.phase;
    untrack(() => {
      if ((want === "flying" || want === "down") && phase === "choose") void land().then(() => (want === "down" || show?.phase === "down") && phase === "flying" && toGround(show?.pose));
    });
  });
  const held = new Set<string>();
  let raf = 0;
  let last = 0;
  const tok = (name: string) => (host ? getComputedStyle(host).getPropertyValue(name).trim() || getComputedStyle(host).color : "");

  function linear(hex: string): [number, number, number] {
    const n = parseInt(hex.replace("#", ""), 16);
    const c = [(n >> 16) & 255, (n >> 8) & 255, n & 255].map((v) => Math.pow(v / 255, 2.2));
    const m = Math.max(...c, 1e-6);
    return [c[0]! / m, c[1]! / m, c[2]! / m];
  }

  let eyeNow = $state<[number, number, number]>([0, 0, 0]);
  let headingNow = $state(90);
  function frame(now: number) {
    raf = requestAnimationFrame(frame);
    const dt = last ? Math.min(0.1, (now - last) / 1000) : 0;
    last = now;
    if (!near || !world || w === 0 || h === 0 || (phase !== "flying" && phase !== "down")) return;
    let eye: [number, number, number];
    let fwd: [number, number, number];
    if (phase === "flying" && track.length) {
      if (!t0) t0 = now;
      const vis = track.slice(shownFrom);
      const u = Math.min(1, (now - t0) / 1000 / flightS);
      let i = keys.findIndex((k) => k >= u);
      if (i < 1) i = Math.min(1, vis.length - 1);
      const a = vis[i - 1] ?? vis[0]!;
      const b = vis[i]!;
      const ka = keys[i - 1] ?? 0;
      const kb = keys[i] ?? 1;
      const f = kb > ka ? Math.min(1, Math.max(0, (u - ka) / (kb - ka))) : 1;
      const tt = a.t_s + (b.t_s - a.t_s) * f;
      const p: [number, number, number] = [a.x + (b.x - a.x) * f, a.y + (b.y - a.y) * f, a.z + (b.z - a.z) * f];
      const ground = groundHeightAt(near, Math.max(-NEAR_HALF, Math.min(NEAR_HALF, p[0])), Math.max(-NEAR_HALF, Math.min(NEAR_HALF, p[1])));
      p[2] = Math.max(p[2], ground + 3);
      eye = p;
      // Look toward the site from high up, ahead along the track as the ground comes near.
      const alt = p[2] - siteZ;
      const toSite = [-p[0], -p[1], siteZ - p[2]];
      const l = Math.hypot(toSite[0]!, toSite[1]!, toSite[2]!) || 1;
      const s = Math.min(1, Math.max(0, 1 - alt / 3000));
      fwd = [(toSite[0]! / l) * (1 - s) + s, (toSite[1]! / l) * (1 - s), (toSite[2]! / l) * (1 - s) - 0.08 * s];
      flyT = tt;
      flyAt = i;
      flyU = u;
      if ((now - t0) / 1000 >= flightS) toGround();
    } else if (watching) {
      const gz = groundAt(walk.x, walk.y).z;
      eye = [walk.x, walk.y, gz + (drone ? 40 : eyeM)];
      fwd = [Math.cos(walk.pitch) * Math.cos(walk.yaw), Math.cos(walk.pitch) * Math.sin(walk.yaw), Math.sin(walk.pitch)];
    } else {
      const speed = (drone ? 14 : 1.4) * (held.has("Shift") ? 4 : 1);
      const turn = 1.4 * dt;
      if (held.has("ArrowLeft") || held.has("a")) walk.yaw += turn;
      if (held.has("ArrowRight") || held.has("d")) walk.yaw -= turn;
      const go = (held.has("ArrowUp") || held.has("w") ? 1 : 0) - (held.has("ArrowDown") || held.has("s") ? 1 : 0);
      if (go) {
        walk.x = Math.max(-WALK_HALF, Math.min(WALK_HALF, walk.x + Math.cos(walk.yaw) * go * speed * dt));
        walk.y = Math.max(-WALK_HALF, Math.min(WALK_HALF, walk.y + Math.sin(walk.yaw) * go * speed * dt));
      }
      const gz = groundAt(walk.x, walk.y).z;
      eye = [walk.x, walk.y, gz + (drone ? 40 : eyeM)];
      fwd = [Math.cos(walk.pitch) * Math.cos(walk.yaw), Math.cos(walk.pitch) * Math.sin(walk.yaw), Math.sin(walk.pitch)];
    }
    eyeNow = eye;
    if (!watching && now - sentAt > 250) publish(now);
    headingNow = ((90 - (Math.atan2(fwd[1], fwd[0]) * 180) / Math.PI) % 360 + 360) % 360;
    const sun = sunDirection(site!.lat, sunHour, sunDecl);
    const air = world.air;
    if (using === "gpu" && gpu) {
      buildMeshes();
      // The lander where it touched down (LR6), on the ground there; its lamps lit once the sun is near the horizon.
      const lights = lamps && sun[2] < 0.1;
      const lk = phase === "down" && landerH > 0 ? `${landerH}|${lights}` : "";
      if (lk !== landerSet) {
        landerSet = lk;
        const lamp = lights ? lampRadiance(landerH, world.host_lsun ?? 0, world.orbit_m ?? 0) : 0;
        const c = linear(thermalColour(LAMP_K));
        gpu.setLander(lk ? { at_m: [0, 0], ground_m: groundAt(0, 0).z, heading_deg: 0, height_m: landerH, lights: lamp > 0, lamp: [c[0] * lamp, c[1] * lamp, c[2] * lamp] } : null);
        landerShown = lk ? (lamp > 0 ? "lit" : "dark") : "";
        // A lit hull is the brightest large thing in view: the eye adapts to it (and the stars fade, as they do).
        hullLight = lamp > 0 ? (LANDER_HULL_ALBEDO * lamp * LAMP_SPILL) / Math.PI : 0;
      }
      // What the picture depends on: while it is the same, the view rests and sharpens; any change draws it moving.
      const r = look === "true" ? run : null;
      const key = r ? [look, w, h, ...eye.map((x) => Math.round(x * 1e3)), ...fwd.map((x) => Math.round(x * 1e5)), ...sun.map((x) => Math.round(x * 1e5)), seaNow ? now : 0, landerSet, siteStars?.key ?? "", lamps].join("|") : "";
      const still = r !== null && key === heldKey;
      heldKey = key;
      const step = still ? r!.rest.next(now) : null;
      // Sharp: the averaged picture stays on screen.
      if (still && !step) return;
      const q = r?.settings(step !== null) ?? null;
      const fin = q?.params["finish"] ?? {};
      const t0 = performance.now();
      gpu.draw({
        ...(q ? { quality: { ...(typeof fin["glare_rungs"] === "number" ? { rungs: fin["glare_rungs"] } : {}), ...(typeof fin["glare_scale"] === "number" ? { glare_scale: fin["glare_scale"] } : {}), render_scale: q.render_scale, ...(step ? { accumulate: step.k, jitter: step.jitter } : {}) } } : {}),
        look,
        eye,
        forward: fwd,
        up: [0, 0, 1],
        fovY: (60 * Math.PI) / 180,
        width: w,
        height: h,
        sun,
        sunRgb: linear(thermalColour(world.star_k)),
        tau: air ? air.zenith_depth : [0, 0, 0],
        scale_height_m: air?.scale_height_m ?? 8000,
        radius_m: world.radius_m,
        sun_radius: world.sun_radius,
        // The eye adapts: a dark sky opens it up until the stars show (night-sky.ts).
        exposure: (seenExposure = look === "true" ? Math.min(adaptedExposure(6, { sun_z: sun[2], tau_green: air ? air.zenith_depth[1] : 0, radius_m: world.radius_m, scale_height_m: air?.scale_height_m ?? 8000 }), hullLight > 0 ? HULL_GREY / hullLight : Infinity) : 6),
        ...(look === "true" && siteStars ? { stars: siteStars } : {}),
        ...(look === "true" && auroraPlan?.layers.length && world.host_lsun && world.orbit_m ? { aurora: skyAurora(auroraPlan, world.host_lsun, world.orbit_m) } : {}),
        ground: tok("--sse-ground"),
        ink: tok("--sse-ink"),
        contour_m: contourFor(eye[2] - siteZ),
        ...(seaNow ? { sea: { wave_m: seaNow.wave_m, period_s: seaNow.period_s, dir_rad: seaNow.wind_m_s < 0 ? Math.PI : 0, gravity_m_s2: world.gravity_m_s2 ?? plan?.gravity_m_s2 ?? 9.8, tide_m: seaNow.tide_m, time_s: now / 1000, ...(seaOptics ? { optics: seaOptics.optics } : {}) } } : {}),
      });
      if (r) {
        const work = performance.now() - t0;
        if (step) r.rest.done(performance.now(), work);
        else {
          // Frames running on: the time between them; a lone frame: its own work.
          const ms = lastDrawn >= 0 && now - lastDrawn < 250 ? Math.max(work, now - lastDrawn) : work;
          r.frame(ms, now);
        }
        lastDrawn = now;
        const st = r.status();
        quality = `${st.tier}|${st.level}|${st.rest}|${st.rest_frames}`;
      } else if (quality) quality = "";
    } else if (using === "canvas" && line) drawLines(line, eye, fwd);
  }
  $effect(() => {
    if (phase === "flying" || phase === "down") {
      cancelAnimationFrame(raf);
      last = 0;
      raf = requestAnimationFrame(frame);
    }
  });
  /** The aurora for this frame: its plan at the site, lifted for the view's exposure, and its words kept for the note. */
  function skyAurora(p: AuroraPlan, host_lsun: number, orbit_m: number) {
    const flux = (host_lsun * constant("L_sun")) / (4 * Math.PI * orbit_m * orbit_m);
    const a = auroraSky(p, world!.radius_m, { lat_deg: site!.lat, lon_deg: site!.lon }, "long", flux, seenExposure);
    const words = a ? `${p.note} ${a.says}` : "";
    if (words !== auroraSays) auroraSays = words;
    return a;
  }
  const contourFor = (alt: number) => (alt > 20000 ? 500 : alt > 3000 ? 100 : alt > 300 ? 20 : 10);

  /** No GPU: the ground ahead as lines of its profile (the line-terrain style), nearest drawn last. */
  function drawLines(c: HTMLCanvasElement, eye: [number, number, number], fwd: [number, number, number]) {
    const dpr = typeof devicePixelRatio === "number" ? devicePixelRatio : 1;
    c.width = Math.round(w * dpr);
    c.height = Math.round(h * dpr);
    const g = c.getContext("2d")!;
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    const bg = tok("--sse-ground");
    g.fillStyle = bg;
    g.fillRect(0, 0, w, h);
    const fl = Math.hypot(fwd[0], fwd[1]) || 1;
    const fx = fwd[0] / fl;
    const fy = fwd[1] / fl;
    const focal = h / 2 / Math.tan(Math.PI / 6);
    const proj = (x: number, y: number, z: number) => {
      const dx = x - eye[0];
      const dy = y - eye[1];
      const along = dx * fx + dy * fy;
      const side = dx * fy - dy * fx;
      if (along < 1) return null;
      return { sx: w / 2 + (side / along) * focal, sy: h / 2 - ((z - eye[2]) / along) * focal + fwd[2] * focal };
    };
    const p = near!;
    g.strokeStyle = tok("--sse-ink");
    g.lineWidth = 1;
    const rows = 48;
    for (let r = rows; r >= 1; r--) {
      const d = Math.pow(r / rows, 2) * NEAR_HALF;
      g.beginPath();
      let started = false;
      for (let s = -40; s <= 40; s++) {
        const side = (s / 40) * d * 0.9;
        const x = eye[0] + fx * d + fy * side;
        const y = eye[1] + fy * d - fx * side;
        if (Math.abs(x) > p.half_m || Math.abs(y) > p.half_m) continue;
        const q = proj(x, y, groundHeightAt(p, x, y) - (d * d) / (2 * world!.radius_m));
        if (!q) continue;
        if (started) g.lineTo(q.sx, q.sy);
        else ((started = true), g.moveTo(q.sx, q.sy));
      }
      g.lineTo(w, h);
      g.lineTo(0, h);
      g.closePath();
      g.fillStyle = bg;
      g.fill();
      g.stroke();
    }
  }

  // Look about: drag turns the head (or the drone); keys walk.
  let drag: { x: number; y: number } | null = null;
  function down(e: PointerEvent) {
    if ((e.target as HTMLElement).closest("button, input, label, select, details")) return;
    drag = { x: e.clientX, y: e.clientY };
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    (e.currentTarget as HTMLElement).focus();
  }
  function move(e: PointerEvent) {
    if (!drag || phase !== "down" || watching) return;
    walk.yaw -= (e.clientX - drag.x) * 0.004;
    walk.pitch = Math.max(-1.4, Math.min(1.4, walk.pitch - (e.clientY - drag.y) * 0.004));
    drag = { x: e.clientX, y: e.clientY };
  }
  const up = () => (drag = null);
  const keyDown = (e: KeyboardEvent) => {
    if (watching) return;
    if (["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "w", "a", "s", "d", "Shift"].includes(e.key)) {
      held.add(e.key);
      e.preventDefault();
    }
  };
  const keyUp = (e: KeyboardEvent) => held.delete(e.key);
  function press(k: string, on: boolean) {
    if (on) held.add(k);
    else held.delete(k);
  }

  const events = $derived(plan ? plan.descent.events.filter((e) => phase !== "flying" || e.t_s <= flyT) : []);
  const tel = $derived(plan && phase === "flying" ? plan.descent.telemetry[Math.min(plan.descent.telemetry.length - 1, shownFrom + flyAt)] : null);
  const fromSite = $derived(Math.hypot(eyeNow[0], eyeNow[1]));
  const horizon = $derived(world ? horizonDistance(Math.max(0.1, eyeNow[2] - (near ? groundAt(eyeNow[0], eyeNow[1]).z : 0)), world.radius_m) : 0);
  const DIRS = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
  const compass = $derived(DIRS[Math.round(headingNow / 45) % 8]);
  /** The nearest lava tube opening, while one lies within the near ground. */
  const caveLine = $derived(openingLine(nearestOpening(openings, eyeNow[0], eyeNow[1])));
  /** Landmarks near enough to name over the ground, with their bearing. */
  const nearMarks = $derived(
    site && world
      ? landmarks
          .map((l) => {
            const la = (site!.lat * Math.PI) / 180;
            const dl = ((l.lon_deg - site!.lon) * Math.PI) / 180;
            const lb = (l.lat_deg * Math.PI) / 180;
            const c = Math.sin(la) * Math.sin(lb) + Math.cos(la) * Math.cos(lb) * Math.cos(dl);
            const d = Math.acos(Math.min(1, Math.max(-1, c))) * world!.radius_m;
            const brg = ((Math.atan2(Math.sin(dl) * Math.cos(lb), Math.cos(la) * Math.sin(lb) - Math.sin(la) * Math.cos(lb) * Math.cos(dl)) * 180) / Math.PI + 360) % 360;
            return { l, d, brg };
          })
          .filter((m) => m.d < 300000)
          .slice(0, 6)
      : [],
  );
</script>

{#snippet hourControls()}
  <span class="time-presets" role="group" aria-label="Time of day">
    {#each TIMES as t (t.id)}<button type="button" aria-pressed={nearTime === t.id} onclick={() => setLocal(t.local)} data-time={t.id}>{t.label}</button>{/each}
  </span>
  {#if clock}<label>Local hours from now <input type="range" min="-12" max="12" step="0.25" bind:value={shift} data-hour /> <span data-local-time>{localTime}</span></label>{:else}<label>Time of day <input type="range" min="-100" max="100" step="1" bind:value={hour} data-hour /> <span data-local-time>{localTime}</span></label>{/if}
{/snippet}

<section class="surface" bind:this={host} data-surface data-watching={watching || undefined} data-phase={phase} data-look={look} data-hud={hud ? "on" : "off"} data-tier={using} data-site={site ? `${site.lat.toFixed(2)},${site.lon.toFixed(2)}` : ""} data-verdict={plan?.verdict ?? ""} data-walk={phase === "down" ? `${eyeNow[0].toFixed(1)},${eyeNow[1].toFixed(1)}` : ""} data-floor-m={near ? near.spacing_m.toFixed(0) : ""} data-stars={siteStars ? siteStars.dirs.length / 3 : 0} data-exposure={Math.round(seenExposure)} data-lander-drawn={landerShown || undefined}>
  {#if watching && !show}
    <p class="quiet" data-surface-waiting>Waiting for the game master to send a landing.</p>
  {:else if !subject || subject.kind === "star"}
    <p class="quiet">Choose a world to land on.</p>
  {:else if failed}
    <p class="quiet" data-surface-failed>{failed}</p>
  {:else if !tiles}
    <p class="quiet">Reading {subject.label}'s surface…</p>
  {:else if phase === "choose" || phase === "loading"}
    <div class="choose">
      <div class="map" role="button" tabindex="0" aria-label="Press anywhere on {subject.label} to land there" onclick={pickMap} onkeydown={() => {}} data-site-map>
        <canvas bind:this={mapCanvas}></canvas>
        {#each pins as l (l.id)}
          {@const p = toMap(l.lat_deg, l.lon_deg)}
          <button class="pin" class:remarkable={l.remarkable.length > 0} style:left="{p.x}%" style:top="{p.y}%" title="{l.name ?? l.original_name}: {l.kind}" onclick={(e) => (e.stopPropagation(), pickLandmark(l))} data-landmark={l.id}></button>
        {/each}
        {#if site}
          {@const p = toMap(site.lat, site.lon)}
          <span class="site" style:left="{p.x}%" style:top="{p.y}%" data-site-mark></span>
        {/if}
      </div>
      <aside class="card">
        <h3>{watching ? `Landing on ${subject.label}` : `Where to land on ${subject.label}`}</h3>
        {#if watching}<p class="quiet">The game master has chosen where to land; this screen follows.</p>{:else}<p class="quiet">Pick a landmark, or press anywhere on the map. Every spot is allowed; its dangers are shown.</p>{/if}
        {#if !watching}
        <ul class="marks">
          {#each landmarks.slice(0, 12) as l (l.id)}
            <li><button onclick={() => pickLandmark(l)} class:on={site?.label === (l.name ?? l.original_name)} data-landmark-row={l.id}>{l.name ?? l.original_name}<span class="quiet"> {l.kind}{l.remarkable.length ? `, ${l.remarkable[0]}` : ""}</span></button></li>
          {/each}
          {#if !landmarks.length}<li class="quiet">Reading the landscape's names…</li>{/if}
        </ul>
        {/if}
        {#if site}
          <div class="verdict" data-verdict-card data-worst={worst}>
            <h4>{site.label || "This spot"} <span class="quiet">{fmtLat(site.lat)}, {fmtLon(site.lon)}</span></h4>
            {#if watching}
              <p class="quiet" data-lander-shown>Lander: {typeof show?.lander === "string" ? (LANDER_WORDS[show.lander as keyof typeof LANDER_WORDS] ?? show.lander) : "the party's ship"}</p>
            {:else}
            <label class="lander">Lander
              <select bind:value={landerPick} data-lander>
                {#if partyHere}<option value="party">{partyHere.name} (the party's ship)</option>{/if}
                {#each STOCK_LANDERS as id (id)}<option value={id}>{LANDER_WORDS[id]}</option>{/each}
              </select>
            </label>
            {/if}
            {#if planning}<p class="quiet">Planning the descent…</p>{/if}
            {#if planFailed}<p class="quiet">{planFailed}</p>{/if}
            {#if plan}
              <p class="says {worst}"><strong>{DESCENT_VERDICT_WORDS[plan.verdict] ?? plan.verdict}</strong>: {plan.descent.outcome.method.replace(/-/g, " ")}, {fmtT(plan.descent.outcome.duration_s)} from {fmtM(plan.entry.altitude_m)} up, peak {plan.descent.outcome.peak_g.toFixed(1)} g.</p>
              <ul class="hazards">
                {#each plan.hazards as z (z.kind)}<li class={z.level} data-hazard={z.kind} data-level={z.level}><span class="lv">{z.level}</span> {z.kind}: {z.why}</li>{/each}
              </ul>
              <details><summary>How the site was read</summary><ul>{#each plan.site.why as y, i (i)}<li>{y}</li>{/each}{#each plan.descent.outcome.why as y, i (i)}<li>{y}</li>{/each}</ul></details>
              {#if !watching}<div class="times" data-times>{@render hourControls()}</div>{/if}
              {#if watching}<p class="quiet" data-watch-phase>{phase === "loading" ? "Reading the ground…" : "Waiting for the game master to land."}</p>{:else}<button class="land" onclick={land} disabled={phase === "loading"} data-land>{phase === "loading" ? "Reading the ground…" : worst === "danger" ? "Land anyway" : "Land"}</button>{/if}
              {#if groundFailed}<p class="quiet">{groundFailed}</p>{/if}
            {/if}
          </div>
        {/if}
      </aside>
    </div>
  {/if}
  {#if tiles}
    <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
    <div class="stage" class:hidden={!(phase === "flying" || phase === "down")} bind:clientWidth={w} bind:clientHeight={h} role="application" aria-label={watching ? `The ground at ${site ? fmtLat(site.lat) : ""}, as the game master sees it` : `The ground at ${site ? fmtLat(site.lat) : ""}: drag to look about, arrow keys or W A S D to walk`} tabindex="0" onpointerdown={down} onpointermove={move} onpointerup={up} onpointercancel={up} onkeydown={keyDown} onkeyup={keyUp} data-ground>
      <canvas bind:this={canvas} class:hidden={using === "canvas"} data-quality={quality || undefined}></canvas>
      <canvas bind:this={line} class:hidden={using !== "canvas"}></canvas>
      {#if hud}
        <div class="hud" data-surface-hud>
          <span>{compass} {Math.round(headingNow)}°</span>
          <span>{fmtM(eyeNow[2] - siteZ)} above the site</span>
          <span>{fmtM(fromSite)} from it</span>
          <span>horizon {fmtM(horizon)}</span>
          {#if seaNow}<span data-sea>waves {seaNow.wave_m.toFixed(1)} m every {Math.round(seaNow.period_s)} s · tide {seaNow.tide_m >= 0 ? "+" : ""}{seaNow.tide_m.toFixed(2)} m</span>{/if}
          {#if caveLine}<span data-cave-opening>{caveLine}</span>{/if}
          {#each nearMarks as m (m.l.id)}<span class="quiet">{m.l.name ?? m.l.original_name} {fmtM(m.d)} at {Math.round(m.brg)}°</span>{/each}
        </div>
      {/if}
      {#if phase === "flying" && plan}
        <div class="log" data-descent-log>
          <p class="clock">T+{fmtT(flyT)} {#if tel}· {fmtM(tel.altitude_m)} · {Math.round(tel.speed_m_s)} m/s · {tel.phase}{/if}</p>
          {#each events as e (e.t_s + e.kind)}<p>T+{fmtT(e.t_s)} {e.text}</p>{/each}
          {#if !watching}<button onclick={skip} data-skip>Skip to the ground</button>{/if}
        </div>
      {:else if watching}
        <p class="walk quiet" data-following>Following the game master on the ground{drone ? " (drone)" : ""}.</p>
      {:else}
        <div class="walk" data-walk-pad>
          <button onpointerdown={() => press("ArrowLeft", true)} onpointerup={() => press("ArrowLeft", false)} onpointerleave={() => press("ArrowLeft", false)} aria-label="Turn left">◀</button>
          <button onpointerdown={() => press("ArrowUp", true)} onpointerup={() => press("ArrowUp", false)} onpointerleave={() => press("ArrowUp", false)} aria-label="Walk forward" data-walk-forward>▲</button>
          <button onpointerdown={() => press("ArrowDown", true)} onpointerup={() => press("ArrowDown", false)} onpointerleave={() => press("ArrowDown", false)} aria-label="Walk back">▼</button>
          <button onpointerdown={() => press("ArrowRight", true)} onpointerup={() => press("ArrowRight", false)} onpointerleave={() => press("ArrowRight", false)} aria-label="Turn right">▶</button>
          <label><input type="checkbox" bind:checked={drone} data-drone /> Drone</label>
          <label title="The lander's floodlamps, lit once the sun is down"><input type="checkbox" bind:checked={lamps} data-lamps /> Lamps</label>
          {@render hourControls()}
          <button onclick={takeOff} data-take-off>Take off</button>
        </div>
      {/if}
    </div>
    {#if phase === "flying" || phase === "down"}
    <p class="note quiet" data-ground-note>
      The ground is the kernel's to {near ? Math.round(near.spacing_m) : "?"} m (its fact floor here); finer than that within {fmtM(DETAIL_HALF)} of the site it is decoration sized to the site's roughness. You can walk {fmtM(WALK_HALF)} from where you landed; further means taking off.
      {#if openings.length}{openings.length} stretch{openings.length === 1 ? "" : "es"} of fallen lava tube roof lie within 25 km (the kernel's cave survey): within {fmtM(DETAIL_HALF)} each is cut down to the tube's floor; going inside waits for interior sites.{/if}
      {#if world && !world.air}No air: the sky is black and nothing dims the far ground.{:else}Sky from single scattering in the kernel's air optics.{/if}
      {#if look === "true" && auroraSays}<span data-aurora-sky>{auroraSays}</span>{/if}
      {#if seaNow}{seaNow.line ?? ""}{seaNow.line ? "." : ""} {seaOptics ? `${seaWords(seaOptics.optics, seaOptics.missing)} ` : ""}{seaOptics?.optics.trains ? `The sea is drawn as the kernel's own ${seaOptics.optics.trains.length} wave trains for this liquid` : "The sea is drawn as a few wave trains of that height and period round the wind"}{seaOptics?.optics.whitecap_fraction ? `, white with froth over ${Math.round(seaOptics.optics.whitecap_fraction * 100)}% of it` : ""}; the tide stands {Math.abs(seaNow.tide_m).toFixed(2)} m {seaNow.tide_m >= 0 ? "above" : "below"} mean sea level.{/if}
      {#if watching}The sun stands where the game master's clock puts it, local time {localTime}.{:else if clock}The sun stands where {subject?.label}'s spin and its place on its orbit put it at the time in view: local time {localTime}{clock.solar_day_s === Infinity ? " (it keeps one face to its star)" : ""}, declination {sunDecl.toFixed(1)}°{shift ? `, moved ${shift > 0 ? "+" : ""}${shift} local h by hand` : ""}.{:else}The world's spin is not on record, so the hour is chosen by hand and the sun stands as at an equinox.{/if} Drawn on {reason}.
    </p>
    {/if}
  {/if}
</section>

<style>
  .surface {
    display: grid;
    gap: var(--sse-space-2, 8px);
    color: var(--sse-ink);
  }
  .choose {
    display: grid;
    grid-template-columns: minmax(0, 2fr) minmax(220px, 1fr);
    gap: var(--sse-space-3, 12px);
    align-items: start;
  }
  @media (max-width: 760px) {
    .choose {
      grid-template-columns: 1fr;
    }
  }
  .map {
    position: relative;
    cursor: crosshair;
    border: 1px solid var(--sse-ink-faint, var(--sse-ink));
    line-height: 0;
  }
  .map canvas {
    width: 100%;
    height: auto;
    image-rendering: auto;
  }
  .pin {
    position: absolute;
    width: 9px;
    height: 9px;
    margin: -5px 0 0 -5px;
    border-radius: 50%;
    border: 1px solid var(--sse-ground);
    background: var(--sse-ink);
    padding: 0;
    cursor: pointer;
  }
  .pin.remarkable {
    background: var(--sse-accent);
  }
  .site {
    position: absolute;
    width: 16px;
    height: 16px;
    margin: -8px 0 0 -8px;
    border: 2px solid var(--sse-accent);
    border-radius: 50%;
    pointer-events: none;
  }
  .card h3,
  .card h4 {
    margin: 0 0 4px;
  }
  .marks {
    list-style: none;
    padding: 0;
    margin: 0 0 8px;
    max-height: 220px;
    overflow: auto;
  }
  .marks button {
    background: none;
    border: none;
    color: inherit;
    text-align: left;
    padding: 2px 0;
    cursor: pointer;
  }
  .marks button.on {
    color: var(--sse-accent);
  }
  .hazards {
    list-style: none;
    padding: 0;
    margin: 4px 0;
  }
  .lv {
    display: inline-block;
    min-width: 4.5em;
    font-size: 0.8em;
    text-transform: uppercase;
    color: var(--sse-ink-muted, var(--sse-ink));
  }
  .hazards .danger .lv,
  .says.danger strong {
    color: var(--sse-danger, var(--sse-accent));
  }
  .hazards .caution .lv {
    color: var(--sse-caution, var(--sse-accent));
  }
  .land {
    margin-top: 6px;
  }
  .stage {
    position: relative;
    height: min(64vh, 640px);
    outline: none;
    touch-action: none;
  }
  .stage canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }
  .hidden {
    display: none;
  }
  .hud {
    position: absolute;
    left: 8px;
    bottom: 52px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 0.85em;
    color: var(--sse-ink);
    background: color-mix(in srgb, var(--sse-ground) 70%, transparent);
    padding: 4px 8px;
    pointer-events: none;
  }
  .log {
    position: absolute;
    right: 8px;
    bottom: 8px;
    width: min(300px, 45%);
    max-height: calc(100% - 16px);
    overflow: auto;
    font-size: 0.85em;
    background: color-mix(in srgb, var(--sse-ground) 75%, transparent);
    padding: 6px 8px;
  }
  .log p {
    margin: 0 0 3px;
  }
  .clock {
    font-variant-numeric: tabular-nums;
  }
  .walk {
    position: absolute;
    left: 8px;
    bottom: 8px;
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
    background: color-mix(in srgb, var(--sse-ground) 75%, transparent);
    padding: 4px 8px;
  }
  .note {
    font-size: 0.85em;
    margin: 0;
  }
  .quiet {
    color: var(--sse-ink-muted, var(--sse-ink));
  }
  .times {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 6px;
    align-items: center;
    margin: 6px 0;
  }
  .time-presets {
    display: inline-flex;
    gap: 2px;
  }
  .time-presets button[aria-pressed="true"] {
    background: var(--sse-ink);
    color: var(--sse-ground);
  }
</style>
