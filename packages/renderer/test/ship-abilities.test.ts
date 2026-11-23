// V3 gap item 14, stage 4: abilities shape the ship. Checks, against the kernel's own ship_at_world: a shield sized for
// a pass keeps its heating at the margin of its limit (q ∝ 1/√r_n), wings sized for a runway hold the stall at the
// margin (S v² fixed), a drive that can hover carries no wings, a scoop's intake is the propellant over density,
// speed and time, and the fitted parts appear in the plans and the model.
import { test, before, after } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { designShip, type DesignRules, type DesignShip, type ShipStyle } from "../src/ship-design.ts";
import { fitAbilities, fittedChecks, outlineOf, type AbilityRules, type AskShipAtWorld } from "../src/ship-abilities.ts";
import { shipMesh } from "../src/ship-mesh.ts";
import { constant } from "../../ui-contract/src/index.ts";
import { shipViews } from "../src/ship-design.ts";
import type { Propellants } from "../src/ship.ts";

const pack = (n: string) => JSON.parse(readFileSync(new URL(`../../../packs/${n}/pack.json`, import.meta.url), "utf8"));
const SHIPS = pack("base-ships").provides.other;
const RULES = SHIPS.ship_design as DesignRules & { abilities: AbilityRules };
const STYLES = SHIPS.ship_styles as Record<string, ShipStyle>;
const AB = RULES.abilities;
// Test fixtures only: saturated liquids at their normal boiling points (NIST Chemistry WebBook).
const STORE: Propellants = {
  "hydrogen-oxygen": [
    { name: "liquid hydrogen", share: 1 / 7, kg_m3: 70.85, src: "nist-webbook" },
    { name: "liquid oxygen", share: 6 / 7, kg_m3: 1141, src: "nist-webbook" },
  ],
  deuterium: [{ name: "liquid deuterium", share: 1, kg_m3: 162.4, src: "nist-webbook" }],
};
// A small crewed lander and a winged shuttle (chemical, too weak to hover at Earth), and a torch that can hover.
const LANDER: DesignShip = { name: "Lander", accel_g: 0.6, sustained: false, exhaust_km_s: 3.6, wet_t: 40, dry_t: 15, fuels: ["hydrogen-oxygen"], crew: 4, cargo_t: 2 };
const SHUTTLE: DesignShip = { name: "Shuttle", accel_g: 0.3, sustained: false, exhaust_km_s: 4.4, wet_t: 120, dry_t: 60, fuels: ["hydrogen-oxygen"], crew: 6, cargo_t: 10 };
const CUTTER: DesignShip & { refuel: string[] } = { ...pack("base-nav").provides.other.nav.ships["rescue-cutter"], crew: 4, cargo_t: 10 };

let close: () => Promise<void>;
let ask: AskShipAtWorld;
let calls = 0;
before(async () => {
  const { Session } = await import("../../ui-contract/src/session.ts");
  const { gmSeat } = await import("../../ui-contract/src/seat.ts");
  const { nodeKernel } = await import("../../ui-contract/test/node-kernel.ts");
  const k = nodeKernel();
  close = () => k.close();
  const s = await Session.open(k.client, gmSeat(), { id: "abilities", seed: "abilities" }, [pack("base-physics"), pack("base-reference"), pack("base-nav")]);
  ask = (a) => (calls++, s.shipAtWorld(a));
});
after(() => close());

const layoutOf = (s: DesignShip, style: string) => designShip(s, STORE, RULES, style, STYLES[style]!, 1);
const fitted = (s: DesignShip, style: string, abilities: Parameters<typeof fitAbilities>[5]) => fitAbilities(ask, { fixture: "sol" }, s, layoutOf(s, style), STYLES[style]!.hull, abilities, AB);

test("with nothing asked of it, the ship only gets its shape in air and the kernel is not asked", async () => {
  const before = calls;
  const r = await fitted(LANDER, "spine", {});
  assert.equal(calls, before);
  assert.equal(r.report, null);
  const o = outlineOf(layoutOf(LANDER, "spine"));
  assert.ok(Math.abs(r.ship.aero!.ref_area_m2 / (Math.PI * o.front_r ** 2) - 1) < 1e-12);
  assert.equal(r.ship.shield, undefined);
});

test("a shield sized for Mars keeps the heating inside its limit, and weighs its areal density over its face", async () => {
  const r = await fitted(LANDER, "spine", { aerobrake: ["Mars"], land: ["Mars"] });
  const sh = r.fit.shield!;
  assert.ok(sh, "an aerobraking lander carries a shield");
  assert.ok(Math.abs(r.ship.shield!.nose_radius_m / sh.base_r - AB.nose_over_base) < 1e-12);
  const mars = r.report!.worlds.find((w) => w.name === "Mars")!;
  for (const [c, id] of [["aerocapture", "aerocapture.heating"], ["entry", "entry.heating"]] as const) {
    const d = mars.checks.find((x) => x.id === c)!.dangers!.find((x) => x.id === id)!;
    assert.ok(d.figure <= d.limit * (AB.margin + 0.05), `${id}: ${d.figure} against ${d.limit}`);
  }
  const kind = AB.shields.find((k) => k.name === sh.name)!;
  const face = Math.PI * sh.base_r ** 2;
  if (kind.kind === "reusable") assert.ok(Math.abs(sh.mass_t / ((kind.areal_kg_m2! * face) / 1e3) - 1) < 1e-9);
  else assert.ok(sh.mass_t >= (kind.min_areal_kg_m2! * face) / 1e3 - 1e-9);
  assert.deepEqual(r.fit.legs, { gear: "legs" });
  assert.equal(r.ship.landing!.max_touchdown_m_s, AB.legs.max_touchdown_m_s);
});

test("wings sized for a runway land at the margin of the runway's speed", async () => {
  const r = await fitted(SHUTTLE, "sleek", { land: ["Earth"] });
  assert.equal(r.fit.lift?.form, "wings");
  assert.deepEqual(r.fit.runway, ["Earth"]);
  const d = r.report!.worlds[0]!.checks.find((c) => c.id === "runway-landing")!.dangers!.find((x) => x.id === "runway.speed")!;
  // Stall goes as 1/√S, so the area that gives the margin lands exactly there.
  assert.ok(Math.abs(d.figure / (AB.margin * d.limit) - 1) < 1e-9, `${d.figure} against ${AB.margin * d.limit}`);
  assert.equal(r.ship.aero!.lift_area_m2, r.fit.lift!.area_m2);
  assert.ok(Math.abs((r.fit.lift!.span_m * r.fit.lift!.span_m) / r.fit.lift!.area_m2 - AB.wing.aspect) < 1e-9);
  // The card shows the runway for a winged lander, not the ballistic entry.
  assert.deepEqual(fittedChecks(r).map((c) => c.check.id), ["runway-landing", "powered-landing", "take-off"]);
});

test("a drive strong enough to hover carries no wings, and a scoop holds the propellant over density, speed and time", async () => {
  const r = await fitted(CUTTER, "spine", { land: ["Mars"] });
  assert.equal(r.fit.lift, undefined);
  assert.match(r.notes.join(" "), /strong enough to hover and set it down at Mars/);
  const kg = (CUTTER.wet_t - CUTTER.dry_t) * 1e3;
  assert.ok(Math.abs(r.fit.scoop!.area_m2 / (kg / (AB.scoop.skim_density_kg_m3 * AB.scoop.skim_speed_m_s * AB.scoop.fill_days * constant("day"))) - 1) < 1e-12);
});

test("fitted parts appear in the plans and the model", async () => {
  const r = await fitted(SHUTTLE, "sleek", { aerobrake: ["Earth"], land: ["Earth"] });
  const plain = layoutOf(SHUTTLE, "sleek");
  const l = designShip(SHUTTLE, STORE, RULES, "sleek", STYLES.sleek!, 1, {}, r.fit);
  const kinds = new Set(l.pieces.map((p) => p.system));
  for (const k of ["shield", "wings", "legs"] as const) assert.ok(kinds.has(k), k);
  // The shield leads: nothing reaches further forward.
  const shield = l.pieces.find((p) => p.system === "shield")!;
  assert.equal(Math.max(...l.pieces.map((p) => p.x1)), shield.x1);
  assert.equal(l.length_m, shield.x1);
  assert.deepEqual(l.needs, plain.needs, "abilities never resize the systems");
  const m = shipMesh(l, { segments: 16, panel_m: 2 });
  assert.ok(m.surfaces.shield && m.surfaces.shield.indices.length > 0);
  // Wings hang low on the hull in the model as in the side view.
  const v = shipViews(l, [560, 220]);
  assert.ok(v.side.paths.some((p) => p.system === "wings"));
  assert.ok(v.top.callouts.some((c) => /^wings, \d+ m²$/.test(c.text)));
});
