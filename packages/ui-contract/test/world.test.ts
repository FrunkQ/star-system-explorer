// V6 cutaway in Node: the view's ladder words a world stack exactly as the
// kernel's sse-world ladder() does (the Strata mockup, line for line), and
// the cut's radii keep shells in order on both scales.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { checkWorldStack, cutRadius, formatKm, ladder, regionCounts, type WorldStack } from "../src/world.ts";

const tarn = JSON.parse(readFileSync(new URL("../../../tests/fixtures/worlds/tarn.json", import.meta.url), "utf8")) as WorldStack;

// Same table as crates/sse-world/src/tests.rs STRATA.
const STRATA: [string, string, string][] = [
  ["+400 km", "Tarn High, station", "orbit"],
  ["+52 km", "Highwater, aerostat", "storm belt"],
  ["0", "Biomes, 3 cities, Ridge 4 volcano", "surface"],
  ["−0.4 km", "Undercroft, lava-tube city", "equator shelf"],
  ["−3 km", "Polar ice shell", "south pole"],
  ["−3 to −11 km", "Sub-ice brine ocean", "south pole"],
  ["−9 km", "Magma chamber", "under Ridge 4"],
  ["−35 km", "Crust ends, 9 plates", "shell"],
  ["−2,900 km", "Liquid core, dynamo on", "shell"],
];

test("the ladder reproduces the kernel's Strata lines", () => {
  assert.deepEqual(checkWorldStack(tarn), []);
  assert.deepEqual(
    ladder(tarn).map((l) => [l.depth, l.text, l.where]),
    STRATA,
  );
});

test("every fixture world gets a ladder and the shell app's copy matches tarn", () => {
  for (const n of ["earth", "europa", "mars", "titan", "venus"]) {
    const w = JSON.parse(readFileSync(new URL(`../../../tests/fixtures/worlds/${n}.json`, import.meta.url), "utf8")) as WorldStack;
    assert.deepEqual(checkWorldStack(w), [], n);
    assert.ok(ladder(w).length > 0, n);
  }
  const doc = JSON.parse(readFileSync(new URL("../../shell/src/documents/kestrel.worlds.json", import.meta.url), "utf8")) as { stacks: WorldStack[] };
  const copy = doc.stacks[0]!;
  assert.deepEqual({ ...copy, body: tarn.body, name: tarn.name }, tarn);
});

test("km words and region counts", () => {
  assert.equal(formatKm(-2_900_000), "−2,900");
  assert.equal(formatKm(400_000), "+400");
  assert.equal(formatKm(-400), "−0.4");
  assert.equal(formatKm(0), "0");
  assert.match(regionCounts(tarn), /9 plates/);
});

test("cut radii keep order and the log scale shows a 3 km lid", () => {
  for (const scale of ["true", "log"] as const) {
    const top = cutRadius(tarn, 120_000, scale);
    const surf = cutRadius(tarn, 0, scale);
    const lid = cutRadius(tarn, -3000, scale);
    const centre = cutRadius(tarn, -tarn.radius_m, scale);
    assert.equal(top, 1);
    assert.ok(top > surf && surf > lid && lid > centre && centre >= 0, scale);
  }
  // To scale the lid is a hair; on the log scale it is a visible band (more than 2% of the radius).
  assert.ok(cutRadius(tarn, 0, "true") - cutRadius(tarn, -3000, "true") < 0.001);
  assert.ok(cutRadius(tarn, 0, "log") - cutRadius(tarn, -3000, "log") > 0.02);
});
