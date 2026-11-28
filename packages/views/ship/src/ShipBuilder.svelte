<!--
  The ship builder (CM14b), built to the interactive mock-up Alex accepted (sse4/handover-screens/CM-builder-mockup/,
  compound ships design 8A.4, rulings 14A.2 to 14A.4, his two modes of 27 Nov 11:47 and his "where is the ship
  builder" of 28 Nov 13:17). One dark working screen: parts on the left by era with a "Parts from" year slider; the
  build in 3D over a blueprint room whose side, top and front drawings never turn with it; glowing connection points
  to press or drag a part onto; the selected part, the build tree and paint on the right; and the shopping list along
  the bottom, counting down to a plan or showing what the build can do as it grows. It keeps a recipe, never a
  drawing. The sums are the parts' own figures (renderer ship-build.ts); every judgement of what the ship can do at a
  world is the kernel's (`ship_at_world`). With `readonly` it is the picture alone: the build turning over its room.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { PAINT_SWATCHES, budgets, buildLayout, driveFacts, freePoints, fromLayout, designShip, readPropellants, readTankWalls, shipViews, startBuilder, wallKey, type BuildRecipe, type BuiltPart, type BuilderMarker, type BuilderScene, type DesignRules, type DesignShip, type FreePoint, type Propellants, type ShipStyle, type TankWall } from "../../../renderer/src/index.ts";
  import { eraOf, paletteByUse, sayNumber, shipRecordOf, verdictWords, type NavShip, type Part, type PartsPalette, type ShipAtWorld, type ViewContext } from "../../../ui-contract/src/index.ts";

  let {
    ctx,
    ship = null,
    saved = null,
    onkeep,
    readonly = false,
    catalogue = [],
    onopen,
  }: {
    ctx: ViewContext;
    /** The ship to open as parts when nothing is saved: a catalogue ship, or a construct's record. */
    ship?: DesignShip | null;
    /** A build kept before. */
    saved?: BuildRecipe | null;
    /** Keep the build with its ship (offered on the Build desk). */
    onkeep?: (r: BuildRecipe) => void;
    /** The picture alone: a card off its edit desk. */
    readonly?: boolean;
    /** Example ships to open (the catalogue, the default start), and how to open one ("" for a blank build). */
    catalogue?: readonly { id: string; name: string; drive?: string }[];
    onopen?: (id: string) => void;
  } = $props();

  type Pack = { rules: DesignRules; styles: Record<string, ShipStyle> };
  let pack = $state<Pack | null>(null);
  let palette = $state<PartsPalette | null>(null);
  let missing = $state<string | null>(null);
  // Packs served beside the page (scripts/build-shell.sh).
  const SHIPS_PACK = "packs/base-ships.json";
  const PARTS_PACK = "packs/base-parts.json";
  const read = (path: string) => fetch(new URL(path, document.baseURI)).then((r) => r.json());
  $effect(() => {
    void Promise.all([read(SHIPS_PACK), read(PARTS_PACK)])
      .then(([s, p]: [{ provides: { other: { ship_design: DesignRules; ship_styles: Record<string, ShipStyle> } } }, { provides: { other: { palette: PartsPalette; examples?: Example[]; sources?: Record<string, string> } } }]) => {
        pack = { rules: s.provides.other.ship_design, styles: s.provides.other.ship_styles };
        palette = p.provides.other.palette;
        partSources = p.provides.other.sources ?? {};
        examples = p.provides.other.examples ?? [];
      })
      .catch((e) => (missing = `The parts could not be read (${e instanceof Error ? e.message : String(e)}).`));
  });

  // Example ships assembled from parts (the parts pack's, as the mock-up's): each is only a recipe, opened as it is.
  type Example = { id: string; name: string; about: string; year?: number; build: BuildRecipe };
  let examples = $state<Example[]>([]);
  function openExample(e: Example) {
    push();
    recipe = structuredClone($state.snapshot(e.build) as BuildRecipe);
    notes = [];
    opened = `example:${e.id}`;
    selected = null;
    placing = null;
    if (e.year !== undefined && e.year > year) year = e.year;
    reframe++;
    toast(`${e.name} opened. Every part can be picked, resized, repainted or removed.`);
  }
  const blank = (name: string): BuildRecipe => ({ format: "sse.build/1", name, parts: [] });
  let recipe = $state<BuildRecipe>(untrack(() => saved ?? blank(ship?.name ?? "New ship")));
  let opened = $state<string | null>(untrack(() => (saved ? "saved" : null)));
  let notes = $state<string[]>([]);
  let fuels = $state<Propellants>({});
  let reframe = $state(1);
  // Tank walls from the kernel's thin-wall flow, cached by what each wall depends on (renderer ship-walls.ts), so a
  // stretched tank asks again and an unchanged one never does.
  let walls = $state<Record<string, TankWall>>({});
  let partSources = $state<Record<string, string>>({});

  // Open the ship as parts once the palette is here (convert on open); a saved build wins.
  $effect(() => {
    const s = ship;
    if (!s || !pack || !palette || saved) return;
    const key = `${s.name}|${s.wet_t}|${s.dry_t}`;
    if (untrack(() => opened) === key) return;
    const style = Object.keys(pack.styles)[0]!;
    const old = designShip(s, untrack(() => fuels), pack.rules, style, pack.styles[style]!, 1);
    const got = fromLayout(old, s, palette);
    untrack(() => ((recipe = got.recipe), (notes = got.notes), (opened = key), (selected = null), reframe++));
  });
  // The tanks' propellants, read from the kernel's sheet. Only the latest ask lands (an older answer arriving late must
  // not overwrite it).
  let fuelAsk = 0;
  $effect(() => {
    const f = recipe.fuels ?? [];
    const id = ++fuelAsk;
    void readPropellants(ctx.session, f)
      .then((x) => id === fuelAsk && (fuels = x))
      .catch(() => id === fuelAsk && (fuels = {}));
  });

  // --- what is being worked on ---
  let year = $state(2026);
  const NOW = 2026;
  let selected = $state<string | null>(null);
  let placing = $state<string | null>(null);
  let sym = $state(1);
  let hover = $state<number | null>(null);
  let tab = $state<"part" | "tree" | "paint">("part");
  let viewStyle = $state<"flat" | "3d" | "wire">("3d");
  let cut = $state(false);
  let seen = $state<"above" | "tilted" | "edge" | "free">("tilted");
  let sheet = $state<"palette" | "inspector" | null>(null);

  const style = $derived(pack ? (pack.styles.spine ?? Object.values(pack.styles)[0]!) : null);
  const layout = $derived(pack && palette ? buildLayout(recipe, palette, fuels, pack.rules, walls) : null);
  $effect(() => {
    const placed = layout?.placed ?? [];
    const have = untrack(() => walls);
    if (!placed.some((q) => { const k = wallKey(q); return k !== null && !(k in have); })) return;
    void readTankWalls(ctx.session, placed, have)
      .then((got) => Object.keys(got).length && (walls = { ...untrack(() => walls), ...got }))
      .catch(() => {});
  });
  const massOf = (key: string): string => layout?.totals.mass_from[key] ?? "";
  const facts = $derived(layout ? driveFacts(layout.record) : null);
  // A part with no mass yet leaves the build's masses unknown: delta-v and push are said as needing them.
  const massKnown = $derived(!!layout && !layout.totals.unmassed.length);
  const needsMass = $derived(layout?.totals.unmassed.length ? `needs part masses (${layout.totals.unmassed.length > 2 ? `${layout.totals.unmassed.slice(0, 2).join(", ")} and more` : layout.totals.unmassed.join(", ")})` : "");
  const free = $derived(layout && placing ? freePoints(recipe, layout.placed, { sym }) : []);
  const partOf = (id: string) => palette?.parts.find((p) => p.id === id) ?? null;
  const placingPart = $derived(placing ? partOf(placing) : null);
  const chosen = $derived(layout?.placed.find((q) => q.key === selected) ?? null);
  // Copies clipped together round a hull move and paint together (the mock-up's symmetry groups).
  const groupOf = (key: string | null): string[] => {
    const b = recipe.parts.find((x) => x.key === key);
    if (!b) return [];
    const g = b.sym;
    return g ? recipe.parts.filter((x) => x.sym === g).map((x) => x.key) : [b.key];
  };

  // The ghost: the part being placed, tried at the hovered point.
  const ghostParts = (f: FreePoint, p: Part): BuiltPart[] => {
    const n = f.place === "hull" ? sym : 1;
    const g = n > 1 ? `sym-${recipe.parts.length}` : undefined;
    return Array.from({ length: n }, (_, i) => ({ key: i ? `ghost-${i}` : "ghost", part: p.id, on: f.key, place: f.place, ...(f.place === "hull" ? { along: f.along ?? 0.5, clock_deg: ((f.clock_deg ?? 0) + (i * 360) / n) % 360 } : {}), ...(g ? { sym: g } : {}) }));
  };
  const shown = $derived.by(() => {
    if (!pack || !palette || !layout) return null;
    const f = hover !== null ? free[hover] : undefined;
    if (!f || !placingPart) return layout;
    return buildLayout({ ...recipe, parts: [...recipe.parts, ...ghostParts(f, placingPart)] }, palette, fuels, pack.rules);
  });

  // --- the plan and the shopping list ---
  let planId = $state<string>(untrack(() => (saved?.plan ? "saved" : "")));
  let planShip = $state<DesignShip | null>(untrack(() => saved?.plan ?? null));
  $effect(() => {
    const id = planId;
    if (id === "") return void (planShip = null);
    if (id === "this") return void (planShip = ship);
    if (id === "saved") return;
    void shipRecordOf(ctx.session, id).then((r) => planId === id && (planShip = r.ok ? (r.ship as unknown as DesignShip) : null));
  });
  const bars = $derived(planShip && layout && pack ? budgets({ ...planShip, ...(recipe.crew !== undefined ? { crew: recipe.crew } : {}) }, layout.totals, fuels, pack.rules) : []);

  // --- editing, with undo ---
  let undo: string[] = [];
  const push = () => {
    undo.push(JSON.stringify($state.snapshot(recipe)));
    if (undo.length > 60) undo.shift();
  };
  function doUndo() {
    const u = undo.pop();
    if (!u) return toast("Nothing to undo");
    recipe = JSON.parse(u) as BuildRecipe;
    selected = null;
    report = null;
  }
  let n = 0;
  const keyFor = (id: string) => {
    let k: string;
    do k = `${id.replace(/[^a-z0-9]+/gi, "-")}-${++n}`;
    while (recipe.parts.some((b) => b.key === k));
    return k;
  };
  function arm(p: Part) {
    if (readonly) return;
    if (placing === p.id) return disarm();
    const from = p.year ?? (palette?.eras ? eraOf(p, palette.eras)?.from : undefined);
    if (from !== undefined && from > year) ((year = from), toast(`Moved the year to ${from} so the ${p.name.toLowerCase()} is offered${from > NOW ? " (a predicted part)" : ""}.`));
    // The first part needs no point: it starts the build.
    if (!recipe.parts.length) {
      push();
      const b: BuiltPart = { key: keyFor(p.id), part: p.id };
      recipe = { ...recipe, parts: [b], ...(p.kind === "tank" && !recipe.fuels?.length && ship?.fuels ? { fuels: [...ship.fuels] } : {}) };
      selected = b.key;
      tab = "part";
      reframe++;
      return;
    }
    placing = p.id;
    hover = null;
  }
  function disarm() {
    placing = null;
    hover = null;
  }
  function commit(i: number) {
    const f = free[i];
    if (!f || !placingPart) return;
    push();
    const parts = ghostParts(f, placingPart).map((b) => ({ ...b, key: keyFor(placingPart.id) }));
    recipe = { ...recipe, parts: [...recipe.parts, ...parts], ...(placingPart.kind === "tank" && !recipe.fuels?.length && (planShip ?? ship)?.fuels ? { fuels: [...(planShip ?? ship)!.fuels!] } : {}) };
    report = null;
    const on = layout?.placed.find((q) => q.key === f.key)?.part.name ?? "the build";
    toast(`${placingPart.name}${parts.length > 1 ? ` ×${parts.length}` : ""} clipped ${f.place === "hull" ? "on the hull" : f.place === "bay" ? "inside" : f.place === "front" ? "in front" : "behind"} of ${on.toLowerCase()}.`);
    // The part stays picked after clipping (the mock-up's default), and the next drop clips the same part again.
    selected = parts[0]!.key;
    tab = "part";
    hover = null;
  }
  function edit(keys: string[], f: (b: BuiltPart, part: Part) => BuiltPart) {
    if (readonly || !palette) return;
    recipe = { ...recipe, parts: recipe.parts.map((b) => { const p = partOf(b.part); return keys.includes(b.key) && p ? f(b, p) : b; }) };
    report = null;
  }
  // Stretch and widen by the part's own law; real hardware says it comes in its sizes.
  const sizeOf = (b: BuiltPart, p: Part) => ({ L: b.length_m ?? p.shape.length_m, R: b.radius_m ?? p.shape.radius_m });
  function setSize(keys: string[], L: number | null, R: number | null) {
    edit(keys, (b) => ({ ...b, ...(L !== null ? { length_m: Math.max(0.2, L) } : {}), ...(R !== null ? { radius_m: Math.max(0.05, R) } : {}) }));
  }
  function setClock(keys: string[], c: number) {
    const base = recipe.parts.find((b) => b.key === keys[0])?.clock_deg ?? 0;
    edit(keys, (b) => ({ ...b, clock_deg: ((((b.clock_deg ?? 0) - base + c) % 360) + 360) % 360 }));
  }
  function removeSel() {
    if (readonly || !selected) return;
    push();
    // A part comes off with everything clipped to it, and its copies round the hull with it.
    const gone = new Set(groupOf(selected));
    for (let grew = true; grew; ) {
      grew = false;
      for (const b of recipe.parts) if (b.on && gone.has(b.on) && !gone.has(b.key)) (gone.add(b.key), (grew = true));
    }
    recipe = { ...recipe, parts: recipe.parts.filter((b) => !gone.has(b.key)) };
    toast(`Removed ${gone.size} part${gone.size > 1 ? "s" : ""}.`);
    selected = null;
    report = null;
  }
  function paint(keys: string[], c: string) {
    push();
    edit(keys, (b) => ({ ...b, paint: c }));
  }
  // Paints are the build's own data (a recipe value, like a name), not the screen's colours.
  const PAINTS = PAINT_SWATCHES;

  // --- toasts and popovers ---
  let toastText = $state<string | null>(null);
  let toastTimer: ReturnType<typeof setTimeout> | undefined;
  function toast(t: string) {
    toastText = t;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toastText = null), 3600);
  }
  let pop = $state<"ships" | "can" | "notes" | null>(null);

  // "What can it do?": the kernel judges the build as it stands at a few worlds; nothing is refused.
  const WORLDS = ["Earth", "Mars", "Venus"];
  let report = $state<{ ship: NavShip; worlds: ShipAtWorld[] } | null>(null);
  let asking = $state(false);
  let askFailed = $state<string | null>(null);
  async function whatCanItDo() {
    if (!layout) return;
    pop = "can";
    asking = true;
    askFailed = null;
    const r = await ctx.session.shipAtWorld({ fixture: "sol", ship: layout.record as unknown as NavShip, worlds: WORLDS });
    asking = false;
    if (r.ok) report = { ship: r.report.ship, worlds: r.report.worlds };
    else askFailed = r.message;
  }

  // --- the stage: 3D over the fixed blueprint room ---
  let stage = $state<HTMLElement | null>(null);
  let canvas = $state<HTMLCanvasElement | null>(null);
  let scene = $state<BuilderScene | null>(null);
  let noGpu = $state(false);
  let box = $state({ w: 800, h: 500 });
  const PRESET = { above: { yaw: -90, pitch: 89 }, tilted: { yaw: -62, pitch: 24 }, edge: { yaw: -90, pitch: 0 } } as const;
  let cam = $state({ yaw: -62, pitch: 24, zoom: 1, pan: [0, 0] as [number, number] });
  $effect(() => {
    if (!canvas || !style?.skin || viewStyle === "flat") return;
    const c = canvas;
    const skin = style.skin;
    let gone = false;
    void startBuilder(c, skin, { segments: 24 }).then((m) => {
      if (gone) return m?.dispose();
      scene = m;
      noGpu = !m;
    });
    return () => {
      gone = true;
      scene?.dispose();
      scene = null;
    };
  });
  $effect(() => {
    if (!stage) return;
    const ro = new ResizeObserver(() => stage && (box = { w: Math.max(1, stage.clientWidth), h: Math.max(1, stage.clientHeight) }));
    ro.observe(stage);
    return () => ro.disconnect();
  });
  const markers = $derived<BuilderMarker[]>(free.map((f) => ({ at: f.at, kind: f.place === "hull" ? "hull" : f.place === "bay" ? "bay" : "end" })));
  let lastFrame = 0;
  $effect(() => {
    const s = scene;
    const l = shown;
    if (!s || !l || !l.pieces.length) return;
    const paints: Record<string, string> = {};
    for (const b of recipe.parts) if (b.paint?.startsWith("#")) paints[b.key] = b.paint;
    const frame = reframe !== lastFrame;
    lastFrame = reframe;
    s.show({ layout: l, placed: l.placed, name: recipe.name, ghost: hover !== null ? "ghost" : null, selected: groupOf(selected), paints, markers, hover, wire: viewStyle === "wire", cut, frame });
    s.draw({ yaw_deg: cam.yaw, pitch_deg: cam.pitch, zoom: cam.zoom, pan: cam.pan }, box.w, box.h);
  });
  // "Seen from": a short turn to the preset; a drag makes it Free.
  let tween = 0;
  function seeFrom(s: typeof seen) {
    seen = s;
    if (viewStyle === "flat") viewStyle = "3d";
    if (s === "free") return;
    const to = PRESET[s];
    const from = { ...cam };
    const t0 = performance.now();
    const id = ++tween;
    const reduce = matchMedia("(prefers-reduced-motion: reduce)").matches;
    const step = (t: number) => {
      if (id !== tween) return;
      const k = reduce ? 1 : Math.min(1, (t - t0) / 240);
      const e = 1 - (1 - k) ** 3;
      cam = { yaw: from.yaw + (to.yaw - from.yaw) * e, pitch: from.pitch + (to.pitch - from.pitch) * e, zoom: from.zoom + (1 - from.zoom) * e, pan: [from.pan[0] * (1 - e), from.pan[1] * (1 - e)] };
      if (k < 1) requestAnimationFrame(step);
    };
    requestAnimationFrame(step);
  }

  // --- pointer: turn, slide, pick, place, stretch ---
  let tip = $state<{ x: number; y: number; html: string } | null>(null);
  let down: { x: number; y: number; moved: boolean; shift: boolean } | null = null;
  let paletteDrag: { part: Part; x: number; y: number; moved: boolean } | null = null;
  const local = (e: PointerEvent | WheelEvent) => {
    const r = stage!.getBoundingClientRect();
    return { x: e.clientX - r.left, y: e.clientY - r.top, inside: e.clientX >= r.left && e.clientX <= r.right && e.clientY >= r.top && e.clientY <= r.bottom };
  };
  const pickAt = (x: number, y: number) => scene?.pick(x, y, box.w, box.h) ?? { marker: null, part: null };
  const placeWords = (f: FreePoint) => {
    const on = layout?.placed.find((q) => q.key === f.key)?.part.name.toLowerCase() ?? "";
    return f.place === "hull" ? `<b>On the hull</b> of ${on} · ${Math.round(f.clock_deg ?? 0)}° round, ${Math.round((f.along ?? 0.5) * 100)}% along${sym > 1 ? ` · ×${sym}` : ""}` : f.place === "bay" ? `<b>Inside</b> ${on}` : `<b>${f.place === "front" ? "In front" : "Behind"}</b> of ${on}`;
  };
  function hoverAt(L: { x: number; y: number; inside: boolean }) {
    if (!L.inside || viewStyle === "flat") return void (tip = null);
    const h = pickAt(L.x, L.y);
    if (placing) {
      if (h.marker !== hover) hover = h.marker;
      const f = h.marker !== null ? free[h.marker] : undefined;
      tip = f ? { x: L.x, y: L.y, html: placeWords(f) } : null;
    } else {
      const q = h.part ? layout?.placed.find((p) => p.key === h.part) : undefined;
      tip = q ? { x: L.x, y: L.y, html: `${q.part.name} · <span class="mut">${q.built.on ? (q.built.place === "hull" ? "on the hull" : q.built.place === "bay" ? "inside" : q.built.place === "front" ? "in front" : "behind") : "root"}</span>` } : null;
    }
  }
  function stageDown(e: PointerEvent) {
    down = { x: e.clientX, y: e.clientY, moved: false, shift: e.shiftKey };
    (e.currentTarget as Element).setPointerCapture(e.pointerId);
  }
  function stageMove(e: PointerEvent) {
    if (down) {
      const dx = e.clientX - down.x;
      const dy = e.clientY - down.y;
      if (!down.moved && Math.hypot(dx, dy) > 5) down.moved = true;
      if (down.moved && viewStyle !== "flat") {
        if (down.shift || e.buttons === 4) cam = { ...cam, pan: [cam.pan[0] - (dx / box.w) * 2, cam.pan[1] + (dy / box.h) * 2] };
        else ((cam = { ...cam, yaw: cam.yaw - dx * 0.4, pitch: Math.max(-89, Math.min(89, cam.pitch + dy * 0.4)) }), seen !== "free" && (seen = "free"));
        down.x = e.clientX;
        down.y = e.clientY;
        tween++;
      }
      return;
    }
    hoverAt(local(e));
  }
  function stageUp(e: PointerEvent) {
    const d = down;
    down = null;
    if (!d || d.moved) return;
    const L = local(e);
    if (viewStyle === "flat") return;
    const h = pickAt(L.x, L.y);
    if (placing) return void (h.marker !== null && commit(h.marker));
    selected = h.part;
    if (h.part) tab = "part";
  }
  function stageWheel(e: WheelEvent) {
    if (viewStyle === "flat") return;
    e.preventDefault();
    const L = local(e);
    // Over the selected part the wheel stretches it, Shift widens; elsewhere it zooms.
    if (!placing && selected && !readonly) {
      const h = pickAt(L.x, L.y);
      const grp = groupOf(selected);
      if (h.part && grp.includes(h.part) && chosen) {
        const p = chosen.part;
        const k = e.deltaY < 0 ? 1.06 : 1 / 1.06;
        const base = partOf(chosen.built.part);
        if (base && (e.shiftKey ? base.scale.widen : base.scale.stretch)) {
          if (!wheelBusy) push();
          wheelBusy = true;
          clearTimeout(wheelT);
          wheelT = setTimeout(() => (wheelBusy = false), 500);
          const s = sizeOf(chosen.built, base);
          if (e.shiftKey) setSize(grp, null, s.R * k);
          else setSize(grp, s.L * k, null);
        } else toast(`${p.name} comes in the sizes its pack offers.`);
        return;
      }
    }
    cam = { ...cam, zoom: Math.max(0.3, Math.min(40, cam.zoom * Math.exp(-e.deltaY * 0.0015))) };
  }
  let wheelBusy = false;
  let wheelT: ReturnType<typeof setTimeout> | undefined;
  // Drag a part from the list onto a glowing point; a press without a drag arms it.
  function paletteDown(e: PointerEvent, p: Part) {
    paletteDrag = { part: p, x: e.clientX, y: e.clientY, moved: false };
    e.preventDefault();
  }
  function docMove(e: PointerEvent) {
    const d = paletteDrag;
    if (!d) return;
    if (!d.moved && Math.hypot(e.clientX - d.x, e.clientY - d.y) > 6) {
      d.moved = true;
      if (placing !== d.part.id) arm(d.part);
      sheet = null;
    }
    if (d.moved && stage) hoverAt(local(e));
  }
  function docUp(e: PointerEvent) {
    const d = paletteDrag;
    if (!d) return;
    paletteDrag = null;
    if (!d.moved) {
      arm(d.part);
      if (matchMedia("(max-width: 820px)").matches) sheet = null;
      return;
    }
    if (!stage) return;
    const L = local(e);
    if (L.inside) {
      const h = pickAt(L.x, L.y);
      if (h.marker !== null) commit(h.marker);
    }
  }
  function key(e: KeyboardEvent) {
    if ((e.target as HTMLElement | null)?.matches?.("input,select,textarea") || readonly) return;
    if (e.key === "Escape") (placing ? disarm() : (selected = null), (pop = null));
    if ((e.key === "Delete" || e.key === "Backspace") && selected) (removeSel(), e.preventDefault());
    if ((e.ctrlKey || e.metaKey) && e.key === "z") (doUndo(), e.preventDefault());
  }

  // --- the parts list, by what parts are for (the mock-up's groups), every part shown; those not yet flown by the year are faint ---
  const fromYear = (p: Part) => p.year ?? (palette?.eras ? eraOf(p, palette.eras)?.from : undefined);
  const groups = $derived(palette ? paletteByUse(palette) : []);
  const later = (p: Part) => (fromYear(p) ?? -Infinity) > year;
  const predicted = (p: Part) => (fromYear(p) ?? -Infinity) > NOW;
  const dryOf = (p: Part) => p.figures.dry_t?.v ?? p.figures.mass_t?.v;
  const kindIcon = (k: Part["kind"]) => (k === "engine" || k === "habitat" ? "M3 14 L8 3 L14 3 L19 14 Z" : k === "node" ? "M11 3 A5.5 5.5 0 1 1 10.99 3 Z" : k === "solar" || k === "radiator" ? "M2 6 L20 6 L20 11 L2 11 Z M6 6 L6 11 M10 6 L10 11 M14 6 L14 11" : k === "dish" ? "M3 5 Q11 15 19 5 M11 10 L11 15" : k === "truss" ? "M2 5 L20 5 L20 12 L2 12 Z M2 12 L6 5 L10 12 L14 5 L18 12" : "M4 5 L18 5 Q21 8.5 18 12 L4 12 Q1 8.5 4 5 Z");

  // --- the build tree ---
  type TreeRow = { key: string; depth: number; name: string; where: string };
  const tree = $derived.by(() => {
    const out: TreeRow[] = [];
    const visit = (on: string | undefined, depth: number) => {
      for (const b of recipe.parts.filter((x) => x.on === on)) {
        out.push({ key: b.key, depth, name: partOf(b.part)?.name ?? b.part, where: b.on ? (b.place === "hull" ? `hull ${Math.round(b.clock_deg ?? 0)}°` : b.place ?? "front") : "root" });
        visit(b.key, depth + 1);
      }
    };
    visit(undefined, 0);
    return out;
  });

  // --- the plans, for Chart ---
  const views = $derived(viewStyle === "flat" && layout?.pieces.length ? shipViews(layout, [Math.max(200, box.w - 40), Math.max(120, box.h / 2 - 60)]) : null);

  // --- words ---
  const t = (v: number) => (v >= 100 ? `${sayNumber(v, 0, { decimals: 0 })} t` : `${sayNumber(v, 0, { decimals: v >= 10 ? 1 : 2 })} t`);
  const m3 = (v: number) => `${sayNumber(v, 0, { decimals: v >= 100 ? 0 : 1 })} m³`;
  const kms = (v: number) => `${sayNumber(v / 1000, 0, { decimals: 2 })} km/s`;
  const share = (b: { need: number; built: number }) => (b.need > 0 ? Math.max(0, Math.min(1, b.built / b.need)) : 1);
  const whereWords = (b: BuiltPart) => {
    const on = recipe.parts.find((x) => x.key === b.on);
    const onName = on ? (partOf(on.part)?.name ?? on.part).toLowerCase() : "";
    return !b.on ? "The root of the build" : b.place === "hull" ? `On the hull of ${onName} · ${Math.round(b.clock_deg ?? 0)}° round, ${Math.round((b.along ?? 0.5) * 100)}% along` : b.place === "bay" ? `Inside ${onName}` : `${b.place === "behind" ? "Behind" : "In front of"} ${onName}`;
  };
  const shopping = $derived(bars.filter((b) => b.left > 0).map((b) => `${sayNumber(b.left, 0, { decimals: 0 })} ${b.unit} more ${b.label.toLowerCase()}`));
  const allNotes = $derived(layout ? [...notes, ...layout.notes] : notes);
  const hint = $derived(placing && placingPart ? `Placing <b>${placingPart.name}</b>${sym > 1 ? ` ×${sym}` : ""}: press a glowing point to clip it on · Esc to stop` : !recipe.parts.length ? "Press a part on the left to start the build" : "Drag to turn · Shift-drag to slide · wheel to zoom · press a part to select it");

  // The working screen fills the area it is given; Full screen gives it the whole display.
  let root = $state<HTMLElement | null>(null);
  const fullscreen = () => (document.fullscreenElement ? document.exitFullscreen() : root?.requestFullscreen?.())?.catch?.(() => {});
</script>

<svelte:document onpointermove={docMove} onpointerup={docUp} onkeydown={key} />

{#snippet stageView()}
  <section class="stage" class:flat={viewStyle === "flat"} bind:this={stage} data-builder-stage>
    {#if viewStyle === "flat"}
      {#if views}
        <div class="plans">
          {#each [["Side · from port", views.side], ["Top · from above", views.top]] as const as [title, v] (title)}
            <figure>
              <figcaption class="flatlabel">{title}</figcaption>
              <svg viewBox="0 0 {Math.max(200, box.w - 40)} {Math.max(120, box.h / 2 - 60)}" role="img" aria-label="{title} of {recipe.name}">
                {#each v.paths as p, i (i)}
                  <path class="{p.system}{p.member && groupOf(selected).includes(p.member) ? ' picked' : ''}" d={p.d} data-part-key={p.member} onclick={() => p.member && !readonly && ((selected = p.member), (tab = "part"))} role="presentation" />
                {/each}
              </svg>
            </figure>
          {/each}
        </div>
      {/if}
    {:else}
      <canvas bind:this={canvas} data-ship-model data-parts={shown?.placed.length ?? 0} data-markers={markers.length} onpointerdown={stageDown} onpointermove={stageMove} onpointerup={stageUp} onpointercancel={() => (down = null)} onpointerleave={() => (tip = null)} onwheel={stageWheel}></canvas>
      {#if noGpu}<p class="nogpu">This device cannot draw 3D: Chart shows the build's side and top.</p>{/if}
    {/if}
    <div class="viewbar">
      <div class="vgroup"><span class="lbl">View style</span>
        <div class="seg" role="group" aria-label="View style">
          <button type="button" aria-pressed={viewStyle === "flat"} onclick={() => (viewStyle = "flat")} data-view="chart">Chart</button>
          <button type="button" aria-pressed={viewStyle === "3d"} onclick={() => (viewStyle = "3d")} data-view="3d">3D</button>
          <button type="button" disabled title="True is held: the photoreal look comes with the renderer's True path">True</button>
          <button type="button" aria-pressed={viewStyle === "wire"} onclick={() => (viewStyle = "wire")} data-view="wire">Wireframe</button>
        </div>
      </div>
      <div class="vgroup"><span class="lbl">Seen from</span>
        <div class="seg" role="group" aria-label="Seen from">
          {#each [["above", "Above"], ["tilted", "Tilted"], ["edge", "Edge-on"], ["free", "Free"]] as const as [s, w] (s)}
            <button type="button" aria-pressed={seen === s} onclick={() => seeFrom(s)} data-seen={s}>{w}</button>
          {/each}
        </div>
      </div>
      {#if !readonly}
        <button type="button" class="chip" class:on={cut} aria-pressed={cut} onclick={() => ((cut = !cut), viewStyle === "flat" && (viewStyle = "3d"))} title="Open bays and habitats on this side, so what they hold shows" data-cutaway>Cutaway</button>
        <button type="button" class="chip" disabled title="Sensor views come with sensor parts and their fields of view (CM13 part 3)">Sensor views</button>
        <button type="button" class="chip" onclick={doUndo} title="Undo (Ctrl+Z)" data-undo>Undo</button>
      {/if}
    </div>
    {#if !readonly && viewStyle !== "flat"}
      <div class="legend" aria-label="Connection points">
        <span style="color: var(--b-accent)"><i></i>In front / behind</span>
        <span style="color: var(--b-good)"><i></i>On the hull</span>
        <span style="color: var(--b-violet)"><i></i>Inside a bay</span>
      </div>
      <div class="hint" data-hint>{@html hint}</div>
    {/if}
    {#if tip}<div class="tip" style="left: {tip.x}px; top: {tip.y}px">{@html tip.html}</div>{/if}
    {#if !readonly}
      <div class="phonebar"><button type="button" class="btn" onclick={() => (sheet = "palette")}>Parts</button><button type="button" class="btn" onclick={() => (sheet = "inspector")}>Selected · paint</button></div>
    {/if}
  </section>
{/snippet}

{#if readonly}
  <section class="builder readonly" data-builder aria-label="Ship from its parts" bind:this={root}>
    {#if missing}<p class="note">{missing}</p>{/if}
    {@render stageView()}
  </section>
{:else}
  <section class="builder" data-builder aria-label="Ship builder" bind:this={root}>
    <header class="top">
      <div class="place"><span class="desk">Edit ›</span><h1>Ship builder</h1></div>
      <label class="namefield"><span class="lbl">Name</span><input value={recipe.name} maxlength="40" aria-label="Ship name" oninput={(e) => (recipe = { ...recipe, name: e.currentTarget.value })} data-name /></label>
      <label class="era"><span class="lbl">Parts from</span><input type="range" min="1940" max="2250" step="1" bind:value={year} aria-label="Parts flown by this year" data-year /><b data-year-out>{year}</b>{#if year > NOW}<span class="tag">predicted parts</span>{/if}</label>
      <div class="grow"></div>
      {#if (catalogue.length || examples.length) && onopen}<button type="button" class="btn" onclick={() => (pop = pop === "ships" ? null : "ships")} data-example-ships>Example ships ▾</button>{/if}
      {#if onkeep}<button type="button" class="btn" onclick={() => onkeep?.($state.snapshot(recipe) as BuildRecipe)} disabled={!recipe.parts.length} data-keep-build>Keep with the ship</button>{/if}
      <button type="button" class="btn primary" onclick={whatCanItDo} disabled={asking || !layout?.pieces.length || !massKnown} title={massKnown ? undefined : `What it can do ${needsMass}`} data-what-can-it-do>{asking ? "Asking..." : "What can it do?"}</button>
      <button type="button" class="btn icon" onclick={fullscreen} title="Full screen" aria-label="Full screen">⛶</button>
    </header>

    <div class="work">
      <aside class="palette" class:open={sheet === "palette"} aria-label="Parts" data-palette>
        <div class="panel-h"><h2>Parts</h2><span class="lbl">{palette?.parts.length ?? 0} parts</span><button type="button" class="btn small sheetclose" onclick={() => (sheet = null)}>Done</button></div>
        <div class="symrow"><span class="lbl">Clip on</span>
          <div class="seg" role="group" aria-label="Copies round the hull">
            {#each [1, 2, 3, 4] as k (k)}<button type="button" aria-pressed={sym === k} onclick={() => (sym = k)} data-sym={k}>×{k}</button>{/each}
          </div>
          <span class="note">copies round the hull</span>
        </div>
        {#each groups as g (g.label)}
          <div class="grp">
            <span class="lbl">{g.label}</span>
            {#each g.parts as p (p.id)}
              {@const yr = fromYear(p)}
              {@const dry = dryOf(p)}
              <button type="button" class="part" class:armed={placing === p.id} class:later={later(p)} onpointerdown={(e) => paletteDown(e, p)} onkeydown={(e) => (e.key === "Enter" || e.key === " ") && (e.preventDefault(), arm(p))} data-palette-part={p.id} title={p.year ? `First flew ${p.year}` : p.advance ? `Figures from the tech advance ${p.advance}` : "A shape: geometry only"}>
                <svg class="ic" viewBox="0 0 22 16" aria-hidden="true"><path d={kindIcon(p.kind)} /></svg>
                <span><span class="nm">{p.name}</span><span class="yr">{yr !== undefined ? `${later(p) ? "from " : ""}${yr}` : "any year"}{dry !== undefined ? ` · ${sayNumber(dry, 0, { decimals: dry >= 10 ? 0 : 2 })} t dry` : ""}</span></span>
                {#if predicted(p)}<span class="pred">predicted</span>{/if}
              </button>
            {/each}
          </div>
        {/each}
      </aside>

      {@render stageView()}

      <aside class="inspector" class:open={sheet === "inspector"} aria-label="Selected part and paint">
        <div class="tabs" role="tablist">
          <button type="button" role="tab" aria-selected={tab === "part"} onclick={() => (tab = "part")} data-tab="part">Selected</button>
          <button type="button" role="tab" aria-selected={tab === "tree"} onclick={() => (tab = "tree")} data-tab="tree">Build tree</button>
          <button type="button" role="tab" aria-selected={tab === "paint"} onclick={() => (tab = "paint")} data-tab="paint">Paint and name</button>
          <button type="button" class="btn small sheetclose" onclick={() => (sheet = null)}>Done</button>
        </div>
        {#if tab === "part"}
          {#if chosen}
            {@const base = partOf(chosen.built.part)}
            {@const grp = groupOf(chosen.key)}
            <div class="sect" data-picked={chosen.key}>
              <h3 class="selname">{chosen.part.name}{grp.length > 1 ? ` ×${grp.length}` : ""}</h3>
              <div class="where">{whereWords(chosen.built)}</div>
              <div>{#if fromYear(chosen.part) !== undefined}<span class="pill">{fromYear(chosen.part)}</span>{/if} {#if chosen.part.maker}<span class="pill">{chosen.part.maker}</span>{/if} {#if predicted(chosen.part)}<span class="pill pred">predicted</span>{/if}</div>
            </div>
            {#if base}
              {@const s = sizeOf(chosen.built, base)}
              <div class="sect">
                <div class="row"><label for="b-len">Length</label><input id="b-len" type="range" min={Math.log(base.shape.length_m / 4)} max={Math.log(base.shape.length_m * 8)} step="0.01" value={Math.log(s.L)} disabled={!base.scale.stretch} onpointerdown={push} oninput={(e) => setSize(grp, Math.exp(+e.currentTarget.value), null)} data-length /><output>{sayNumber(s.L, 0, { decimals: 1 })} m</output></div>
                <div class="row"><label for="b-r">Across</label><input id="b-r" type="range" min={Math.log(base.shape.radius_m / 4)} max={Math.log(base.shape.radius_m * 6)} step="0.01" value={Math.log(s.R)} disabled={!base.scale.widen} onpointerdown={push} oninput={(e) => setSize(grp, null, Math.exp(+e.currentTarget.value))} data-across /><output>{sayNumber(2 * s.R, 0, { decimals: 1 })} m</output></div>
                {#if chosen.built.place === "hull"}
                  <div class="row"><label for="b-c">Round</label><input id="b-c" type="range" min="0" max="359" step="1" value={chosen.built.clock_deg ?? 0} onpointerdown={push} oninput={(e) => setClock(grp, +e.currentTarget.value)} data-clock /><output>{Math.round(chosen.built.clock_deg ?? 0)}°</output></div>
                {/if}
                {#if !base.scale.stretch && !base.scale.widen}<p class="note">Real hardware: it comes in the sizes its pack offers.</p>{:else}<p class="note">Over the part, the mouse wheel stretches it and Shift + wheel widens it. The figures follow.</p>{/if}
              </div>
            {/if}
            <div class="sect">
              <dl class="kv">
                {#each Object.entries(chosen.part.figures) as [k, f] (k)}
                  <dt>{k.replaceAll("_", " ")}</dt><dd>{sayNumber(f.v, 0, { decimals: Math.abs(f.v) >= 100 ? 0 : 2 })} {f.unit} {#if f.mark !== "cited"}<span class="mark">{f.mark}</span>{/if}</dd>
                {/each}
              </dl>
              {#if massOf(chosen.key)}<p class="note" data-mass-from title={Object.values(partSources).length ? (partSources[(chosen.part.figures.dry_t ?? chosen.part.figures.mass_t)?.src ?? ""] ?? "") : ""}>Mass: {massOf(chosen.key)}.</p>{:else}<p class="note" data-mass-from="none">Mass: not known yet, so the build's Δv waits for it.</p>{/if}
              {#if chosen.part.to_read?.length}<p class="note">Still to read: {chosen.part.to_read.join("; ")}</p>{/if}
            </div>
            <div class="sect actions"><button type="button" class="btn" onclick={removeSel} data-remove>Remove{groupOf(chosen.key).length > 1 ? " all copies" : ""}</button></div>
          {:else}
            <div class="sect help">
              <p class="note"><b>Pick a part</b> on the left, then press one of the glowing points to clip it on. Or drag it straight onto a point.</p>
              <p class="note">Press a part of the ship to select it. Over a selected part the <b>mouse wheel stretches</b> it and <b>Shift + wheel widens</b> it. The figures follow.</p>
              <p class="note">Drag the empty space to turn the ship. The blueprint behind stays still: its side, top and front drawings come from the same build and fill in as you clip parts on.</p>
            </div>
          {/if}
        {:else if tab === "tree"}
          <div class="sect"><div class="tree" data-tree>
            {#each tree as r (r.key)}
              <button type="button" class:sel={groupOf(selected).includes(r.key)} style="padding-left: {4 + r.depth * 12}px" onclick={() => ((selected = r.key), (tab = "part"))}><span>{r.name}</span><span class="pl">{r.where}</span></button>
            {:else}<p class="note">Nothing yet.</p>{/each}
          </div></div>
        {:else}
          <div class="sect">
            <label class="lbl" for="b-name">Name</label>
            <input id="b-name" class="inp" value={recipe.name} maxlength="40" oninput={(e) => (recipe = { ...recipe, name: e.currentTarget.value })} />
          </div>
          <div class="sect">
            <span class="lbl">{chosen ? `Paint ${chosen.part.name.toLowerCase()}` : "Paint: select a part first"}</span>
            <div class="swatches">
              {#each PAINTS as c (c)}<button type="button" class="sw" style="background: {c}" aria-label="Paint {c}" aria-pressed={chosen?.built.paint === c} disabled={!chosen} onclick={() => chosen && paint(groupOf(chosen.key), c)} data-paint={c}></button>{/each}
            </div>
            <button type="button" class="btn small" disabled={!chosen?.built.paint} onclick={() => { const c = chosen?.built.paint; if (c) paint(recipe.parts.filter((b) => { const k = partOf(b.part)?.kind; return k !== "solar" && k !== "radiator" && k !== "engine"; }).map((b) => b.key), c); toast("Painted the hull. Solar wings, radiators and engines keep their own surfaces."); }}>Paint the whole hull</button>
            <p class="note">Lettering and badges come later; the name is kept with the build.</p>
          </div>
        {/if}
      </aside>
    </div>

    <footer class="budgets" data-totals>
      <div class="brief">
        <span class="lbl">Shopping list for</span>
        <select class="inp" bind:value={planId} aria-label="What the ship must do" data-plan>
          <option value="">Nothing set: show what it can do</option>
          {#if ship}<option value="this">As capable as {ship.name}</option>{/if}
          {#if saved?.plan}<option value="saved">The kept plan ({saved.plan.name})</option>{/if}
          {#each catalogue.filter((c) => c.name !== ship?.name) as c (c.id)}<option value={c.id}>As capable as {c.name}</option>{/each}
        </select>
      </div>
      <div class="bars" data-budgets={bars.length ? "" : undefined}>
        {#if bars.length}
          {#each bars as b (b.id)}
            <div class="bar" class:done={b.left <= 0 && b.left > -b.need * 0.5} class:over={b.left < -b.need * 0.5} data-budget={b.id}>
              <div class="t"><span>{b.label}</span><b>need {sayNumber(b.need, 0, { decimals: 0 })} {b.unit}</b></div>
              <div class="track"><div class="fill" style="width: {share(b) * 100}%"></div></div>
              <div class="left" data-left={b.left}>{b.left <= 0 ? `covered · ${sayNumber(-b.left, 0, { decimals: 0 })} ${b.unit} spare` : `${sayNumber(b.left, 0, { decimals: 0 })} ${b.unit} to go`}</div>
            </div>
          {/each}
        {:else if layout && facts}
          {#if massKnown}
            <div class="bar fig"><div class="t"><span>Δv</span></div><div class="left" data-dv>{kms(facts.delta_v_m_s)}</div></div>
            <div class="bar fig"><div class="t"><span>Mass full / empty</span></div><div class="left">{t(layout.record.wet_t)} / {t(layout.record.dry_t)}</div></div>
          {:else}
            <div class="bar fig" data-needs-mass title={needsMass}><div class="t"><span>Δv and mass</span></div><div class="left muted">needs part masses</div></div>
            <div class="bar fig"><div class="t"><span>Propellant</span></div><div class="left">{t(layout.totals.propellant_t)}</div></div>
          {/if}
          <div class="bar fig"><div class="t"><span>Tank room</span></div><div class="left">{m3(Object.values(layout.totals.tank_m3).reduce((a, v) => a + v, 0))}</div></div>
          <div class="bar fig"><div class="t"><span>Room aboard</span></div><div class="left">{m3(layout.totals.habitat_m3)}</div></div>
          <div class="bar fig"><div class="t"><span>Radiator</span></div><div class="left">{sayNumber(layout.totals.radiator_m2, 0, { decimals: 0 })} m²</div></div>
          <div class="bar fig"><div class="t"><span>Size</span></div><div class="left">{sayNumber(layout.length_m, 0, { decimals: 1 })} m long</div></div>
        {/if}
      </div>
      <div class="shop">
        {#if shopping.length}<span><b>Parts you should use:</b> {shopping.join(" · ")}</span>{/if}
        {#if layout && facts && massKnown}<span>{#if bars.length}Δv <b data-dv>{kms(facts.delta_v_m_s)}</b> · {/if}Mass about <b>{t(layout.record.wet_t)}</b> · thrust to weight at Earth <b>{sayNumber(layout.record.accel_g, 0, { decimals: 2 })}</b></span>{:else if layout && needsMass}<span data-needs-mass>Δv, mass and push: {needsMass}, until their sizing blocks read them</span>{/if}
        <button type="button" class="btn small notesbtn" onclick={() => (pop = pop === "notes" ? null : "notes")} data-notes>{allNotes.length ? `${allNotes.length} note${allNotes.length > 1 ? "s" : ""} to look at` : "No warnings"}</button>
      </div>
    </footer>

    {#if pop}
      <div class="pop" role="dialog" aria-label={pop === "ships" ? "Example ships" : pop === "can" ? "What can it do?" : "Notes on this build"}>
        <button type="button" class="close" aria-label="Close" onclick={() => (pop = null)}>×</button>
        {#if pop === "ships"}
          <h3>Example ships</h3>
          <div class="sub">Each opens as the same parts, so you can pull any of them apart.</div>
          <div class="ships">
            {#if examples.length}<span class="lbl">Assembled from parts</span>{/if}
            {#each examples as e (e.id)}<button type="button" onclick={() => ((pop = null), openExample(e))} data-example-ship={e.id}><b>{e.name}</b><span>{e.about}{e.year !== undefined ? ` · ${e.year}` : ""}</span></button>{/each}
            {#if catalogue.length}<span class="lbl">From the catalogue (converted from the old layout)</span>{/if}
            {#each catalogue as c (c.id)}<button type="button" onclick={() => ((pop = null), onopen?.(c.id))} data-catalogue-ship={c.id}><b>{c.name}</b>{#if c.drive}<span>{c.drive}</span>{/if}</button>{/each}
            <button type="button" onclick={() => ((pop = null), onopen?.(""))} data-catalogue-ship=""><b>Nothing</b><span>a blank build</span></button>
          </div>
        {:else if pop === "can"}
          <h3>What can it do?</h3>
          <div class="sub">The physics' verdict on {recipe.name} as built, at a few worlds. It never refuses a build.</div>
          {#if asking}<p class="note">Asking...</p>{/if}
          {#if askFailed}<p class="note" data-ask-failed>{askFailed}</p>{/if}
          {#if report}
            <ul class="verdicts" data-can-do>
              {#each report.worlds as w (w.body)}
                {@const can = w.checks.filter((c) => c.verdict !== "not-here")}
                {#if can.length}
                  {#each can as c (c.id)}<li class={c.verdict}>{w.name}, {c.words.toLowerCase()}: {verdictWords(c.verdict)}</li>{/each}
                {:else}
                  <li class="not-here">{w.name}: nothing it can do there as built{w.checks[0]?.why?.[0] ? ` (${w.checks[0].why[0]})` : ""}</li>
                {/if}
              {/each}
            </ul>
          {/if}
        {:else}
          <h3>Notes on this build</h3>
          <div class="sub">It never refuses a build. These say what the parts cannot count yet, and what was carried over.</div>
          {#if allNotes.length}<ul class="notes">{#each allNotes as w, i (i)}<li>{w}</li>{/each}</ul>{:else}<p class="note">Nothing to warn about.</p>{/if}
        {/if}
      </div>
    {/if}
    {#if toastText}<div class="toast" role="status">{toastText}</div>{/if}
    {#if missing}<p class="note">{missing}</p>{/if}
  </section>
{/if}

<style>
  /* The builder's own dark blueprint look, from the mock-up (one look on purpose). */
  .builder {
    --b-ground: var(--sse-bench-ground); --b-raised: var(--sse-bench-raised); --b-raised2: var(--sse-bench-raised-2); --b-ink: var(--sse-bench-ink); --b-muted: var(--sse-bench-ink-muted); --b-faint: var(--sse-bench-ink-faint);
    --b-accent: var(--sse-bench-accent); --b-accent-ink: var(--sse-bench-accent-ink); --b-line: var(--sse-bench-line); --b-good: var(--sse-bench-good); --b-warn: var(--sse-bench-warn); --b-bad: var(--sse-bench-bad); --b-violet: var(--sse-bench-violet);
    --b-ease: var(--sse-ease);
    --b-mono: var(--sse-font-mono);
    color-scheme: dark; background: var(--b-ground); color: var(--b-ink); font-size: 13px; position: relative;
    display: grid; grid-template-rows: auto minmax(0, 1fr) auto; height: min(760px, 90vh); min-height: 420px; border: 1px solid var(--b-line); border-radius: 8px; overflow: hidden; min-width: 0;
  }
  .builder:fullscreen { border-radius: 0; height: 100vh !important; }
  .builder.readonly { display: block; min-height: 0; height: 360px; }
  .builder.readonly .stage { height: 100%; }
  button, input, select { font: inherit; color: inherit; }
  button { cursor: pointer; }
  .top { display: flex; flex-wrap: wrap; align-items: center; gap: 8px 14px; padding: 8px 16px; border-bottom: 1px solid var(--b-line); }
  .place { display: flex; align-items: baseline; gap: 8px; white-space: nowrap; }
  .place .desk { color: var(--b-muted); font-size: 12px; }
  .place h1 { font-weight: 700; font-size: 17px; margin: 0; }
  .namefield, .era { display: flex; align-items: center; gap: 6px; color: var(--b-muted); }
  .namefield input { background: var(--b-raised); border: 1px solid var(--b-line); border-radius: 6px; padding: 5px 8px; width: 150px; font-weight: 500; }
  .era input { width: 150px; accent-color: var(--b-accent); }
  .era b { font-family: var(--b-mono); color: var(--b-ink); font-weight: 500; min-width: 3.2em; }
  .era .tag { font-size: 10.5px; color: var(--b-violet); }
  .grow { flex: 1; }
  .btn { background: var(--b-raised); border: 1px solid var(--b-line); border-radius: 7px; padding: 6px 11px; transition: transform 0.12s var(--b-ease), border-color 0.12s; }
  .btn:hover:not(:disabled) { border-color: color-mix(in srgb, var(--b-accent) 55%, var(--b-line)); transform: translateY(-1px); }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn.primary { background: var(--b-accent); color: var(--sse-bench-on-accent); border-color: var(--b-accent); font-weight: 600; }
  .btn.small { padding: 3px 8px; font-size: 12px; }
  .btn.icon { padding: 6px 9px; }
  .lbl { font-size: 10.5px; text-transform: uppercase; letter-spacing: 0.08em; color: var(--b-faint); }
  .work { display: grid; grid-template-columns: 232px minmax(0, 1fr) 272px; grid-template-rows: minmax(0, 1fr); min-height: 0; }
  aside { min-height: 0; overflow: auto; border-right: 1px solid var(--b-line); background: var(--b-ground); }
  .inspector { border-right: 0; border-left: 1px solid var(--b-line); }
  .panel-h { display: flex; align-items: center; justify-content: space-between; gap: 8px; padding: 10px 12px 6px; position: sticky; top: 0; background: var(--b-ground); z-index: 2; }
  .panel-h h2 { font-size: 13px; margin: 0; font-weight: 600; }
  .symrow { display: flex; align-items: center; gap: 6px; padding: 4px 12px 10px; flex-wrap: wrap; }
  .grp { padding: 4px 8px 8px; }
  .grp > .lbl { padding: 6px 4px 4px; display: block; }
  .pop .ships > .lbl { display: block; padding: 8px 2px 2px; }
  .left.muted { color: var(--b-muted); font-size: 0.9em; }
  .part { display: grid; grid-template-columns: 22px 1fr auto; align-items: center; gap: 8px; width: 100%; text-align: left; background: transparent; border: 1px solid transparent; border-radius: 7px; padding: 5px 6px; touch-action: none; transition: background 0.12s, border-color 0.12s; }
  .part:hover { background: var(--b-raised); border-color: var(--b-line); }
  .part.armed { border-color: var(--b-accent); background: color-mix(in srgb, var(--b-accent) 12%, var(--b-raised)); }
  .part.later { opacity: 0.45; }
  .part .ic { width: 22px; height: 16px; fill: none; stroke: var(--b-accent-ink); stroke-width: 1.4; }
  .part .nm { display: block; font-size: 12.5px; line-height: 1.2; }
  .part .yr { display: block; font-family: var(--b-mono); font-size: 10.5px; color: var(--b-faint); }
  .pred { font-family: var(--b-mono); font-size: 9.5px; color: var(--b-violet); letter-spacing: 0.04em; }
  .seg { display: inline-flex; border: 1px solid var(--b-line); border-radius: 7px; overflow: hidden; }
  .seg button { background: transparent; border: 0; padding: 4px 8px; font-size: 12px; color: var(--b-muted); border-right: 1px solid var(--b-line); }
  .seg button:last-child { border-right: 0; }
  .seg button[aria-pressed="true"] { background: color-mix(in srgb, var(--b-accent) 18%, transparent); color: var(--b-ink); }
  .seg button:disabled { color: var(--b-faint); cursor: not-allowed; }
  .stage { position: relative; min-width: 0; min-height: 0; overflow: hidden; background: radial-gradient(120% 90% at 50% 40%, var(--b-raised) 0%, var(--b-ground) 70%); }
  .stage.flat { --g1: color-mix(in srgb, var(--b-accent) 7%, transparent); --g2: color-mix(in srgb, var(--b-accent) 3.5%, transparent); background-color: var(--b-ground); background-image: linear-gradient(var(--g1) 1px, transparent 1px), linear-gradient(90deg, var(--g1) 1px, transparent 1px), linear-gradient(var(--g2) 1px, transparent 1px), linear-gradient(90deg, var(--g2) 1px, transparent 1px); background-size: 120px 120px, 120px 120px, 24px 24px, 24px 24px; }
  canvas { position: absolute; inset: 0; width: 100%; height: 100%; display: block; touch-action: none; cursor: grab; }
  .plans { position: absolute; inset: 52px 20px 20px; display: grid; grid-template-rows: 1fr 1fr; gap: 8px; }
  .plans figure { margin: 0; position: relative; min-height: 0; }
  .plans svg { width: 100%; height: 100%; display: block; }
  .flatlabel { font-family: var(--b-mono); font-size: 11px; letter-spacing: 0.08em; color: var(--b-accent-ink); text-transform: uppercase; }
  .plans path { stroke: var(--b-accent); stroke-width: 1; fill: color-mix(in srgb, var(--b-accent) 6%, transparent); stroke-linejoin: round; cursor: pointer; }
  .plans path.picked { stroke: var(--b-ink); stroke-width: 2; fill: color-mix(in srgb, var(--b-accent) 20%, transparent); }
  .viewbar { position: absolute; top: 10px; left: 12px; right: 12px; display: flex; flex-wrap: wrap; gap: 8px 12px; align-items: center; pointer-events: none; }
  .viewbar > * { pointer-events: auto; }
  .viewbar .seg { background: color-mix(in srgb, var(--b-ground) 82%, transparent); backdrop-filter: blur(6px); }
  .vgroup { display: flex; align-items: center; gap: 6px; }
  .chip { background: color-mix(in srgb, var(--b-ground) 82%, transparent); border: 1px solid var(--b-line); border-radius: 999px; padding: 4px 10px; font-size: 12px; color: var(--b-muted); }
  .chip:disabled { color: var(--b-faint); cursor: not-allowed; }
  .chip.on { color: var(--b-accent); border-color: var(--b-accent); }
  .hint { position: absolute; left: 12px; bottom: 12px; background: color-mix(in srgb, var(--b-ground) 88%, transparent); border: 1px solid var(--b-line); border-radius: 999px; padding: 6px 14px; font-size: 12px; color: var(--b-muted); white-space: nowrap; max-width: calc(100% - 190px); overflow: hidden; text-overflow: ellipsis; pointer-events: none; }
  .hint :global(b), .tip :global(b) { color: var(--b-ink); font-weight: 500; }
  .tip { position: absolute; pointer-events: none; background: var(--b-raised2); border: 1px solid var(--b-line); border-radius: 6px; padding: 4px 8px; font-size: 12px; white-space: nowrap; transform: translate(12px, -130%); }
  .tip :global(.mut) { color: var(--b-muted); }
  .legend { position: absolute; right: 12px; bottom: 12px; display: flex; flex-direction: column; gap: 3px; font-size: 11px; color: var(--b-muted); background: color-mix(in srgb, var(--b-ground) 85%, transparent); border: 1px solid var(--b-line); border-radius: 8px; padding: 6px 9px; }
  .legend i { display: inline-block; width: 9px; height: 9px; border-radius: 50%; margin-right: 6px; vertical-align: -1px; border: 2px solid currentColor; box-sizing: border-box; }
  .nogpu { position: absolute; left: 12px; top: 56px; color: var(--b-muted); }
  .tabs { display: flex; gap: 2px; padding: 8px 10px 0; border-bottom: 1px solid var(--b-line); position: sticky; top: 0; background: var(--b-ground); z-index: 3; }
  .tabs button { background: transparent; border: 0; padding: 7px 9px; color: var(--b-muted); border-bottom: 2px solid transparent; font-size: 12.5px; }
  .tabs button[aria-selected="true"] { color: var(--b-ink); border-bottom-color: var(--b-accent); }
  .sect { padding: 10px 12px 12px; border-bottom: 1px solid var(--b-line); display: grid; gap: 8px; }
  .sect:last-child { border-bottom: 0; }
  .row { display: grid; grid-template-columns: 64px 1fr auto; align-items: center; gap: 8px; }
  .row label { color: var(--b-muted); font-size: 12px; }
  .row input[type="range"] { width: 100%; accent-color: var(--b-accent); }
  .row output { font-family: var(--b-mono); font-size: 11.5px; min-width: 4.5em; text-align: right; }
  .kv { display: grid; grid-template-columns: 1fr auto; gap: 3px 10px; font-size: 12px; margin: 0; }
  .kv dt { color: var(--b-muted); }
  .kv dd { margin: 0; font-family: var(--b-mono); font-size: 11.5px; text-align: right; }
  .mark { font-size: 10px; color: var(--b-warn); font-family: var(--b-mono); }
  .selname { font-weight: 600; font-size: 15px; margin: 0; }
  .where { color: var(--b-muted); font-size: 12px; }
  .pill { display: inline-block; font-size: 10.5px; border: 1px solid var(--b-line); border-radius: 999px; padding: 1px 7px; color: var(--b-muted); }
  .note { font-size: 11.5px; color: var(--b-muted); line-height: 1.45; margin: 0; }
  .help .note { margin-bottom: 8px; }
  .note :global(b), .help b { color: var(--b-ink); }
  .tree { display: grid; gap: 1px; font-size: 12px; }
  .tree button { display: flex; align-items: center; gap: 6px; background: transparent; border: 0; border-radius: 5px; padding: 3px 4px; text-align: left; width: 100%; }
  .tree button:hover { background: var(--b-raised); }
  .tree button.sel { background: color-mix(in srgb, var(--b-accent) 16%, transparent); }
  .tree .pl { color: var(--b-faint); font-size: 11px; margin-left: auto; white-space: nowrap; }
  .inp { background: var(--b-raised); border: 1px solid var(--b-line); border-radius: 6px; padding: 5px 7px; width: 100%; box-sizing: border-box; }
  .swatches { display: flex; flex-wrap: wrap; gap: 6px; }
  .sw { width: 26px; height: 26px; border-radius: 6px; border: 2px solid var(--b-line); padding: 0; }
  .sw[aria-pressed="true"] { border-color: var(--b-accent); }
  .budgets { border-top: 1px solid var(--b-line); padding: 8px 16px; display: grid; grid-template-columns: auto 1fr; gap: 8px 16px; align-items: start; }
  .brief { display: grid; gap: 6px; min-width: 190px; }
  .brief select { max-width: 240px; }
  .bars { display: grid; grid-template-columns: repeat(6, minmax(110px, 1fr)); gap: 10px; min-width: 0; }
  .bar { display: grid; gap: 3px; min-width: 0; }
  .bar .t { display: flex; justify-content: space-between; gap: 6px; font-size: 11.5px; color: var(--b-muted); }
  .bar .t b { color: var(--b-ink); font-weight: 500; }
  .bar .track { height: 6px; border-radius: 3px; background: var(--b-raised2); overflow: hidden; position: relative; }
  .bar .fill { position: absolute; inset: 0 auto 0 0; border-radius: 3px; background: var(--b-accent); transition: width 0.22s var(--b-ease); }
  .bar .left { font-family: var(--b-mono); font-size: 11px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .bar.fig .left { font-size: 13px; }
  .bar.done .fill { background: var(--b-good); }
  .bar.done .left { color: var(--b-good); }
  .bar.over .fill { background: var(--b-warn); }
  .shop { grid-column: 2; display: flex; flex-wrap: wrap; gap: 6px 14px; align-items: center; font-size: 12px; color: var(--b-muted); }
  .shop b { color: var(--b-ink); font-weight: 500; }
  .notesbtn { margin-left: auto; }
  .pop { box-sizing: border-box; position: absolute; z-index: 50; right: 16px; top: 52px; background: var(--b-raised); border: 1px solid var(--b-line); border-radius: 12px; box-shadow: var(--sse-card-shadow); width: min(420px, calc(100% - 24px)); max-height: min(70%, 680px); overflow: auto; padding: 14px 16px; animation: grow 0.18s var(--b-ease); transform-origin: 90% 0; }
  @keyframes grow { from { opacity: 0; transform: scale(0.92); } to { opacity: 1; transform: none; } }
  .pop h3 { margin: 0 0 2px; font-size: 16px; }
  .pop .sub { color: var(--b-muted); font-size: 12px; margin-bottom: 10px; }
  .pop .close { position: absolute; top: 8px; right: 10px; background: transparent; border: 0; color: var(--b-muted); font-size: 18px; }
  .ships { display: grid; gap: 6px; }
  .ships button { display: grid; text-align: left; background: var(--b-ground); border: 1px solid var(--b-line); border-radius: 9px; padding: 8px 10px; }
  .ships button:hover { border-color: var(--b-accent); }
  .ships b { font-size: 13.5px; }
  .ships span { color: var(--b-muted); font-size: 11.5px; }
  .verdicts, .notes { margin: 0; padding-left: 1.2em; font-size: 12.5px; display: grid; gap: 4px; }
  .verdicts .dangerous { color: var(--b-bad); }
  .toast { position: absolute; left: 50%; bottom: 110px; transform: translateX(-50%); background: var(--b-raised2); border: 1px solid var(--b-line); padding: 8px 14px; border-radius: 9px; z-index: 60; font-size: 12.5px; animation: grow 0.16s var(--b-ease); max-width: calc(100% - 32px); }
  .phonebar, .sheetclose { display: none; }
  @media (max-width: 820px) {
    .builder:not(.readonly) { grid-template-rows: auto minmax(360px, 1fr) auto; }
    .work { grid-template-columns: minmax(0, 1fr); }
    aside { position: absolute; left: 0; right: 0; bottom: 0; max-height: 62%; z-index: 40; border: 0; border-top: 1px solid var(--b-line); border-radius: 14px 14px 0 0; transform: translateY(105%); transition: transform 0.22s var(--b-ease); box-shadow: var(--sse-card-shadow); visibility: hidden; }
    aside.open { transform: none; visibility: visible; }
    .inspector { border-left: 0; }
    .era input { width: 100px; }
    .namefield input { width: 110px; }
    .bars { display: flex; overflow-x: auto; gap: 12px; padding-bottom: 2px; }
    .bar { min-width: 130px; }
    .budgets { grid-template-columns: minmax(0, 1fr); }
    .shop { grid-column: 1; }
    .legend { display: none; }
    .phonebar { display: flex; gap: 8px; position: absolute; left: 12px; bottom: 12px; right: 12px; justify-content: center; }
    .hint { bottom: 58px; }
    .viewbar .lbl, .namefield .lbl, .era .lbl, .grow { display: none; }
    .sheetclose { display: inline-block; margin-left: auto; }
    .seg button { padding: 4px 7px; }
  }
  @media (prefers-reduced-motion: reduce) { * { animation: none !important; transition: none !important; } }
</style>
