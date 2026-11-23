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
-->
<script module lang="ts">
  import type { Landmark, SurfaceTile } from "../../../ui-contract/src/index.ts";
  // The world's map tiles (a second or so) and its landmarks, kept per world for the page's life.
  const globes = new Map<string, SurfaceTile[]>();
  const marks = new Map<string, Landmark[]>();
</script>

<script lang="ts">
  import { onDestroy, untrack } from "svelte";
  import { constant, DEFAULT_VIEW, DESCENT_VERDICT_WORDS, LANDER_WORDS, landmarkOrder, sampleAt, sampleColour, STOCK_LANDERS, suggestLander, thermalColour, viewOptions, type DescentPlan, type Json, type StockLander, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { airFactsOf, airFromOptics, airRequest, decoration, descentTrack, floorFromRefusal, playbackKeys, groundHeightAt, groundMesh, horizonDistance, levelFor, patchKeys, patchPoint, readPatch, startSurface, sunDirection, type Air, type Patch, type SurfaceGpu } from "../../../renderer/src/index.ts";
  import { SURFACE_CONTROLS } from "./controls.ts";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();

  const ws = $derived(ctx.workspace);
  const subject = $derived(scene.subjects.find((s) => s.address === ws.selection));
  const shown = $derived(viewOptions(SURFACE_CONTROLS, ws.view ?? DEFAULT_VIEW).shown);
  // On the ground Chart draws as 3D: the chart is the map you choose on.
  const look = $derived<"true" | "3d" | "wireframe">(shown.style === "true" ? "true" : shown.style === "wireframe" ? "wireframe" : "3d");
  const hud = $derived(shown.hud);
  const flightS = $derived(Math.min(120, Math.max(6, Number(params.flight_s ?? 24))));
  const eyeM = $derived(Math.min(3, Math.max(0.3, Number(params.eye_m ?? 1.7))));

  // ── The world: map tiles, landmarks, air, star ─────────────────────────────────────────────
  type World = { radius_m: number; pressure_bar: number; t_k: number; air: Air | null; star_k: number; sun_radius: number; tilt_deg: number };
  let tiles = $state<SurfaceTile[] | null>(null);
  let landmarks = $state<Landmark[]>([]);
  let world = $state<World | null>(null);
  let failed = $state<string | null>(null);
  let gen = 0;
  $effect(() => {
    const sub = subject;
    const sys = scene.system;
    const g = ++gen;
    untrack(() => reset());
    tiles = null;
    landmarks = [];
    world = null;
    failed = null;
    if (!sub || sub.kind === "star") return;
    const key = `${sys}|${sub.address}`;
    (async () => {
      const hit = globes.get(key);
      const t = hit ? { ok: true as const, tiles: hit } : await ctx.session.surfaceTiles(sys, sub.address);
      if (g !== gen) return;
      if (!t.ok) return void (failed = t.message);
      if (!t.tiles.length) return void (failed = `${sub.label} has no surface the kernel has built`);
      globes.set(key, t.tiles);
      tiles = t.tiles;
      const rs = await ctx.session.resolvedSystem(sys);
      if (g !== gen || !rs.ok) return;
      const facts = rs.result.facts as Record<string, Record<string, Json>>;
      const f = facts[sub.address] ?? {};
      const surf = rs.result.surfaces.find((x) => x.address === sub.address) as { gravity_m_s2?: number; t_surface_now_k?: number; reference_radius_m?: number } | undefined;
      const radius = Number(f["body.radius"] ?? surf?.reference_radius_m ?? 0) || (t.tiles[0]!.spacing_m * t.tiles[0]!.samples * 2) / Math.PI;
      // The star: up the chain of hosts to the one with a temperature; its disc's size from its radius and distance.
      let host = f["parent"] as string | undefined;
      let dist = Number(f["orbit.semi_major_axis"] ?? 0);
      let star_k = 0;
      let star_r = 0;
      for (let i = 0; host && i < 6; i++) {
        const h = facts[host] ?? {};
        if (typeof h["star.effective_temperature"] === "number") {
          star_k = h["star.effective_temperature"] as number;
          star_r = Number(h["body.radius"] ?? 0);
          break;
        }
        dist = Number(h["orbit.semi_major_axis"] ?? dist);
        host = h["parent"] as string | undefined;
      }
      const af = airFactsOf(f, surf);
      let air: Air | null = null;
      const req = af ? airRequest(sub.address, af) : null;
      if (af && req) {
        const ao = await ctx.session.readAirOptics([req.air]);
        if (g !== gen) return;
        air = ao.ok && ao.airs[0] ? airFromOptics(ao.airs[0] as Parameters<typeof airFromOptics>[0], req) : null;
      }
      world = {
        radius_m: radius,
        pressure_bar: Number(f["body.surface_pressure"] ?? 0) / 1e5,
        t_k: surf?.t_surface_now_k ?? 0,
        air,
        star_k: star_k || constant("T_sun"),
        sun_radius: star_r > 0 && dist > 0 ? star_r / dist : 0.0047,
        tilt_deg: (Number(f["body.axial_tilt"] ?? 0) * 180) / Math.PI,
      };
      lander = suggestLander(world.pressure_bar, world.t_k);
      const lm: Landmark[] = marks.get(key) ?? (await ctx.session.surfaceLandmarks(sys, sub.address).then((r) => (r.ok ? r.landmarks : [])));
      if (g !== gen) return;
      marks.set(key, lm);
      landmarks = landmarkOrder(lm);
    })();
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
  let lander = $state<StockLander>("capsule");
  let plan = $state<DescentPlan | null>(null);
  let planning = $state(false);
  let planFailed = $state<string | null>(null);
  let pgen = 0;
  $effect(() => {
    const s = site;
    const l = lander;
    const sub = subject;
    const sys = scene.system;
    const g = ++pgen;
    plan = null;
    planFailed = null;
    if (!s || !sub) return;
    planning = true;
    void ctx.session.planDescent({ system: sys, body: sub.address, lat_deg: s.lat, lon_deg: s.lon, lander: l }).then((r) => {
      if (g !== pgen) return;
      planning = false;
      if (r.ok) plan = r.plan;
      else planFailed = r.message;
    });
  });
  function pickMap(e: MouseEvent) {
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const lon = -180 + (360 * (e.clientX - r.left)) / r.width;
    const lat = 90 - (180 * (e.clientY - r.top)) / r.height;
    site = { lat: Math.round(lat * 100) / 100, lon: Math.round(lon * 100) / 100, label: "" };
  }
  const pickLandmark = (l: Landmark) => (site = { lat: l.lat_deg, lon: l.lon_deg, label: l.name ?? l.original_name });
  const worst = $derived(plan ? (plan.hazards.some((h) => h.level === "danger") ? "danger" : plan.hazards.some((h) => h.level === "caution") ? "caution" : "ok") : "");
  const fmtLat = (v: number) => `${Math.abs(v).toFixed(2)}° ${v >= 0 ? "N" : "S"}`;
  const fmtLon = (v: number) => `${Math.abs(v).toFixed(2)}° ${v >= 0 ? "E" : "W"}`;
  const fmtT = (s: number) => (s >= 3600 ? `${Math.floor(s / 3600)} h ${Math.round((s % 3600) / 60)} min` : s >= 90 ? `${Math.floor(s / 60)} min ${Math.round(s % 60)} s` : `${Math.round(s)} s`);
  const fmtM = (m: number) => (Math.abs(m) >= 10000 ? `${(m / 1000).toFixed(0)} km` : Math.abs(m) >= 1000 ? `${(m / 1000).toFixed(1)} km` : `${Math.round(m)} m`);

  // ── The ground round the site ─────────────────────────────────────────────────────────────
  let phase = $state<"choose" | "loading" | "flying" | "down">("choose");
  let near = $state<Patch | null>(null);
  let far = $state<Patch | null>(null);
  let groundFailed = $state<string | null>(null);
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
      const r = await ctx.session.surfaceTilesAt(scene.system, sub.address, keys.slice(0, 24));
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
  /** Down: facing on along the way the lander came in (from the west). */
  function toGround() {
    walk = { x: 0, y: 0, yaw: 0, pitch: -0.05 };
    phase = "down";
  }
  function takeOff() {
    phase = "choose";
    near = null;
    far = null;
    built = false;
  }
  function reset() {
    phase = "choose";
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
    if (g.cover === 0 && Math.abs(x) < DETAIL_HALF && Math.abs(y) < DETAIL_HALF) g.z += decoration(x, y, roughness, p.spacing_m);
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
      if ((now - t0) / 1000 >= flightS) toGround();
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
    headingNow = ((90 - (Math.atan2(fwd[1], fwd[0]) * 180) / Math.PI) % 360 + 360) % 360;
    const sun = sunDirection(site!.lat, hour, 0);
    const air = world.air;
    if (using === "gpu" && gpu) {
      buildMeshes();
      gpu.draw({
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
        exposure: 6,
        ground: tok("--sse-ground"),
        ink: tok("--sse-ink"),
        contour_m: contourFor(eye[2] - siteZ),
      });
    } else if (using === "canvas" && line) drawLines(line, eye, fwd);
  }
  $effect(() => {
    if (phase === "flying" || phase === "down") {
      cancelAnimationFrame(raf);
      last = 0;
      raf = requestAnimationFrame(frame);
    }
  });
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
    if (!drag || phase !== "down") return;
    walk.yaw -= (e.clientX - drag.x) * 0.004;
    walk.pitch = Math.max(-1.4, Math.min(1.4, walk.pitch - (e.clientY - drag.y) * 0.004));
    drag = { x: e.clientX, y: e.clientY };
  }
  const up = () => (drag = null);
  const keyDown = (e: KeyboardEvent) => {
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

<section class="surface" bind:this={host} data-surface data-phase={phase} data-look={look} data-hud={hud ? "on" : "off"} data-tier={using} data-site={site ? `${site.lat.toFixed(2)},${site.lon.toFixed(2)}` : ""} data-verdict={plan?.verdict ?? ""} data-walk={phase === "down" ? `${eyeNow[0].toFixed(1)},${eyeNow[1].toFixed(1)}` : ""} data-floor-m={near ? near.spacing_m.toFixed(0) : ""}>
  {#if !subject || subject.kind === "star"}
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
        <h3>Where to land on {subject.label}</h3>
        <p class="quiet">Pick a landmark, or press anywhere on the map. Every spot is allowed; its dangers are shown.</p>
        <ul class="marks">
          {#each landmarks.slice(0, 12) as l (l.id)}
            <li><button onclick={() => pickLandmark(l)} class:on={site?.label === (l.name ?? l.original_name)} data-landmark-row={l.id}>{l.name ?? l.original_name}<span class="quiet"> {l.kind}{l.remarkable.length ? `, ${l.remarkable[0]}` : ""}</span></button></li>
          {/each}
          {#if !landmarks.length}<li class="quiet">Reading the landscape's names…</li>{/if}
        </ul>
        {#if site}
          <div class="verdict" data-verdict-card data-worst={worst}>
            <h4>{site.label || "This spot"} <span class="quiet">{fmtLat(site.lat)}, {fmtLon(site.lon)}</span></h4>
            <label class="lander">Lander
              <select bind:value={lander} data-lander>
                {#each STOCK_LANDERS as id (id)}<option value={id}>{LANDER_WORDS[id]}</option>{/each}
              </select>
            </label>
            {#if planning}<p class="quiet">Planning the descent…</p>{/if}
            {#if planFailed}<p class="quiet">{planFailed}</p>{/if}
            {#if plan}
              <p class="says {worst}"><strong>{DESCENT_VERDICT_WORDS[plan.verdict] ?? plan.verdict}</strong>: {plan.descent.outcome.method.replace(/-/g, " ")}, {fmtT(plan.descent.outcome.duration_s)} from {fmtM(plan.entry.altitude_m)} up, peak {plan.descent.outcome.peak_g.toFixed(1)} g.</p>
              <ul class="hazards">
                {#each plan.hazards as z (z.kind)}<li class={z.level} data-hazard={z.kind} data-level={z.level}><span class="lv">{z.level}</span> {z.kind}: {z.why}</li>{/each}
              </ul>
              <details><summary>How the site was read</summary><ul>{#each plan.site.why as y, i (i)}<li>{y}</li>{/each}{#each plan.descent.outcome.why as y, i (i)}<li>{y}</li>{/each}</ul></details>
              <button class="land" onclick={land} disabled={phase === "loading"} data-land>{phase === "loading" ? "Reading the ground…" : worst === "danger" ? "Land anyway" : "Land"}</button>
              {#if groundFailed}<p class="quiet">{groundFailed}</p>{/if}
            {/if}
          </div>
        {/if}
      </aside>
    </div>
  {/if}
  {#if tiles}
    <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
    <div class="stage" class:hidden={!(phase === "flying" || phase === "down")} bind:clientWidth={w} bind:clientHeight={h} role="application" aria-label="The ground at {site ? fmtLat(site.lat) : ''}: drag to look about, arrow keys or W A S D to walk" tabindex="0" onpointerdown={down} onpointermove={move} onpointerup={up} onpointercancel={up} onkeydown={keyDown} onkeyup={keyUp} data-ground>
      <canvas bind:this={canvas} class:hidden={using === "canvas"}></canvas>
      <canvas bind:this={line} class:hidden={using !== "canvas"}></canvas>
      {#if hud}
        <div class="hud" data-surface-hud>
          <span>{compass} {Math.round(headingNow)}°</span>
          <span>{fmtM(eyeNow[2] - siteZ)} above the site</span>
          <span>{fmtM(fromSite)} from it</span>
          <span>horizon {fmtM(horizon)}</span>
          {#each nearMarks as m (m.l.id)}<span class="quiet">{m.l.name ?? m.l.original_name} {fmtM(m.d)} at {Math.round(m.brg)}°</span>{/each}
        </div>
      {/if}
      {#if phase === "flying" && plan}
        <div class="log" data-descent-log>
          <p class="clock">T+{fmtT(flyT)} {#if tel}· {fmtM(tel.altitude_m)} · {Math.round(tel.speed_m_s)} m/s · {tel.phase}{/if}</p>
          {#each events as e (e.t_s + e.kind)}<p>T+{fmtT(e.t_s)} {e.text}</p>{/each}
          <button onclick={skip} data-skip>Skip to the ground</button>
        </div>
      {:else}
        <div class="walk" data-walk-pad>
          <button onpointerdown={() => press("ArrowLeft", true)} onpointerup={() => press("ArrowLeft", false)} onpointerleave={() => press("ArrowLeft", false)} aria-label="Turn left">◀</button>
          <button onpointerdown={() => press("ArrowUp", true)} onpointerup={() => press("ArrowUp", false)} onpointerleave={() => press("ArrowUp", false)} aria-label="Walk forward" data-walk-forward>▲</button>
          <button onpointerdown={() => press("ArrowDown", true)} onpointerup={() => press("ArrowDown", false)} onpointerleave={() => press("ArrowDown", false)} aria-label="Walk back">▼</button>
          <button onpointerdown={() => press("ArrowRight", true)} onpointerup={() => press("ArrowRight", false)} onpointerleave={() => press("ArrowRight", false)} aria-label="Turn right">▶</button>
          <label><input type="checkbox" bind:checked={drone} data-drone /> Drone</label>
          <label>Time of day <input type="range" min="-100" max="100" step="1" bind:value={hour} data-hour /></label>
          <button onclick={takeOff} data-take-off>Take off</button>
        </div>
      {/if}
    </div>
    {#if phase === "flying" || phase === "down"}
    <p class="note quiet" data-ground-note>
      The ground is the kernel's to {near ? Math.round(near.spacing_m) : "?"} m (its fact floor here); finer than that within {fmtM(DETAIL_HALF)} of the site it is decoration sized to the site's roughness. You can walk {fmtM(WALK_HALF)} from where you landed; further means taking off.
      {#if world && !world.air}No air: the sky is black and nothing dims the far ground.{:else}Sky from single scattering in the kernel's air optics.{/if}
      The sun is placed for the equinox at the hour chosen. Drawn on {reason}.
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
</style>
