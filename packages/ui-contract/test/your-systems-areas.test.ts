// Map layers on Your systems (OV3, OV4) through the real kernel: a pack's area layer, two systems from a V3 starmap
// held by two powers, read back with holdings_in and coloured out by areasFromHoldings at the campaign map's places.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { areasFromHoldings } from "../src/areas.ts";
import { overlaySpecsOf } from "../src/overlay.ts";
import { placeOnMap, readCampaignSystems, reachInMapUnits } from "../src/your-systems.ts";
import { nodeKernel } from "./node-kernel.ts";

const layerPack = {
  format: "sse.pack/1", id: "test.powers", version: "0.1.0", kind: ["universe"], kernel_range: "^0.1", licence: "Apache-2.0",
  provides: { other: { overlays: [{ id: "so.powers", label: "Powers of the Reach", areas: { by: { relation: "holds" }, answers: { from: "actors" }, level: "auto", fill: { between_stars: { reach: { value: 4, unit: "pc" } } }, weigh_by: "count", look: "sse.look.atlas", tier: "obvious" } }] } },
};

test("two systems held by two powers colour the campaign map, met halfway", async () => {
  const k = nodeKernel();
  try {
    const packs = [JSON.parse(readFileSync("packs/base-physics/pack.json", "utf8")), layerPack];
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
    const sample = JSON.parse(readFileSync("packages/shell/src/documents/sample-sol.v3.json", "utf8"));
    const twin = structuredClone(sample.systems[0]);
    twin.name = "Twin";
    twin.position = { x: 4, y: 3, z: 0 };
    const a = await s.adopt({ import_v3: JSON.stringify({ ...sample, systems: [sample.systems[0], twin] }) });
    assert.ok(a.ok, JSON.stringify(a));
    const read = await readCampaignSystems(s);
    assert.ok(read.ok);
    const placed = read.systems.filter((x) => x.position);
    assert.equal(placed.length, 2);
    const [one, two] = placed as [(typeof placed)[0], (typeof placed)[0]];
    for (const [place, key] of [[one.system, "crown"], [two.system, "league"]])
      assert.equal(((await k.client.call({ op: "hold", place, layer: "so.powers", key, status: "holds", audience: "everyone", at_s: 0 } as never)) as { type: string }).type, "held");
    const h = await s.holdingsIn({ bounds: one.system.split("/")[0]!, layer: "so.powers" });
    assert.ok(h.ok, JSON.stringify(h));
    assert.deepEqual([...new Set(h.holdings.map((x) => x.key))].sort(), ["crown", "league"]);
    const spec = overlaySpecsOf(s.stack().base).find((o) => o.areas);
    assert.equal(spec?.id, "so.powers");
    // The sample's map gives no unit, so the reach reads as map units (the view says so).
    const reach = reachInMapUnits({ value: 4, unit: "pc" }, one.unit) ?? 4;
    const items = areasFromHoldings(h.holdings, { frame: "galaxy-plane-pc", bounds: [-6, -6, 10, 9], positionOf: (p) => placeOnMap(p, placed), reach, inkOf: () => ({ slot: 1 }), cells: 80 });
    assert.deepEqual([...new Set(items.map((i) => i.key))].sort(), ["crown", "league"], "each power has its own area");
    assert.ok(items.every((i) => i.rings.length > 0));
  } finally {
    await k.close();
  }
});
