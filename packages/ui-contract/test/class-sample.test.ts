// V14i in Node: the class gallery's sample worlds. `classSample` makes a world for each class asked on a what-if
// branch (kernel `class_sample`), the branch's world reads back as six level-0 tiles through `surfaceTiles`, nothing
// reaches the campaign's state hash, and a player's seat is refused.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat, tableSeat } from "../src/seat.ts";
import { globeStats, type Globe } from "../src/globe.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);
const text = (p: string) => readFileSync(new URL(p, root), "utf8");
const packs = [JSON.parse(text("packs/base-physics/pack.json")), JSON.parse(text("packs/base-time/pack.json"))];
const atlas = JSON.parse(text("packs/base-atlas/pack.json"));

test("a class's sample world reads back on its branch, and the campaign does not move", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
    const before = await s.stateHash();
    const r = await s.classSample(["rocky", "iron-rich", "no-such-class"], [atlas]);
    assert.ok(r.ok, JSON.stringify(r));
    assert.deepEqual(r.result.samples.map((x) => x.class), ["rocky", "iron-rich"]);
    assert.equal(r.result.refused[0]?.class, "no-such-class");
    const rocky = r.result.samples[0]!;
    assert.ok(rocky.surface && rocky.fit > 0.5, JSON.stringify(rocky));
    const t = await s.surfaceTiles(rocky.system, rocky.body, r.result.branch);
    assert.ok(t.ok, JSON.stringify(t));
    const globe: Globe = [0, 1, 2, 3, 4, 5].map((f) => t.tiles.find((x) => x.key.face === f && x.key.level === 0));
    assert.equal(globe.filter((x) => x).length, 6, "six level-0 tiles");
    const st = globeStats(globe);
    console.log(JSON.stringify({ covered: st.covered, materials: st.materials.slice(0, 6), hi: st.highest_m, lo: st.lowest_m }));
    assert.ok(await s.discardBranch(r.result.branch));
    assert.equal(await s.stateHash(), before, "a sample never reaches the campaign");
    const p = await Session.open(k.client, tableSeat(), { id: "kestrel", seed: "k" }, packs).catch(() => null);
    if (p) assert.equal((await p.classSample(["rocky"], [atlas])).ok, false, "a player's seat makes no samples");
  } finally {
    k.close();
  }
});
