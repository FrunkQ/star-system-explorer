// The ship builder's model (CM14a): parts clipped at their points lay out end to end and on the hull; the plans and the
// model draw a build like any ship; the totals are the parts' own figures summed, with what they cannot count named;
// stretching a tank grows its room and its load by its own law; a plan's budgets count down as parts go on and go
// past zero honestly; a catalogue ship opens as parts; the palette groups by era and offers by exact year.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { designShip, shipViews, type DesignRules, type DesignShip, type ShipStyle } from "../src/ship-design.ts";
import { budgets, buildLayout, freePoints, fromLayout, placeBuild, type BuildRecipe } from "../src/ship-build.ts";
import { shipMesh } from "../src/ship-mesh.ts";
import { type Propellants } from "../src/ship.ts";
import { eraOf, paletteByEra, type PartsPalette } from "../../ui-contract/src/index.ts";

const json = (p: string) => JSON.parse(readFileSync(new URL(p, import.meta.url), "utf8"));
const PALETTE = json("../../../packs/base-parts/pack.json").provides.other.palette as PartsPalette;
const SHIPS = json("../../../packs/base-nav/pack.json").provides.other.nav.ships as Record<string, DesignShip>;
const PACK = json("../../../packs/base-ships/pack.json").provides.other;
const RULES = PACK.ship_design as DesignRules;
const STYLES = PACK.ship_styles as Record<string, ShipStyle>;
// Test fixture only: NIST saturated liquid hydrogen at boiling (as ship-design.test.ts).
const STORE: Propellants = { hydrogen: [{ name: "liquid hydrogen", share: 1, kg_m3: 70.85, src: "nist-webbook" }] };

const stack: BuildRecipe = {
  format: "sse.build/1",
  name: "Test tug",
  fuels: ["hydrogen"],
  crew: 2,
  parts: [
    { key: "hab", part: "habitat-4m" },
    { key: "tank", part: "tank-4m", on: "hab", place: "behind" },
    { key: "engine", part: "engine-rl10a-3", on: "tank", place: "behind" },
    { key: "rad", part: "radiator-panel", on: "tank", place: "hull", along: 0.5, clock_deg: 90 },
    { key: "dish", part: "dish", on: "hab", place: "hull", along: 0.5, clock_deg: 0 },
  ],
};

test("parts clip end to end on the axis and onto the hull, the stern at 0", () => {
  const { placed, notes } = placeBuild(stack, PALETTE);
  assert.deepEqual(notes, []);
  const at = Object.fromEntries(placed.map((q) => [q.key, q]));
  assert.equal(Math.min(...placed.map((q) => q.x0)), 0);
  // The engine's top meets the tank's stern; the tank's nose meets the habitat's stern.
  assert.ok(Math.abs(at.engine!.x0 + at.engine!.part.shape.length_m - at.tank!.x0) < 1e-9);
  assert.ok(Math.abs(at.tank!.x0 + at.tank!.part.shape.length_m - at.hab!.x0) < 1e-9);
  // Clock 90 is towards +y: the radiator sits on the tank's surface there, the dish on top of the habitat.
  assert.ok(Math.abs(at.rad!.y - 2) < 1e-9 && Math.abs(at.rad!.z) < 1e-9);
  assert.ok(at.dish!.z > 1.99);
});

test("a build draws in the plans and the 3D model like any ship", () => {
  const l = buildLayout(stack, PALETTE, STORE, RULES);
  const v = shipViews(l, [600, 300]);
  assert.ok(v.side.paths.length >= 4, "parts drawn in the side view");
  const m = shipMesh(l, { segments: 12 });
  assert.ok(Object.keys(m.surfaces).length >= 3, "hull, tanks and drive surfaces");
  assert.ok(l.length_m > 12 && l.length_m < 20, `${l.length_m}`);
});

test("totals are the parts' own figures summed, and what they cannot count is named", () => {
  const l = buildLayout(stack, PALETTE, STORE, RULES);
  const t = l.totals;
  const tankRoom = PALETTE.parts.find((p) => p.id === "tank-4m")!.figures.volume_m3!.v;
  assert.equal(t.tank_m3.hydrogen, tankRoom);
  // Load is room times the sheet's density, nothing more.
  assert.ok(Math.abs(t.propellant_t - (tankRoom * 70.85) / 1e3) < 1e-9);
  const rl10 = PALETTE.parts.find((p) => p.id === "engine-rl10a-3")!.figures;
  assert.equal(t.thrust_n, rl10.thrust_n!.v);
  assert.equal(t.exhaust_km_s, rl10.exhaust_km_s!.v);
  assert.ok(t.missing.some((m) => m.includes("No mass given")), t.missing.join(" | "));
  assert.equal(l.record.exhaust_km_s, rl10.exhaust_km_s!.v);
  assert.ok(l.record.wet_t > l.record.dry_t);
});

test("stretching a tank grows its room and its load by its own law", () => {
  const long: BuildRecipe = { ...stack, parts: stack.parts.map((b) => (b.key === "tank" ? { ...b, length_m: 16 } : b)) };
  const a = buildLayout(stack, PALETTE, STORE, RULES).totals;
  const b = buildLayout(long, PALETTE, STORE, RULES).totals;
  assert.ok(Math.abs(b.tank_m3.hydrogen! / a.tank_m3.hydrogen! - 2) < 1e-9);
  assert.ok(b.length_m > a.length_m + 7.9);
});

test("a plan's budgets count down as parts go on, and go past zero honestly", () => {
  const plan: DesignShip = { ...SHIPS["nuclear-thermal"]!, name: "Plan", crew: 2 };
  const empty = buildLayout({ ...stack, parts: stack.parts.slice(0, 1) }, PALETTE, STORE, RULES).totals;
  const some = buildLayout(stack, PALETTE, STORE, RULES).totals;
  const b0 = budgets(plan, empty, STORE, RULES);
  const b1 = budgets(plan, some, STORE, RULES);
  const tank0 = b0.find((b) => b.id === "tanks")!;
  const tank1 = b1.find((b) => b.id === "tanks")!;
  assert.equal(tank0.left, tank0.need);
  assert.ok(tank1.left < tank0.left);
  const big: BuildRecipe = { ...stack, parts: stack.parts.map((b) => (b.key === "tank" ? { ...b, length_m: 4000 } : b)) };
  const over = budgets(plan, buildLayout(big, PALETTE, STORE, RULES).totals, STORE, RULES).find((b) => b.id === "tanks")!;
  assert.ok(over.left < 0, "overshoot shown, not clipped");
});

test("free points offer the open ends and the hull, not what is taken", () => {
  const { placed } = placeBuild(stack, PALETTE);
  const free = freePoints(stack, placed);
  assert.ok(free.some((f) => f.key === "hab" && f.place === "front"));
  assert.ok(!free.some((f) => f.key === "hab" && f.place === "behind"), "the habitat's stern holds the tank");
  assert.ok(!free.some((f) => f.key === "tank" && f.place === "behind"), "the tank's stern holds the engine");
  assert.ok(!free.some((f) => f.key === "tank" && f.place === "hull" && f.clock_deg === 90), "the radiator is there");
  assert.ok(free.some((f) => f.key === "tank" && f.place === "hull" && f.clock_deg === 270));
});

test("a catalogue ship laid out by the old generator opens as parts, and redraws about the same size", () => {
  const ship: DesignShip = { ...SHIPS["nuclear-thermal"]!, crew: 4 };
  const old = designShip(ship, STORE, RULES, "spine", STYLES.spine!, 1);
  const { recipe, notes } = fromLayout(old, ship, PALETTE);
  assert.ok(recipe.parts.length >= 3, JSON.stringify(recipe.parts));
  assert.ok(recipe.parts.some((b) => b.part === "tech-nuclear-thermal"), "its drive is the tech part for its fuel");
  assert.ok(recipe.parts.some((b) => b.part.startsWith("tank-")));
  const l = buildLayout(recipe, PALETTE, STORE, RULES);
  assert.ok(Math.abs(l.length_m - old.length_m) / old.length_m < 0.35, `${l.length_m} vs ${old.length_m}`);
  assert.equal(l.totals.tank_m3.hydrogen !== undefined, true);
  void notes;
});

test("the palette groups by era (any length, future eras too) and offers by exact first-flight year", () => {
  const eras = PALETTE.eras!;
  assert.ok(eras.some((e) => e.future) && eras.some((e) => !e.future));
  const lengths = new Set(eras.filter((e) => e.from !== undefined && e.to !== undefined).map((e) => e.to! - e.from!));
  assert.ok(lengths.size > 1, "eras vary in length");
  const saturn = PALETTE.parts.find((p) => p.id === "stage-saturn-s-ic")!;
  assert.equal(eraOf(saturn, eras)?.id, "space-race");
  const in1965 = paletteByEra(PALETTE, 1965).flatMap((g) => g.parts.map((p) => p.id));
  assert.ok(!in1965.includes("stage-saturn-s-ic"), "the picker reads the exact year: not flown by 1965");
  assert.ok(paletteByEra(PALETTE, 1967).some((g) => g.era?.id === "space-race" && g.parts.some((p) => p.id === "stage-saturn-s-ic")));
  assert.ok(!in1965.includes("tech-fusion-torch"));
  assert.ok(paletteByEra(PALETTE, 2100).some((g) => g.era?.id === "fusion" && g.parts.some((p) => p.id === "tech-fusion-torch")));
  assert.equal(paletteByEra(PALETTE, 1965)[0]!.label, "Shapes");
});
