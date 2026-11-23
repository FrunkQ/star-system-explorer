// Example packs 4.1: the Traveller pack's standard ships (twelve starships, six small craft). Each is a construct
// template and a ship the voyage planner and the ship designer read, so its hull is our own, drawn from its parts.
// Checks: the list is whole and each template points at its ship; the designer draws every one in every style, with a
// hold of exactly the ship's cargo tonnage in displacement tons and a jump tank of a tenth of the hull per parsec; the
// pack carries Mongoose's notice word for word and the Packs screen lists it.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { packShips } from "../../packages/ui-contract/src/route.ts";
import { packSummary } from "../../packages/ui-contract/src/packs-edit.ts";
import type { PackDoc } from "../../packages/ui-contract/src/physics.ts";
import { designShip, needsOf, type DesignRules, type DesignShip, type ShipStyle } from "../../packages/renderer/src/ship-design.ts";
import type { Propellants } from "../../packages/renderer/src/ship.ts";

const json = (p: string) => JSON.parse(readFileSync(new URL(p, import.meta.url), "utf8"));
const TRAVELLER = json("../../packs/traveller/pack.json");
const SHIPS_PACK = json("../../packs/base-ships/pack.json").provides.other;
const RULES = SHIPS_PACK.ship_design as DesignRules;
const STYLES = SHIPS_PACK.ship_styles as Record<string, ShipStyle>;
const SHEET = json("../../packs/base-reference/pack.json").provides.reference.substances as { id: string; phases: { density_liquid?: { v: number; src: string } } }[];
const liquid = (id: string, name: string) => {
  const d = SHEET.find((s) => s.id === id)!.phases.density_liquid!;
  return [{ name, share: 1, kg_m3: d.v, src: d.src }];
};
const STORE: Propellants = { hydrogen: liquid("h2", "liquid hydrogen"), deuterium: liquid("d2", "liquid deuterium") };
const DTON_M3 = (TRAVELLER.provides.units as { id: string; scale: number }[]).find((u) => u.id === "dton")!.scale;
const TEMPLATES = TRAVELLER.provides.other.templates.templates as { id: string; tags: string[]; facts: Record<string, unknown> }[];
const near = (a: number, b: number, what: string) => assert.ok(Math.abs(a - b) <= 1e-6 * Math.max(1, Math.abs(b)), `${what}: ${a} against ${b}`);

test("the standard list is whole, and each template points at a ship the planner flies", () => {
  const ships = packShips([TRAVELLER]);
  assert.equal(ships.length, 18);
  const starships = TEMPLATES.filter((t) => t.tags.includes("starship"));
  const craft = TEMPLATES.filter((t) => t.tags.includes("small_craft") && t.facts["construct.nav_ship"]);
  assert.equal(starships.length, 12);
  assert.equal(craft.length, 6);
  for (const t of [...starships, ...craft]) {
    const s = ships.find((x) => x.id === t.facts["construct.nav_ship"]);
    assert.ok(s, t.id);
    assert.equal(s.pack, "sse.traveller");
    assert.equal(s.ship.accel_g, t.facts["traveller.thrust_g"], t.id);
    const jump = (s.ship as { jump?: { rating_pc: number } }).jump;
    assert.equal(jump?.rating_pc ?? 0, t.facts["traveller.jump"], t.id);
  }
  // The roles the role programs play (voyages design): every one has a ship.
  for (const role of ["explorer", "courier", "miner", "trader", "patrol", "pirate"]) assert.ok(starships.some((t) => t.tags.includes(role)), role);
  // A higher pack's ship with the same id replaces it.
  const over = { id: "mine", provides: { other: { nav_ships: { ships: { "traveller.trader": { ...ships[1]!.ship, name: "Our trader" } } } } } };
  const both = packShips([TRAVELLER, over]);
  assert.equal(both.length, 18);
  assert.equal(both.find((x) => x.id === "traveller.trader")!.ship.name, "Our trader");
});

test("the designer draws every standard ship from its parts, in every style", () => {
  for (const { id, ship } of packShips([TRAVELLER])) {
    const t = TEMPLATES.find((x) => x.facts["construct.nav_ship"] === id)!;
    const hull = t.facts["traveller.hull_dt"] as number;
    const jump = t.facts["traveller.jump"] as number;
    const s = ship as DesignShip & { jump?: { tank_t: number } };
    const n = needsOf(s, STORE, RULES);
    // The hold is the ship's cargo in displacement tons, exactly.
    near(n.cargo_m3, (t.facts["traveller.cargo_dt"] as number) * DTON_M3, `${id} hold`);
    // The jump tank: a tenth of the hull per parsec, in liquid hydrogen.
    const jt = n.tanks.filter((k) => k.name.startsWith("jump fuel")).reduce((a, k) => a + k.volume_m3, 0);
    near(jt, 0.1 * hull * jump * DTON_M3, `${id} jump tank`);
    for (const [styleId, style] of Object.entries(STYLES)) {
      const l = designShip(s, STORE, RULES, styleId, style, 1);
      assert.ok(l.length_m > 0 && Number.isFinite(l.length_m), `${id} in ${styleId}`);
      assert.ok(l.pieces.length > 0, `${id} in ${styleId}`);
    }
  }
});

test("the pack carries Mongoose's notice word for word, and the Packs screen lists it", () => {
  const n = packSummary(TRAVELLER as PackDoc).notice;
  assert.ok(n);
  assert.ok(n.text.startsWith("The Traveller, 2300AD, Twilight: 2000 and Dark Conspiracy games in all forms are owned by Mongoose Publishing."));
  assert.ok(n.text.includes("Traveller is a registered trademark of Mongoose Publishing."));
  assert.ok(!JSON.stringify(TRAVELLER).includes("Far Future Enterprises"));
  assert.ok(!packSummary(TRAVELLER as PackDoc).other.includes("notice"));
});
