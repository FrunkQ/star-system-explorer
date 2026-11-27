<!--
  System 3D (V6): one frame at a time, in perspective. The centre body sits
  at the origin and the bodies orbiting it lie on log-scaled rings. Every
  position is composed in f64 relative to the camera and only then narrowed
  for the GPU (handover/rendering-rules.md). Zooming in on a body with moons
  hands over to its frame; zooming out of a planet's frame hands back. Tiers:
  GPU (three.js WebGPU, WebGL2 when the device has no WebGPU), else Canvas
  2D drawing the same projection. The overlay buttons are the keyboard path.
  True (V10a): the same frame against black space, each world wrapped in its
  own surface (given in `surfaces`, read from its tiles) and lit by its star,
  the star in its own colour; given `star` (V10c), the star itself as the
  kernel has it, turning on its spin with its spots and flares. Everything that
  moves (the star, the worlds round their orbits, the belts' members) moves on
  the campaign's clock (`clock`, CLK1): still while it is stopped. HUD (every style, Alex 23 Nov): orbit lines,
  belts, names and locator rings; off, names show on hover only and True
  looks like a photograph. Scale Real: distances and sizes in proportion.
-->
<script lang="ts">
  import { onDestroy, tick, untrack } from "svelte";
  import { closeMin, grazeLook, groundUnder, orbitEye, orbitOf, zoomNear, type GroundView, longGain, basis, trueLook as trueFace, shadeFor, umbraKey, BELT_DUST, beltAt, beltParticles, drawStarNow, GLOW_REACH_DEG, poleOf, magShape, placeMag, type FieldReadJson, type MagShape, thermalGrey, THERMAL_DECADES, turnAt, project, startGpu, tree, type Camera, type Gpu, type GpuGlow, type GpuLight, type StarNow, type LiveStar, type TrueTexture, type Vec3, type AirShape, type PlumeFacts, plumeNote, LIBRARY, TrueRun, cometNote, cometTails, comaPicture, ionColour, isActive, showing, hexLinear, linearCss, exposeAurora, type AuroraDraw, type AuroraPlan, type CometActivity } from "../../../renderer/src/index.ts";
  import { SYSTEM_TRUE } from "../../../renderer/src/recipes.ts";
  import { clockAt, constant, motionWords, phaseAt, pictureSpan, SEEN_ELEVATION, thermalColour, tooFast, type PictureClock, type RangeRing, type SeenFrom } from "../../../ui-contract/src/index.ts";
  import { reducedMotion } from "../../../chart-grammar/src/motion.ts";
  import { rangeRingLines } from "./range-rings.ts";
  import { bandLines, hasChildren, layout, layoutReal, placedBodies, shellDots, type Band3D, type Body3D } from "./bodies.ts";

  let {
    bodies,
    selection,
    onselect,
    title,
    tier = "auto",
    labels = true,
    distanceNote = "",
    bands = [],
    seen = "free",
    onturned,
    trueLook = false,
    surfaces = new Map(),
    plumes = new Map(),
    aurorae = new Map(),
    airs = new Map(),
    umbras = new Map(),
    starColour = "",
    trueNote = "",
    realScale = false,
    hud = true,
    star = null,
    band = "visible",
    fields = new Map(),
    pencil,
    reframe = 0,
    sky,
    long = false,
    onground,
    oncamera,
    aim = null,
    onaimed,
    range = [],
    clock = null,
  }: {
    bands?: Band3D[];
    bodies: Body3D[];
    selection: string | null;
    onselect: (id: string) => void;
    title: string;
    tier?: string;
    labels?: boolean;
    distanceNote?: string;
    seen?: SeenFrom;
    onturned?: () => void;
    /** The True view style (V10a); the GPU tier draws it, the canvas tier keeps the plain look. */
    trueLook?: boolean;
    /** True look: each world's surface by body id, as far as it has been read. */
    surfaces?: ReadonlyMap<string, TrueTexture>;
    /** True look: each world's plume facts by body id (the kernel's surface record, plumes.ts), when it has any. */
    plumes?: ReadonlyMap<string, PlumeFacts>;
    /** True look: each world's aurora plan by body id (renderer true-aurora.ts), drawn as shells in Visible. */
    aurorae?: ReadonlyMap<string, { plan: AuroraPlan; radius_m: number }>;
    /** True look: each world's air, by body id (renderer true-air.ts `readTrueFace`): its clouds alone, drawn over its ground, and its air's rim (VP-A3, VP-A4). */
    airs?: ReadonlyMap<string, { ground?: TrueTexture; clouds?: TrueTexture; air?: AirShape; flatten?: number; sea?: { slope_variance: number; normal_reflectance: number } }>;
    /** True look: the light each body's air bends into its shadow on another (renderer eclipse-read.ts `readUmbras`), by `umbraKey(caster, receiver)`; none, a black umbra. */
    umbras?: ReadonlyMap<string, [number, number, number]>;
    /** True look: the star's colour (its surface temperature's, as a CSS colour). */
    starColour?: string;
    /** True look: what is still being read, said under the chart. */
    trueNote?: string;
    /** Scale: Real (Alex, 23 Nov 2026): distances in proportion and every body at its true size on that scale. */
    realScale?: boolean;
    /** HUD (True only): orbit lines, names and locator rings over the picture; off, it looks like a photograph. */
    hud?: boolean;
    /** True look: the star as the kernel has it (live-star.ts), drawn turning, spotted and flaring. */
    star?: LiveStar | null;
    /** True look: the band it is seen in (rule 12). Visible, X-ray (the star's corona; worlds dark) and Infrared (each
     * body's own heat, mapped grey) are drawn; the others draw Visible and say so. */
    band?: string;
    /** Mag (V10e): each world's field facts (read_field) by body id; worlds without are drawn dark and fieldless. */
    fields?: ReadonlyMap<string, FieldReadJson>;
    /** Who knows (track SV, pencil and ink): true for a body the table does not know, drawn in pencil like the
     * chart's names (quiet, dotted); read afresh whenever the table's knowledge changes. */
    pencil?: (id: string) => boolean;
    /** Bumped each time a fixed "Seen from" is picked: the camera goes back to that framing (turn, zoom and slide). */
    reframe?: number;
    /** True: the sky's stars from this system (stars-behind.ts), drawn behind everything (VP2). */
    sky?: { key: string; dirs: Float32Array; share: Float32Array };
    /** True: a long exposure that lifts the stars (a composite; the note says so). */
    long?: boolean;
    /** Close-up: the ground under the camera while it is near the world at the centre in True (null when it leaves). */
    onground?: (g: GroundView | null) => void;
    /** PC2: the camera about the world at the centre (eye direction, radii out, field of view), for a bookmark or a
     * postcard made here; null when the camera has slid off it or the centre is the star. */
    oncamera?: (c: { about: string; dir: Vec3; radii: number; fov_deg: number } | null) => void;
    /** A postcard's or bookmark's camera to go to, once the world it is about is at the centre; `onaimed` says it was taken. */
    aim?: { about: string; dir: Vec3; radii: number } | null;
    onaimed?: () => void;
    /** Range rings (RK12), drawn with the HUD round each holder (a body id) in this frame. */
    range?: readonly { holder: string; rings: readonly RangeRing[] }[];
    /** The picture's clock (CLK1, ui-contract picture-clock.ts): every motion here reads its time from it; stopped, the picture is still. */
    clock?: PictureClock | null;
  } = $props();
  const trueBand = $derived(band === "xray" || band === "infrared" ? band : "visible");
  const overlay = $derived(hud);
  // Mag (V10e): the picture gives way to the fields, in 3D and True alike: each world's lines and the wind's bubble.
  const mag = $derived(band === "mag");
  // Traced once per change of facts (a few ms a world), never per frame.
  const magShapes = $derived.by(() => {
    const out = new Map<string, MagShape>();
    if (!mag) return out;
    for (const [id, r] of fields) {
      const s = magShape(r);
      if (s) out.set(id, s);
    }
    return out;
  });
  const isTrue = $derived(trueLook);

  const root = $derived(bodies.find((b) => b.parent === null)?.id ?? bodies[0]?.id ?? "");
  let frame = $state<string | null>(null);
  const centreId = $derived(frame && bodies.some((b) => b.id === frame) ? frame : root);
  // The picture's time (CLK1): seconds since the campaign's epoch, from the clock. While it runs, a frame loop moves it
  // on; stopped, it sits at the time in view and nothing moves. Positions follow every frame (at most thirty a
  // second); the star's face, drawn on the CPU, ten times a second.
  const PICTURES_PER_S = 30;
  const STAR_PICTURES_PER_S = 10;
  let pictureT = $state(0);
  let starClockT = $state(0);
  $effect(() => {
    const c = clock;
    if (!c) return;
    const at = (now: number) => clockAt(c, now);
    pictureT = starClockT = at(performance.now());
    if (!(c.rate > 0)) return;
    let raf = 0;
    let last = 0;
    let lastStar = 0;
    const step = (now: number) => {
      if (now - last >= 1000 / PICTURES_PER_S - 1) ((last = now), (pictureT = at(now)));
      if (now - lastStar >= 1000 / STAR_PICTURES_PER_S - 1) ((lastStar = now), (starClockT = at(now)));
      raf = requestAnimationFrame(step);
    };
    raf = requestAnimationFrame(step);
    return () => cancelAnimationFrame(raf);
  });
  // Each body where its orbit has carried it since the kernel's last answer (its own period, mean motion).
  const moving = $derived(clock ? bodies.map((b) => (b.period_s && b.phase_at_s !== undefined ? { ...b, phase_deg: phaseAt(b.phase_deg, b.phase_at_s, b.period_s, pictureT) } : b)) : bodies);
  const lay = $derived(realScale ? layoutReal(moving, centreId) : layout(moving, centreId));
  const t = $derived(lay ? tree(lay.nodes) : tree([]));
  const byId = (id: string | null) => moving.find((b) => b.id === id);

  // A body chosen elsewhere opens the frame it is drawn in (a moon: its planet's).
  $effect(() => {
    const s = byId(selection);
    if (!s || s.id === centreId) return;
    const p = s.parent;
    if (p && p !== root && p !== centreId) frame = p;
    else if (p === root && centreId !== root && s.id !== centreId) frame = null;
  });

  // Camera: orbiting the centre, in display units of this frame.
  let yaw = $state(-0.6);
  let pitch = $state(0.55);
  let dist = $state(26);
  // Slide (rule 11): where the camera looks, in this frame's display units.
  let pan = $state<[number, number]>([0, 0]);
  // "Seen from" (rule 12): picking a fixed view frames it afresh (height, turn, zoom and slide), even when it is the
  // one already shown; Free keeps what the hands did.
  // Only a real change of either reframes: the props are read again whenever the view choice changes at all (HUD,
  // band, exposure), and those must leave the camera where it is.
  let framedAs = "";
  $effect(() => {
    const s = seen;
    const key = `${s}|${reframe ?? 0}`;
    if (key === framedAs) return;
    framedAs = key;
    if (s === "free") return;
    pitch = SEEN_ELEVATION[s];
    yaw = -0.6;
    dist = 26;
    pan = [0, 0];
  });
  // Changing the scale changes what a distance means: start the camera again from its usual place.
  let lastScale: boolean | null = null;
  $effect(() => {
    const r = realScale;
    if (lastScale !== null && r !== lastScale) ((dist = 26), (pan = [0, 0]));
    lastScale = r;
  });
  // Real scale lets the camera come much closer: the worlds are small. True lets it come down to a grazing orbit over
  // the world at the centre (close-up, Alex 25 Nov): a few km over its ground (renderer close-up.ts).
  const centreBody = $derived(lay?.centre);
  const centreR = $derived(!centreBody || centreBody.parent === null ? 0 : realScale ? (lay ? lay.dist(centreBody.radius_m ?? 0) : 0) : 0.55);
  const closeUp = $derived(isTrue && centreR > 0 && pan[0] === 0 && pan[1] === 0);
  const MIN = $derived(closeUp ? closeMin(centreR, centreBody?.radius_m) : realScale ? 0.3 : 4);
  const MAX = 60;
  let w = $state(0);
  let h = $state(0);
  const FOV = (45 * Math.PI) / 180;
  const eyeAt = $derived<Vec3>([pan[0] + dist * Math.cos(pitch) * Math.cos(yaw), pan[1] + dist * Math.cos(pitch) * Math.sin(yaw), dist * Math.sin(pitch)]);
  const cam = $derived<Camera>({
    frame: centreId,
    eye: eyeAt,
    // Near the world the view tips toward its horizon; from three radii out it looks at the centre.
    look: closeUp ? grazeLook(eyeAt, [0, 0, 0], centreR, FOV) : [pan[0], pan[1], 0],
    fovY: FOV,
    width: Math.max(1, w),
    height: Math.max(1, h),
  });
  // The near plane comes down under the camera's height, so the ground is never clipped.
  const nearPlane = $derived(closeUp ? Math.min(1e-3, Math.max(1e-9, (dist - centreR) * 0.2)) : 1e-3);
  // The ground under the camera, for the detail ladder's deep tiles (VP3 part 4), at most five times a second.
  let groundAt = 0;
  let groundSent: string | null = null;
  $effect(() => {
    if (!onground) return;
    const g = closeUp && centreBody ? groundUnder(centreBody.id, cam.eye, [0, 0, 0], centreR, centreBody.radius_m, cam.fovY, cam.height) : null;
    const now = performance.now();
    const k = g ? `${g.body}|${g.toward.join(",")}|${g.m_per_px}` : "";
    if (k === groundSent || (g && groundSent && now - groundAt < 200)) return;
    groundAt = now;
    groundSent = k;
    onground(g);
  });

  // PC2: the camera kept for a bookmark or postcard made here, and a kept camera taken back (once its world is centred).
  let cameraSent = "";
  let aimedAt = $state("");
  $effect(() => {
    if (!oncamera) return;
    const e = centreBody && pan[0] === 0 && pan[1] === 0 ? orbitEye({ yaw, pitch, dist }, centreR) : null;
    const c = e && centreBody ? { about: centreBody.id, dir: e.dir, radii: e.radii, fov_deg: (FOV * 180) / Math.PI } : null;
    const k = c ? `${c.about}|${c.dir.join(",")}|${c.radii}` : "";
    if (k === cameraSent) return;
    cameraSent = k;
    oncamera(c);
  });
  $effect(() => {
    const a = aim;
    if (!a || !centreBody) return;
    // Its world first comes to the centre (the frame it is drawn in), then the camera goes where it was.
    if (a.about !== centreBody.id) {
      if (byId(a.about)) untrack(() => ((flying += 1), (tweening = false), (frame = a.about)));
      return;
    }
    if (!(centreR > 0)) return;
    untrack(() => {
      flying += 1;
      tweening = false;
      const o = orbitOf(a.dir, a.radii, centreR);
      yaw = o.yaw;
      pitch = o.pitch;
      dist = Math.min(MAX, Math.max(MIN, o.dist));
      pan = [0, 0];
      aimedAt = a.about;
      onaimed?.();
    });
  });

  function enter(id: string) {
    flying += 1; // a fly-in still under way stops here, or it would drag the camera on in the new frame
    tweening = false;
    frame = id;
    dist = 26;
    onselect(id);
  }
  // A click on a body flies in to its close-up (Alex, 24 Nov: "do some zoom tricks to fool the user into feeling we are
  // zooming"). Not one continuous zoom across scales: the camera rushes at the body in this frame, the frame changes
  // to the body's own while the eye is close, and the camera eases back out to show it with its moons. Two short
  // moves that read as one; no motion when the person asks for less.
  let flying = 0;
  // Whether a fly-in is under way, from the first move to the last, said on the chart (data-flying) so a test can wait
  // for it to finish rather than guess from the camera holding still.
  let tweening = $state(false);
  let flights = 0;
  const easeIn = (u: number) => u * u * u;
  const easeOut = (u: number) => 1 - (1 - u) ** 3;
  function tweenCam(to: { pan: [number, number]; dist: number }, ms: number, ease: (u: number) => number): Promise<boolean> {
    const run = ++flying;
    const from = { pan: [...pan] as [number, number], dist };
    const t0 = performance.now();
    return new Promise((done) => {
      const step = (now: number) => {
        if (run !== flying) return done(false);
        const u = Math.min(1, (now - t0) / ms);
        const e = ease(u);
        pan = [from.pan[0] + (to.pan[0] - from.pan[0]) * e, from.pan[1] + (to.pan[1] - from.pan[1]) * e];
        // Distance in proportion, so the rush feels even from far out to close in.
        dist = from.dist * (to.dist / from.dist) ** e;
        if (u < 1) requestAnimationFrame(step);
        else done(true);
      };
      requestAnimationFrame(step);
    });
  }
  // A world with no moons fills more of the view in True, where its face is the point (about 60% of the height).
  const closeDist = (id: string) => (hasChildren(bodies, id) ? 26 : isTrue && !realScale ? 2.2 : 9);
  // Arriving at a world in True, the camera turns to three-quarter light: this far round from its star (about 50°),
  // as a globe is lit to be photographed, so its craters and ranges cast shadows near the terminator. Seen from the
  // star's side (full phase) the same relief shows no shadow at all and a world reads flat (Alex, 25 Nov).
  const LIT_TURN = 0.9;
  async function litTurn(id: string) {
    if (!isTrue || hasChildren(bodies, id)) return;
    await tick();
    const b = byId(id);
    if (!b || centreId !== id) return;
    const l = lightOf(b);
    if (Math.hypot(l[0], l[1]) > 0) yaw = Math.atan2(l[1], l[0]) + LIT_TURN;
  }
  async function flyTo(id: string) {
    onselect(id);
    if (id === centreId) return;
    const node = lay?.nodes.find((n) => n.id === id);
    if (!node || reducedMotion() || typeof requestAnimationFrame !== "function") {
      enter(id);
      dist = closeDist(id);
      await litTurn(id);
      return;
    }
    const flight = ++flights;
    tweening = true;
    try {
      const near = Math.max(MIN, Math.min(3, dist / 6));
      if (!(await tweenCam({ pan: [node.local[0], node.local[1]], dist: near }, 380, easeIn))) return;
      frame = id;
      pan = [0, 0];
      dist = 2.5;
      const at = flying;
      await litTurn(id);
      if (at !== flying) return; // stopped (‹ back, a zoom across frames) while it turned to the light
      await tweenCam({ pan: [0, 0], dist: closeDist(id) }, 620, easeOut);
    } finally {
      if (flight === flights) tweening = false;
    }
  }
  function leave() {
    // Coming back out stops a fly-in still under way: left running, it carried on setting the camera after the frame
    // changed, so a quick "‹ back" (or a slow device) left the view part-way into a close-up of the frame just left.
    flying += 1;
    tweening = false;
    const back = centreId;
    frame = byId(centreId)?.parent && byId(centreId)!.parent !== root ? byId(centreId)!.parent : null;
    dist = 26;
    onselect(back);
  }
  // Rule 4: zoom past what the frame carries hands over to the chosen body's frame, or back up.
  function zoomBy(f: number) {
    // Near a world in True a step scales the height over its ground, so the last kilometres take as many steps as the first.
    const next = closeUp ? zoomNear(dist, centreR, f) : dist * f;
    const sel = byId(selection);
    if (next < MIN && sel && sel.id !== centreId && hasChildren(bodies, sel.id)) return enter(sel.id);
    if (next > MAX && centreId !== root) return leave();
    dist = Math.min(MAX, Math.max(MIN, next));
  }

  // Ink from the theme: marks without a colour draw in it.
  let host: HTMLDivElement | undefined = $state();
  // Theme tokens, read at draw time; the computed text colour stands in for a token a theme leaves out.
  const token = (name: string, fallback?: string) => {
    if (!host) return "";
    const cs = getComputedStyle(host);
    return cs.getPropertyValue(name).trim() || (fallback ? cs.getPropertyValue(fallback).trim() : "") || cs.color;
  };
  // Readable: sizes for reading. Real: the body's own radius on the frame's scale, but never under a pixel or two at
  // the camera's distance, so a world too small to show as a disc still shows as a point of light (fudge register).
  const sizeOf = (b: Body3D, centre: boolean) => {
    // A belt member is drawn smaller than a world (it is one), but large enough to see its shape when chosen.
    if (!realScale) return centre ? 0.55 : b.small ? 0.06 : 0.13 * Math.max(0.6, (b.mark?.size ?? 5) / 5);
    const n = lay?.nodes.find((x) => x.id === b.id);
    const far = n ? Math.hypot(n.local[0] - cam.eye[0], n.local[1] - cam.eye[1], n.local[2] - cam.eye[2]) : Math.hypot(...cam.eye);
    const px = (2 * Math.tan(cam.fovY / 2) * far) / Math.max(1, h);
    return Math.max((b.parent === null ? 1.5 : 0.75) * px, lay ? lay.dist(b.radius_m ?? 0) : 0);
  };

  // Camera-relative positions, f64 until the last step (frames.ts).
  const SEG = 96;
  function ringPoints(r: number): Vec3[] {
    const out: Vec3[] = [];
    for (let i = 0; i <= SEG; i++) {
      const a = (i / SEG) * 2 * Math.PI;
      out.push([Math.fround(r * Math.cos(a) - cam.eye[0]), Math.fround(r * Math.sin(a) - cam.eye[1]), Math.fround(-cam.eye[2])]);
    }
    return out;
  }
  const relOf = (id: string): Vec3 => {
    const n = lay!.nodes.find((x) => x.id === id)!;
    return [Math.fround(n.local[0] - cam.eye[0]), Math.fround(n.local[1] - cam.eye[1]), Math.fround(n.local[2] - cam.eye[2])];
  };
  const shown = $derived(lay ? [lay.centre, ...lay.placed] : []);
  // Belts and the Oort cloud belong to the star's frame; a sphere far out is drawn just past the outermost orbit.
  const bandLocal = $derived(lay && centreId === root ? bands.flatMap((b) => bandLines(b, lay!.dist, 11.5, SEG).map((pts) => ({ b, pts }))) : []);
  const shellLocal = $derived(lay && centreId === root ? bands.map((b) => ({ b, pts: shellDots(b, lay!.dist, 14) })).filter((x) => x.pts.length) : []);
  // Range rings (RK12): metres from the frame's centre into display units by the frame's own scale (log or Real), so
  // a ring round a world bends as the orbits do; loops kept in frame-local units, projected by each tier.
  const rangeLocal = $derived.by(() => {
    if (!lay || !range.length) return [];
    const toLocal = (p: readonly [number, number, number]): Vec3 => {
      const r = Math.hypot(p[0], p[1]);
      const d = r > 0 ? lay!.dist(r) : 0;
      return [r > 0 ? (d * p[0]) / r : 0, r > 0 ? (d * p[1]) / r : 0, 0];
    };
    return range.flatMap((g) => {
      const b = g.holder === lay!.centre.id ? null : lay!.placed.find((x) => x.id === g.holder);
      if (b === undefined) return [];
      const t = ((b?.phase_deg ?? 0) * Math.PI) / 180;
      const centre: [number, number, number] = b ? [b.a_m! * Math.cos(t), b.a_m! * Math.sin(t), 0] : [0, 0, 0];
      return rangeRingLines(g.rings, centre, toLocal, { ring: "accent", odds: "faint" });
    });
  });
  const rel = (p: [number, number, number]): Vec3 => [Math.fround(p[0] - cam.eye[0]), Math.fround(p[1] - cam.eye[1]), Math.fround(p[2] - cam.eye[2])];
  const pins = $derived(shown.map((b) => ({ b, p: project(t, cam, b.id) })));

  let canvas: HTMLCanvasElement | undefined = $state();
  let gpu = $state<Gpu | null>(null);
  let using = $state<"gpu" | "canvas" | "">("");
  let reason = $state("");
  $effect(() => {
    if (!canvas || using) return;
    const want = tier;
    if (want === "canvas") {
      using = "canvas";
      reason = "canvas tier chosen";
      return;
    }
    void startGpu(canvas, { timing: true }).then((g) => {
      if (g) {
        gpu = g;
        using = "gpu";
        reason = g.backend === "webgpu" ? "WebGPU" : "WebGL2 (no WebGPU here)";
      } else {
        using = "canvas";
        reason = "no GPU path on this device";
      }
    });
  });
  onDestroy(() => gpu?.dispose());

  // Quality in True (VP4 part 2b, VP5): one controller places the view by the device's measured speed, steps it down
  // on slow frames by the system-true recipe, and sharpens the picture once the view holds still (the finish keeps the
  // running mean of jittered frames). `qv` redraws when its settings change; `restTick` draws the next sharpening frame.
  let run: TrueRun | null = null;
  let qv = $state(0);
  let restTick = $state(0);
  let drawnRestTick = 0;
  let restTimer: ReturnType<typeof setTimeout> | undefined;
  let quality = $state("");
  const viewPixels = () => {
    const dpr = typeof devicePixelRatio === "number" ? devicePixelRatio : 1;
    return Math.max(1, Math.round(w * h * dpr * dpr));
  };
  $effect(() => {
    if (!isTrue || using !== "gpu" || !gpu) return;
    const r = new TrueRun({ recipe: SYSTEM_TRUE, library: LIBRARY, pixels: untrack(viewPixels), backend: gpu.backend, source: "render.system", onChange: () => qv++ });
    run = r;
    void r.place();
    return () => {
      clearTimeout(restTimer);
      r.dispose();
      run = null;
      quality = "";
    };
  });
  $effect(() => {
    const px = viewPixels();
    untrack(() => run?.update({ pixels: px }));
  });
  /** Ask for the next sharpening frame when it falls due (none while moving, or once sharp). */
  function restLater(now: number) {
    clearTimeout(restTimer);
    const r = run;
    if (!r) return;
    const due = r.rest.dueIn(now);
    if (Number.isFinite(due)) restTimer = setTimeout(() => restTick++, due);
  }
  onDestroy(() => clearTimeout(restTimer));

  // The star's light: from the star itself when it is in the frame, else from where it lies as seen from this frame's
  // centre (opposite the centre's own place on its orbit round the star).
  // Hot glow (hot-glow.ts): the star's temperature, radius and distance from a world, through its parents.
  function daylightOf(b: Body3D): { t_k: number; r_m: number; d_m: number } | null {
    const st = bodies.find((x) => x.parent === null);
    let top = b;
    while (top.parent !== null && top.parent !== st?.id) {
      const up = bodies.find((x) => x.id === top.parent);
      if (!up) break;
      top = up;
    }
    return st?.t_k && st.radius_m && top.a_m ? { t_k: st.t_k, r_m: st.radius_m, d_m: top.a_m } : null;
  }
  // Aurorae (aurora.ts): the world's plan under the view's exposure, its star's light at the top of its air as the
  // point light's strength (3) gives it, so the shells and the lit ground share one exposure; Long lifts faint light.
  function auroraOf(b: Body3D): AuroraDraw | null {
    const a = aurorae.get(b.id);
    const day = daylightOf(b);
    if (!a || !day) return null;
    const flux = constant("sigma_SB") * day.t_k ** 4 * (day.r_m / day.d_m) ** 2;
    return exposeAurora(a.plan, a.radius_m, long ? "long" : "eye", 3, flux);
  }
  const auroraNote = $derived(isTrue && trueBand === "visible" && !mag ? shown.filter((b) => aurorae.has(b.id)).map((b) => `${b.name}: ${aurorae.get(b.id)!.plan.note}`).join(" ") : "");
  const neg = (v: Vec3): Vec3 => [-v[0], -v[1], -v[2]];
  // From a world toward its light, camera-relative (as lightFor gives it).
  function lightOf(b: Body3D): Vec3 {
    const l = lightFor();
    const r = relOf(b.id);
    return l.at ? [l.at[0] - r[0], l.at[1] - r[1], l.at[2] - r[2]] : (l.from ?? [0, 0, 1]);
  }
  const plumeNotes = $derived(isTrue && trueBand === "visible" && !mag ? shown.filter((b) => plumes.has(b.id)).map((b) => plumeNote(b.name, plumes.get(b.id)!)).join(" ") : "");

  function lightFor(): GpuLight {
    const c = lay!.centre;
    if (c.parent === null) return { at: relOf(c.id) };
    // The star lies opposite the world that circles it, whatever frame inside that world's family is shown: a moon is
    // lit from where its planet's star is, not from its own place round the planet (the Moon was lit from behind the
    // camera, evenly, with no day and night).
    let top = c;
    for (let up = byId(top.parent); up && up.parent !== null; up = byId(up.parent)) top = up;
    const a = (top.phase_deg * Math.PI) / 180;
    return { from: [-Math.cos(a), -Math.sin(a), 0.15] };
  }

  // The living star (V10c) on the picture's clock (CLK1): it turns on its own spin, its cells live their own lives and
  // its flares rise and fade at their own speeds, as fast as the clock runs. Its time is its age at the campaign's
  // epoch plus the clock's seconds, so the same date always shows the same face. Ten pictures a second; each shows
  // any flare that peaked since the last, and the cells blur as a picture stands for more of their lives.
  const starRate = $derived(clock?.rate ?? 0);
  const starT = $derived(star ? star.age_s + starClockT : 0);
  const starK = $derived(bodies.find((b) => b.parent === null)?.t_k ?? star?.teff ?? constant("T_sun"));
  const bandNote = $derived(
    !isTrue
      ? ""
      : trueBand === "xray"
        ? "X-rays, in mapped colour: the star's corona on a log scale, brighter over its spots and flares. Worlds give off almost none, so they are dark."
        : trueBand === "infrared"
          ? `Thermal infrared (10 µm), in mapped grey on a log scale ${THERMAL_DECADES} decades deep from the star: each body glows by its own temperature, the same over its whole disc (the Light panel's near-infrared recipes paint the flat chart).`
          : band !== "visible" && band !== "mag"
            ? "True draws Visible, Infrared and X-ray for now; this band is drawn as Visible."
            : "",
  );
  const magNote = $derived(
    !mag
      ? ""
      : using === "canvas"
        ? "Mag lines are drawn on the GPU tier; open a world for its field's cut."
        : !magShapes.size
          ? "Mag: no world here has field facts yet. Fields come from the Solar System's own bodies for now; other worlds get theirs once the kernel publishes a field for every world."
          : `Mag: ${magShapes.size === 1 ? "one world's" : `${magShapes.size} worlds'`} magnetic fields from the kernel's facts (read_field), traced as on each world's own Mag layer. Orange where the field points away from the world, blue where it points in; the faint outlines are the wind's bubble (magnetopause), its nose toward the star.${realScale ? " Real scale: each field at its true size." : " Readable scale: each field is drawn to its world's enlarged size and stopped short of the nearest body drawn; Real shows them at their true size."}`,
  );
  // What the star showed in its last picture (star-now.ts): flares on screen, ejections leaving, words for the note.
  let starNow = $state<StarNow | null>(null);
  const flaringNow = $derived(starNow ? starNow.shown.length : 0);
  const calm = reducedMotion();
  // The face is drawn at about the size the disc shows on the screen, in steps so its texture is not remade each zoom.
  const FACE_STEPS = [16, 32, 64, 128, 256];
  const FACE_DISC = 0.92;
  const XRAY_DISC = 0.6;
  let faceBuf: { width: number; data: Uint8ClampedArray } | null = null;
  let faceVersion = 0;
  function faceFor(b: Body3D, radius: number) {
    const s = star;
    if (!s || b.parent !== null) return undefined;
    const r = relOf(b.id);
    const d = Math.hypot(r[0], r[1], r[2]);
    const dpr = typeof devicePixelRatio === "number" ? devicePixelRatio : 1;
    const px = ((radius / Math.max(1e-6, d)) * (h / 2)) / Math.tan(cam.fovY / 2) * 2 * dpr / FACE_DISC;
    const width = FACE_STEPS.find((n) => n >= px) ?? 256;
    if (!faceBuf || faceBuf.width !== width) faceBuf = { width, data: new Uint8ClampedArray(width * width * 4) };
    const xray = trueBand === "xray";
    // In X-rays the corona reaches past the disc: the disc takes less of the square.
    const disc = xray ? XRAY_DISC : FACE_DISC;
    // The glow round the star (star-glow.ts) is an angle on the screen: the disc's angular radius sets its shape.
    const discDeg = (Math.atan(radius / Math.max(1e-9, d)) * 180) / Math.PI;
    if (!haloBuf) haloBuf = { width: HALO_PX, data: new Uint8ClampedArray(HALO_PX * HALO_PX * 4) };
    starNow = drawStarNow(faceBuf, xray ? null : haloBuf, s, { t: starT, clock: starRate, span: starRate / STAR_PICTURES_PER_S, calm, light: xray ? "xray" : "visible", disc, lat_deg: 7, disc_deg: discDeg });
    const version = ++faceVersion;
    halo = xray ? null : { key: `star-glow:${b.id}`, version, texture: haloBuf, rel: r, radius: d * Math.tan((GLOW_REACH_DEG * Math.PI) / 180) };
    return { key: `star:${b.id}:${xray ? "x" : "v"}`, version, width, data: faceBuf.data, disc };
  }
  // The halo: the eye's glare round the star and any mass ejection leaving it, added to the scene as light.
  const HALO_PX = 160;
  let haloBuf: { width: number; data: Uint8ClampedArray } | null = null;
  let halo: GpuGlow | null = null;

  // Mag lines in camera-relative points: each world's field drawn to its drawn size, x toward the star. Readable
  // enlarges the worlds, so a field there is stopped short of the nearest other body drawn (fudge register); Real
  // draws every field at its true size.
  function magRings(outInk: string, inInk: string, edge: string): { points: Vec3[]; colour: string }[] {
    if (!mag || !lay || !magShapes.size) return [];
    const starNode = lay.nodes.find((n) => n.id === root);
    const from = lightFor();
    const out: { points: Vec3[]; colour: string }[] = [];
    for (const b of shown) {
      const s = magShapes.get(b.id);
      const n = lay.nodes.find((x) => x.id === b.id);
      if (!s || !n) continue;
      const radius = realScale ? lay.dist(b.radius_m ?? 0) : sizeOf(b, b === lay.centre);
      if (!(radius > 0)) continue;
      const sun: [number, number] = starNode && starNode.id !== b.id ? [starNode.local[0] - n.local[0], starNode.local[1] - n.local[1]] : "from" in from && from.from ? [from.from[0], from.from[1]] : [-n.local[0], -n.local[1]];
      let reach = Infinity;
      if (!realScale) for (const o of lay.nodes) if (o.id !== b.id) reach = Math.min(reach, 0.45 * Math.hypot(o.local[0] - n.local[0], o.local[1] - n.local[1], o.local[2] - n.local[2]));
      const placed = placeMag(s, n.local as [number, number, number], radius, sun, reach);
      for (const r of placed.runs) out.push({ points: r.points.map(rel), colour: r.out ? outInk : inInk });
      for (const e of placed.bubble) out.push({ points: e.map(rel), colour: edge });
    }
    return out;
  }

  // Belts in True: a haze of points between each disc belt's drawn edges, in the star's frame, as faint as starlight
  // on dust allows a screen to show (fudge register: the count is a drawing's, not the belt's).
  const BELT_POINTS = 2400;
  // Each member keeps its own orbit (made once per belt) and goes round at the Kepler rate √(GM/a³) on the picture's
  // clock (CLK1), so the inner edge laps the outer; the same date always puts every member in the same place.
  const beltMembers = $derived(bands.filter((b) => b.shape !== "sphere").map((b) => ({ b, ps: beltParticles(b.a_in_m, b.a_out_m, 0.08, BELT_POINTS) })));
  const beltHazes = $derived(lay && centreId === root ? beltMembers.map(({ b, ps }) => ({ b, pts: beltAt(ps, b.gm ? pictureT : 0, b.gm ?? 1, (a) => lay!.dist(a), 11.5) })) : []);
  // What moves faster than the pictures can show (CLK1, picture-clock.ts `tooFast`): drawn where it truly is at each
  // picture, never slowed to look smooth, and named in the note. The star's cells blur instead (granuleBlur): a
  // picture that stands for many of their lives shows their average, as a long exposure would.
  const fastNames = $derived.by(() => {
    if (!clock || !(clock.rate > 0) || !lay) return [] as string[];
    const span = pictureSpan(clock, PICTURES_PER_S);
    const out = lay.placed.filter((b) => b.period_s && tooFast(b.period_s, span)).map((b) => b.name);
    const spin = Number(star?.activity.rotation_period_s ?? 0);
    if (isTrue && star && tooFast(spin, pictureSpan(clock, STAR_PICTURES_PER_S))) out.unshift("the star's spin");
    const inner = beltMembers.filter(({ b }) => b.gm && tooFast(2 * Math.PI * Math.sqrt(b.a_in_m ** 3 / b.gm), span)).map(({ b }) => `the ${b.name}`);
    return [...out, ...(isTrue && centreId === root ? inner : [])];
  });
  // Said while the clock runs, and in True while it is stopped (where a still star could look broken).
  const motionNote = $derived(clock && (clock.rate > 0 || (isTrue && using === "gpu")) ? motionWords(clock, fastNames) : "");
  const ringed = $derived(shown.filter((b) => b.rings?.length).map((b) => b.name));
  const ringsAssumed = $derived(shown.some((b) => !b.ring_kernel && b.rings?.some((r) => r.tau === null || r.albedo === null)));
  // The kernel's rings (VP-R3): what it says about the one in the middle of the frame, else a line for all of them.
  const kernelRinged = $derived(shown.filter((b) => b.ring_kernel));
  const ringWhy = $derived(kernelRinged.find((b) => b.id === centreId)?.ring_kernel?.why.slice(0, 3).join("; ") ?? "");
  const ringNote = $derived(
    [
      kernelRinged.length
        ? `Rings of ${kernelRinged.map((b) => b.name).join(", ")} as the kernel works them out: bands, gaps and waves placed by the moons, depth varying across each band, grains coloured by what they are made of, lit from each side with the opposition surge and backlit dust.${ringWhy ? ` ${ringWhy[0]!.toUpperCase()}${ringWhy.slice(1)}.` : ""}`
        : "",
      ringed.length > kernelRinged.length ? `Rings of ${shown.filter((b) => b.rings?.length && !b.ring_kernel).map((b) => b.name).join(", ")} from their edges, optical depth and albedo${ringsAssumed ? " (a ring with none on record is drawn as a middling one)" : ""}, in the world's equator.` : "",
      beltHazes.length ? "Belts are a haze of points between their edges, each going round at the Kepler rate on the campaign's clock (the inner edge laps the outer): their number is a drawing's, not the belt's." : "",
    ]
      .filter(Boolean)
      .join(" "),
  );

  // Comets in True (renderer comet.ts): an active belt member's coma and tails, from the kernel's activity, in the
  // star's frame. A tail grain s metres down the tail and w across it is drawn where the frame's own scale puts a point
  // that far from the star: s along the line from the star (tails point away from it), w as the angle it makes there,
  // so a tail bends the same way the orbits do on the log scale. Bodies are drawn larger than their scale in Readable,
  // so the coma is drawn at least three of the nucleus's drawn radii round it (fudge register, COMET-coma).
  const COMA_PICTURE = 64;
  const cometLooks = $derived.by(() => {
    if (!isTrue || !lay || centreId !== root) return [];
    const starRgb = hexLinear(thermalColour(starK));
    const out: { name: string; act: CometActivity; coma: { key: string; version: number; texture: { width: number; data: Uint8ClampedArray }; radius: number; at: [number, number, number] }; dots: { points: [number, number, number][]; colour: string; glow: number; size: number }[] }[] = [];
    for (const b of lay.placed) {
      const act = b.small?.activity;
      if (!isActive(act) || !(b.a_m && b.a_m > 0)) continue;
      const n = lay.nodes.find((x) => x.id === b.id);
      if (!n) continue;
      const r = b.a_m;
      const phi = Math.atan2(n.local[1], n.local[0]);
      const place = (s: number, w: number, h: number): [number, number, number] => {
        const R = r + s;
        const d = Math.min(14, lay!.dist(R));
        return [d * Math.cos(phi + w / R), d * Math.sin(phi + w / R), d * (h / R)];
      };
      const tails = cometTails(act);
      const toPts = (g: Float64Array) => {
        const pts: [number, number, number][] = [];
        for (let i = 0; i < g.length; i += 3) pts.push(place(g[i]!, g[i + 1]!, g[i + 2]!));
        return pts;
      };
      const strength = showing(act.dust_kg_s);
      const drawn = sizeOf(b, false);
      const radius = Math.max(3 * drawn, lay.dist(r) * ((act.coma_km * 1000) / r));
      out.push({
        name: b.name,
        act,
        coma: { key: `coma|${b.id}|${Math.round(strength * 1000)}`, version: 1, texture: comaPicture(starRgb, strength, COMA_PICTURE), radius, at: [n.local[0], n.local[1], n.local[2]] },
        dots: [
          ...(tails.dust.length ? [{ points: toPts(tails.dust), colour: linearCss(starRgb), glow: 0.5 * tails.dustShows, size: 1.5 }] : []),
          ...(tails.ion.length ? [{ points: toPts(tails.ion), colour: linearCss(ionColour(act)), glow: 0.5 * tails.ionShows, size: 1.5 }] : []),
        ],
      });
    }
    return out;
  });
  const smallShown = $derived(isTrue ? shown.filter((b) => b.small) : []);
  const smallNote = $derived(
    smallShown.length
      ? [`Belt members drawn in their own shapes, as the kernel resolves them: the largest, 10th, 100th and 1000th largest of each belt, where their orbits put them (${smallShown.length} shown).`, ...cometLooks.map((c) => cometNote(c.name, c.act))].join(" ")
      : "",
  );

  let canvas2d: HTMLCanvasElement | undefined = $state();
  $effect(() => {
    if (!lay || w === 0 || h === 0) return;
    const ink = token("--sse-ink");
    const faint = token("--sse-ink-faint", "--sse-ink-muted");
    const accent = token("--sse-accent");
    // A shell's haze is faint ink: it surrounds everything and must not crowd the orbits.
    const haze = token("--sse-ink-faint", "--sse-ink-muted");
    const ground = token("--sse-ground", "--sse-ground-raised");
    const fieldOut = token("--sse-field-out", "--sse-accent");
    const fieldIn = token("--sse-field-in", "--sse-ink");
    const colour = (b: Body3D) => b.mark?.colour ?? ink;
    if (using === "gpu" && gpu) {
      const { f, u } = basis(cam.eye, cam.look);
      if (isTrue) {
        // True: worlds in their own surfaces, a world not read yet plain and faint; only the selected orbit drawn.
        const tex = (b: Body3D) => surfaces.get(b.id);
        halo = null;
        // Shadows and eclipses (renderer eclipse.ts): every body where it really is, each world's casters turned onto
        // the way the drawing lights it. Visible light only; a star casts none on itself.
        const placed = trueBand === "visible" && !mag ? placedBodies(bodies) : null;
        const starP = placed ? [...placed.values()].find((p) => byId(p.id)?.parent === null) : undefined;
        const shadeOf = (b: Body3D) => {
          const me = placed?.get(b.id);
          if (!placed || !starP || !me || b.parent === null) return null;
          const others = [...placed.values()].filter((p) => p.id !== b.id && p.id !== starP.id).map((p) => {
            const u = umbras.get(umbraKey(p.id, b.id));
            return u ? { ...p, umbra: u } : p;
          });
          return shadeFor(me, others, starP, lightOf(b));
        };
        const items = shown.map((b) => {
            const t = tex(b);
            const isStar = b.parent === null;
            const radius = sizeOf(b, b === lay!.centre);
            // Infrared draws every body by its own heat, the star included; X-ray draws the star's corona and leaves
            // the worlds dark (they give off almost none).
            const face = isStar && !mag && trueBand !== "infrared" ? faceFor(b, radius) : undefined;
            // Mag: the worlds dark under their fields, the star a dim grey (its own field is not drawn yet).
            const emit = mag ? (isStar ? 0.35 : magShapes.has(b.id) ? 0.18 : 0.08) : trueBand === "infrared" ? thermalGrey(b.t_k ?? 0, starK) : trueBand === "xray" && !isStar ? 0 : undefined;
            // The body's face from the one door every True view uses (renderer true-body.ts): surface, rings, and
            // whatever looks other builders hook in there.
            // Its own heat in Visible (hot-glow.ts) and its plumes (plumes.ts), visible light only.
            const visible = emit === undefined && !isStar;
            const pf = visible ? plumes.get(b.id) : undefined;
            const toLight = lightOf(b);
            const a = airs.get(b.id);
            const look = trueFace({ key: b.id, star: isStar, colour: isStar ? starColour || colour(b) : faint, ...(b.radius_m ? { radius_m: b.radius_m } : {}), ...(a?.ground && a.clouds ? { surface: a.ground, clouds: a.clouds } : t ? { surface: t } : {}), ...(a?.air && emit === undefined ? { air: a.air } : {}), ...(a?.flatten ? { flatten: a.flatten } : {}), ...(b.rings && emit === undefined ? { rings: b.rings, ...(b.ring_kernel ? { ring_kernel: b.ring_kernel } : {}) } : {}), pole: poleOf(b.tilt_rad) , ...(face ? { face } : {}), ...(visible && b.t_k ? { heat: { t_k: b.t_k, star: daylightOf(b) } } : {}), ...(pf && b.radius_m ? { plumes: { facts: pf, toLight, toEye: neg(relOf(b.id)) } } : {}), ...(b.small ? { figure: b.small.figure } : {}), ...(visible && !mag ? { aurora: auroraOf(b) } : {}) });
            const shade = emit === undefined && !isStar ? shadeOf(b) : null;
            return { id: b.id, rel: relOf(b.id), radius, ring: b.mark?.ring ?? false, selected: b.id === selection, ...look, ...(emit !== undefined ? { emit } : {}), ...(shade ? { shade } : {}) };
          });
        // The controller's settings: moving, the placed tier's; on a sharpening frame, the still passes at full size.
        void qv;
        const tick = restTick;
        const now = performance.now();
        const r = run;
        const resting = r !== null && tick !== drawnRestTick;
        drawnRestTick = tick;
        const step = resting ? r!.rest.next(now) : null;
        // Sharp (or out of time): the mean on screen stays as it is. Still waiting (a timer fired a hair early): ask again.
        if (resting && !step) return void untrack(() => restLater(performance.now()));
        const q = r?.settings(step !== null) ?? null;
        const fin = q?.params["finish"] ?? {};
        const bodiesQ = q?.params["bodies"] ?? {};
        const t0 = performance.now();
        gpu.draw({
          ...(typeof bodiesQ["sphere_detail"] === "number" ? { sphere_detail: bodiesQ["sphere_detail"] } : {}),
          ...(step ? { jitter: step.jitter } : {}),
          items,
          rings: [...(hud ? lay.placed.map((b) => ({ points: ringPoints(lay!.ring.get(b.id)!), colour: b.id === selection ? accent : faint })) : []), ...(hud ? rangeLocal.map((l) => ({ points: l.points.map((q) => rel(q as [number, number, number])), colour: l.kind === "even odds" ? faint : accent })) : []), ...magRings(fieldOut, fieldIn, faint)],
          // Belts as haze in visible light only: in the other bands they give off next to nothing a screen could show.
          dots: trueBand === "visible" && !mag ? [...beltHazes.map(({ pts }) => ({ points: pts.map(rel), colour: BELT_DUST, size: 1.5 })), ...cometLooks.flatMap((c) => c.dots.map((d) => ({ ...d, points: d.points.map(rel) })))] : [],
          glows: [...(halo ? [halo] : []), ...(trueBand === "visible" && !mag ? cometLooks.map((c) => ({ ...c.coma, rel: rel(c.coma.at) })) : [])],
          forward: f,
          up: u,
          fovY: cam.fovY,
          width: w,
          height: h,
          ground,
          light: lightFor(),
          // The shared finish (VP1). While the star draws its own glow (SG1's sprite) the eye's glare stays off, so
          // the star is not glared twice; the glow moves onto the finish when the star is drawn at its real brightness.
          finish: {
            glare: halo === null,
            ...(typeof fin["glare_rungs"] === "number" ? { rungs: fin["glare_rungs"] } : {}),
            ...(typeof fin["glare_scale"] === "number" ? { glare_scale: fin["glare_scale"] } : {}),
            ...(q ? { render_scale: q.render_scale } : {}),
            ...(step ? { accumulate: step.k } : {}),
          },
          near: nearPlane,
          ...(sky && trueBand === "visible" && !mag ? { stars: { ...sky, gain: long ? longGain(sky.share, cam.fovY / h) : 1 } } : {}),
        });
        // The GPU's own time where the device has timestamps (a frame late), else the work of asking for the frame.
        const ms = Math.max(performance.now() - t0, gpu.gpuMs ?? 0);
        if (r) {
          // A sharpening frame adds to the mean; any other frame is the view moving (the governor's signal).
          if (step) r.rest.done(performance.now(), ms);
          else if (!resting) r.frame(ms, now);
          const st = r.status();
          quality = `${st.tier}|${st.level}|${st.rest}|${st.rest_frames}`;
          untrack(() => restLater(performance.now()));
        }
        // PC2: each True frame is offered to a still being taken (photograph-view.ts), in the same task as the draw so
        // the canvas still holds it; how sharp it is goes with it.
        untrack(() => host)?.dispatchEvent(new CustomEvent("sse-frame", { bubbles: true, detail: { canvas, rest: r ? r.status().rest : "sharp", backend: gpu.backend } }));
        return;
      }
      gpu.draw({
        items: shown.map((b) => ({ id: b.id, rel: relOf(b.id), radius: sizeOf(b, b === lay!.centre), colour: colour(b), ring: b.mark?.ring ?? false, selected: b.id === selection })),
        rings: [...magRings(fieldOut, fieldIn, faint), ...(!hud ? [] : [
          ...rangeLocal.map((l) => ({ points: l.points.map((q) => rel(q as [number, number, number])), colour: l.kind === "even odds" ? faint : accent })),
          ...bandLocal.map(({ b, pts }) => ({ points: pts.map(rel), colour: b.id === selection ? accent : faint })),
          ...lay.placed.map((b) => ({ points: ringPoints(lay!.ring.get(b.id)!), colour: b.id === selection ? accent : faint })),
        ])],
        dots: hud ? shellLocal.map(({ b, pts }) => ({ points: pts.map(rel), colour: b.id === selection ? accent : haze })) : [],
        forward: f,
        up: u,
        fovY: cam.fovY,
        width: w,
        height: h,
        ground,
      });
    } else if (using === "canvas" && canvas2d) {
      const dpr = typeof devicePixelRatio === "number" ? devicePixelRatio : 1;
      canvas2d.width = Math.round(w * dpr);
      canvas2d.height = Math.round(h * dpr);
      const g = canvas2d.getContext("2d")!;
      g.setTransform(dpr, 0, 0, dpr, 0, 0);
      g.clearRect(0, 0, w, h);
      // Belts and the Oort cloud first, dotted and faint, under the orbits (all HUD).
      g.setLineDash([2, 4]);
      for (const { b, pts } of hud ? bandLocal : []) {
        g.beginPath();
        let started = false;
        for (const q of pts) {
          const p = project(t, cam, lay.centre.id, q);
          if (!p) continue;
          if (started) g.lineTo(p.x, p.y);
          else (g.moveTo(p.x, p.y), (started = true));
        }
        g.strokeStyle = b.id === selection ? accent : faint;
        g.lineWidth = 1;
        g.stroke();
      }
      g.setLineDash([]);
      for (const { b, pts } of hud ? shellLocal : []) {
        g.fillStyle = b.id === selection ? accent : haze;
        for (const q of pts) {
          const p = project(t, cam, lay.centre.id, q);
          if (p) g.fillRect(p.x - 0.75, p.y - 0.75, 1.5, 1.5);
        }
      }
      // Range rings (RK12) with the HUD, as projected loops.
      for (const l of hud ? rangeLocal : []) {
        g.beginPath();
        let started = false;
        for (const q of l.points) {
          const p = project(t, cam, lay.centre.id, q as [number, number, number]);
          if (!p) continue;
          if (started) g.lineTo(p.x, p.y);
          else (g.moveTo(p.x, p.y), (started = true));
        }
        g.strokeStyle = l.kind === "even odds" ? faint : accent;
        g.lineWidth = 1;
        g.stroke();
      }
      // Orbits as projected polylines, in the same camera-relative arithmetic.
      for (const b of hud ? lay.placed : []) {
        const r = lay.ring.get(b.id)!;
        g.beginPath();
        let started = false;
        for (let i = 0; i <= SEG; i++) {
          const a = (i / SEG) * 2 * Math.PI;
          const p = project(t, cam, lay.centre.id, [r * Math.cos(a), r * Math.sin(a), 0]);
          if (!p) continue;
          if (started) g.lineTo(p.x, p.y);
          else (g.moveTo(p.x, p.y), (started = true));
        }
        g.strokeStyle = b.id === selection ? accent : faint;
        g.lineWidth = b.id === selection ? 2 : 1;
        g.stroke();
      }
      const k = h / 2 / Math.tan(cam.fovY / 2);
      for (const { b, p } of [...pins].sort((x, y) => (y.p?.depth ?? 0) - (x.p?.depth ?? 0))) {
        if (!p) continue;
        g.beginPath();
        g.arc(p.x, p.y, Math.max(2, (sizeOf(b, b === lay.centre) * k) / p.depth), 0, 2 * Math.PI);
        g.fillStyle = colour(b);
        g.fill();
      }
    }
  });

  // Dragging turns the camera about the centre.
  // With a mouse, dragging turns it and says so (the choice becomes Free); Shift-drag slides. On touch, one finger
  // scrolls the page until Free is chosen, so the chart never traps a scroll; in Free, two fingers slide.
  let dragFrom: { x: number; y: number; yaw: number; pitch: number; pan: [number, number]; slide: boolean } | null = null;
  const touches = new Map<number, { x: number; y: number }>();
  function down(e: PointerEvent) {
    // A right-click asks for the object menu (below), never turns or picks.
    if (e.button === 2 || (e.target as HTMLElement).closest("button")) return;
    if (e.pointerType === "touch") {
      if (seen !== "free") return;
      touches.set(e.pointerId, { x: e.clientX, y: e.clientY });
    }
    dragFrom = { x: e.clientX, y: e.clientY, yaw, pitch, pan: [...pan], slide: e.shiftKey || touches.size >= 2 };
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }
  function move(e: PointerEvent) {
    if (!dragFrom) return;
    const dx = e.clientX - dragFrom.x;
    const dy = e.clientY - dragFrom.y;
    if (Math.abs(dx) + Math.abs(dy) > 3 && seen !== "free") onturned?.();
    if (dragFrom.slide || touches.size >= 2) {
      // Slide along the plane, in the direction the screen moved.
      const k = dist / Math.max(1, h);
      const c = Math.cos(yaw);
      const s2 = Math.sin(yaw);
      pan = [dragFrom.pan[0] + (dx * s2 + dy * c) * k, dragFrom.pan[1] + (-dx * c + dy * s2) * k];
      return;
    }
    yaw = dragFrom.yaw - dx * 0.008;
    pitch = Math.min(1.45, Math.max(-1.45, dragFrom.pitch + dy * 0.008));
  }
  // A click picks the body nearest the pointer, not whichever pin happens to lie on top (at Real scale the inner worlds'
  // pins overlap) and not only a 26 px pin: anywhere on a body's drawn disc, or a little round a point of light.
  const radiusPx = (b: Body3D, depth: number) => (sizeOf(b, b === lay?.centre) * (b.mark?.ring ? 1.9 : 1) * (h / 2 / Math.tan(cam.fovY / 2))) / depth;
  function nearestAt(clientX: number, clientY: number): string | null {
    if (!host) return null;
    const r = host.getBoundingClientRect();
    const x = clientX - r.left;
    const y = clientY - r.top;
    let best: { id: string; d: number } | null = null;
    for (const { b, p } of pins) {
      if (!p) continue;
      const d = Math.hypot(p.x - x, p.y - y);
      if (d > Math.max(22, radiusPx(b, p.depth) + 6)) continue;
      if (!best || d < best.d) best = { id: b.id, d };
    }
    return best?.id ?? null;
  }
  function up(e: PointerEvent) {
    touches.delete(e.pointerId);
    const still = dragFrom && Math.abs(e.clientX - dragFrom.x) + Math.abs(e.clientY - dragFrom.y) < 5;
    dragFrom = null;
    if (!still || e.type !== "pointerup") return;
    const id = nearestAt(e.clientX, e.clientY);
    if (id) void flyTo(id);
  }
  // The menu on any object (V3 gap 38): a right-click near a body on the canvas asks for that body's menu, as a
  // right-click on its pin does (the pin carries data-address, which the shell watches).
  function context(e: MouseEvent) {
    if (e.shiftKey || (e.target as HTMLElement).closest("[data-address]")) return;
    const id = nearestAt(e.clientX, e.clientY);
    if (!id) return;
    e.preventDefault();
    host?.dispatchEvent(new CustomEvent("sse:object-menu", { bubbles: true, detail: { address: id, x: e.clientX, y: e.clientY } }));
  }
  function dbl(e: MouseEvent) {
    const id = nearestAt(e.clientX, e.clientY);
    if (id && id !== centreId && hasChildren(bodies, id)) enter(id);
  }
  function wheel(e: WheelEvent) {
    e.preventDefault();
    zoomBy(e.deltaY > 0 ? 1.12 : 1 / 1.12);
  }
  function key(e: KeyboardEvent) {
    const k = e.key;
    if (k.startsWith("Arrow") && seen !== "free") onturned?.();
    if (k === "ArrowLeft") yaw -= 0.12;
    else if (k === "ArrowRight") yaw += 0.12;
    else if (k === "ArrowUp") pitch = Math.min(1.45, pitch + 0.1);
    else if (k === "ArrowDown") pitch = Math.max(-1.45, pitch - 0.1);
    else if (k === "+" || k === "=") zoomBy(1 / 1.25);
    else if (k === "-") zoomBy(1.25);
    else if (k === "Escape" && centreId !== root) leave();
    else return;
    e.preventDefault();
  }
  const sel = $derived(byId(selection));
  const canEnter = $derived(sel && sel.id !== centreId && sel.parent === centreId && hasChildren(bodies, sel.id) ? sel : undefined);
</script>

<!-- The canvas area takes drag, wheel and arrow keys to turn the camera; the body buttons inside are the controls. -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
<div
  class="s3d"
  bind:this={host}
  data-aimed={aimedAt || undefined}
  bind:clientWidth={w}
  bind:clientHeight={h}
  role="application"
  aria-roledescription="3D chart"
  aria-label={centreId === root ? `3D chart of ${title}` : `3D chart of ${byId(centreId)?.name} and its moons`}
  tabindex="0"
  onpointerdown={down}
  onpointermove={move}
  onpointerup={up}
  onpointercancel={up}
  ondblclick={dbl}
  oncontextmenu={context}
  onwheel={wheel}
  data-seen={seen}
  style:touch-action={seen === "free" ? "none" : "pan-y"}
  onkeydown={key}
  data-tier={using}
  data-frame={centreId}
  data-look={isTrue && using === "gpu" ? "true" : "plain"}
  data-scale={realScale ? "real" : "readable"}
  data-hud={overlay ? "on" : "off"}
  data-dist={Math.round(dist * 1000) / 1000}
  data-flying={tweening ? "yes" : undefined}
  class:true-look={isTrue && using === "gpu"}
>
  <canvas bind:this={canvas} class="gpu" class:off={using !== "gpu"} aria-hidden="true" data-quality={quality || undefined}></canvas>
  <canvas bind:this={canvas2d} class="c2d" class:off={using !== "canvas"} aria-hidden="true"></canvas>
  {#each pins as { b, p } (b.id)}
    {#if p && p.x > -40 && p.x < w + 40 && p.y > -40 && p.y < h + 40}
      <button
        type="button"
        class="pin"
        class:sel={b.id === selection && overlay}
        class:locate={realScale && overlay}
        style="left: {p.x}px; top: {p.y}px; --r: {Math.max(7, radiusPx(b, p.depth))}px"
        aria-pressed={b.id === selection}
        data-body={b.id}
        data-address={b.id}
        data-x={p.x.toFixed(2)}
        data-y={p.y.toFixed(2)}
        onclick={(e) => void flyTo(e.detail ? (nearestAt(e.clientX, e.clientY) ?? b.id) : b.id)}
        ondblclick={(e) => (e.stopPropagation(), dbl(e))}
        data-known={pencil ? (pencil(b.id) ? "no" : "yes") : undefined}
      ><span class="dot"></span>{#if labels}<span class="name" class:hover={!hud} class:pencil={pencil?.(b.id)}>{b.name}{#if pencil?.(b.id)}<span class="sr"> (not known to the table)</span>{/if}</span>{/if}</button>
    {/if}
  {/each}
  <div class="bar">
    {#if centreId !== root}<button type="button" onclick={leave}>‹ {byId(root)?.name ?? title}</button><span>{hasChildren(bodies, centreId) ? "Moons of" : "Close up of"} <b>{byId(centreId)?.name}</b></span>{/if}
    {#if canEnter}<button type="button" onclick={() => enter(canEnter.id)}>Moons of {canEnter.name} ›</button>{/if}
  </div>
  <div class="zoom" role="group" aria-label="Zoom">
    <button type="button" aria-label="Zoom in" onclick={() => zoomBy(1 / 1.25)}>+</button>
    <button type="button" aria-label="Zoom out" onclick={() => zoomBy(1.25)}>−</button>
  </div>
  <p class="note">
    {#if lay?.unplaced.length}<span>Position not known: {lay.unplaced.map((b) => b.name).join(", ")}.</span>{/if}
    <span>{realScale ? "Real scale: distances and sizes in proportion (a world smaller than a pixel shows as a point)" : "Distances on a log scale, sizes not to scale"}{distanceNote}. Drawn with {using === "gpu" ? reason : using === "canvas" ? `Canvas 2D (${reason})` : "…"}.</span>
    {#if isTrue && (using === "canvas" || trueNote)}<span data-true-note>{using === "canvas" ? "True needs a GPU: this device draws the chart's look." : trueNote}</span>{/if}
    {#if isTrue && using === "gpu" && ringNote && trueBand === "visible" && !mag}<span data-true-rings={ringed.length} data-true-belts={beltHazes.length}>{ringNote}</span>{/if}
    {#if using === "gpu" && plumeNotes}<span data-true-plumes>{plumeNotes}</span>{/if}
    {#if isTrue && using === "gpu" && auroraNote}<span data-true-aurorae={shown.filter((b) => aurorae.has(b.id)).length}>{auroraNote}</span>{/if}
    {#if isTrue && using === "gpu" && smallNote && trueBand === "visible" && !mag}<span data-true-small={smallShown.length} data-true-comets={cometLooks.length}>{smallNote}</span>{/if}
    {#if isTrue && bandNote && using === "gpu"}<span data-true-band={trueBand}>{bandNote}</span>{/if}
    {#if mag}<span data-true-band="mag" data-mag-worlds={magShapes.size}>{magNote}</span>{/if}
    {#if motionNote}<span data-clock-rate={clock?.rate ?? 0} data-clock-fast={fastNames.length}>{motionNote}</span>{/if}
    {#if isTrue && star && using === "gpu"}<span data-star-live={flaringNow ? "flaring" : "quiet"} data-star-turn={turnAt(star, starT).toFixed(1)} data-star-ejections={starNow?.ejections.length ?? 0} data-star-calm={calm ? "yes" : "no"}>The star turns on its own spin, its spots go round with it and its cells live their own lives, on the campaign's clock{flaringNow ? "; it is flaring now" : ""}. {(starNow?.words ?? []).join(" ")}</span>{/if}
  </p>
</div>

<style>
  .s3d {
    position: relative;
    width: 100%;
    height: 100%;
    min-height: 240px;
    overflow: hidden;
    touch-action: none;
    background: var(--sse-ground);
    outline: none;
    cursor: grab;
  }
  .s3d:focus-visible {
    box-shadow: inset 0 0 0 2px var(--sse-focus);
  }
  canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }
  .off {
    visibility: hidden;
  }
  .pin {
    position: absolute;
    transform: translate(-50%, -50%);
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px;
    border: 0;
    background: transparent;
    color: var(--sse-ink);
    font-family: var(--sse-font-display);
    font-style: italic;
    font-size: 14px;
    cursor: pointer;
  }
  /* Pencil (track SV): a name the table does not know, quiet and dotted, the same as on the flat chart. */
  .pin .name.pencil {
    color: var(--sse-ink-muted);
    text-decoration: underline dotted;
    text-underline-offset: 3px;
  }
  .sr {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0 0 0 0);
  }
  .pin .dot {
    width: 14px;
    height: 14px;
    border-radius: 50%;
  }
  /* Real scale: a faint ring finds a world too small to see (its name on hover in True). */
  .pin.locate .dot {
    box-shadow: 0 0 0 1px var(--sse-ink-faint, var(--sse-ink-muted));
  }
  .pin.sel .dot {
    outline: 1.5px dashed var(--sse-accent);
    outline-offset: 3px;
  }
  .pin:focus-visible .dot {
    outline: 2px dashed var(--sse-focus);
    outline-offset: 3px;
  }
  .pin .name {
    position: absolute;
    left: calc(50% + var(--r) + 6px);
    white-space: nowrap;
    text-shadow: 0 0 3px var(--sse-ground), 0 0 3px var(--sse-ground);
  }
  /* HUD off: names only on hover or focus. True: text sits on a chip over black space. */
  .pin .name.hover {
    visibility: hidden;
  }
  .pin:hover .name.hover,
  .pin:focus-visible .name.hover {
    visibility: visible;
  }
  .true-look .pin .name,
  .true-look .note span {
    background: var(--sse-ground);
    padding: 0 4px;
    text-shadow: none;
  }
  /* Low in the stage: the view controls run along the top. */
  .bar {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    bottom: 64px;
    /* Over the note: in True the note can run to more lines than the gap below the bar, and it must never cover
       the way back out of a close-up. */
    z-index: 2;
    display: flex;
    gap: 10px;
    align-items: center;
    font-size: 13px;
    color: var(--sse-ink-muted);
  }
  .bar b {
    color: var(--sse-ink);
    font-family: var(--sse-font-display);
    font-style: italic;
    font-weight: normal;
  }
  .bar button,
  .zoom button {
    min-height: 36px;
    min-width: 40px;
    padding: 0 10px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    cursor: pointer;
  }
  .zoom {
    position: absolute;
    right: 12px;
    bottom: 12px;
    display: flex;
  }
  .zoom button + button {
    border-left: 0;
  }
  .note {
    position: absolute;
    left: 12px;
    bottom: 8px;
    right: 110px;
    margin: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 4px 12px;
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
</style>
