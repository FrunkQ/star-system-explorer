// Belt members in True (render audit item 3): the system view resolves a few members of each belt through the
// kernel's `small_body` and draws them where their orbits put them now (packages/views/system-3d/src/small-sample.ts).
import { test } from "node:test";
import assert from "node:assert/strict";
import { placeOnOrbit, readSmallSample, SAMPLE_RANKS, type SmallReader } from "../../packages/views/system-3d/src/small-sample.ts";
import { fromSmallSample } from "../../packages/views/system-3d/src/bodies.ts";

const orbit = { a_au: 2.5, e: 0.2, peri_deg: 30, node_deg: 10, mean_anomaly_deg: 0 };

test("a body at perihelion is a(1 − e) out, at aphelion a(1 + e), and Kepler's equation holds between", () => {
  const p = placeOnOrbit(orbit);
  assert.ok(Math.abs(p.r_au - 2.0) < 1e-9);
  assert.ok(Math.abs(p.longitude_deg - 40) < 1e-9, "node + perihelion argument at perihelion");
  const a = placeOnOrbit({ ...orbit, mean_anomaly_deg: 180 });
  assert.ok(Math.abs(a.r_au - 3.0) < 1e-9);
  assert.ok(Math.abs(a.longitude_deg - 220) < 1e-9);
  // A quarter of the period after perihelion it is past a quarter of the way round (it moves fastest there).
  const q = placeOnOrbit({ ...orbit, mean_anomaly_deg: 90 });
  assert.ok(q.longitude_deg - 40 > 90, `${q.longitude_deg}`);
});

function member(rank: number, r_au?: number) {
  return {
    address: `u:t/sys:1/cloud:0/obj:${rank}`, cloud: "u:t/sys:1/cloud:0", rank, diameter_km: 1000 / rank, albedo: 0.1, weathering: 0.5, type_label: "C",
    makeup: { ice: rank === 10 ? 0.5 : 0 },
    orbit,
    figure: { kind: "irregular", axes_m: [600e3 / rank, 500e3 / rank, 400e3 / rank] as [number, number, number], lobes: [], relief: 0.05 },
    ...(rank === 10 ? { activity: { r_au: r_au ?? 2.0, active: true, dominant: "h2o", gas_kg_s: 10, dust_kg_s: 5, coma_km: 1e4, dust_tail_km: 1e6, ion_tail_km: 1e7 } } : {}),
  };
}

test("each belt is sampled at its ranks, an icy member's activity asked again where it is now, and a refused rank ends that belt", async () => {
  const asks: { cloud?: string; rank?: number; r_au?: number }[] = [];
  const k: SmallReader = {
    async smallBody(a) {
      asks.push({ cloud: a.cloud, rank: a.rank, r_au: a.r_au });
      if (a.cloud === "u:t/sys:1/cloud:1" && (a.rank ?? 0) > 1) return { ok: false, code: "not_found", message: "no such member" };
      return { ok: true, body: member(a.rank ?? 1, a.r_au), notables: [] };
    },
  };
  const got = await readSmallSample(k, "u:t/sys:1", [{ address: "u:t/sys:1/cloud:0", name: "Main belt" }, { address: "u:t/sys:1/cloud:1", name: "Outer belt" }]);
  assert.deepEqual(got.map((s) => s.rank), [...SAMPLE_RANKS, 1]);
  assert.equal(got[0]!.name, "Main belt, largest (C type)");
  assert.equal(got[1]!.name, "Main belt, 10th largest (C type)");
  const icy = got[1]!;
  assert.equal(asks.filter((x) => x.rank === 10 && x.r_au !== undefined).length, 1, "the icy one is asked again at its distance now");
  assert.ok(Math.abs(icy.activity!.r_au - 2.0) < 1e-9);
  assert.deepEqual(icy.figure.axes_m, [60e3, 50e3, 40e3]);
  assert.equal(icy.radius_m, 50e3);
  const b = fromSmallSample(icy, "u:t/sys:1/star:0");
  assert.equal(b.parent, "u:t/sys:1/star:0");
  assert.ok(Math.abs(b.a_m! - 2.0 * 1.495978707e11) < 1e3);
  assert.equal(b.phase_deg, 40);
});

test("stops between requests when the view no longer wants them", async () => {
  let n = 0;
  const k: SmallReader = { async smallBody(a) { n++; return { ok: true, body: member(a.rank ?? 1), notables: [] }; } };
  const got = await readSmallSample(k, "s", [{ address: "c", name: "B" }], () => n < 2);
  assert.equal(got.length, 2);
});
