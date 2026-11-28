// Tank walls in the builder (CM14c part 3): a tank's wall comes from the thin-wall flow on the tank's own cited
// pressure and factors and the data sheet's alloy row; the build then counts it, says where it came from, and stops
// saying the tank needs a mass. An engine with a cited dry mass counts it too; a part with neither still needs one.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { buildTotals, placeBuild, type BuildRecipe } from "../src/ship-build.ts";
import { readTankWalls, wallKey, type WallReader } from "../src/ship-walls.ts";
import type { DeriveRef, PartsPalette } from "../../ui-contract/src/index.ts";

const PARTS = JSON.parse(readFileSync(new URL("../../../packs/base-parts/pack.json", import.meta.url), "utf8")).provides.other;
const PALETTE = PARTS.palette as PartsPalette;
const SHEET = JSON.parse(readFileSync(new URL("../../../packs/base-reference/pack.json", import.meta.url), "utf8")).provides.reference;

/** The sheet's rows as the kernel's read_matter gives them, and the flow's one block (m = phi p V rho / s). */
const asks: DeriveRef[] = [];
const reader: WallReader = {
  async readMatter({ table, id }) {
    const row = (SHEET[table!] as { id: string }[]).find((r) => r.id === id);
    return row ? { ok: true, value: { id, table, row, packs: [], sources: [], used_by: [] } as never } : { ok: false, code: "unknown", message: "no row" };
  },
  async derive(ref) {
    asks.push(ref);
    const v = (n: string) => (ref.inputs[n] as { value: number }).value;
    return { ok: true, values: { "ship.wall_mass": { value: (v("vessel_shape") * v("tank_pressure") * v("stowed_volume") * v("wall_density")) / v("wall_stress") } } };
  },
};

test("a 4 m tank's wall comes from its cited pressure and factors and the sheet's alloy, through the flow", async () => {
  const r: BuildRecipe = { format: "sse.build/1", name: "tank", parts: [{ key: "t", part: "tank-4m" }] };
  const { placed } = placeBuild(r, PALETTE);
  assert.ok(wallKey(placed[0]!), "the palette's tank carries what its wall needs");
  const walls = await readTankWalls(reader, placed, {});
  const w = walls[wallKey(placed[0]!)!];
  assert.ok(w, "the wall is read");
  const ask = asks.at(-1)!;
  assert.equal(ask.flow, "sse.base.ship-tank-wall");
  // 40 psia, Al 2219-T87 at the lower of 476 MPa / 1.4 and 393 MPa / 1.25, 2840 kg/m3, 100.53 m3, a long cylinder.
  assert.ok(Math.abs((ask.inputs.wall_stress as { value: number }).value - 314.4e6) < 1e5, "allowed stress is yield over 1.25");
  assert.ok(Math.abs(w.kg - 500.9) < 1.5, `wall ${w.kg} kg`);
  const t = buildTotals(r, placed, {}, walls);
  assert.deepEqual(t.unmassed, [], "the tank no longer needs a mass");
  assert.ok(Math.abs(t.dry_t - w.kg / 1e3) < 1e-9);
  assert.match(t.mass_from.t!, /^501 kg, wall from the thin-wall pressure block/);
  assert.equal(Object.keys(await readTankWalls(reader, placed, walls)).length, 0, "an unchanged tank is never asked again");
});

test("an engine with a cited dry mass counts it; a part with none still needs one", () => {
  const r: BuildRecipe = { format: "sse.build/1", name: "pair", parts: [{ key: "e", part: "engine-j-2" }, { key: "l", part: "landing-leg", on: "e", place: "hull", along: 0.5, clock_deg: 90 }] };
  const { placed } = placeBuild(r, PALETTE);
  const t = buildTotals(r, placed, {}, {}, PALETTE.mass_by_kind);
  assert.ok(Math.abs(t.dry_t - 1.438) < 1e-9, `J-2 ${t.dry_t} t`);
  assert.match(t.mass_from.e!, /^1\.4 t, dry mass from wiki-engines/);
  assert.deepEqual(t.unmassed, [placed[1]!.part.name], "legs have no kind row, so they still need a mass");
});

test("a generic part with no mass of its own weighs as its kind's cited row says, by its own figure", () => {
  const r: BuildRecipe = { format: "sse.build/1", name: "can", parts: [{ key: "h", part: "habitat-4m" }, { key: "w", part: "solar-panel", on: "h", place: "hull", along: 0.5, clock_deg: 90 }] };
  const { placed } = placeBuild(r, PALETTE);
  const t = buildTotals(r, placed, {}, {}, PALETTE.mass_by_kind);
  // Destiny: 14,515 kg in pi 2.15^2 8.5 m3 is 117.6 kg/m3, times the 4 m can's 100.53 m3; Dawn's wing: 63 kg over
  // 19.09 m2, times the panel's 20 m2.
  assert.ok(Math.abs(t.dry_t - (11.825 + 0.066)) < 0.01, `dry ${t.dry_t} t`);
  assert.deepEqual(t.unmassed, []);
  assert.match(t.mass_from.h!, /^11\.8 t, by its kind: the ISS Destiny laboratory/);
  assert.match(t.mass_from.w!, /^66 kg, by its kind: a Dawn solar wing/);
});
