// The atmosphere pass's numbers come from the kernel (light track,
// `read_air_optics`). Earth's air from its pressure, temperature, gravity and
// mix must give Bucholtz's (1995) sea-level Rayleigh coefficients (4.84, 11.49
// and 28.6 per million metres at 680, 550 and 440 nm) and a scale height near
// 8.4 km; Mars and Titan follow from their own facts. The renderer only builds
// the request and reads the answer.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../../ui-contract/src/session.ts";
import { gmSeat } from "../../ui-contract/src/seat.ts";
import { nodeKernel } from "../../ui-contract/test/node-kernel.ts";
import { air, airFactsOf, airFromOptics, airRequest, type AirFacts } from "../src/atmosphere.ts";

const root = new URL("../../../", import.meta.url);
const text = (p: string) => readFileSync(new URL(p, root), "utf8");
const near = (a: number, b: number, tol: number) => Math.abs(a - b) / b <= tol;

async function withKernel(f: (s: Session) => Promise<void>) {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, [JSON.parse(text("packs/base-physics/pack.json"))]);
    await f(s);
  } finally {
    await k.close();
  }
}

async function optics(s: Session, f: AirFacts) {
  const req = airRequest("test:air", f)!;
  const r = await s.readAirOptics([req.air]);
  assert.ok(r.ok, JSON.stringify(r));
  return airFromOptics(r.airs[0] as Parameters<typeof airFromOptics>[0], req);
}

test("Earth, Mars and Titan from the kernel's air optics", async () => {
  await withKernel(async (s) => {
    const e = await optics(s, { pressure_pa: 101325, temperature_k: 288, gravity_m_s2: 9.81, radius_m: 6.371e6, composition: { N2: 0.78, O2: 0.21, Ar: 0.0093 } });
    const want = [4.84e-6, 11.49e-6, 28.6e-6];
    e.rayleigh_m.forEach((b, i) => assert.ok(near(b, want[i]!, 0.03), `band ${i}: ${b} vs ${want[i]}`));
    assert.ok(near(e.scale_height_m, 8430, 0.03), `H = ${e.scale_height_m}`);
    assert.ok(near(e.mean_molar_mass, 28.96, 0.01), `mu = ${e.mean_molar_mass}`);
    assert.ok(near(e.zenith_depth[1], e.rayleigh_m[1] * e.scale_height_m, 0.02), "tau is beta times H for an isothermal column");
    const m = await optics(s, { pressure_pa: 610, temperature_k: 210, gravity_m_s2: 3.71, radius_m: 3.39e6, composition: { CO2: 0.95, N2: 0.028, Ar: 0.02 } });
    assert.ok(near(m.scale_height_m, 10.8e3, 0.08), `Mars H = ${m.scale_height_m}`);
    assert.ok(m.zenith_depth[2] < 0.02, `Mars's blue zenith depth is small: ${m.zenith_depth[2]}`);
    const t = await optics(s, { pressure_pa: 146700, temperature_k: 94, gravity_m_s2: 1.352, radius_m: 2.575e6, composition: { N2: 0.95, CH4: 0.05 } });
    assert.ok(t.scale_height_m > 15e3 && t.scale_height_m < 25e3, `Titan H = ${t.scale_height_m}`);
    assert.ok(t.zenith_depth[2] > 1, `Titan's blue zenith column is optically thick: ${t.zenith_depth[2]}`);
  });
});

test("no air, unknown gases and missing facts are said, not invented", () => {
  assert.equal(airRequest("x", { pressure_pa: 0, temperature_k: 300, gravity_m_s2: 3, radius_m: 1e6, composition: { N2: 1 } }), null);
  const r = airRequest("x", { pressure_pa: 1e5, temperature_k: 300, gravity_m_s2: 9, radius_m: 6e6, composition: { N2: 0.9, SO2: 0.1 } })!;
  assert.deepEqual(r.unknown_gases, ["SO2"]);
  assert.ok(near(r.unknown_share, 0.1, 1e-9));
  assert.deepEqual((r.air as { gases: unknown[] }).gases, [{ gas: "n2", fraction: 1 }]);
  assert.equal(airRequest("x", { pressure_pa: 1e5, temperature_k: 300, gravity_m_s2: 9, radius_m: 6e6, composition: { Xq: 1 } }), null);
  assert.equal(air({ pressure_pa: 1e5, temperature_k: 300, gravity_m_s2: 9, radius_m: 6e6, composition: { N2: 1 } }), null, "nothing drawn until the kernel has answered");
  assert.equal(airFactsOf({ "body.surface_pressure": 101300 }, { gravity_m_s2: 9.8, t_surface_now_k: 285, reference_radius_m: 6.4e6 }), null, "no gas mix, no air");
  assert.ok(airFactsOf({ "body.surface_pressure": 101300, "v3.atmosphere": { composition: { N2: 0.78, O2: 0.21 } } }, { gravity_m_s2: 9.8, t_surface_now_k: 285, reference_radius_m: 6.4e6 }));
});
