// The Formation screen's chart (F10) in Node: a grown and adopted system's record is read back (kernel
// `formation_record`) and drawn as tracks in time and distance, from the run's own record. An authored system has
// no record and says why. Reading writes nothing.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { formationChart, massWidth, stageAt, chartXY } from "../src/formation-chart.ts";
import { formationOf } from "../src/formation-record.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);
const text = (p: string) => readFileSync(new URL(p, root), "utf8");
const packs = [JSON.parse(text("packs/base-physics/pack.json")), JSON.parse(text("packs/base-time/pack.json"))];

test("a grown system's history reads back and draws as tracks, lines, events and stages", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
    const req = { seed: "first light", system: "u:kestrel/sys:3" };
    assert.ok((await s.adopt({ formation: req })).ok);
    const h = await s.stateHash();
    const src = await formationOf(s, req.system);
    assert.equal(src.kind, "run", JSON.stringify(src).slice(0, 300));
    if (src.kind !== "run") return;
    assert.equal(src.seed, "first light");
    assert.equal(src.drift, false, "the same pipeline grows the same history");
    assert.equal(await s.stateHash(), h, "reading the record writes nothing");

    const c = formationChart(src.run, { names: {}, seed: src.seed });
    assert.equal(JSON.stringify(formationChart(src.run, { seed: src.seed }).tracks), JSON.stringify(c.tracks), "the same run gives the same chart");
    // Scales: decades that hold everything, the system's age inside the span.
    assert.ok(c.t_top <= 0.1 && c.t_bottom >= c.now && c.now > 100, `${c.t_top} ${c.t_bottom} ${c.now}`);
    assert.ok(c.a_in < c.a_out);
    // Every body is a track; the ones that last are the run's worlds, ending where the run put them.
    const lasting = c.tracks.filter((t) => t.end.how === "lasts");
    assert.equal(lasting.length, src.run.run.worlds.length);
    const final = (src.run.run as unknown as { upheaval: { worlds: { id: string; a_au: number }[] } }).upheaval.worlds;
    for (const w of final) assert.ok(Math.abs(lasting.find((t) => t.id === w.id)!.end.a - w.a_au) < 1e-9, `${w.id} ends at its orbit`);
    // Tracks thicken as they grow: no segment is lighter than the one before (mass only grows or merges in).
    for (const t of c.tracks) for (let i = 1; i < t.segs.length; i++) assert.ok(t.segs[i]!.m >= t.segs[i - 1]!.m - 1e-12, `${t.id} loses mass at ${t.segs[i]!.t0}`);
    assert.ok(c.tracks.every((t) => t.segs.every((g) => g.t1 >= g.t0 && g.a0 > 0 && g.a1 > 0)));
    // First light throws worlds out and merges others: tracks that end that way.
    assert.ok(c.tracks.some((t) => t.end.how === "ejected"), "a rogue runs off the edge");
    const merged = c.tracks.find((t) => t.end.how === "merged");
    assert.ok(merged && merged.end.how === "merged" && c.tracks.some((t) => t.id === merged.end.into), "a merged track joins another");
    // The disc's five lines while the gas lasts, the water line leaning inward.
    assert.equal(c.lines.length, 5);
    const water = c.lines.find((l) => l.i === 2)!;
    assert.ok(water.pts[water.pts.length - 1]!.a < water.pts[0]!.a, "the water ice line moves in as the disc cools");
    assert.ok(water.pts.every((p) => p.t <= c.gas_until));
    // Events sit on a track; violent ones have words.
    assert.ok(c.events.length > 0 && c.events.every((e) => e.a > 0 && e.t > 0));
    assert.ok(c.events.some((e) => e.kind === "ejected" && e.short.includes("rogue")));
    assert.ok(c.upheaval && c.upheaval.to > c.upheaval.from);
    // The nine stages in order, and the slice finds its stage.
    assert.deepEqual(c.stages.map((x) => x.n), [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assert.equal(stageAt(c, c.now).n, 9);
    assert.equal(stageAt(c, c.upheaval!.from * 1.01).n, 6);
    // The inherited panel in words.
    const keys = c.inherited.map((r) => r.key);
    for (const key of ["star", "metals", "c_o", "disc", "gas", "cluster", "al26", "seed"]) assert.ok(keys.includes(key), `inherited ${key}`);
    assert.ok(c.inherited.every((r) => r.value && !/NaN|undefined/.test(r.value)), JSON.stringify(c.inherited));
    assert.deepEqual(c.worked_at.map((r) => r.value), ["Standard", "drawn", "not here"]);
    // The drawing scale: log both ways.
    const xy = chartXY(c, 1000, 1000);
    assert.ok(Math.abs(xy.y(c.t_top)) < 1e-9 && Math.abs(xy.y(c.t_bottom) - 1000) < 1e-9);
    assert.ok(Math.abs(xy.t(xy.y(41)) - 41) < 1e-6);
    assert.ok(massWidth(1e-4) < massWidth(1) && massWidth(1) < massWidth(300));
  } finally {
    await k.close();
  }
});

test("an imported system has no formation history, and says why", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
    const a = await s.adopt({ import_v3: text("tests/fixtures/v3/synthetic-sol.json") });
    assert.ok(a.ok);
    const src = await formationOf(s, a.result.systems[0]!);
    assert.equal(src.kind, "none", JSON.stringify(src));
    if (src.kind === "none") {
      assert.equal(src.source, "v3");
      assert.match(src.why, /^This system came from an SSE 3 save/);
    }
    assert.equal((await formationOf(s, "u:kestrel/sys:99")).kind, "failed");
  } finally {
    await k.close();
  }
});
