<!--
  System chart view: the star at the centre, orbits on a log-radial scale,
  zones and belts as bands, unsurveyed territory hatched, and every body a
  keyboard-reachable control. Marks come from the symbology; the view only
  places them. A planet with moons opens its own frame: the planet at the
  centre and its moons about it (handover/rendering-rules.md), in kilometres.
-->
<script lang="ts" module>
  import { withFiner, TILE_SAMPLES, type AirShape as AirKept, type FieldReadJson, type LiveStar, type TrueTexture as Kept } from "../../../renderer/src/index.ts";
  /** Surfaces read for True, by system and world, kept for the page (a string: why the world has none to show). */
  const trueSurfaces = new Map<string, Kept | string>();
  /** Plume facts from the same reads (the world's surface record), by system and world. */
  const truePlumes = new Map<string, import("../../../renderer/src/index.ts").PlumeFacts>();
  /** What a world's True face could not show as it is (renderer true-air.ts), by system and world. */
  const trueSaid = new Map<string, string[]>();
  /** Its clouds alone, its ground under them, and its air (VP-A3, VP-A4), by system and world. */
  type TrueAir = { ground?: Kept; clouds?: Kept; air?: AirKept; flatten?: number };
  const trueAirs = new Map<string, TrueAir>();
  /** Lays finer tiles (VP3 part 2) over the ground under a world's clouds; the new entry, or null when nothing changed. */
  function finerGround(key: string, fine: Parameters<typeof withFiner>[1]): TrueAir | null {
    const a = trueAirs.get(key);
    if (!a?.ground?.faces || !fine.length) return null;
    const ground = withFiner(a.ground, fine);
    if (ground === a.ground) return null;
    const next = { ...a, ground };
    trueAirs.set(key, next);
    return next;
  }
  /** Stars read for True's living star, by system and star (a string: why there is none). */
  const liveStars = new Map<string, LiveStar | string>();
  /** read_field's Solar System fixture, read once for the page (Mag, V10e). */
  let solFields: Promise<FieldReadJson[]> | null = null;
</script>

<script lang="ts">
  import { untrack } from "svelte";
  import { KM_PER_AU, gearing, nearestCatch, resonanceCatches, roundFigures, chosenReading, sayText, sheetUnitFactor, isShell, asPlayers, distanceWords, lightWords, separation, hasLayer, hasTool, childrenOf, commitEdit, factEdit, formsHere, roundTo, sayNumber, frameFor, legendFor, primaryOf, sig, subjectOf, viewOptions, hudShows, placeLabels, textWidth, DEFAULT_VIEW, type Box, type Json, type SceneSubject, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { SYSTEM_CHART_CONTROLS } from "./controls.ts";
  import RangeRings from "./RangeRings.svelte";
  import { chartProject, holderAt } from "./range-place.ts";
  import { System3D, bandsFromScene, fromSmallSample, fromSystemScene, readSmallSample, type SmallReader, type SmallSample } from "../../system-3d/src/index.ts";
  import { plumeFacts, readLiveStar, readTrueFace, readTrueShape, starsBehind, starCss, type AirShape, type FaceReader, type PlumeFacts, type ShapeReader, type StarReader, type TrueTexture, type GroundView } from "../../../renderer/src/index.ts";
  import { finerTiles, readDeep, readFiner } from "../../body/src/finer-tiles.ts";
  import { ChartGround, CompositeStack, HatchDefs, Hatch, Ink, Legend, Mark, Pencilled, PEN_WORDS, penOf, penStyle, Stamp, markExtent, distanceAt, scaleRings, easing, fitScale, motionMs, polar, radius } from "../../../chart-grammar/src/index.ts";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();

  const prefix = "sc";
  let w = $state(0);
  let h = $state(0);
  let zoom = $state(1);
  // What lies over the chart, in its own pixels (measured in measureOverlays below).
  let overlays = $state<Box[]>([]);
  $effect(() => {
    zoom = Number(params.zoom);
  });

  const narrow = $derived(w < 700);
  const cx = $derived(w * (narrow ? 0.5 : Number(params.centre_x)));
  // On a phone the key and the zoom sit over the foot of the chart: the system is drawn in the band above them, so no
  // world hides under them (overlays are measured below; until then the whole height).
  const band = $derived.by(() => {
    if (!narrow) return { top: 0, bottom: h };
    const top = Math.max(0, ...overlays.filter((o) => o.y + o.h < h * 0.4).map((o) => o.y + o.h));
    const bottom = Math.min(h, ...overlays.filter((o) => o.y > h * 0.5).map((o) => o.y));
    return bottom - top > 160 ? { top, bottom } : { top: 0, bottom: h };
  });
  const cy = $derived((band.top + band.bottom) / 2);
  const star = $derived(primaryOf(scene));
  // Orbits are local to their parent (handover/rendering-rules.md). The chart shows one frame at a time:
  // the star's (frame null) with its planets, or a planet's with its moons. Choosing a moon anywhere opens
  // its planet's frame; choosing a planet returns to the star's unless that planet is the centre.
  let frame = $state<string | null>(null);
  const framed = $derived(frame ? scene.subjects.find((s) => s.address === frame) : undefined);
  const centre = $derived(framed ?? star);
  const orbiting = $derived(centre ? childrenOf(scene, centre.address) : []);
  const moonsOf = (s: SceneSubject) => childrenOf(scene, s.address).length;
  $effect(() => {
    const sel = ctx.workspace.selection;
    if (frame && !framed) frame = null;
    if (sel && sel !== frame) {
      const f = frameFor(scene, sel);
      if (f !== frame) frame = f;
    }
  });
  // Entering a planet's frame grows the chart out of the planet; leaving shrinks back into the system (U12).
  function frameMotion(into: boolean, from?: [number, number]) {
    const el = svgEl;
    const ms = motionMs(el ?? null);
    if (!el || ms <= 0 || typeof el.animate !== "function") return;
    const [ox, oy] = from ?? [cx, cy];
    el.animate(
      into
        ? [{ transformOrigin: `${ox}px ${oy}px`, transform: "scale(0.4)", opacity: 0 }, { transformOrigin: `${ox}px ${oy}px`, transform: "none", opacity: 1 }]
        : [{ transformOrigin: `${cx}px ${cy}px`, transform: "scale(1.6)", opacity: 0 }, { transformOrigin: `${cx}px ${cy}px`, transform: "none", opacity: 1 }],
      { duration: ms * 1.4, easing: easing(el) },
    );
  }
  function openFrame(s: SceneSubject) {
    frameMotion(true, pos(s));
    frame = s.address;
    zoom = Number(params.zoom);
    ctx.workspace.select(s.address);
  }
  function closeFrame() {
    const back = frame;
    frameMotion(false);
    frame = null;
    zoom = Number(params.zoom);
    if (back) ctx.workspace.select(back);
  }
  const bands = $derived(frame ? [] : scene.subjects.filter((s) => s.band_au));
  // A system read from a V3 save is authored there: its names (even "Earth") and values are the save's, not the physics'.
  const authoredImport = $derived(!scene.fixture && scene.subjects.some((s) => s.stamps.includes("imported")));
  const catalogued = $derived(!scene.fixture && !authoredImport && scene.subjects.length > 0 && scene.subjects.every((s) => s.stamps.includes("catalogued")));
  const scale = $derived(
    fitScale(
      // A sphere round the system (the Oort cloud) is drawn as a haze from where it begins; the fit frames the worlds and
      // belts, so they fill the chart instead of huddling in the middle of a cloud a thousand times wider.
      [...orbiting.map((s) => s.orbit!.a_au), ...bands.filter((b) => !isShell(b)).flatMap((b) => b.band_au!)],
      narrow ? 22 : 34,
      Math.max(60, Math.min(w, band.bottom - band.top) * (narrow ? 0.47 : 0.44) * zoom),
    ),
  );
  const r = (au: number) => radius(au, drag ? drag.scale : scale);
  const surveyR = $derived(r(scene.surveyed_to_au));
  // How far out a drag may move an orbit: the survey edge in the star's frame, a little past the outermost moon in a planet's.
  const limitAu = $derived(frame ? scale.outer_au * 2 : scene.surveyed_to_au);
  const km = (au: number) => `${sig(au * KM_PER_AU, 3).replace(/\B(?=(\d{3})+(?!\d))/g, ",")} km`;
  // An orbit's size as the units engine says it (ED1a): AU round a star, thousand km round a planet, in the campaign's
  // chosen reading, at the workspace depth.
  const orbitAbout = $derived({ class: frame ? "moon" : "planet", what: "orbit" });
  const dist = (au: number) =>
    sayText(au * sheetUnitFactor("au"), "length", orbitAbout, { depth: ctx.workspace.depth ?? "science", reading: chosenReading(ctx.workspace.unit_choices, "length", orbitAbout) });
  const unsurveyed = $derived(
    `M0 0H${w}V${h}H0Z M${cx - surveyR} ${cy}a${surveyR} ${surveyR} 0 1 0 ${surveyR * 2} 0a${surveyR} ${surveyR} 0 1 0 ${-surveyR * 2} 0Z`,
  );
  // The grid is the scale (log-radial, so not squares): a solid ring at each power of ten (AU round a star, km round a
  // planet), fainter rings at 2 and 5 times it when there is room, out to the chart's corners.
  const rings = $derived(scaleRings(scale, Math.hypot(Math.max(cx, w - cx), Math.max(cy, h - cy)), frame ? 1 / KM_PER_AU : 1));
  const decades = $derived(rings.filter((g) => g.major));
  const unitWord = $derived(frame ? "km" : "AU");

  const label = (s: SceneSubject) => s.label;
  // The GM's lens (U9) decides which layers are drawn; it never changes the data.
  const L = $derived(ctx.workspace.lens);
  // HUD off (the shared view controls, Alex 23 Nov): the chart shows the bodies and their belts as they are, without
  // the overlay of scale rings, distances, orbit lines, names, the survey hatch or the temperate zone.
  const on = (id: Parameters<typeof hasLayer>[1]) => hasLayer(L, id) && hudShows(shownView.hud, id);
  // What the lens's audience knows: each body's data view, from the kernel's scene for them.
  const known = $derived(ctx.workspace.lensScene ? new Map(ctx.workspace.lensScene.subjects.map((x) => [x.address, x.view])) : null);
  const through = $derived(asPlayers(L) && known !== null);
  const knows = (s: SceneSubject) => !known || s === star || known.has(s.address);
  const pencilled = (id: string) => { const s = all.find((x) => x.address === id); return !!s && !knows(s); };
  // How the table knows most bodies here: said once in the legend, and under a name only where it differs.
  const usualView = $derived.by(() => {
    if (!known) return null;
    const n = new Map<string, number>();
    for (const v of known.values()) if (v) n.set(v, (n.get(v) ?? 0) + 1);
    let best: string | null = null;
    for (const [v, c] of n) if (best === null || c > n.get(best)!) best = v;
    return best;
  });
  const knownCount = $derived(known ? scene.subjects.filter((x) => (x.orbit || x.band_au) && known.has(x.address)).length : 0);
  const bodyCount = $derived(scene.subjects.filter((x) => x.orbit || x.band_au).length);
  // The key is generated from the symbology for the bodies on this chart.
  const key = $derived.by(() => {
    const host = ctx.symbology;
    if (!host) return null;
    const m = host.marker();
    const doc = host.registry().find((d) => d.id === m.id);
    return { entries: legendFor(m, { present: scene.subjects.map(subjectOf), names: host.names(), order: Array.isArray(doc?.legend) ? doc.legend : [] }), note: m.collapse, name: doc?.name ?? m.id };
  });

  // Dragging a body along its radius moves its orbit; the edit is an intent
  // sent when the drag ends. The scale is frozen during the drag.
  // ED3 part 2: the drag is geared (the further the pointer from the line, the less a move counts; Shift a tenth) and
  // lightly magnetic (round figures, resonances with the neighbours; named as it catches; Alt runs free).
  let drag = $state<{ address: string; a_au: number; moved: boolean; scale: typeof scale; lastD: number; rpx: number; caught: string | null } | null>(null);
  let problem = $state<string | null>(null);
  const editable = (s: SceneSubject) => ctx.workspace.mode === "edit" && hasTool(L, "drag") && !!s.orbit && !!s.facts.find((f) => f.field === "a_au" && f.editable);
  // The world stays put while dragging; a ghost and its orbit line follow the pointer (editing design section 4).
  const aOf = (s: SceneSubject) => s.orbit!.a_au;
  const rr = (au: number) => radius(au, drag ? drag.scale : scale);
  // Range rings (RK12): whatever the host feeds (a pack's weapons), placed round each holder shown on this chart.
  const rangeProject = $derived(chartProject(cx, cy, rr));
  const rangeSets = $derived(
    (ctx.rangeRings?.() ?? []).flatMap((g) => {
      const s = all.find((b) => b.address === g.holder);
      return s && g.rings.length ? [{ ...g, centre_m: s === centre ? ([0, 0] as [number, number]) : holderAt(s.orbit) }] : [];
    }),
  );
  const pos = (s: SceneSubject): [number, number] => polar(cx, cy, rr(aOf(s)), s.orbit!.phase_deg);
  let svgEl: SVGSVGElement | undefined = $state();

  // Rounded to three figures in the frame's own unit (AU for planets, km for moons), held in AU.
  function round(a: number): number {
    return frame ? Number(sig(a * KM_PER_AU, 3)) / KM_PER_AU : Number(sig(a, 3));
  }

  function startDrag(e: PointerEvent, s: SceneSubject) {
    if (!editable(s) || e.button !== 0) return;
    svgEl!.setPointerCapture(e.pointerId);
    const box = svgEl!.getBoundingClientRect();
    drag = { address: s.address, a_au: s.orbit!.a_au, moved: false, scale, lastD: Math.hypot(e.clientX - box.left - cx, e.clientY - box.top - cy), rpx: radius(s.orbit!.a_au, scale), caught: null };
  }

  function moveDrag(e: PointerEvent) {
    if (!drag) return;
    const box = svgEl!.getBoundingClientRect();
    const [x, y] = [e.clientX - box.left - cx, e.clientY - box.top - cy];
    const d = Math.hypot(x, y);
    const s = scene.subjects.find((q) => q.address === drag!.address);
    if (!s?.orbit) return;
    // Off the line: the pointer's distance from the radius through the world.
    const [ux, uy] = polar(0, 0, 1, s.orbit.phase_deg);
    const off = Math.abs(x * uy - y * ux);
    const lo = radius(drag.scale.inner_au, drag.scale);
    const hi = radius(limitAu, drag.scale);
    const rpx = Math.min(hi, Math.max(lo, drag.rpx + (d - drag.lastD) * gearing(off, e.shiftKey)));
    const raw = Math.min(limitAu, Math.max(drag.scale.inner_au, distanceAt(rpx, drag.scale)));
    const neighbours = scene.subjects.filter((q) => q.address !== s.address && q.orbit && q.parent === s.parent).map((q) => ({ label: q.label, a: q.orbit!.a_au }));
    const c = e.altKey ? null : nearestCatch(rpx, [...roundFigures(raw, frame ? KM_PER_AU : 1, frame ? "km" : "AU"), ...resonanceCatches(raw, neighbours)], (v) => radius(v, drag!.scale));
    const a = c ? c.at : round(raw);
    drag = { ...drag, rpx, lastD: d, caught: c?.says ?? null, ...(a !== drag.a_au ? { a_au: a, moved: true } : {}) };
  }

  // ED4, Add with the ghost first (editing design section 5): while the shell is adding round the body at this chart's
  // centre, a ghost and its orbit follow the pointer with the distance; a press drops it there (Add here opens with that
  // orbit), Esc puts it away. Catches as the drag's: round figures, resonances with the worlds already there.
  const adding = $derived.by(() => {
    const a = ctx.workspace.adding;
    return a && a.a_au === null && centre && a.host === centre.address ? a : null;
  });
  let ghostAt = $state<{ a_au: number; x: number; y: number; caught: string | null } | null>(null);
  function moveGhost(e: PointerEvent) {
    if (!adding) return void (ghostAt = null);
    const box = svgEl!.getBoundingClientRect();
    const [x, y] = [e.clientX - box.left, e.clientY - box.top];
    const d = Math.hypot(x - cx, y - cy);
    const raw = Math.min(limitAu, Math.max(scale.inner_au, distanceAt(d, scale)));
    const neighbours = orbiting.filter((q) => q.orbit).map((q) => ({ label: q.label, a: q.orbit!.a_au }));
    const c = e.altKey ? null : nearestCatch(d, [...roundFigures(raw, frame ? KM_PER_AU : 1, frame ? "km" : "AU"), ...resonanceCatches(raw, neighbours)], (v) => radius(v, scale));
    ghostAt = { a_au: c ? c.at : round(raw), x, y, caught: c?.says ?? null };
  }
  // ED4 part 2a: while the ghost rests on an orbit round a star, one formation run round a star like this one says
  // what forms there (the first of the runs Add here offers, so the two agree). Each orbit is asked once.
  let formsAt = $state<{ key: string; words: string } | null>(null);
  const formsSaid = new Map<string, string>();
  const formsKey = (a_au: number) => `${scene.system} ${centre?.address} ${roundTo(a_au, 3)}`;
  $effect(() => {
    const g = ghostAt;
    const c = centre;
    const system = scene.system;
    const m = c?.facts.find((f) => f.field === "mass_sun")?.value;
    if (!adding || !g || !c || c.kind !== "star" || !system || typeof m !== "number") return;
    const key = formsKey(g.a_au);
    const said = formsSaid.get(key);
    if (said !== undefined) return void (formsAt = { key, words: said });
    const t = setTimeout(async () => {
      const r = await formsHere(ctx.session, { system, star_mass_sun: m, a_au: g.a_au, tries: 1 });
      if (!r.ok) return;
      const f = r.found[0];
      const words = f ? `forms a world of ${sayNumber(f.mass_earth, 2)} Earth masses here` : "no world forms here in one run";
      formsSaid.set(key, words);
      formsAt = { key, words };
    }, 400);
    return () => clearTimeout(t);
  });
  function dropGhost(e: PointerEvent) {
    if (!adding) return;
    e.stopPropagation();
    e.preventDefault();
    if (!ghostAt) moveGhost(e);
    if (ghostAt) ctx.workspace.adding = { ...adding, a_au: ghostAt.a_au };
    ghostAt = null;
  }
  $effect(() => {
    if (!adding) return void (ghostAt = null);
    const cancel = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      e.preventDefault();
      e.stopPropagation();
      ctx.workspace.adding = null;
    };
    window.addEventListener("keydown", cancel, true);
    return () => window.removeEventListener("keydown", cancel, true);
  });

  // Esc while dragging puts the orbit back and writes nothing (editing design section 4).
  $effect(() => {
    if (!drag) return;
    const cancel = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      e.preventDefault();
      e.stopPropagation();
      drag = null;
    };
    window.addEventListener("keydown", cancel, true);
    return () => window.removeEventListener("keydown", cancel, true);
  });

  async function endDrag() {
    const d = drag;
    if (!d) return;
    const s = scene.subjects.find((x) => x.address === d.address)!;
    if (!d.moved || d.a_au === s.orbit!.a_au) {
      drag = null;
      ctx.workspace.select(d.address);
      return;
    }
    await moveOrbit(s, d.a_au);
    drag = null;
  }

  async function moveOrbit(s: SceneSubject, a: number) {
    const f = s.facts.find((x) => x.field === "a_au")!;
    const e = factEdit(f, a);
    const r = await commitEdit(ctx.session, ctx.workspace, {
      address: s.address,
      field: e.field,
      value: e.value,
      display: dist(a),
      before: { value: e.before, display: f.display },
    });
    problem = r.status === "applied" ? null : r.status === "pending" ? "Sent to the GM to approve." : r.status === "refused" ? r.reason : r.message;
    ctx.workspace.select(s.address);
  }
  const showLabel = (s: SceneSubject) =>
    on("labels") && (params.labels === "all" || (params.labels === "selected" && ctx.workspace.selection === s.address));

  const focusables: SVGGElement[] = $state([]);
  function onkey(e: KeyboardEvent, i: number, s: SceneSubject) {
    if ((e.key === "PageUp" || e.key === "PageDown") && editable(s)) {
      e.preventDefault();
      void moveOrbit(s, round(s.orbit!.a_au * (e.key === "PageUp" ? 1.05 : 1 / 1.05)));
    } else if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      if (measuring) measurePick(s);
      else ctx.workspace.select(s.address);
    } else if (e.key === "Escape" && frame) {
      e.preventDefault();
      closeFrame();
    } else if (["ArrowRight", "ArrowDown", "ArrowLeft", "ArrowUp"].includes(e.key)) {
      e.preventDefault();
      const n = focusables.length;
      const next = e.key === "ArrowRight" || e.key === "ArrowDown" ? (i + 1) % n : (i - 1 + n) % n;
      focusables[next]?.focus();
    }
  }
  const all = $derived(centre ? [centre, ...orbiting] : orbiting);
  const selected = $derived(scene.subjects.find((s) => s.address === ctx.workspace.selection));
  // Where each name sits (chart-labels.ts): round its mark, clear of the other names and marks, the belt names and the
  // scale figures, inside the chart. A body that is not named takes no place.
  const bandLabels = $derived(
    (on("zones") && on("labels") ? bands : []).map((b) => {
      const [lx, ly] = polar(cx, cy, isShell(b) ? r(b.band_au![0]) + 14 : r(b.band_au![1]) + 10, b.kind === "zone" ? -62 : isShell(b) ? 132 : 58);
      const text = `${label(b)}${isShell(b) ? " · sphere" : ""}`;
      const bw = Math.ceil(text.length * 11 * 0.78);
      return { b, lx, ly, text, box: { x: lx, y: ly - 11, w: bw, h: 13 } as Box };
    }),
  );
  // What lies over the chart (the view controls, tools, card, key and zoom: anything marked data-chart-overlay), in the
  // chart's own pixels, measured after each layout change: no name or scale figure is put underneath.
  function measureOverlays() {
    const svg = svgEl;
    if (!svg) return;
    const sb = svg.getBoundingClientRect();
    const root = svg.closest(".stage") ?? svg.parentElement ?? document;
    overlays = [...root.querySelectorAll("[data-chart-overlay]")]
      .map((e) => e.getBoundingClientRect())
      .filter((b) => b.width > 0 && b.height > 0 && b.right > sb.left && b.left < sb.right && b.bottom > sb.top && b.top < sb.bottom)
      .map((b) => ({ x: b.left - sb.left, y: b.top - sb.top, w: b.width, h: b.height }));
  }
  $effect(() => {
    void [w, h, ctx.workspace.selection, frame, ctx.workspace.view, ctx.workspace.desk];
    const raf = requestAnimationFrame(measureOverlays);
    // Cards and dialogs open with a short motion; measure again once they have settled.
    const t = setTimeout(measureOverlays, 450);
    return () => (cancelAnimationFrame(raf), clearTimeout(t));
  });
  const under = (b: Box) => overlays.some((o) => b.x < o.x + o.w && o.x < b.x + b.w && b.y < o.y + o.h && o.y < b.y + b.h);
  const distLabels = $derived(
    (on("distances") ? decades : [])
      .filter((g) => cy - r(g.au) > 12)
      .map((g) => ({ g, y: cy - r(g.au) - 4, text: frame ? km(g.au) : `${sayNumber(g.units, 0, { upTo: 3 })} AU` }))
      .filter((d) => !under({ x: cx + 2, y: d.y - 10, w: d.text.length * 6.6 + 4, h: 12 })),
  );
  const named = $derived(all.filter((s) => showLabel(s) && (s.orbit || s === centre) && !(s === centre && !frame)));
  const places = $derived.by(() => {
    if (!w || !h) return new Map();
    const sel = ctx.workspace.selection;
    const bodies = all.map((s) => {
      const atCentre = s === centre;
      const [x, y] = atCentre ? [cx, cy] : pos(s);
      const m0 = ctx.mark(s);
      const m = atCentre && frame ? { ...m0, size: m0.size * 2.2 } : m0;
      const px = s.address === sel ? 18 : 16;
      return { s, b: { id: s.address, x, y, r: Math.max(markExtent(m), m.size) + 3, w: textWidth(label(s), px), h: px, first: s.address === sel } };
    });
    const label_ = new Set(named.map((s) => s.address));
    // Bodies that are not named still keep their marks clear.
    const clear: Box[] = [
      ...overlays.map((o) => ({ ...o, w: o.w, h: o.h })),
      ...bandLabels.map((l) => l.box),
      ...distLabels.map((d) => ({ x: cx + 2, y: d.y - 10, w: d.text.length * 6.6 + 4, h: 12 })),
      ...bodies.filter((x) => !label_.has(x.s.address)).map((x) => ({ x: x.b.x - x.b.r, y: x.b.y - x.b.r, w: x.b.r * 2, h: x.b.r * 2 })),
    ];
    const placed = placeLabels(bodies.filter((x) => label_.has(x.s.address)).map((x) => x.b), clear, { x: 4, y: 4, w: w - 8, h: h - 8 });
    return new Map(placed.map((p) => [p.id, p]));
  });

  const canOpen = $derived(!frame && selected && selected !== star && moonsOf(selected) > 0 ? selected : undefined);
  // V6: the same frame in 3D (view only; edits stay on the 2D chart).
  // The standard view controls (rule 12) choose it: Chart is this flat log chart, 3D turns. A view without a shell's
  // choice (a screen document elsewhere) stays on Chart.
  const viewed = $derived(viewOptions(SYSTEM_CHART_CONTROLS, ctx.workspace.view ?? DEFAULT_VIEW));
  const shownView = $derived(viewed.shown);
  // Pencil (track SV): a body the table does not know is drawn in pencil on the GM's chart, on the Build and Run desks
  // whatever the lens, and wherever a lens carries the "What the table knows" layer; never through a player lens.
  const pencilOn = $derived(!!known && !through && hudShows(shownView.hud, "known") && (hasLayer(L, "known") || ctx.workspace.desk === "build" || ctx.workspace.desk === "run"));
  const dims = $derived(shownView.style === "3d" || shownView.style === "true" ? "3d" : "2d");
  // True (V10a): each world's surface is read from its level-0 tiles (the Body globe's own, about 3 s a world in the
  // kernel), one at a time, the selected world first, and kept for this page. Only on the GM's seat, which reads tiles.
  const looksTrue = $derived(shownView.style === "true");
  // True (VP2): the sky's stars from this system's place, read once per system and drawn behind everything; from the
  // Sun's place, laid square on the system's frame, when the system has none yet (fudge register).
  const SKY_LIMIT = 6.5;
  const SQUARE = { to_galactic: [[1, 0, 0], [0, 1, 0], [0, 0, 1]] as [[number, number, number], [number, number, number], [number, number, number]] };
  let skyBehind = $state<{ key: string; dirs: Float32Array; share: Float32Array } | null>(null);
  $effect(() => {
    const sys = scene.system;
    if (!looksTrue || skyBehind?.key === sys) return;
    // The worlds' surfaces first: they share the kernel with the sky, and a world's face matters more than its stars.
    if (ctx.session.seat.authority === "gm" && (!reading || reading.done < reading.of)) return;
    untrack(() =>
      void (async () => {
        const ask = { limit_mag: SKY_LIMIT, band: "V", bands: ["B", "V"] };
        let r = await ctx.session.skyFromHere({ ...ask, observer: sys });
        if (!r.ok) r = await ctx.session.skyFromHere(ask);
        const sky = r.ok ? (r.sky as { frame?: unknown; stars?: { dir: number[]; luminosity_lsun: number; distance_pc: number; teff_k?: number; components?: { teff_k: number }[] }[] }) : null;
        if (!sky?.stars || scene.system !== sys) return;
        const f = sky.frame as { to_galactic?: unknown } | null | undefined;
        const frame = f && typeof f === "object" && Array.isArray(f.to_galactic) ? (f as typeof SQUARE) : SQUARE;
        const stars = sky.stars.map((x) => ({ dir: [x.dir[0]!, x.dir[1]!, x.dir[2]!] as [number, number, number], luminosity_lsun: x.luminosity_lsun, distance_pc: x.distance_pc, teff_k: x.teff_k ?? x.components?.[0]?.teff_k ?? 5800 }));
        skyBehind = { key: sys, ...starsBehind(stars, frame) };
      })(),
    );
  });
  let surfaces = $state<Map<string, TrueTexture>>(new Map());
  // Each world's plume facts from the same read (its surface record), for plumes above the limb.
  let plumes = $state<Map<string, PlumeFacts>>(new Map());
  let airs = $state<Map<string, TrueAir>>(new Map());
  let reading = $state<{ done: number; of: number; none: number; why: string } | null>(null);
  let trueGen = 0;
  $effect(() => {
    const sys = scene.system;
    const sel = ctx.workspace.selection;
    const g = ++trueGen;
    if (!looksTrue || ctx.session.seat.authority !== "gm") return void (reading = null);
    const worlds = scene.subjects.filter((s) => s.orbit && s.kind !== "star" && s.kind !== "belt");
    const order = [...worlds.filter((s) => s.address === sel), ...worlds.filter((s) => s.address !== sel)];
    // The reader runs untracked: worlds already kept are put in `surfaces` without waiting, so a tracked read of
    // `surfaces` here would make the effect depend on what it writes and run again without end (the tab froze on
    // choosing a world once one surface was kept, Alex 23 Nov).
    untrack(() => void (async () => {
      let none = 0;
      let noneWhy = "";
      let done = 0;
      for (const s of order) {
        const key = `${sys}|${s.address}`;
        if (!trueSurfaces.has(key)) {
          reading = { done, of: order.length, none, why: noneWhy };
          // Ground, then today's clouds and haze; a giant is its cloud decks (renderer true-air.ts). The surface record
          // read on the way carries the world's plume facts (renderer plumes.ts).
          let record: unknown = null;
          const k = ctx.session as unknown as FaceReader;
          // The session's own methods, bound to it (they use its private state), with its tile read noting the record.
          const reader = new Proxy(k, {
            get: (t, p) =>
              p === "surfaceTiles"
                ? async (a: string, b: string) => {
                    const r = await ctx.session.surfaceTiles(a, b);
                    if (r.ok) record = r.record ?? null;
                    return r;
                  }
                : typeof (t as unknown as Record<string | symbol, unknown>)[p] === "function"
                  ? ((t as unknown as Record<string | symbol, (...x: unknown[]) => unknown>)[p]!).bind(t)
                  : (t as unknown as Record<string | symbol, unknown>)[p],
          });
          const f = await readTrueFace(reader, sys, s.address);
          trueSurfaces.set(key, typeof f === "string" ? f : f.tex);
          if (typeof f !== "string") trueSaid.set(key, f.said);
          const pf = plumeFacts(record);
          if (pf) truePlumes.set(key, pf);
          // Its clouds alone over its ground, and its air's rim (VP-A3, VP-A4).
          // And how much its spin flattens it (TA6).
          const shape = await readTrueShape(ctx.session as unknown as ShapeReader, sys, s.address);
          if (typeof shape !== "string") trueSaid.set(key, [...(trueSaid.get(key) ?? []), `${label(s)}: ${shape.said}.`]);
          const flat = typeof shape === "string" ? {} : { flatten: shape.f };
          if (typeof f !== "string" && (f.clouds || f.air || "flatten" in flat)) trueAirs.set(key, { ...(f.ground ? { ground: f.ground } : {}), ...(f.clouds ? { clouds: f.clouds } : {}), ...(f.air ? { air: f.air } : {}), ...flat });
          // Finer tiles already read on this page (VP3 part 2) are laid over the ground at once.
          const kept = trueSurfaces.get(key);
          if (kept && typeof kept !== "string") trueSurfaces.set(key, withFiner(kept, finerTiles(key)));
          finerGround(key, finerTiles(key));
        }
        const kp = truePlumes.get(key);
        if (kp && g === trueGen) plumes = new Map(plumes).set(s.address, kp);
        if (g !== trueGen) return;
        const t = trueSurfaces.get(key);
        if (t && typeof t !== "string") surfaces = new Map(surfaces).set(s.address, t);
        else (none++, (noneWhy = String(t)));
        const a = trueAirs.get(key);
        if (a && airs.get(s.address) !== a) airs = new Map(airs).set(s.address, a);
        done++;
      }
      reading = { done, of: order.length, none, why: noneWhy };
      // VP3 part 2: finer tiles for the chosen world only, four at a time, once it has been left chosen for a while
      // (finer-tiles.ts); stops between requests when another world is chosen or True is left.
      const chosen = order[0];
      const ck = chosen && chosen.address === sel ? `${sys}|${chosen.address}` : "";
      const base = ck ? trueSurfaces.get(ck) : undefined;
      // A world wears the finer faces on its ground: its True face when that is the ground alone, or the ground under
      // its own cloud layer (VP-A4), which is what gpu.ts draws when the clouds are there.
      const under = ck ? trueAirs.get(ck)?.ground : undefined;
      if (ck && base && typeof base !== "string" && (base.faces || under?.faces) && g === trueGen) {
        // Close-up: the ground under the camera is read first when the camera is over this world; from further out,
        // the side the camera sees (so the face in view sharpens first, not the far side).
        const below = untrack(() => groundView);
        const facing = (a: string) => untrack(() => (ctx.workspace.camera?.about === a ? ([...ctx.workspace.camera.eye_dir] as [number, number, number]) : undefined));
        void readFiner(ctx.session, sys, chosen!.address, () => g === trueGen, (fine) => {
          const now = trueSurfaces.get(ck);
          if (!now || typeof now === "string") return;
          const t = withFiner(now, fine);
          trueSurfaces.set(ck, t);
          surfaces = new Map(surfaces).set(chosen!.address, t);
          const a = finerGround(ck, fine);
          if (a) airs = new Map(airs).set(chosen!.address, a);
        }, below?.body === chosen!.address ? below.toward : () => facing(chosen!.address));
      }
    })());
  });
  // Close-up (Alex, 25 Nov): the ground under the 3D camera while it is low over a world in True; the detail ladder's
  // deep tiles (VP3 part 4) read it.
  let groundView = $state<GroundView | null>(null);
  // VP3 part 4: once the camera has stayed low over the chosen world for DEEP_SETTLE_MS, the tiles under it are read
  // at the screen's own scale (down to the kernel's fact floor) and laid over its ground as a patch. A newer view
  // stops the read between requests; the last patch stays where it was read (it is the world's real ground).
  const DEEP_SETTLE_MS = 600;
  let deepGen = 0;
  /** The level of the deep patch last laid, for the page's checks (data-deep-level). */
  let deepLevel = $state<number | null>(null);
  $effect(() => {
    const below = groundView;
    const sys = scene.system;
    const sel = ctx.workspace.selection;
    const my = ++deepGen;
    if (!looksTrue || ctx.session.seat.authority !== "gm" || !below || below.body !== sel) return;
    const radius_m = untrack(() => bodies3d.find((b) => b.id === below.body)?.radius_m);
    if (!radius_m) return;
    // Only a world whose ground True already wears (faces read from its tiles), as for the finer tiles: a world the
    // kernel has no surface for is never asked again, so a close fly-in does not queue kernel work in front of the
    // chart's own (a world with none answered each read with a refusal, and on a slow runner held the kernel).
    const key0 = `${sys}|${below.body}`;
    const worn = untrack(() => trueSurfaces.get(key0));
    if (!worn || typeof worn === "string" || !(worn.faces || untrack(() => trueAirs.get(key0)?.ground?.faces))) return;
    const timer = setTimeout(() => untrack(() => void (async () => {
      const p = await readDeep(ctx.session, sys, below.body, { toward: below.toward, m_per_px: below.m_per_px }, radius_m, TILE_SAMPLES, () => my === deepGen);
      if (!p || my !== deepGen) return;
      deepLevel = p.level;
      const key = `${sys}|${below.body}`;
      const t = trueSurfaces.get(key);
      if (t && typeof t !== "string") {
        const next = { ...t, patch: p };
        trueSurfaces.set(key, next);
        surfaces = new Map(surfaces).set(below.body, next);
      }
      const a = trueAirs.get(key);
      if (a?.ground) {
        const next = { ...a, ground: { ...a.ground, patch: p } };
        trueAirs.set(key, next);
        airs = new Map(airs).set(below.body, next);
      }
    })()), DEEP_SETTLE_MS);
    return () => clearTimeout(timer);
  });
  // The living star (V10c): read once a star from the kernel (its facts and activity), on the GM's seat.
  let liveStar = $state<LiveStar | null>(null);
  let liveStarWhy = $state("");
  $effect(() => {
    const sys = scene.system;
    const a = star?.address;
    if (!looksTrue || !a || ctx.session.seat.authority !== "gm") return;
    const key = `${sys}|${a}`;
    const kept = liveStars.get(key);
    const show = (r: LiveStar | string) => ((liveStar = typeof r === "string" ? null : r), (liveStarWhy = typeof r === "string" ? r : ""));
    if (kept !== undefined) return void show(kept);
    let live = true;
    const num = (k: string) => Number(star?.facts.find((x) => x.field === k)?.value);
    void readLiveStar(ctx.session as unknown as StarReader, sys, a, { teff_k: num("teff_k"), mass_sun: num("mass_sun"), radius_sun: num("radius_sun") }, "system").then((r) => {
      liveStars.set(key, r);
      if (live) show(r);
    });
    return () => {
      live = false;
    };
  });
  // Mag (V10e): each world's field facts. Until the kernel publishes a field for every world (MF2), read_field's
  // Solar System fixture is the source, matched by name inside the campaign's Solar System, as on the Body screen.
  let fields = $state.raw<Map<string, FieldReadJson>>(new Map());
  $effect(() => {
    const sys = scene.system;
    const subjects = scene.subjects;
    if (dims !== "3d" || !viewed.layers.some((l) => l.band === "mag") || !/\/sys:sol$/.test(sys)) return void (fields = new Map());
    let live = true;
    solFields ??= ctx.session.readField({ fixture: "solar-system" }).then((r) => (r.ok ? (r.bodies as unknown as FieldReadJson[]) : []));
    void solFields.then((read) => {
      if (!live) return;
      const m = new Map<string, FieldReadJson>();
      for (const s of subjects) {
        const b = read.find((x) => x.name === s.label);
        if (b) m.set(s.address, b);
      }
      fields = m;
    });
    return () => {
      live = false;
    };
  });
  const starColour = $derived.by(() => {
    const t = Number(star?.facts.find((f) => f.field === "teff_k")?.value);
    return Number.isFinite(t) && t > 0 ? starCss(t) : "";
  });
  const surfaceNote = $derived(
    ctx.session.seat.authority !== "gm"
      ? "Worlds are drawn plain: their surfaces are read on the GM's seat."
      : !reading
        ? ""
        : reading.done < reading.of
          ? `Reading surfaces: ${reading.done} of ${reading.of} worlds (a few seconds each).`
          : reading.none
            ? `${reading.none} of ${reading.of} worlds are drawn plain, with no surface to show (the kernel: ${reading.why}).`
            : "",
  );
  // The long exposure (the default, Alex 24 Nov) says its lift, so the picture never claims to be what an eye sees;
  // at the eye's exposure the faint stars speak for themselves.
  const skyNote = $derived(skyBehind && shownView.exposure === "long" ? "Long exposure: faint light is lifted, a composite." : "");
  // Each thing a face could not show, said once for the system (a colour the sheet lacks, a giant's dark deep air).
  const airNote = $derived(reading && reading.done >= reading.of ? [...new Set([...trueSaid].filter(([k]) => k.startsWith(`${scene.system}|`)).flatMap(([, v]) => v))].join(" ") : "");
  const trueNote = $derived([surfaceNote, airNote, liveStarWhy ? `The star is drawn plain: ${liveStarWhy}` : "", skyNote].filter(Boolean).join(" "));
  // Belt members in True (render audit item 3): a few members of each belt round the star resolved by the kernel
  // (system-3d small-sample.ts), read once per system after the worlds' surfaces (they share the kernel), GM seat only.
  let smalls = $state<{ key: string; list: SmallSample[] } | null>(null);
  const smallsKept = new Map<string, SmallSample[]>();
  let smallsAsking = "";
  $effect(() => {
    const sys = scene.system;
    if (!looksTrue || ctx.session.seat.authority !== "gm" || smalls?.key === sys || smallsAsking === sys) return;
    if (!reading || reading.done < reading.of) return;
    const star = primaryOf(scene)?.address;
    const belts = scene.subjects.filter((s) => s.kind === "belt" && s.shape !== "sphere" && s.band_au && (!s.parent || s.parent === star)).map((s) => ({ address: s.address, name: s.label }));
    smallsAsking = sys;
    untrack(() =>
      void (async () => {
        const kept = smallsKept.get(sys);
        const list = kept ?? (await readSmallSample(ctx.session as unknown as SmallReader, sys, belts, () => scene.system === sys && looksTrue));
        smallsAsking = "";
        // Cut short (another system, or True left): nothing kept, it is asked again next time.
        if (scene.system !== sys || !looksTrue) return;
        smallsKept.set(sys, list);
        smalls = { key: sys, list };
      })(),
    );
  });
  const bodies3d = $derived.by(() => {
    if (dims !== "3d") return [];
    const out = fromSystemScene(scene, ctx.mark);
    const star = out.find((b) => b.parent === null)?.id;
    return looksTrue && star && smalls?.key === scene.system ? [...out, ...smalls.list.map((s) => fromSmallSample(s, star))] : out;
  });
  // PC2: the 3D camera about the world at the centre, kept on the workspace for a bookmark or postcard made here (the
  // pose's distance in metres from the world's radius); a bookmark's camera handed to the 3D view once its world is centred.
  function keepCamera(c: { about: string; dir: [number, number, number]; radii: number; fov_deg: number } | null) {
    const r = c ? untrack(() => bodies3d.find((b) => b.id === c.about)?.radius_m) : null;
    ctx.workspace.camera = c && r ? { about: c.about, eye_dir: c.dir, eye_m: c.radii * r, up: [0, 0, 1], fov_deg: c.fov_deg, exposure: { mode: shownView.exposure === "eye" ? "eye" : "long", ev: 0 }, keep: [0, 0, 1, 1] } : null;
  }
  const aim3d = $derived.by(() => {
    const a = ctx.workspace.aim;
    const r = a ? bodies3d.find((b) => b.id === a.about)?.radius_m : null;
    return a && r ? { about: a.about, dir: a.eye_dir, radii: a.eye_m / r } : null;
  });
  $effect(() => {
    if (dims !== "3d") untrack(() => (ctx.workspace.camera = null));
    return () => untrack(() => (ctx.workspace.camera = null));
  });
  const bands3d = $derived(dims === "3d" && on("zones") ? bandsFromScene(scene) : []);
  // Measure (V14): pick two bodies; the chart draws the line and says how far apart they are now.
  let measuring = $state(false);
  let picks = $state<string[]>([]);
  function measurePick(s: SceneSubject) {
    picks = picks.length >= 2 || picks.includes(s.address) ? [s.address] : [...picks, s.address];
  }
  function toggleMeasure() {
    measuring = !measuring;
    picks = [];
  }
  const sep = $derived(picks.length === 2 ? separation(scene, picks[0]!, picks[1]!) : null);
  const at = (address: string): [number, number] | null => {
    const s = all.find((x) => x.address === address);
    if (!s) return null;
    return s === centre ? [cx, cy] : pos(s);
  };
  const zoomBy = (f: number) => (zoom = Math.min(6, Math.max(0.5, zoom * f)));
  // Hover peek (dynamic minimalism): with a mouse, resting on a body shows its
  // name and kind beside it; the card is one click away. Not on touch, and not
  // for the body already selected, which the card already describes.
  let peek = $state<{ address: string; x: number; y: number; r: number } | null>(null);
  const peeked = $derived(peek && peek.address !== ctx.workspace.selection && !drag && !measuring ? all.find((s) => s.address === peek!.address) : undefined);
</script>

<div class="chart" bind:clientWidth={w} bind:clientHeight={h} data-deep-level={deepLevel ?? undefined} data-true-read={reading ? `${reading.done}/${reading.of}` : undefined}>
  {#if dims === "3d"}
    {#snippet in3d(band: string, base: boolean)}
      <System3D bodies={bodies3d} bands={bands3d} selection={ctx.workspace.selection} onselect={(id) => ctx.workspace.select(id)} title={scene.title} seen={shownView.seen} onturned={() => ctx.workspace.turned?.()} trueLook={looksTrue} {surfaces} {airs} {plumes} {starColour} {trueNote} star={liveStar} {band} {fields} realScale={shownView.scale === "real"} hud={base && shownView.hud} reframe={shownView.framed} pencil={base && pencilOn ? pencilled : undefined} range={ctx.rangeRings?.() ?? []} sky={skyBehind ?? undefined} long={shownView.exposure === "long"} onground={(g) => (groundView = g)} oncamera={keepCamera} aim={aim3d} onaimed={() => (ctx.workspace.aim = null)} />
    {/snippet}
    {#if viewed.composite}
      <!-- Composite (XO17): the system drawn once per band and laid into one picture. -->
      <CompositeStack layers={viewed.layers} dark={looksTrue ? true : undefined}>{#snippet children(band, base)}{@render in3d(band, base)}{/snippet}</CompositeStack>
    {:else}
      {@render in3d(shownView.band, true)}
    {/if}
  {:else}
  <ChartGround>
    {#if w > 0 && h > 0}
      <svg
        bind:this={svgEl}
        width={w}
        height={h}
        viewBox="0 0 {w} {h}"
        role="group"
        aria-label={framed ? `Chart of ${framed.label} and its moons` : `System chart of ${scene.title}`}
        onpointermove={(e) => (moveDrag(e), moveGhost(e))}
        onpointerdowncapture={dropGhost}
        onpointerleave={() => (ghostAt = null)}
        onpointerup={endDrag}
        onpointercancel={() => (drag = null)}
        class:dragging={drag !== null}
        class:adding={adding !== null}
      >
        <HatchDefs {prefix} />
        {#if on("grid")}
          <g class="rings" aria-hidden="true" data-scale-rings={decades.map((g) => g.units).join(" ")}>
            {#each rings as g (g.au)}<circle class:major={g.major} cx={cx} cy={cy} r={r(g.au)} />{/each}
          </g>
        {/if}
        {#if params.unsurveyed_hatch && on("unsurveyed") && !frame}
          <Hatch {prefix} d={unsurveyed} pattern="hatch" opacity={0.5} />
          <Ink circle={{ cx, cy, r: surveyR }} tone="muted" dash="dashed" />
        {/if}
        {#each on("zones") ? bands : [] as b (b.address)}
          {@const r0 = r(b.band_au![0])}
          {@const r1 = r(b.band_au![1])}
          {#if b.kind === "zone"}
            {#if shownView.hud}<Ink circle={{ cx, cy, r: (r0 + r1) / 2 }} tone="zone" width={r1 - r0} />{/if}
          {:else if isShell(b)}
            <!-- A sphere seen from above: a haze from its inner edge out past the chart, not a ring in the plane. -->
            {@const edge = Math.min(r1, Math.hypot(w, h))}
            <!-- Its near and far sides lie over the whole system too, so the disc inside the inner edge is tinted, more faintly. -->
            <circle class="shell-in" cx={cx} cy={cy} r={r0} />
            <circle class="shell" cx={cx} cy={cy} r={(r0 + edge) / 2} stroke-width={Math.max(1, edge - r0)} data-shape="sphere" data-address={b.address} />
            <Ink circle={{ cx, cy, r: r0 }} tone="muted" dash="dotted" />
          {:else}
            <Ink circle={{ cx, cy, r: (r0 + r1) / 2 }} tone="muted" dash="belt" />
          {/if}
        {/each}
        {#each distLabels as d (d.g.au)}
          <text class="dist" x={cx + 4} y={d.y} aria-hidden="true">{d.text}</text>
        {/each}
        {#each orbiting as s (s.address)}
          {@const sel = ctx.workspace.selection === s.address}
          {#if drag?.address === s.address}
            <Ink circle={{ cx, cy, r: rr(s.orbit!.a_au) }} tone="faint" dash="dashed" />
            <Ink circle={{ cx, cy, r: rr(drag.a_au) }} tone="accent" weight="heavy" dash="dashed" />
          {:else if on("orbits") || sel}
            <!-- Resting on a body lifts its orbit out of the grid, so the eye can follow it round. -->
            <Ink circle={{ cx, cy, r: rr(s.orbit!.a_au) }} tone={sel ? "accent" : peek?.address === s.address ? "ink" : "faint"} weight={sel ? "heavy" : "line"} />
          {/if}
        {/each}
        {#each bandLabels as l (l.b.address)}
          <text class="band {l.b.kind}" class:sphere={isShell(l.b)} x={l.lx} y={l.ly} aria-hidden="true">{l.text}</text>
        {/each}
        <!-- A press on a name chooses its body: the name's box is a quiet target under every world's own hit circle, so
             a name that lands near another world never takes that world's press or drag (the names drawn on top take none). -->
        <g class="name-hits" aria-hidden="true">
          {#each all as s (s.address)}
            {@const p = places.get(s.address)}
            <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions (the body's own mark is the keyboard's way in) -->
            {#if p}<rect x={p.box.x} y={p.box.y} width={p.box.w} height={p.box.h} data-name-hit={s.address} onclick={() => (measuring ? measurePick(s) : ctx.workspace.select(s.address))} />{/if}
          {/each}
        </g>
        {#each all as s, i (s.address)}
          {@const atCentre = s === centre}
          {@const [x, y] = atCentre ? [cx, cy] : pos(s)}
          {@const m0 = ctx.mark(s)}
          {@const m = atCentre && frame ? { ...m0, size: m0.size * 2.2 } : m0}
          {@const moons = !frame && !atCentre ? moonsOf(s) : 0}
          {@const view = known?.get(s.address)}
          {@const sel = ctx.workspace.selection === s.address}
          <g
            bind:this={focusables[i]}
            class="body"
            data-address={s.address}
            class:unknown={through && !knows(s)}
            data-known={known ? (knows(s) ? (view ?? "yes") : "no") : undefined}
            role="button"
            tabindex="0"
            aria-label="{label(s)}{s.subtitle ? `, ${s.subtitle}` : ''}{through && !knows(s) ? ', not known to them' : ''}"
            aria-pressed={sel}
            aria-describedby={editable(s) ? "chart-drag-hint" : undefined}
            onpointerdown={(e) => (measuring ? measurePick(s) : editable(s) && !atCentre ? startDrag(e, s) : ctx.workspace.select(s.address))}
            ondblclick={() => moons > 0 && openFrame(s)}
            onpointerenter={(e) => e.pointerType === "mouse" && (peek = { address: s.address, x, y, r: markExtent(m) })}
            onpointerleave={() => peek?.address === s.address && (peek = null)}
            onkeydown={(e) => onkey(e, i, s)}
            data-moons={moons || undefined}
          >
            <circle class="hit" cx={x} cy={y} r={Math.max(22, m.size + 12)} />
            {#if pencilOn && !atCentre && !knows(s)}
              <!-- Pencil (SV3, the shared pencil kit): the mark of a body the table does not know is quiet, inside a thin dotted ring. -->
              <g data-pencil><Pencilled pen={penOf("gm_only")} cx={x} cy={y} r={markExtent(m)}><Mark mark={m} cx={x} cy={y} {prefix} /></Pencilled></g>
            {:else}
              <!-- The mark lifts a little under the pointer (dynamic minimalism): a body answers before it is pressed. -->
              <g class="lift" style="transform-origin: {x}px {y}px"><Mark mark={m} cx={x} cy={y} {prefix} /></g>
            {/if}
            {#each { length: on("moons") ? Math.min(moons, 4) : 0 } as _, k (k)}
              {@const [mx, my] = polar(x, y, markExtent(m) + 5, 200 + k * 28)}
              <circle class="moontick" cx={mx} cy={my} r="1.6" aria-hidden="true" />
            {/each}
            {#if sel}<circle class="sel" cx={x} cy={y} r={Math.max(markExtent(m) + 4, m.size * 2 + 6)} />{/if}
            {#if drag?.address === s.address}
              {@const [gx, gy] = polar(cx, cy, rr(drag.a_au), s.orbit!.phase_deg)}
              <g class="ghost" data-ghost aria-hidden="true"><Mark mark={m} cx={gx} cy={gy} {prefix} /></g>
              <text class="dragv" data-drag-readout data-caught={drag.caught ?? undefined} x={gx + m.size + 9} y={gy + m.size + 16}>{dist(drag.a_au)}{drag.caught && !drag.caught.startsWith("a round") ? ` · ${drag.caught}` : ""}</text>
            {/if}
          </g>
        {/each}
        {#if adding && ghostAt}
          {@const gr = radius(ghostAt.a_au, scale)}
          {@const [gx, gy] = polar(cx, cy, gr, (Math.atan2(cy - ghostAt.y, ghostAt.x - cx) * 180) / Math.PI)}
          <g class="add-ghost" data-add-ghost aria-hidden="true">
            <Ink circle={{ cx, cy, r: gr }} tone="accent" weight="heavy" dash="dashed" />
            <circle class="ghost-dot" cx={gx} cy={gy} r="6" />
            <text class="dragv" data-add-readout x={gx + 14} y={gy + 20}>{dist(ghostAt.a_au)} round {centre?.label}{ghostAt.caught && !ghostAt.caught.startsWith("a round") ? ` · ${ghostAt.caught}` : ""}</text>
            {#if formsAt && formsAt.key === formsKey(ghostAt.a_au)}<text class="dragv forms" data-add-forms x={gx + 14} y={gy + 36}>{formsAt.words}</text>{/if}
          </g>
        {/if}
        <!-- Names in their own layer, over every mark (chart-labels.ts places them); a press on a name chooses its body. -->
        <!-- Range rings (RK12), under the HUD like the grid: round each weapon's holder, bent by the chart's scale. -->
        {#if hudShows(shownView.hud, "range") && rangeSets.length}
          {#each rangeSets as r (r.holder + r.label)}<RangeRings rings={r.rings} centre_m={r.centre_m} project={rangeProject} label={r.label} />{/each}
        {/if}
        <g class="names">
          {#each all as s (s.address)}
            {#if places.get(s.address)}
              {@const p = places.get(s.address)}
              {@const atCentre = s === centre}
              {@const view = known?.get(s.address)}
              {@const sel = ctx.workspace.selection === s.address}
              <!-- Who knows (track SV, pencil and ink): a name the table does not know is drawn in pencil, quiet and dotted,
                   with no words; one they know is in ink, with how they know it ("long range") under it. -->
              {@const pencil = pencilOn && !atCentre && !view && !knows(s)}
              <text class="name" class:selname={sel} class:pencil style={pencil ? penStyle(penOf("gm_only"), "name") : undefined} class:unknown={through && !knows(s)} x={p.x} y={p.y} text-anchor={p.anchor} data-label-side={p.side} data-label-for={s.address} aria-hidden="true">{label(s)}{#if pencil}<title>{PEN_WORDS.pencil}</title>{/if}</text>
              {#if on("known") && known && !atCentre && view && view !== usualView}
                <text class="knownv" x={p.x} y={p.y + 13} text-anchor={p.anchor} aria-hidden="true">{view.replace(/-/g, " ")}</text>
              {/if}
            {/if}
          {/each}
        </g>
        {#if measuring && picks.length}
          {@const p0 = at(picks[0]!)}
          {@const p1 = picks[1] ? at(picks[1]) : null}
          {#if p0}<circle class="mpick" cx={p0[0]} cy={p0[1]} r="14" />{/if}
          {#if p1}<circle class="mpick" cx={p1[0]} cy={p1[1]} r="14" />{/if}
          {#if p0 && p1}<line class="mline" x1={p0[0]} y1={p0[1]} x2={p1[0]} y2={p1[1]} />{/if}
        {/if}
      </svg>
      {#if peeked && peek}
        {#key peek.address}
          <div class="peek" class:up={peek.y > h - 90} style="left: {peek.x}px; top: {peek.y}px; --off: {peek.r + 14}px" aria-hidden="true" data-peek={peek.address}>
            <b>{label(peeked)}</b>{#if peeked.subtitle}<span>{peeked.subtitle}</span>{/if}
          </div>
        {/key}
      {/if}
      {#if measuring}
        <div class="measure" role="status" aria-label="Measurement" data-measure={sep ? sep.now_au : picks.length}>
          {#if sep}
            <b>{sep.from} to {sep.to}</b>
            <span>Now {distanceWords(sep.now_au)}, {lightWords(sep.light_s)}.</span>
            {#if sep.range_au && sep.range_au[0] !== sep.range_au[1]}<span>Closest {distanceWords(sep.range_au[0])}, farthest {distanceWords(sep.range_au[1])}.</span>{/if}
            <span class="fine">From the orbits as drawn: circular, in one plane, at the ruler's time. Distances on the chart are on a log scale; these are true.</span>
          {:else if picks.length === 1}
            <span>Now pick a second body.</span>
          {:else}
            <span>Measure: pick two bodies on the chart.</span>
          {/if}
        </div>
      {/if}
      {#if framed}
        <div class="frame" role="group" aria-label="Chart frame" data-chart-overlay>
          <button type="button" onclick={closeFrame}>‹ {scene.title}</button>
          <span>Moons of <b>{framed.label}</b>, distances from its centre</span>
        </div>
      {:else if canOpen}
        <div class="frame" data-chart-overlay>
          <button type="button" onclick={() => openFrame(canOpen)}>Moons of {canOpen.label} ›</button>
        </div>
      {/if}
      <p id="chart-drag-hint" class="sr">Drag along the radius, or press Page Up or Page Down, to move the orbit. Further from the line it moves more slowly, Shift slower still; it catches at round figures and at resonances with its neighbours, and Alt lets it run free.</p>
      {#if problem}<p class="problem" role="alert">{problem}</p>{/if}
      <div class="legend" class:short={narrow} data-chart-overlay>
        {#if authoredImport}<span class="authored-import" data-authored-import><Stamp kind="authored" title="Every name and value here was written in a V3 save, not modelled by SSE4's physics. A body called Earth is the save's Earth, not the real one." /><span class="words">As written in a V3 save, not modelled</span></span>{/if}
        {#if catalogued}<span class="authored-import" data-catalogued><Stamp kind="catalogued" title="Every value here is a measurement from a catalogue, with its source on the card's why?." /><span class="words">As measured, each value with its source</span></span>{/if}
        {#if scene.fixture}<Stamp kind="fixture" title="The sample system's inputs are fixture data; every derived value comes from the kernel's base-physics pack." />{/if}
        <span data-scale-note>{narrow ? `Log scale${frame ? ", km" : ""}` : on("grid") && decades.length ? `Log scale${frame ? ", in kilometres" : ""}: each bold dotted ring is ten times the last (${decades.slice(0, 2).map((g) => `${sayNumber(g.units, 0, { upTo: 3 })} ${unitWord}`).join(", ")}…)` : `Distances on a log scale${frame ? ", in kilometres" : ""}`}</span>
        {#if on("known") && known && usualView && !through}<span class="usual-known" data-usual-known={usualView}>The table knows these at {usualView.replace(/-/g, " ")}; dotted names, not at all</span>{/if}
        {#if through}<span class="through" data-known-count={knownCount}>{L?.label}: {knownCount} of {bodyCount} known</span>{/if}
        {#if key?.entries.length && on("key")}
          <details class="key">
            <summary>Key · {key.name}</summary>
            <div class="key-body"><Legend entries={key.entries} prefix="sck" note={key.note} label="Key to the chart's marks" /></div>
          </details>
        {/if}
        {#if params.unsurveyed_hatch && on("unsurveyed") && !frame && !narrow}<span>Hatched: unsurveyed beyond {scene.surveyed_to_au} AU</span>{/if}
      </div>
      <div class="zoom" role="group" aria-label="Zoom" data-chart-overlay>
        <button type="button" aria-label="Zoom in" onclick={() => zoomBy(1.25)}>+</button>
        <button type="button" aria-label="Zoom out" onclick={() => zoomBy(0.8)}>−</button>
        <button type="button" aria-label={frame ? "Fit the moons" : "Fit the system"} onclick={() => (zoom = Number(params.zoom))}>Fit</button>
        <button type="button" aria-pressed={measuring} onclick={toggleMeasure} title="Measure the distance between two bodies">Measure</button>
      </div>
    {/if}
  </ChartGround>
  {/if}
</div>

<style>
  .chart {
    position: relative;
    width: 100%;
    height: 100%;
  }
  svg {
    display: block;
    font-family: var(--sse-font-body);
  }
  .body {
    cursor: pointer;
    outline: none;
  }
  .hit {
    fill: transparent;
  }
  .lift {
    transition: transform var(--sse-motion-quick) var(--sse-ease);
  }
  .body:hover .lift {
    transform: scale(1.3);
  }
  .dragging .body:hover .lift {
    transform: none;
  }
  .dragging,
  .dragging .body {
    cursor: grabbing;
  }
  .dragging {
    user-select: none;
  }
  svg.adding {
    cursor: crosshair;
  }
  .ghost-dot {
    fill: none;
    stroke: var(--sse-accent);
    stroke-width: 2;
    stroke-dasharray: 3 2;
  }
  .ghost {
    opacity: 0.55;
    pointer-events: none;
  }
  .dragv.forms {
    fill: var(--sse-ink-muted);
    font-size: 11px;
  }
  .dragv {
    fill: var(--sse-accent-ink);
    font-family: var(--sse-font-mono);
    font-size: 12px;
    paint-order: stroke;
    stroke: var(--sse-ground);
    stroke-width: 4px;
  }
  .moontick {
    fill: var(--sse-ink);
  }
  .frame {
    position: absolute;
    left: 92px;
    right: 16px;
    /* Below the shell's view controls (interface style rule 12), which sit at the top of the stage. */
    top: calc(var(--viewbar-h, 68px) + 16px);
    flex-wrap: wrap;
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
    color: var(--sse-ink-muted);
  }
  .frame b {
    color: var(--sse-ink);
    font-family: var(--sse-font-display);
    font-style: italic;
    font-weight: normal;
  }
  .frame button {
    min-height: 36px;
    padding: 0 12px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    font-size: 13px;
    cursor: pointer;
  }
  .sr {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
  }
  .problem {
    position: absolute;
    left: 24px;
    top: 128px;
    margin: 0;
    padding: 6px 10px;
    border: var(--sse-line) solid var(--sse-accent-ink);
    background: var(--sse-ground-raised);
    color: var(--sse-accent-ink);
    font-size: 13px;
  }
  .body:focus-visible .hit {
    stroke: var(--sse-focus);
    stroke-width: 2;
    stroke-dasharray: 3 3;
  }
  .sel {
    fill: none;
    stroke: var(--sse-accent);
    stroke-width: 1.2;
    stroke-dasharray: 2 3;
    transform-box: fill-box;
    transform-origin: center;
    animation: sel-in var(--sse-motion) var(--sse-ease);
  }
  /* The selection ring closes in on the body chosen. */
  @keyframes sel-in {
    from {
      transform: scale(1.7);
      opacity: 0;
    }
  }
  .body:hover .hit {
    fill: var(--sse-rule);
    fill-opacity: 0.45;
  }
  .peek {
    position: absolute;
    transform: translate(-50%, var(--off));
    display: flex;
    align-items: center;
    flex-direction: column;
    gap: 1px;
    padding: 5px 9px;
    background: var(--sse-panel-ground);
    color: var(--sse-panel-ink);
    border: var(--sse-frame-width) solid var(--sse-frame-line);
    box-shadow: 3px 3px 0 var(--sse-frame-line);
    font-size: 12px;
    pointer-events: none;
    white-space: nowrap;
    z-index: 2;
    animation: peek-in var(--sse-motion-quick) var(--sse-ease);
  }
  @keyframes peek-in {
    from {
      opacity: 0;
    }
  }
  .peek.up {
    transform: translate(-50%, calc(-100% - var(--off)));
  }
  .peek b {
    font-family: var(--sse-title-font);
    font-weight: 600;
    font-size: 13px;
  }
  .peek span {
    color: var(--sse-panel-muted);
  }
  .name-hits rect {
    fill: transparent;
    cursor: pointer;
  }
  .name {
    pointer-events: none;
    fill: var(--sse-ink);
    font-family: var(--sse-font-display);
    font-style: italic;
    font-size: 16px;
    paint-order: stroke;
    stroke: var(--sse-ground);
    stroke-width: 4px;
  }
  .selname {
    font-size: 18px;
  }
  .band {
    fill: var(--sse-ink-muted);
    font-size: 11px;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    paint-order: stroke;
    stroke: var(--sse-ground);
    stroke-width: 4px;
  }
  .body.unknown,
  .name.unknown {
    opacity: 0.25;
  }
  .knownv {
    font-family: var(--sse-font-body);
    font-size: 10px;
    font-style: italic;
    fill: var(--sse-ink-muted);
  }
  .through {
    color: var(--sse-accent-ink);
  }
  /* The legend is a caption laid over the chart: its words never take a tap meant for a body. */
  .legend {
    pointer-events: none;
  }
  .legend :global(.stamp),
  .legend .key {
    pointer-events: auto;
  }
  .authored-import {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .shell-in {
    fill: var(--sse-ink-muted);
    fill-opacity: 0.03;
    pointer-events: none;
  }
  .shell {
    fill: none;
    stroke: var(--sse-ink-muted);
    stroke-opacity: 0.07;
    pointer-events: none;
  }
  .band.zone {
    fill: var(--sse-zone-ink);
  }
  /* Scale rings are the grid, not orbits: grid ink, dotted, so a solid line on the chart is always an orbit. */
  .rings circle {
    fill: none;
    stroke: var(--sse-grid);
    stroke-width: var(--sse-line);
    stroke-dasharray: 1 5;
    opacity: 0.7;
  }
  .rings circle.major {
    stroke: var(--sse-ink-faint);
    stroke-width: calc(var(--sse-line) * 1.5);
    stroke-dasharray: 1 3;
    opacity: 1;
  }
  .dist {
    fill: var(--sse-ink-muted);
    font-family: var(--sse-font-mono);
    font-size: 10px;
  }
  .legend {
    position: absolute;
    left: 24px;
    bottom: 30px;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px 14px;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .key summary {
    cursor: pointer;
    color: var(--sse-ink);
  }
  .key[open] .key-body {
    position: absolute;
    left: 0;
    bottom: calc(100% + 6px);
    width: min(360px, calc(100vw - 48px));
    padding: 10px 12px;
    background: var(--sse-panel-ground);
    border: var(--sse-frame-width) solid var(--sse-frame-line);
    box-shadow: var(--sse-frame-shadow);
  }
  .measure {
    position: absolute;
    left: 24px;
    bottom: 64px;
    max-width: min(420px, calc(100% - 32px));
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px 12px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    font-family: var(--sse-font-body);
    font-size: 13px;
    color: var(--sse-ink);
  }
  .measure .fine {
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  .mline {
    stroke: var(--sse-accent);
    stroke-width: 1.5;
    stroke-dasharray: 5 4;
    pointer-events: none;
  }
  .mpick {
    fill: none;
    stroke: var(--sse-accent);
    stroke-width: 1.5;
    pointer-events: none;
  }
  .zoom {
    position: absolute;
    right: 16px;
    bottom: 16px;
    display: flex;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
  }
  .zoom button {
    min-width: 44px;
    height: 40px;
    border: 0;
    background: transparent;
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    font-size: 15px;
    cursor: pointer;
  }
  .zoom button[aria-pressed="true"] {
    background: var(--sse-ink);
    color: var(--sse-on-ink);
  }
  .zoom button + button {
    border-left: var(--sse-line) solid var(--sse-ink);
  }
  @media (max-width: 760px) {
    .frame {
      left: 16px;
      top: 12px;
    }
    .frame span {
      display: none;
    }
    .authored-import .words {
      display: none;
    }
    .legend {
      left: 16px;
      right: 250px;
      bottom: 12px;
      font-size: 11px;
    }
    .zoom {
      bottom: 12px;
      right: 12px;
    }
  }
</style>
