// V6 cutaway in Node: the view's ladder words a world stack exactly as the
// kernel's sse-world ladder() does (the Strata mockup, line for line), and
// the cut's radii keep shells in order on both scales.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { checkWorldStack, cutRadius, formatKm, ladder, regionCounts, type WorldStack } from "../src/world.ts";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { nodeKernel } from "./node-kernel.ts";

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

test("the kernel serves every fixture; its ladder and this port agree; the campaign's links resolve", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, [JSON.parse(readFileSync(new URL("../../../packs/base-physics/pack.json", import.meta.url), "utf8"))]);
    for (const n of ["earth", "europa", "mars", "tarn", "titan", "venus"]) {
      const r = await s.readWorldStack({ fixture: n });
      assert.ok(r.ok, n);
      assert.equal(r.result.source, `fixture:${n}`);
      assert.deepEqual(r.result.invalid, [], n);
      assert.deepEqual(checkWorldStack(r.result.stack), [], n);
      assert.deepEqual(ladder(r.result.stack).map((l) => [l.depth, l.text, l.where, l.key]), r.result.ladder.map((l) => [l.depth, l.text, l.where, l.key]), `${n}: the TS port words it as the kernel does`);
    }
    const tarn = await s.readWorldStack({ fixture: "tarn" });
    assert.ok(tarn.ok && tarn.result.notes.some((x) => x.startsWith("brine-ocean: ")));
    // Kestrel's Tarn has no built stack yet; the campaign names its fixture.
    const none = await s.readWorldStack({ body: "u:kestrel/sys:1/body:3" });
    assert.ok(!none.ok && none.code === "no_world_stack");
    const links = JSON.parse(readFileSync(new URL("../../shell/src/documents/kestrel.worlds.json", import.meta.url), "utf8")) as { links: { body: string; fixture: string }[] };
    for (const l of links.links) assert.ok((await s.readWorldStack({ fixture: l.fixture })).ok, l.fixture);
  } finally {
    await k.close();
  }
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
