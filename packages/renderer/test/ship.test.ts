// V3 gap item 14, part 1: hulls and plumes from the ship record. Checks: the drive's numbers are the rocket
// equation's (classical at chemical speeds, relativistic for the antimatter probe), every tank holds exactly its
// propellant at the stored density, the shield's mass is its areal density over the crew module's skin, the outline
// builds a three.js lathe the hull's length, and the Chart paths fit their frame.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { LatheGeometry, Vector2 } from "three";
import { bandOf, driveFacts, hullFromShip, hullProfile, narrateShip, shipPaths, type Propellants, type ShipRecord } from "../src/ship.ts";
import { constant } from "../../ui-contract/src/index.ts";

const pack = JSON.parse(readFileSync(new URL("../../../packs/base-nav/pack.json", import.meta.url), "utf8"));
const SHIPS = pack.provides.other.nav.ships as Record<string, ShipRecord>;

// Stored densities as a view would pass them from the sheet's substance rows (phases.density_liquid). Test
// fixtures only: saturated liquids at their normal boiling points (NIST Chemistry WebBook; He-3 from Kerr 1954).
const STORE: Propellants = {
  "hydrogen-oxygen": [
    { name: "liquid hydrogen", share: 1 / 7, kg_m3: 70.85, src: "nist-webbook" },
    { name: "liquid oxygen", share: 6 / 7, kg_m3: 1141, src: "nist-webbook" },
  ],
  hydrogen: [{ name: "liquid hydrogen", share: 1, kg_m3: 70.85, src: "nist-webbook" }],
  deuterium: [{ name: "liquid deuterium", share: 1, kg_m3: 162.4, src: "nist-webbook" }],
  "helium-3": [{ name: "liquid helium-3", share: 1, kg_m3: 58.9, src: "kerr1954" }],
};

const near = (a: number, b: number, rel: number, what: string) => assert.ok(Math.abs(a / b - 1) < rel, `${what}: ${a} against ${b}`);

test("at chemical speeds the drive is Tsiolkovsky's rocket", () => {
  const s = SHIPS["chemical-transfer"];
  const d = driveFacts(s);
  const ve = s.exhaust_km_s * 1e3;
  const F = s.wet_t * 1e3 * s.accel_g * constant("g0");
  near(d.thrust_n, F, 1e-12, "thrust");
  near(d.mass_flow_kg_s, F / ve, 1e-9, "mass flow");
  near(d.jet_power_w, 0.5 * F * ve, 1e-9, "jet power");
  near(d.delta_v_m_s, ve * Math.log(s.wet_t / s.dry_t), 1e-9, "delta-v");
  near(d.burn_s, ((s.wet_t - s.dry_t) * 1e3) / (F / ve), 1e-9, "burn");
  // A water molecule's worth (18 u) of exhaust carries a few eV: chemistry.
  assert.ok(d.ev_per_u * 18 > 1 && d.ev_per_u * 18 < 10, `${d.ev_per_u * 18} eV`);
});

test("the antimatter probe stays under light speed and its exhaust can make gamma rays", () => {
  const s = SHIPS["relativistic-probe"];
  const d = driveFacts(s);
  const c = constant("c");
  const beta = (s.exhaust_km_s * 1e3) / c;
  near(d.delta_v_m_s, c * Math.tanh(beta * Math.log(s.wet_t / s.dry_t)), 1e-12, "relativistic delta-v");
  assert.ok(d.delta_v_m_s < c);
  assert.ok(d.delta_v_m_s < s.exhaust_km_s * 1e3 * Math.log(s.wet_t / s.dry_t), "less than the classical answer");
  const gamma = 1 / Math.sqrt(1 - beta * beta);
  near(d.mass_flow_kg_s * gamma * beta * c, d.thrust_n, 1e-9, "momentum flux is the thrust");
  assert.equal(bandOf(d.photon_m), "gamma rays");
  assert.equal(bandOf(driveFacts(SHIPS["fusion-torch"]).photon_m), "X-rays");
});

test("each tank holds its propellant at the stored density, and the shield is its areal density over the skin", () => {
  for (const [id, s] of Object.entries(SHIPS)) {
    const h = hullFromShip(s, STORE);
    const tanks = h.parts.filter((p) => p.kind === "tank");
    const prop = tanks.filter((p) => !p.name.startsWith("jump")).reduce((a, p) => a + (p.mass_t ?? 0), 0);
    near(prop, s.wet_t - s.dry_t, 1e-12, `${id} propellant`);
    for (const p of tanks.filter((q) => !q.stand_in)) near(Math.PI * p.radius_m ** 2 * (p.to_m - p.from_m), p.volume_m3!, 1e-9, `${id} ${p.name}`);
    // Parts follow one another along the axis with no gaps.
    for (let i = 1; i < h.parts.length; i++) assert.equal(h.parts[i].from_m, h.parts[i - 1].to_m);
    assert.equal(h.length_m, h.parts[h.parts.length - 1].to_m);
  }
  const chem = hullFromShip(SHIPS["chemical-transfer"], STORE).parts.filter((p) => p.kind === "tank");
  assert.deepEqual(chem.map((p) => p.name), ["liquid hydrogen", "liquid oxygen"]);
  assert.ok(chem[0].volume_m3! > chem[1].volume_m3!, "a sixth of the mass as hydrogen still needs the bigger tank");
  const torch = hullFromShip(SHIPS["fusion-torch"], STORE);
  const r = torch.radius_m;
  near(torch.shield_t, (40 * 10 * (2 * Math.PI * r * r + 2 * Math.PI * r * r)) / 1e3, 1e-12, "shield mass");
  assert.ok(hullFromShip(SHIPS["jump-courier"], STORE).parts.some((p) => p.name.startsWith("jump fuel")));
  // With the record's fuel shares, deuterium and helium-3 split by their reaction mass ratio.
  const shared = hullFromShip({ ...SHIPS["fusion-torch"], fuel_shares: { deuterium: 2.014, "helium-3": 3.016 } }, STORE).parts;
  near(shared.find((p) => p.name === "liquid deuterium")!.mass_t!, 1000 * (2.014 / 5.03), 1e-12, "deuterium share");
  const probe = hullFromShip(SHIPS["relativistic-probe"], STORE);
  assert.ok(probe.parts.some((p) => p.stand_in && p.name === "antimatter"));
  assert.match(probe.notes.join(" "), /antimatter is stored is not on the master data sheet/);
});

test("the outline builds a three.js lathe as long and as wide as the hull", () => {
  const h = hullFromShip(SHIPS["nuclear-thermal"], STORE);
  const g = new LatheGeometry(hullProfile(h).map(([along, rr]) => new Vector2(rr, along)), 32);
  g.computeBoundingBox();
  const b = g.boundingBox!;
  near(b.max.y - b.min.y, h.length_m, 1e-6, "length");
  near(b.max.x, h.radius_m, 1e-3, "radius");
});

test("Chart paths fit their frame and the words give the drive's facts", () => {
  for (const s of Object.values(SHIPS)) {
    const h = hullFromShip(s, STORE);
    const p = shipPaths(h, driveFacts(s), [320, 160]);
    for (const path of [p.hull, p.plume, p.dividers])
      for (const m of path.matchAll(/(-?[\d.]+) (-?[\d.]+)/g)) {
        const [x, y] = [Number(m[1]), Number(m[2])];
        assert.ok(x >= 0 && x <= 320 && y >= 0 && y <= 160, `${s.name}: ${x} ${y}`);
      }
  }
  const t = narrateShip(hullFromShip(SHIPS["fusion-torch"], STORE), driveFacts(SHIPS["fusion-torch"]));
  assert.match(t, /^Fusion torch: \d+ m long/);
  assert.match(t, /Thrust [\d.]+ MN; the jet carries [\d.]+ TW/);
  assert.match(t, /X-rays/);
});

test("fuels read from the master data sheet: mixtures by mass, densities from the substances", async () => {
  const { Session } = await import("../../ui-contract/src/session.ts");
  const { gmSeat } = await import("../../ui-contract/src/seat.ts");
  const { nodeKernel } = await import("../../ui-contract/test/node-kernel.ts");
  const { readPropellants } = await import("../src/ship.ts");
  const k = nodeKernel();
  try {
    const pack = (n: string) => JSON.parse(readFileSync(new URL(`../../../packs/${n}/pack.json`, import.meta.url), "utf8"));
    const s = await Session.open(k.client, gmSeat(), { id: "ship", seed: "ship" }, [pack("base-physics"), pack("base-reference")]);
    const store = await readPropellants(s, ["hydrogen-oxygen", "deuterium", "helium-3", "antimatter"]);
    assert.deepEqual(store["hydrogen-oxygen"]!.map((p) => Math.round(p.share * 7)), [1, 6]);
    assert.ok(store["hydrogen-oxygen"]![0]!.kg_m3 < 100 && store["hydrogen-oxygen"]![1]!.kg_m3 > 1000, "liquid hydrogen is light, liquid oxygen dense");
    assert.ok(store.deuterium![0]!.kg_m3 > 100 && store.deuterium![0]!.kg_m3 < 200);
    assert.equal(store.antimatter, undefined, "no sheet row: the hull draws a stand-in");
  } finally {
    await k.close();
  }
});
