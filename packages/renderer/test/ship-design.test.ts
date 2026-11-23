// V3 gap item 14, stage 2: ships designed from their parts. Checks: every part holds what the rules say it must
// (quarters from crew, hold from cargo, tanks from propellant, radiators from waste heat by Stefan-Boltzmann); a style
// arranges but never resizes; a recipe rebuilds the same ship; hand overrides win and are said; the call-outs fit and
// never overlap; and every rule in the base-ships pack names a source it lists.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { designShip, narrateDesign, needsOf, remixes, shipViews, type DesignRules, type DesignShip, type ShipLayout, type ShipStyle } from "../src/ship-design.ts";
import { type Propellants, type ShipRecord } from "../src/ship.ts";
import { constant } from "../../ui-contract/src/index.ts";

const json = (p: string) => JSON.parse(readFileSync(new URL(p, import.meta.url), "utf8"));
const SHIPS = json("../../../packs/base-nav/pack.json").provides.other.nav.ships as Record<string, ShipRecord>;
const PACK = json("../../../packs/base-ships/pack.json").provides.other;
const RULES = PACK.ship_design as DesignRules;
const STYLES = PACK.ship_styles as Record<string, ShipStyle>;
// Test fixtures only (the sheet's density_liquid rows are on their way): NIST saturated liquids at boiling.
const STORE: Propellants = {
  "hydrogen-oxygen": [
    { name: "liquid hydrogen", share: 1 / 7, kg_m3: 70.85, src: "nist-webbook" },
    { name: "liquid oxygen", share: 6 / 7, kg_m3: 1141, src: "nist-webbook" },
  ],
  hydrogen: [{ name: "liquid hydrogen", share: 1, kg_m3: 70.85, src: "nist-webbook" }],
  deuterium: [{ name: "liquid deuterium", share: 1, kg_m3: 163.5, src: "nist-webbook" }],
  "helium-3": [{ name: "liquid helium-3", share: 1, kg_m3: 59, src: "kerr1954" }],
};
const ship = (id: string, crew: number, cargo_t: number): DesignShip => ({ ...SHIPS[id]!, crew, cargo_t });
const TORCH = ship("fusion-torch", 12, 200);
const CHEM = ship("chemical-transfer", 4, 20);
const near = (a: number, b: number, rel: number, what: string) => assert.ok(Math.abs(a - b) <= rel * Math.abs(b), `${what}: ${a} against ${b}`);
const vol = (l: ShipLayout, sys: string) =>
  l.pieces.filter((p) => p.system === sys && p.shape === "body").reduce((a, p) => a + (Math.PI / 3) * (p.x1 - p.x0) * (p.r0 * p.r0 + p.r0 * p.r1 + p.r1 * p.r1), 0);
const panelArea = (l: ShipLayout) => l.pieces.filter((p) => p.shape === "panel").reduce((a, p) => a + (p.x1 - p.x0) * Math.abs(p.span ?? 0), 0);

test("each part holds what the rules say it must", () => {
  const n = needsOf(TORCH, STORE, RULES);
  near(n.crew_m3, (12 * RULES.crew.habitable_m3_per_person) / RULES.crew.net_to_gross, 1e-12, "quarters");
  near(n.cargo_m3, 200e3 / RULES.cargo.stowage_kg_m3, 1e-12, "hold");
  const heat = RULES.drive_heat.find((h) => h.fuels.includes("deuterium"))!;
  near(n.waste_w, heat.waste_per_jet * n.drive.jet_power_w, 1e-12, "waste heat");
  near(n.radiator_m2 * 2 * RULES.radiator.emissivity * constant("sigma_SB") * heat.radiator_k ** 4, n.waste_w, 1e-9, "Stefan-Boltzmann");
  const chem = needsOf(CHEM, STORE, RULES);
  assert.equal(chem.radiator_m2, 0, "a chemical drive carries its heat away in the exhaust");
  assert.ok(needsOf(SHIPS["relativistic-probe"]!, STORE, RULES).radiator_m2 > 0, "the probe's heat follows antimatter, not the hydrogen it heats");
  for (const [id, style] of Object.entries(STYLES))
    for (const s of [TORCH, CHEM]) {
      const l = designShip(s, STORE, RULES, id, style, 3);
      near(vol(l, "crew"), l.needs.crew_m3, 1e-9, `${id} ${s.name} quarters`);
      near(vol(l, "cargo"), l.needs.cargo_m3, 1e-9, `${id} ${s.name} hold`);
      near(vol(l, "tanks"), l.needs.tanks.reduce((a, t) => a + t.volume_m3, 0), 1e-9, `${id} ${s.name} tanks`);
      near(panelArea(l), l.needs.radiator_m2, 1e-9, `${id} ${s.name} radiators`);
    }
});

test("a style arranges the parts but never resizes them, and a recipe rebuilds the same ship", () => {
  const a = designShip(TORCH, STORE, RULES, "spine", STYLES.spine!, 7);
  const b = designShip(TORCH, STORE, RULES, "sleek", STYLES.sleek!, 7);
  assert.deepEqual(a.needs, b.needs);
  assert.notDeepEqual(a.pieces, b.pieces);
  assert.deepEqual(designShip(TORCH, STORE, RULES, "spine", STYLES.spine!, 7), a, "same recipe, same ship");
  const options = remixes(TORCH, STORE, RULES, "spine", STYLES.spine!, 6);
  assert.ok(new Set(options.map((l) => JSON.stringify(l.pieces))).size > 1, "remixes give different choices");
  for (const l of options) assert.deepEqual(l.needs, a.needs);
  // A crewless probe has no quarters; the words say why.
  const probe = designShip(SHIPS["relativistic-probe"]!, STORE, RULES, "spine", STYLES.spine!, 1);
  assert.ok(!probe.pieces.some((p) => p.system === "crew"));
  assert.match(narrateDesign(probe, STYLES.spine!), /gives no crew/);
});

test("sizes set by hand win and are said", () => {
  const l = designShip(CHEM, STORE, RULES, "spine", STYLES.spine!, 1, { crew_m3: 400 });
  near(vol(l, "crew"), 400, 1e-9, "overridden quarters");
  assert.match(narrateDesign(l, STYLES.spine!), /Set by hand: crew\./);
});

test("side and top views: every system drawn, call-outs inside the frame and clear of each other", () => {
  for (const s of [TORCH, CHEM])
    for (const [id, style] of Object.entries(STYLES)) {
      const l = designShip(s, STORE, RULES, id, style, 2);
      for (const fit of ["all", "hull"] as const) {
        const v = shipViews(l, [560, 220], fit);
        for (const view of [v.side, v.top]) {
          for (const sys of new Set(l.pieces.map((p) => p.system))) assert.ok(view.paths.some((p) => p.system === sys), `${id} ${sys}`);
          for (const above of [true, false]) {
            const row = view.callouts.filter((c) => c.above === above).sort((p, q) => p.label[0] - q.label[0]);
            for (const c of row) assert.ok(c.label[0] >= 0 && c.label[0] <= 560, `${c.text} at ${c.label[0]}`);
            // Labels that fit side by side never overlap (more text than the frame holds would have to).
            const room = row.reduce((a, c) => a + c.text.length * 5.6 + 10, 0) <= 560;
            for (let i = 1; room && i < row.length; i++) {
              const gap = row[i]!.label[0] - row[i - 1]!.label[0];
              const need = (row[i]!.text.length + row[i - 1]!.text.length) * 2.8 + 10;
              assert.ok(gap >= need - 1e-6, `${row[i - 1]!.text} and ${row[i]!.text} overlap`);
            }
          }
        }
        if (fit === "hull" && s === TORCH) assert.ok(v.clipped, "the torch's radiators run off a hull picture");
      }
    }
  assert.match(narrateDesign(designShip(TORCH, STORE, RULES, "sleek", STYLES.sleek!, 1), STYLES.sleek!), /nowhere to put this much radiator/);
});

test("every rule in the base-ships pack names a source the pack lists", () => {
  const sources = PACK.ship_design.sources as Record<string, string>;
  const srcs = [RULES.crew, RULES.cargo, RULES.radiator, ...RULES.drive_heat].map((r) => (r as { src: string }).src);
  for (const s of srcs) assert.ok(sources[s], `source ${s}`);
});
