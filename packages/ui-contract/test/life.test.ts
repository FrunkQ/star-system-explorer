// U7 in Node: the biosphere view's pure parts against the real kernel's
// life_view for the five example worlds. Earth's epochs fill the axis in
// order, production climbs, the food web links every consumer to food in its
// own habitat, and each scripted case reads as designed: Europa's scan is
// blind to the ice, the red-dwarf desert is a false alarm, the dead Earth
// keeps its record in the rocks.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat, tableSeat } from "../src/seat.ts";
import { LIFE_EXAMPLES, epochBands, foodWeb, productionPath, scanTiers, timeAxis, carbonWords, type LifeView } from "../src/life.ts";
import type { PackDoc } from "../src/physics.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);
const base: PackDoc = JSON.parse(readFileSync(new URL("packs/base-physics/pack.json", root), "utf8"));
const universe = { id: "kestrel", seed: "kestrel reach campaign" };

test("the five example worlds through life_view, drawn and read as designed", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), universe, [base], k.scratch);
    const views: Record<string, LifeView> = {};
    for (const e of LIFE_EXAMPLES) {
      const r = await s.lifeView({ fixture: e.fixture });
      assert.ok(r.ok, `${e.fixture}: ${JSON.stringify(r)}`);
      assert.equal(r.source, `fixture:${e.fixture}`);
      views[e.fixture] = r.view;
    }
    const earth = views.earth!;
    const axis = timeAxis(earth, 1000);
    const bands = epochBands(earth, axis);
    assert.equal(bands.length, 10);
    for (let i = 1; i < bands.length; i++) assert.ok(bands[i]!.x0 >= bands[i - 1]!.x0, "epochs in order");
    assert.ok(bands.some((b) => b.epoch.major && b.epoch.name === "The age of green mats"));
    const prod = productionPath(earth, axis, 0, 100);
    assert.ok(prod.d.startsWith("M") && prod.hi > prod.lo, "production climbs over decades");
    const web = foodWeb(earth.guilds, 600, 300);
    assert.equal(web.nodes.length, earth.guilds.length, "every guild drawn");
    for (const n of web.nodes) assert.ok(n.x >= 0 && n.x <= 600 && n.y >= 0 && n.y <= 300 && n.r >= 4);
    for (const n of web.nodes.filter((x) => x.part === "eats" && x.guild.level > 1)) {
      assert.ok(web.links.some((l) => l.to === n.guild.id && !l.recycle), `${n.guild.plain_name} eats something`);
    }
    const et = scanTiers(earth.signatures);
    assert.match(et[0]!.lines[0]!, /green/);
    assert.match(et[1]!.lines.join(" "), /life candidate, confidence 99%/);
    assert.match(et[3]!.lines.join(" "), /fossils: abundant/);
    assert.match(carbonWords(7.98e16), /^8\.0 × 10¹⁶ g C\/yr$/);

    assert.match(scanTiers(views.europa!.signatures)[1]!.lines.join(" "), /blind to: .*ice/);
    assert.ok(views.europa!.hooks.some((h) => h.id === "hidden_life"));
    const desert = views["red-dwarf-desert"]!;
    assert.equal(desert.signatures.alive, false);
    assert.ok(desert.signatures.candidate && desert.hooks.some((h) => h.id === "false_positive"));
    assert.match(scanTiers(desert.signatures)[2]!.lines[0]!, /no biosphere found/);
    assert.equal(foodWeb(desert.guilds, 600, 300).nodes.length, 0);
    assert.ok(views["dead-earth"]!.hooks.some((h) => h.id === "dead_world_record"));

    // Players never read the whole biosphere; they see it through data views.
    const t = await Session.open(k.client, tableSeat(), universe, [base], k.scratch).catch(() => null);
    if (t) assert.equal((await t.lifeView({ fixture: "earth" })).ok, false);
  } finally {
    k.close();
  }
});
