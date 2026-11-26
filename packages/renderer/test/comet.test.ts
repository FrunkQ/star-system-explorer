// SB4 part 2: comets in True from the kernel's activity (comet.ts).
import { test } from "node:test";
import assert from "node:assert/strict";
import { comaPicture, cometNote, cometTails, ionColour, isActive, showing, tailGrains, type CometActivity } from "../src/comet.ts";

// A CO2-driven member at 11.8 AU, as the kernel's small-bodies contract case answers (tests/golden/areas/small-bodies.jsonl).
const co2: CometActivity = { r_au: 11.77, active: true, dominant: "co2", gas_kg_s: 69623, dust_kg_s: 69623, coma_km: 17159908, dust_tail_km: 7990, ion_tail_km: 2.77e10 };

test("a quiet body draws nothing; an active one draws a coma and both tails", () => {
  assert.equal(isActive({ ...co2, active: false }), false);
  assert.equal(isActive({ ...co2, gas_kg_s: 0.5, dust_kg_s: 0.5 }), false, "under a kilogram a second is too faint to show");
  const quiet = cometTails({ ...co2, active: false });
  assert.equal(quiet.dust.length + quiet.ion.length, 0);
  const t = cometTails(co2);
  assert.ok(t.dust.length > 0 && t.ion.length > 0);
});

test("each tail reaches the kernel's length and no further, straight away from the star", () => {
  const t = cometTails(co2);
  let maxDust = 0;
  let maxIon = 0;
  for (let i = 0; i < t.dust.length; i += 3) (maxDust = Math.max(maxDust, t.dust[i]!)), assert.ok(t.dust[i]! > 0, "every grain is on the side away from the star");
  for (let i = 0; i < t.ion.length; i += 3) maxIon = Math.max(maxIon, t.ion[i]!);
  assert.ok(maxDust <= 7990e3 && maxDust > 0.8 * 7990e3, `dust tail ${maxDust} m`);
  assert.ok(maxIon <= 2.77e13 && maxIon > 0.8 * 2.77e13, `ion tail ${maxIon} m`);
});

test("grains thin out along a tail as one over the distance from the head (equal numbers in each factor of ten)", () => {
  const L = 1e10;
  const g = tailGrains(L, 1e7, 6000, 5);
  const s0 = Math.min(L * 1e-3, 1e6);
  const decades = Math.log10(L / s0);
  const lower = [...Array(6000).keys()].filter((i) => g[i * 3]! < s0 * 10 ** (decades / 2)).length;
  // Half the decades hold half the grains under 1/s; under an even spread the first half would hold under 1%.
  assert.ok(Math.abs(lower / 6000 - 0.5) < 0.05, `share in the first half of the decades ${lower / 6000}`);
  for (let i = 0; i < 6000; i++) assert.ok(Math.hypot(g[i * 3 + 1]!, g[i * 3 + 2]!) <= 1e7 + 1e-6, "within the tail's width");
});

test("brighter with more production, on a log scale capped at full", () => {
  assert.equal(showing(1), 0);
  assert.ok(showing(1e3) > showing(10));
  assert.equal(showing(1e9), 1);
  assert.ok(Math.abs(showing(1e3) - 0.5) < 1e-9);
});

test("the ion tail is blue for a CO or CO2 comet and warmer for a water one", () => {
  const blue = ionColour(co2);
  assert.ok(blue[2] > blue[0], `CO+ ${blue}`);
  const water = ionColour({ ...co2, dominant: "h2o" });
  assert.ok(water[0] > water[2], `H2O+ ${water}`);
});

test("the coma falls off from its centre and is dark at its edge", () => {
  const p = comaPicture([1, 1, 1], 1, 64);
  const at = (x: number, y: number) => p.data[(y * 64 + x) * 4 + 3]!;
  assert.ok(at(32, 32) > at(40, 32) && at(40, 32) > at(56, 32));
  assert.equal(at(0, 0), 0);
  assert.match(cometNote("Belt, 10th largest", co2), /carbon dioxide/);
});
