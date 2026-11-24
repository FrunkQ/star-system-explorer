// Stations and land constructs built from their description (Alex, 24 Nov): a construct's words pick one of about two
// dozen shapes from the base-ships pack, its population sizes it, and its gravity, power and traffic size the parts. A
// 30-person ring and a 20-million ring are the same shape, one far bigger and carrying far more bolt-ons. Self
// contained and swappable: a record and a recipe in, a layout of plain parts out, nothing else read or written.
//
// The megastructure line is read off the physics, never set by hand:
// - a spun hull's hoop stress over its density is g R (Alex's "when that space station becomes a torus"); past what
//   ordinary steel carries it needs the materials further up base-tech's strength_for_weight ladder, and past the
//   carbon bond nothing can carry it;
// - a tower stands only to its material's strength over density over g;
// - when its own gravity holds a walker (escape speed over about 1 m/s), it is a body, not a building.
//
// Axes: z is up (a spun station's axis; on the ground, the vertical), x and y lie in the plane. Metres throughout.
import { constant } from "../../ui-contract/src/index.ts";

export type StationPlace = "space" | "ground" | "air" | "sea";
export type StationForm = "ring" | "cylinder" | "sphere" | "tether" | "truss" | "cluster" | "cross" | "can" | "ball" | "disc" | "dock_ring" | "spike" | "rock" | "derelict" | "dome" | "tower" | "pyramid" | "sprawl" | "buried" | "crater" | "aerostat" | "platform";
export type StationSurface = "hull" | "habitat" | "glass" | "truss" | "radiators" | "solar" | "rock" | "dock";
export type Axis = "x" | "y" | "z";
type V3 = [number, number, number];

/** A construct as the builder reads it (construct.* facts and its words). */
export interface StationRecord {
  name: string;
  /** People aboard or living there (construct.crew_max, else construct.crew). */
  population: number;
  place?: StationPlace;
  mass_kg?: number;
  power_w?: number;
  cargo_t?: number;
  /** construct.modules: each becomes a labelled bolt-on. */
  modules?: string[];
  tags?: string[];
  /** Its description (the template's explain, the GM's notes): what picks the shape. */
  words?: string;
  /** Gravity wanted in a spun station, or the world's surface gravity on the ground, in standard gravities. */
  gravity_g?: number;
}

/** base-ships `station_shapes` entry. */
export interface StationShape {
  name: string;
  form: StationForm;
  place: StationPlace;
  /** Who lives there: a working post or a home (base-ships `station_design.living`). */
  living: string;
  spin: boolean;
  words: string[];
  count?: number;
  spokes?: number;
  /** A ring's tube radius over its radius. */
  aspect?: number;
  length_over_r?: number;
  windows?: number;
  hub_sphere?: boolean;
}

/** base-ships `station_design`. */
export interface StationRules {
  living: { id: string; m3_per_person: number; why: string }[];
  spin: { g_default: number };
  hoop: { safety: number; load_over_shell: number };
  tower: { safety: number };
  self_gravity: { escape_m_s: number };
  mass_kg_per_m3: { v: number };
  power: { w_per_person: number; efficiency: number; radiator_k: number };
  urban_density_per_km2: { v: number };
  docks: { people_per_berth: number; size_share: number; port_bonus: number; classes: { id: string; name: string; max_ship_m: number }[] };
  bolt_on_share: number;
}

/** One rung of base-tech's strength_for_weight ladder (J/kg, which is stress over density). */
export interface StrengthMark {
  mark: "built" | "allowed" | "ceiling" | string;
  v: number;
  what: string;
}

/** What the builder reads from base-tech: the comfortable spin and the strength ladder. */
export interface StationLimits {
  spin_rpm: number;
  strength: StrengthMark[];
}

/** Hand changes kept with the recipe; each wins over the record and is said to be by hand. */
export interface StationOverrides {
  population?: number;
  gravity_g?: number;
  berths?: number;
  living?: string;
}

/** Kept on the construct as `construct.design`: everything needed to build the same station anywhere. */
export interface StationRecipe {
  shape: string;
  seed: number;
  overrides?: StationOverrides;
}

/** One solid. Every part is aligned with an axis, so plans are plain outlines.
 * tube: along `axis` through `c`, `len` long, radius r0 at the low end and r1 at the high end.
 * torus: round `axis` through `c`, radius R, tube radius r. sphere: radius r (hemi: the half above c).
 * box: `size` along x, y, z. panel: flat, facing `axis`, `size[0]` by `size[1]` across it (lumpy rock uses `lumpy`). */
export interface Part {
  surface: StationSurface;
  form: "tube" | "torus" | "sphere" | "box" | "panel";
  c: V3;
  axis: Axis;
  len?: number;
  r0?: number;
  r1?: number;
  R?: number;
  r?: number;
  hemi?: boolean;
  size?: V3;
  /** A rock's roughness, as a share of its radius. */
  lumpy?: number;
  /** Part of the power arrays (panels and the mast that carries them), left out when closing in on the station. */
  array?: boolean;
  label?: string;
}

export interface DockClass {
  id: string;
  name: string;
  max_ship_m: number;
  count: number;
}

export interface Megastructure {
  /** Why it is one, in words. */
  why: string;
  /** The material its hull would need, from the strength ladder; null when even the carbon bond is not enough. */
  needs: string | null;
  /** Past every material: an honesty mark (fictional) on the GM's view. */
  fictional: boolean;
}

export interface StationLayout {
  name: string;
  shape: string;
  seed: number;
  place: StationPlace;
  parts: Part[];
  /** Widest extent, metres. */
  size_m: number;
  population: number;
  living: string;
  /** Pressurised volume the people need. */
  volume_m3: number;
  mass_kg: number;
  power_w: number;
  spin: { g: number; rpm: number; radius_m: number } | null;
  docks: DockClass[];
  mega: Megastructure | null;
  by_hand: (keyof StationOverrides)[];
  notes: string[];
}

/** A small deterministic generator (mulberry32) for the remix choices. */
function rng(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** The shape a record's words fit best on its own place, else the first shape for that place. Ties go to the order
 * of the pack. A shape's word scores once however often it appears. */
export function pickShape(rec: StationRecord, shapes: Record<string, StationShape>): string {
  const place = rec.place ?? "space";
  const text = ` ${[rec.words ?? "", ...(rec.tags ?? []), ...(rec.modules ?? []), rec.name].join(" ").toLowerCase().replace(/[_.]+/g, " ")} `;
  let best: string | null = null;
  let bestScore = 0;
  for (const [id, s] of Object.entries(shapes)) {
    if (s.place !== place) continue;
    // A word must stand as a word ("ring" is not in "refuelling"), longer phrases count more.
    const score = s.words.reduce((a, w) => a + (new RegExp(`[^a-z]${w.replace(/[.*+?^${}()|[\]\\']/g, "\\$&")}s?[^a-z]`).test(text) ? w.split(" ").length : 0), 0);
    if (score > bestScore) ((best = id), (bestScore = score));
  }
  return best ?? Object.keys(shapes).find((id) => shapes[id]!.place === place) ?? Object.keys(shapes)[0]!;
}

const fmt = (x: number) => (x >= 1e4 ? Math.round(x).toLocaleString("en-GB") : x >= 100 ? Math.round(x).toString() : x >= 10 ? x.toFixed(0) : x.toPrecision(2));
export function lengthWords(m: number): string {
  return m >= 1000 ? `${fmt(m / 1000)} km` : `${fmt(m)} m`;
}
function powerWords(w: number): string {
  return w >= 1e9 ? `${fmt(w / 1e9)} GW` : w >= 1e6 ? `${fmt(w / 1e6)} MW` : `${fmt(w / 1e3)} kW`;
}

/** The strength over density a spun hull of radius R needs at gravity g (m/s²): the hoop's own g R, times the load
 * it carries and a safety factor. */
export function hoopNeed(g: number, R: number, rules: StationRules): number {
  return g * R * (1 + rules.hoop.load_over_shell) * rules.hoop.safety;
}

/** Whether a structure needing `need` J/kg is ordinary (the best built material carries it) or a megastructure. */
function strengthVerdict(need: number, limits: StationLimits, what: string): Megastructure | null {
  const ladder = [...limits.strength].sort((a, b) => a.v - b.v);
  const ordinary = ladder.find((m) => m.mark === "built");
  if (!ordinary || need <= ordinary.v) return null;
  const enough = ladder.find((m) => m.v >= need);
  if (!enough) return { why: `${what} needs ${fmt(need / 1e3)} kJ/kg of strength for its weight, past the carbon bond itself`, needs: null, fictional: true };
  return { why: `${what} needs ${fmt(need / 1e3)} kJ/kg of strength for its weight, more than ${ordinary.what} (${fmt(ordinary.v / 1e3)})`, needs: enough.what, fictional: false };
}

/** Build a station from its record. Sizes come from the record and the rules; the shape only arranges. */
export function designStation(rec0: StationRecord, rules: StationRules, shapes: Record<string, StationShape>, limits: StationLimits, recipe: StationRecipe): StationLayout {
  const over = recipe.overrides ?? {};
  const shapeId = shapes[recipe.shape] ? recipe.shape : pickShape(rec0, shapes);
  const shape = shapes[shapeId]!;
  const rnd = rng(recipe.seed);
  const g0 = constant("g0");
  const population = Math.max(1, over.population ?? rec0.population);
  const livingId = over.living ?? shape.living;
  const living = rules.living.find((l) => l.id === livingId) ?? rules.living[0]!;
  const volume = population * living.m3_per_person;
  const place = shape.place;
  const parts: Part[] = [];
  const notes: string[] = [];
  const by_hand = (Object.keys(over) as (keyof StationOverrides)[]).filter((k) => over[k] !== undefined);
  let spin: StationLayout["spin"] = null;
  let mega: Megastructure | null = null;
  const tube = (surface: StationSurface, axis: Axis, c: V3, len: number, r0: number, r1 = r0, label?: string) => parts.push({ surface, form: "tube", axis, c, len, r0, r1, ...(label ? { label } : {}) });
  const box = (surface: StationSurface, c: V3, size: V3, label?: string) => parts.push({ surface, form: "box", axis: "z", c, size, ...(label ? { label } : {}) });
  const panel = (surface: StationSurface, axis: Axis, c: V3, a: number, b: number, label?: string) => parts.push({ surface, form: "panel", axis, c, size: [a, b, 0], ...(label ? { label } : {}) });

  // Spin: the radius one gravity needs at the comfortable rate is the smallest a spun hull can be.
  const gWant = (over.gravity_g ?? rec0.gravity_g ?? rules.spin.g_default) * g0;
  const omegaMax = (limits.spin_rpm * 2 * Math.PI) / 60;
  const rMin = gWant / (omegaMax * omegaMax);
  const spun = (R: number) => {
    const omega = Math.sqrt(gWant / R);
    spin = { g: gWant / g0, rpm: (omega * 60) / (2 * Math.PI), radius_m: R };
    mega = mega ?? strengthVerdict(hoopNeed(gWant, R, rules), limits, `A hull spun to ${fmt(gWant / g0)} g at ${lengthWords(R)} radius`);
  };
  // The hub a spun station docks at: on its axis, not turning, `h` long.
  const hub = (h: number, r: number) => tube("hull", "z", [0, 0, 0], h, r, r, "hub (not spinning): docks at both ends");
  let size = 0;
  let hubTop = 0;
  let hubR = 0;
  let edge = 0;

  switch (shape.form) {
    case "ring": {
      const n = shape.count ?? 1;
      const a = shape.aspect ?? 0.08;
      const v1 = volume / n;
      let R = Math.cbrt(v1 / (2 * Math.PI * Math.PI * a * a));
      let r = a * R;
      if (R < rMin) ((R = rMin), (r = Math.max(1.5, Math.sqrt(v1 / (2 * Math.PI * Math.PI * R)))));
      spun(R);
      const gap = 3 * r + 0.1 * R;
      hubR = Math.max(2, 0.06 * R);
      const h = n * gap + 2 * r;
      hub(h, hubR);
      for (let k = 0; k < n; k++) {
        const z = (k - (n - 1) / 2) * gap;
        parts.push({ surface: "habitat", form: "torus", axis: "z", c: [0, 0, z], R, r, ...(k === 0 ? { label: `${n > 1 ? `${n} rings` : "ring"}, ${lengthWords(2 * R)} across, ${fmt(spin!.rpm)} turns a minute` } : {}) });
        const spokes = shape.spokes ?? 4;
        for (let s = 0; s < spokes; s++) {
          const t = (2 * Math.PI * (s + (k % 2) * 0.5)) / spokes;
          const mid = (R + hubR) / 2;
          // A spoke runs radially; the plans and mesh take x or y, so turn it onto the nearer of the two.
          const ax: Axis = Math.abs(Math.cos(t)) > Math.abs(Math.sin(t)) ? "x" : "y";
          const c: V3 = ax === "x" ? [Math.sign(Math.cos(t)) * mid, 0, z] : [0, Math.sign(Math.sin(t)) * mid, z];
          tube("truss", ax, c, R - hubR - r, Math.max(0.5, 0.25 * r), Math.max(0.5, 0.25 * r), s === 0 && k === 0 ? "spokes (lifts to the hub)" : undefined);
        }
      }
      if (shape.hub_sphere) parts.push({ surface: "hull", form: "sphere", axis: "z", c: [0, 0, 0], r: 2.5 * hubR, label: "central hub" });
      size = 2 * (R + r);
      hubTop = h / 2;
      edge = R + r;
      break;
    }
    case "cylinder": {
      const n = shape.count ?? 1;
      const k = shape.length_over_r ?? 5;
      const v1 = volume / n;
      let R = Math.cbrt(v1 / (Math.PI * k));
      let L = k * R;
      if (R < rMin) ((R = rMin), (L = Math.max(R * 0.5, v1 / (Math.PI * R * R))));
      spun(R);
      for (let i = 0; i < n; i++) {
        const x = n > 1 ? (i - (n - 1) / 2) * 2.4 * R : 0;
        tube("habitat", "z", [x, 0, 0], L, R, R, i === 0 ? `${n > 1 ? `${n} cylinders` : "cylinder"}, ${lengthWords(2 * R)} across and ${lengthWords(L)} long` : undefined);
        const w = shape.windows ?? 0;
        // Windows: strips of glass along the length, lit by mirrors (the land between them is the floor).
        for (let s = 0; s < w; s++) {
          const t = (2 * Math.PI * s) / w;
          panel("glass", Math.abs(Math.cos(t)) > Math.abs(Math.sin(t)) ? "x" : "y", Math.abs(Math.cos(t)) > Math.abs(Math.sin(t)) ? [x + Math.sign(Math.cos(t)) * R * 1.002, 0, 0] : [x, Math.sign(Math.sin(t)) * R * 1.002, 0], 0.5 * R, 0.9 * L, s === 0 && i === 0 ? "windows" : undefined);
        }
        tube("hull", "z", [x, 0, L / 2 + 0.05 * R], 0.1 * R, 0.15 * R, 0.1 * R);
      }
      hubR = 0.15 * R;
      hubTop = L / 2 + 0.1 * R;
      size = Math.max(L, n * 2.4 * R);
      edge = (n > 1 ? 1.2 * R * (n - 1) : 0) + R;
      break;
    }
    case "sphere": {
      let R = Math.cbrt((3 * volume) / (4 * Math.PI));
      if (R < rMin) R = rMin;
      spun(R);
      parts.push({ surface: "habitat", form: "sphere", axis: "z", c: [0, 0, 0], r: R, label: `sphere, ${lengthWords(2 * R)} across, ${fmt(spin!.rpm)} turns a minute` });
      hubR = 0.12 * R;
      hubTop = R + 0.3 * R;
      hub(2 * hubTop, hubR);
      size = 2 * R;
      edge = R;
      break;
    }
    case "tether": {
      // Two modules swung round their middle on a tether: the cheapest spin for a few people.
      const R = rMin;
      spun(R);
      const v1 = volume / 2;
      const r = Math.max(2, Math.cbrt(v1 / (Math.PI * 3)));
      const L = v1 / (Math.PI * r * r);
      tube("habitat", "z", [R, 0, 0], L, r, r, `two modules on a ${lengthWords(2 * R)} tether`);
      tube("habitat", "z", [-R, 0, 0], L, r, r);
      tube("truss", "x", [0, 0, 0], 2 * R, 0.3, 0.3, "tether");
      hubR = Math.max(1.5, 0.5 * r);
      hubTop = L / 2;
      hub(L, hubR);
      size = 2 * R + 2 * r;
      edge = R + r;
      break;
    }
    case "truss":
    case "cluster":
    case "cross":
    case "spike": {
      // Modules of one size; a big post has more of them and bigger ones (the module grows with the cube root).
      const r = Math.max(2.1, 0.6 * Math.cbrt(volume / 40));
      const modLen = 4 * r;
      const nMod = Math.max(2, Math.round(volume / (Math.PI * r * r * modLen)));
      const label = `${nMod} pressurised modules, ${lengthWords(2 * r)} across`;
      if (shape.form === "truss") {
        const span = Math.max(8 * r, nMod * 1.2 * r);
        tube("truss", "x", [0, 0, 0], span, 0.25 * r, 0.25 * r, "main truss");
        const row = Math.min(nMod, 6);
        for (let i = 0; i < nMod; i++) {
          const col = i % row;
          const tier = Math.floor(i / row);
          tube("habitat", "y", [(col - (row - 1) / 2) * 1.3 * modLen * 0.5, (tier % 2 ? -1 : 1) * (r + modLen / 2) * (1 + Math.floor(tier / 2)), 0], modLen, r, r, i === 0 ? label : undefined);
        }
        size = span;
        edge = span / 2;
      } else if (shape.form === "spike") {
        const L = nMod * modLen;
        tube("habitat", "z", [0, 0, 0], L, r, r, label);
        tube("truss", "z", [0, 0, L / 2 + 0.3 * L], 0.6 * L, 0.3 * r, 0.05 * r, "antenna mast");
        size = 1.6 * L;
        edge = r;
        hubTop = L / 2;
      } else {
        // A node with modules radiating from it; a cross reaches its arms far out for berths and sensors.
        const arms = shape.form === "cross" ? 4 : Math.min(6, nMod);
        const per = Math.max(1, Math.ceil(nMod / arms));
        parts.push({ surface: "hull", form: "sphere", axis: "z", c: [0, 0, 0], r: 1.4 * r, label: "node" });
        for (let a = 0; a < arms; a++) {
          const dir: V3 = [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]][a]! as V3;
          const ax: Axis = dir[0] ? "x" : dir[1] ? "y" : "z";
          const L = per * modLen * (shape.form === "cross" ? 1.6 : 1);
          const c: V3 = [dir[0] * (1.4 * r + L / 2), dir[1] * (1.4 * r + L / 2), dir[2] * (1.4 * r + L / 2)];
          tube(shape.form === "cross" ? "truss" : "habitat", ax, c, L, shape.form === "cross" ? 0.35 * r : r, shape.form === "cross" ? 0.35 * r : r, a === 0 ? (shape.form === "cross" ? "arms" : label) : undefined);
          if (shape.form === "cross") tube("habitat", ax, [c[0] * 0.5, c[1] * 0.5, c[2] * 0.5], Math.min(L * 0.5, nMod * modLen / arms), r, r, a === 0 ? label : undefined);
          edge = Math.max(edge, 1.4 * r + L);
        }
        size = 2 * edge;
      }
      hubR = r;
      hubTop = hubTop || r;
      break;
    }
    case "can":
    case "ball":
    case "disc": {
      if (shape.form === "can") {
        const R = Math.cbrt(volume / (Math.PI * 3));
        const L = 3 * R;
        tube("habitat", "z", [0, 0, 0], L, R, R, `hull, ${lengthWords(2 * R)} across and ${lengthWords(L)} long`);
        hubR = 0.3 * R;
        hubTop = L / 2;
        size = L;
        edge = R;
      } else if (shape.form === "ball") {
        const R = Math.cbrt((3 * volume) / (4 * Math.PI)) * 1.15;
        parts.push({ surface: "habitat", form: "sphere", axis: "z", c: [0, 0, 0], r: R, label: `armoured sphere, ${lengthWords(2 * R)} across` });
        hubR = 0.2 * R;
        hubTop = R;
        size = 2 * R;
        edge = R;
      } else {
        const R = Math.cbrt(volume / (Math.PI * 0.2));
        const H = 0.2 * R;
        tube("habitat", "z", [0, 0, 0], H, R, R, `disc, ${lengthWords(2 * R)} across`);
        tube("hull", "z", [0, 0, H / 2 + 0.05 * R], 0.1 * R, 0.6 * R, 0.4 * R, "upper decks");
        tube("hull", "z", [0, 0, -H / 2 - 0.05 * R], 0.1 * R, 0.4 * R, 0.6 * R);
        hubR = 0.15 * R;
        hubTop = H / 2 + 0.1 * R;
        size = 2 * R;
        edge = R;
      }
      break;
    }
    case "dock_ring": {
      // A ring of berths that does not turn, round a core; people live in the core.
      const R0 = Math.cbrt((3 * volume) / (4 * Math.PI));
      parts.push({ surface: "habitat", form: "sphere", axis: "z", c: [0, 0, 0], r: R0, label: `core, ${lengthWords(2 * R0)} across` });
      const R = 3 * R0;
      parts.push({ surface: "truss", form: "torus", axis: "z", c: [0, 0, 0], R, r: 0.12 * R0, label: `dock ring, ${lengthWords(2 * R)} across (not spinning)` });
      for (const [x, y] of [[1, 0], [-1, 0], [0, 1], [0, -1]] as const) tube("truss", x ? "x" : "y", [x * (R0 + R) / 2, y * (R0 + R) / 2, 0], R - R0, 0.06 * R0, 0.06 * R0);
      hubR = 0.2 * R0;
      hubTop = R0;
      size = 2 * R;
      edge = R;
      break;
    }
    case "rock": {
      // A rock hollowed out: the rooms are a third of it, the rest is shielding the rock gives for free.
      const R = Math.cbrt((3 * volume) / (4 * Math.PI * 0.3));
      parts.push({ surface: "rock", form: "sphere", axis: "z", c: [0, 0, 0], r: R, lumpy: 0.18, label: `hollowed rock, ${lengthWords(2 * R)} across` });
      tube("hull", "x", [R * 0.95, 0, 0], 0.2 * R, 0.12 * R, 0.16 * R, "berth mouth");
      hubR = 0.12 * R;
      hubTop = R * 0.8;
      size = 2.2 * R;
      edge = R;
      break;
    }
    case "derelict": {
      // An old ship pressed into use: a long hull with its drive cold and pieces missing.
      const r = Math.max(4, Math.cbrt(volume / (Math.PI * 8)));
      const L = volume / (Math.PI * r * r);
      tube("habitat", "x", [0, 0, 0], L, r, r, `old hull, ${lengthWords(L)} long`);
      tube("hull", "x", [L / 2 + r, 0, 0], 2 * r, r, 0.2 * r, "bow");
      tube("truss", "x", [-L / 2 - 1.5 * r, 0, 0], 3 * r, 0.3 * r, 0.3 * r, "broken spine");
      tube("hull", "x", [-L / 2 - 3.5 * r, rnd() * r - r / 2, 0], 1.2 * r, 0.6 * r, 1.1 * r, "cold drive");
      for (let i = 0; i < 3; i++) if (rnd() < 0.6) box("hull", [(rnd() - 0.5) * L, (rnd() < 0.5 ? -1 : 1) * r * 1.3, 0], [0.8 * r, 0.6 * r, 0.6 * r], i === 0 ? "patched-on modules" : undefined);
      hubR = r;
      hubTop = r;
      size = L + 6 * r;
      edge = L / 2;
      notes.push("A ship serving as a station: its drive and tanks are dead weight, so it holds its place only by being parked.");
      break;
    }
    case "dome":
    case "crater": {
      const n = shape.form === "crater" ? 1 : Math.max(1, Math.min(shape.count ?? 1, Math.ceil(Math.log10(population + 1))));
      const v1 = volume / n;
      const R = Math.cbrt((3 * v1) / (2 * Math.PI));
      for (let i = 0; i < n; i++) {
        const t = (2 * Math.PI * i) / Math.max(1, n - 1);
        const at: V3 = i === 0 ? [0, 0, 0] : [Math.cos(t) * 2.6 * R, Math.sin(t) * 2.6 * R, 0];
        parts.push({ surface: shape.form === "crater" ? "glass" : "habitat", form: "sphere", axis: "z", c: at, r: R, hemi: true, ...(i === 0 ? { label: shape.form === "crater" ? `roof over a crater ${lengthWords(2 * R)} across` : `${n > 1 ? `${n} domes` : "dome"}, ${lengthWords(2 * R)} across` } : {}) });
        if (i > 0) tube("hull", Math.abs(Math.cos(t)) > Math.abs(Math.sin(t)) ? "x" : "y", [at[0] / 2, at[1] / 2, 0.1 * R], 0.6 * R, 0.08 * R, 0.08 * R, i === 1 ? "linking tunnels" : undefined);
      }
      if (shape.form === "crater") parts.push({ surface: "rock", form: "torus", axis: "z", c: [0, 0, 0], R: 1.05 * R, r: 0.15 * R, label: "crater rim" });
      size = n > 1 ? 7.2 * R : 2.3 * R;
      edge = size / 2;
      break;
    }
    case "tower":
    case "pyramid": {
      const k = shape.form === "tower" ? 6 : 0.7;
      // V = s² H for a tower, s² H / 3 for a pyramid; H = k s.
      const s = Math.cbrt((shape.form === "tower" ? 1 : 3) * volume / k);
      const H = k * s;
      if (shape.form === "tower") box("habitat", [0, 0, H / 2], [s, s, H], `tower, ${lengthWords(H)} tall`);
      else tube("habitat", "z", [0, 0, H / 2], H, s / Math.SQRT2, 0.05 * s, `pyramid, ${lengthWords(H)} tall and ${lengthWords(s)} across`);
      const g = (over.gravity_g ?? rec0.gravity_g ?? 1) * g0;
      // A column of one material stands to strength over density over g (halved for safety).
      const need = (g * H * rules.tower.safety) / (shape.form === "pyramid" ? 3 : 1);
      mega = strengthVerdict(need, limits, `A structure ${lengthWords(H)} tall at ${fmt(g / g0)} g`);
      size = Math.max(H, s);
      edge = s;
      break;
    }
    case "sprawl":
    case "buried": {
      // Low blocks over the ground at a town's density; buried, they are mounds of regolith over the rooms.
      const area = (population / rules.urban_density_per_km2.v) * 1e6;
      const side = Math.sqrt(area);
      const blocks = Math.min(64, Math.max(4, Math.round(Math.sqrt(population) / 4)));
      const per = Math.ceil(Math.sqrt(blocks));
      const cell = side / per;
      const h = shape.form === "buried" ? 0.25 * cell : Math.max(6, Math.min(cell * 0.6, (volume / (blocks * cell * cell * 0.5))));
      for (let i = 0; i < per * per; i++) {
        const x = ((i % per) - (per - 1) / 2) * cell;
        const y = (Math.floor(i / per) - (per - 1) / 2) * cell;
        const hh = h * (0.5 + rnd());
        if (shape.form === "buried") tube("rock", "x", [x, y, 0], 0.8 * cell, 0.3 * cell, 0.3 * cell, i === 0 ? "rooms under regolith mounds" : undefined);
        else box("habitat", [x, y, hh / 2], [0.7 * cell, 0.7 * cell, hh], i === 0 ? `${per * per} blocks over ${lengthWords(side)} square` : undefined);
      }
      size = side;
      edge = side / 2;
      break;
    }
    case "aerostat": {
      // A city hung under a gas envelope; the envelope is sized like the habitat, the lift is the atmosphere's.
      const R = Math.cbrt((3 * volume) / (4 * Math.PI));
      parts.push({ surface: "glass", form: "sphere", axis: "z", c: [0, 0, 1.2 * R], r: 1.6 * R, label: `lifting envelope, ${lengthWords(3.2 * R)} across` });
      tube("habitat", "z", [0, 0, -0.6 * R], 0.5 * R, 1.1 * R, 0.8 * R, "city decks");
      size = 3.2 * R;
      edge = 1.6 * R;
      notes.push("A floating city needs air denser than what it holds; the world's atmosphere decides whether it floats.");
      break;
    }
    case "platform": {
      const s = Math.cbrt(volume / 0.2);
      box("habitat", [0, 0, 0.4 * s], [s, s, 0.2 * s], `deck, ${lengthWords(s)} square`);
      for (const [x, y] of [[1, 1], [1, -1], [-1, 1], [-1, -1]] as const) tube("truss", "z", [x * 0.4 * s, y * 0.4 * s, 0], 0.6 * s, 0.06 * s, 0.06 * s, x === 1 && y === 1 ? "legs" : undefined);
      size = s;
      edge = s / 2;
      break;
    }
  }

  // Power and its heat: from the record, else so much a head; solar panels for it, and in space radiators to shed it.
  const power_w = rec0.power_w ?? population * rules.power.w_per_person;
  const S = constant("S_sun");
  const solar_m2 = power_w / (rules.power.efficiency * S);
  const T = rules.power.radiator_k;
  const radiator_m2 = power_w / (2 * 0.9 * constant("sigma_SB") * T ** 4);
  const big = Math.max(edge, 4);
  if (place === "space") {
    // Two wings of each, off the hub's ends: square arrays, so a vast one stays legible.
    const wing = (area: number) => {
      const side = Math.sqrt(area / 2);
      return [side, side] as const;
    };
    const [sSpan, sChord] = wing(solar_m2);
    const [rSpan, rChord] = wing(radiator_m2);
    const zTop = Math.max(hubTop, 2) + sChord / 2 + 0.1 * big;
    const zBot = -(Math.max(hubTop, 2) + rChord / 2 + 0.1 * big);
    for (const side of [1, -1]) {
      panel("solar", "y", [side * (hubR + sSpan / 2), 0, zTop], sSpan, sChord, side > 0 ? `solar panels, ${powerWords(power_w)}` : undefined);
      panel("radiators", "x", [0, side * (hubR + rSpan / 2), zBot], rSpan, rChord, side > 0 ? `radiators at ${Math.round(T)} K` : undefined);
    }
    parts.push({ surface: "truss", form: "tube", axis: "z", c: [0, 0, (zTop + zBot) / 2], len: zTop - zBot, r0: Math.max(0.4, 0.3 * hubR), r1: Math.max(0.4, 0.3 * hubR), array: true });
    size = Math.max(size, 2 * (hubR + sSpan), 2 * (hubR + rSpan), zTop - zBot);
  } else if (place === "ground") {
    const side = Math.sqrt(solar_m2);
    panel("solar", "z", [big + 0.2 * big + side / 2, 0, 0.5], side, side, `solar field, ${powerWords(power_w)}`);
    size = Math.max(size, big + 0.2 * big + side);
    notes.push("On the ground the waste heat goes into the air or the rock, so no radiators are drawn.");
  }

  // Docks: berths by people and trade, each class up half as many, the biggest taking ships up to half its size.
  const port = (rec0.tags ?? []).some((t) => /port|shipyard|yard|dock|refined_fuel/.test(t)) ? rules.docks.port_bonus : 1;
  const total = over.berths ?? Math.max(1, Math.ceil((population / rules.docks.people_per_berth) * port));
  const fits = rules.docks.classes.filter((c, i) => i === 0 || c.max_ship_m <= size * rules.docks.size_share);
  const weights = fits.map((_, i) => 0.5 ** i);
  const wsum = weights.reduce((a, b) => a + b, 0);
  const docks: DockClass[] = fits.map((c, i) => ({ ...c, count: Math.max(i === 0 ? 1 : 0, Math.round((total * weights[i]!) / wsum)) })).filter((d) => d.count > 0);
  // Draw the berths: collars at the hub's ends (space) or pads beside it (ground), each sized to its class.
  let placed = 0;
  for (const d of docks) {
    const shown = Math.min(d.count, 12);
    const rc = Math.max(0.6, 0.03 * d.max_ship_m);
    for (let i = 0; i < shown; i++, placed++) {
      const t = (2 * Math.PI * i) / shown;
      // One call-out for all the berths; the words below the plans list them by class.
      const all = docks.reduce((n, x) => n + x.count, 0);
      const label = i === 0 && d === docks[0] ? (place === "space" ? `${all} berth${all === 1 ? "" : "s"}` : `${all} landing pad${all === 1 ? "" : "s"}`) : undefined;
      if (place === "space") {
        const end = placed % 2 ? -1 : 1;
        const ring = hubR + rc * 2;
        tube("dock", "z", [Math.cos(t) * (shown > 1 ? ring : 0), Math.sin(t) * (shown > 1 ? ring : 0), end * (Math.max(hubTop, 1) + rc)], 2 * rc, rc, rc, label);
      } else {
        const pr = Math.max(5, 0.6 * d.max_ship_m);
        const dist = big + pr * (1.5 + 2.2 * Math.floor(i / 6));
        const a = t + Math.PI;
        tube("dock", "z", [Math.cos(a) * dist, Math.sin(a) * dist, 0.25], 0.5, pr, pr, label);
      }
    }
  }
  if (docks.some((d) => d.count > 12)) notes.push("Up to 12 berths of each class are drawn; the words above give them all.");

  // Bolt-ons: one per listed module, then a few more as the place grows (workshops, tanks, antennas), round the edge.
  const mods = rec0.modules ?? [];
  const extra = Math.max(0, Math.floor(Math.log10(population + 1)) - 1);
  const boltV = Math.max(30, volume * rules.bolt_on_share);
  const b = Math.cbrt(boltV);
  const labelOf = (i: number) => (i < mods.length ? mods[i]! : undefined);
  for (let i = 0; i < mods.length + extra; i++) {
    const t = (2 * Math.PI * (i + rnd() * 0.4)) / (mods.length + extra) + rnd() * 0.3;
    const r = edge + b * (place === "space" ? 0.8 : 1.2);
    const z = place === "space" ? (rnd() - 0.5) * Math.max(hubTop, b) : b / 2;
    const kind = rnd();
    const c: V3 = [Math.cos(t) * r, Math.sin(t) * r, z];
    if (kind < 0.5) box("hull", c, [b, b * (0.6 + rnd() * 0.6), b * (0.5 + rnd() * 0.5)], labelOf(i));
    else tube("hull", "z", c, b * (0.8 + rnd()), 0.45 * b, 0.45 * b, labelOf(i));
  }
  if (mods.length + extra > 0) size = Math.max(size, 2 * (edge + 1.5 * b));

  // Mass, and whether its own gravity makes it a body: escape speed from its edge over the rule's.
  const mass_kg = rec0.mass_kg ?? volume * rules.mass_kg_per_m3.v;
  const reach = Math.max(edge, 1);
  const escape = Math.sqrt((2 * constant("G") * mass_kg) / reach);
  if (escape >= rules.self_gravity.escape_m_s) {
    const why = `Its own gravity holds a walker: ${fmt(mass_kg)} kg within ${lengthWords(reach)} gives an escape speed of ${escape.toFixed(1)} m/s`;
    mega = mega ? { ...(mega as Megastructure), why: `${(mega as Megastructure).why}. ${why}` } : { why, needs: null, fictional: false };
  }
  if (rec0.mass_kg !== undefined) {
    const density = rec0.mass_kg / Math.max(volume, 1);
    if (density > 20 * rules.mass_kg_per_m3.v) notes.push(`Its listed mass (${fmt(rec0.mass_kg)} kg) is heavy for its people: ${fmt(density)} kg for each cubic metre they live in. It may hold industry, armour or rock the builder does not draw.`);
  }
  if (spin && (spin as { radius_m: number }).radius_m === rMin && shape.spin) notes.push(`Few people for a spun hull: the radius is set by the spin rate a person can bear (${fmt(limits.spin_rpm)} turns a minute), not by the room they need.`);
  return { name: rec0.name, shape: shapeId, seed: recipe.seed, place, parts, size_m: size, population, living: living.id, volume_m3: volume, mass_kg, power_w, spin, docks, mega, by_hand, notes };
}

/** The station in a few sentences. */
export function narrateStation(l: StationLayout, shape: StationShape): string {
  const s: string[] = [];
  s.push(`${shape.name} for ${fmt(l.population)} ${l.population === 1 ? "person" : "people"}, ${lengthWords(l.size_m)} across.`);
  if (l.spin) s.push(`It turns ${fmt(l.spin.rpm)} times a minute for ${fmt(l.spin.g)} g at ${lengthWords(l.spin.radius_m)} from the axis.`);
  s.push(`${fmt(l.volume_m3)} cubic metres to live in (${l.living === "home" ? "a home" : "a working post"}), ${powerWords(l.power_w)} of power.`);
  if (l.docks.length) s.push(`Berths: ${l.docks.map((d) => `${d.count} for ${d.name}`).join(", ")}.`);
  if (l.mega) s.push(`A megastructure: ${l.mega.why}.${l.mega.needs ? ` Its hull would need ${l.mega.needs}.` : ""}`);
  return s.join(" ");
}

/** Read the station builder's inputs from the two packs' `other` data (base-ships and base-tech); null when either
 * lacks what the builder needs. */
export function stationPackData(ships: Record<string, unknown>, tech?: Record<string, unknown>): { rules: StationRules; shapes: Record<string, StationShape>; limits: StationLimits } | null {
  const rules = ships.station_design as StationRules | undefined;
  const shapes = ships.station_shapes as Record<string, StationShape> | undefined;
  if (!rules || !shapes) return null;
  const figures = ((tech?.tech as { figures?: { id: string; marks: StrengthMark[] }[] } | undefined)?.figures ?? []);
  const rpm = figures.find((f) => f.id === "spin_rpm")?.marks[0]?.v;
  const strength = figures.find((f) => f.id === "strength_for_weight")?.marks ?? [];
  if (rpm === undefined || !strength.length) return null;
  return { rules, shapes, limits: { spin_rpm: rpm, strength } };
}

export interface StationPath {
  surface: StationSurface;
  d: string;
}
export interface StationCallout {
  text: string;
  anchor: [number, number];
  label: [number, number];
  above: boolean;
}
export interface StationView {
  paths: StationPath[];
  callouts: StationCallout[];
}

/** Top view (looking down: x across, y up the page) and side view (x across, z up), at one scale fitting both in
 * frames of `size` pixels with room above and below for call-outs. `core` draws the station without its solar and
 * radiator arrays (`clipped` says so), which for a big port run to kilometres. */
export function stationViews(l: StationLayout, size: [number, number], fit: "all" | "core" = "all"): { top: StationView; side: StationView; scale: number; clipped: boolean } {
  const [W, H] = size;
  const band = 0.58;
  // Each part's extent along x, y, z.
  const ext = (p: Part): [V3, V3] => {
    const half: V3 = [0, 0, 0];
    const set = (ax: Axis, along: number, across: number) => {
      for (const [k, a] of (["x", "y", "z"] as const).entries()) half[k] = a === ax ? along : across;
    };
    if (p.form === "tube") set(p.axis, (p.len ?? 0) / 2, Math.max(p.r0 ?? 0, p.r1 ?? 0));
    else if (p.form === "torus") set(p.axis, p.r ?? 0, (p.R ?? 0) + (p.r ?? 0));
    else if (p.form === "sphere") {
      const r = (p.r ?? 0) * (1 + (p.lumpy ?? 0));
      return [[p.c[0] - r, p.c[1] - r, p.hemi ? p.c[2] : p.c[2] - r], [p.c[0] + r, p.c[1] + r, p.c[2] + r]];
    } else if (p.form === "box") {
      const s = p.size ?? [0, 0, 0];
      set("z", s[2] / 2, 0);
      (half[0] = s[0] / 2), (half[1] = s[1] / 2);
    } else {
      const s = p.size ?? [0, 0, 0];
      const [u, v] = p.axis === "x" ? [1, 2] : p.axis === "y" ? [2, 0] : [0, 1];
      half[u] = s[0] / 2;
      half[v] = s[1] / 2;
    }
    return [[p.c[0] - half[0], p.c[1] - half[1], p.c[2] - half[2]], [p.c[0] + half[0], p.c[1] + half[1], p.c[2] + half[2]]];
  };
  const shown = l.parts.filter((p) => fit === "all" || !(p.array || p.surface === "solar" || p.surface === "radiators"));
  const lo: V3 = [Infinity, Infinity, Infinity];
  const hi: V3 = [-Infinity, -Infinity, -Infinity];
  for (const p of shown) {
    const [a, b] = ext(p);
    for (let k = 0; k < 3; k++) (lo[k] = Math.min(lo[k]!, a[k]!)), (hi[k] = Math.max(hi[k]!, b[k]!));
  }
  const spanX = Math.max(1, hi[0] - lo[0]);
  const spanUp = Math.max(1, hi[1] - lo[1], hi[2] - lo[2]);
  const scale = Math.min((W * 0.92) / spanX, (H * band) / spanUp);
  const cx = W / 2 - ((lo[0] + hi[0]) / 2) * scale;
  const clipped = fit === "core" && shown.length < l.parts.length;
  const f = (v: number) => v.toFixed(1);
  const view = (up: 1 | 2): StationView => {
    const mid = (lo[up] + hi[up]) / 2;
    const X = (x: number) => cx + x * scale;
    const Y = (v: number) => H / 2 - (v - mid) * scale;
    const rect = (x0: number, y0: number, x1: number, y1: number) => `M${f(X(x0))} ${f(Y(y0))} L${f(X(x1))} ${f(Y(y0))} L${f(X(x1))} ${f(Y(y1))} L${f(X(x0))} ${f(Y(y1))} Z`;
    const circle = (x: number, y: number, r: number) => {
      const R = Math.max(0.4, r * scale);
      return `M${f(X(x) - R)} ${f(Y(y))} a${f(R)} ${f(R)} 0 1 0 ${f(2 * R)} 0 a${f(R)} ${f(R)} 0 1 0 ${f(-2 * R)} 0 Z`;
    };
    const paths: StationPath[] = [];
    const anchors: { at: [number, number]; text: string }[] = [];
    const looking = up === 1 ? "z" : "y";
    // Closed in, the power arrays are left out of the plans rather than drawn across the station.
    for (const p of shown) {
      const u = p.c[up]!;
      let d: string;
      if (p.form === "sphere") {
        const r = (p.r ?? 0) * (1 + 0.5 * (p.lumpy ?? 0));
        d = p.hemi && up === 2 ? `M${f(X(p.c[0] - r))} ${f(Y(u))} A${f(r * scale)} ${f(r * scale)} 0 0 1 ${f(X(p.c[0] + r))} ${f(Y(u))} Z` : circle(p.c[0], u, r);
      } else if (p.form === "torus") {
        const R = p.R ?? 0;
        const r = p.r ?? 0;
        // Seen along its axis a ring is an annulus; side on, a band with round ends.
        d = p.axis === looking ? `${circle(p.c[0], u, R + r)} ${circle(p.c[0], u, Math.max(0, R - r))}` : rect(p.c[0] - R - r, u - r, p.c[0] + R + r, u + r);
      } else if (p.form === "tube") {
        const L = (p.len ?? 0) / 2;
        const r0 = p.r0 ?? 0;
        const r1 = p.r1 ?? r0;
        if (p.axis === looking) d = circle(p.c[0], u, Math.max(r0, r1));
        else if (p.axis === "x") d = `M${f(X(p.c[0] - L))} ${f(Y(u + r0))} L${f(X(p.c[0] + L))} ${f(Y(u + r1))} L${f(X(p.c[0] + L))} ${f(Y(u - r1))} L${f(X(p.c[0] - L))} ${f(Y(u - r0))} Z`;
        else d = `M${f(X(p.c[0] - r0))} ${f(Y(u - L))} L${f(X(p.c[0] + r0))} ${f(Y(u - L))} L${f(X(p.c[0] + r1))} ${f(Y(u + L))} L${f(X(p.c[0] - r1))} ${f(Y(u + L))} Z`;
      } else {
        const [a, b] = ext(p);
        const t = 0.6 / scale;
        d = rect(a[0], Math.min(a[up]!, b[up]! - t), Math.max(b[0], a[0] + t), Math.max(b[up]!, a[up]! + t));
      }
      paths.push({ surface: p.surface, d });
      if (p.label) anchors.push({ at: [Math.min(W - 4, Math.max(4, X(p.c[0]))), Math.min(H / 2 + (H * band) / 2, Math.max(H / 2 - (H * band) / 2, Y(u)))], text: p.label });
    }
    const rank = (s: StationSurface) => (s === "solar" || s === "radiators" ? 0 : s === "truss" ? 1 : s === "dock" ? 3 : 2);
    paths.sort((a, b) => rank(a.surface) - rank(b.surface));
    // Call-outs: along the drawing, alternating above and below, pushed apart so they never overlap.
    const list = anchors.sort((a, b) => a.at[0] - b.at[0]);
    const line = (row: typeof list, above: boolean, drop: number): StationCallout[] => {
      const widths = row.map((a) => a.text.length * 5.6 + 10);
      const xs = row.map((a) => a.at[0]);
      for (let i = 0; i < xs.length; i++) xs[i] = Math.max(xs[i]!, widths[i]! / 2, i ? xs[i - 1]! + (widths[i - 1]! + widths[i]!) / 2 : 0);
      for (let i = xs.length - 1; i >= 0; i--) xs[i] = Math.min(xs[i]!, W - widths[i]! / 2, i < xs.length - 1 ? xs[i + 1]! - (widths[i + 1]! + widths[i]!) / 2 : W);
      return row.map((a, i) => ({ text: a.text, anchor: a.at, label: [xs[i]!, above ? H * 0.08 + drop : H * 0.94 - drop], above }));
    };
    const place = (all: typeof list, above: boolean): StationCallout[] => {
      const width = all.reduce((a, x) => a + x.text.length * 5.6 + 10, 0);
      if (width > W * 0.95 && all.length > 1) return [0, 1].flatMap((k) => line(all.filter((_, i) => i % 2 === k), above, k * 13));
      return line(all, above, 0);
    };
    return { paths, callouts: [...place(list.filter((_, i) => i % 2 === 0), true), ...place(list.filter((_, i) => i % 2 === 1), false)] };
  };
  return { top: view(1), side: view(2), scale, clipped };
}
