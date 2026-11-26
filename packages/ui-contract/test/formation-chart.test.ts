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
import { howWorkedOut, storyFor, worldStory } from "../src/formation-story.ts";
import { chartFate, fatePlan, withFate, FATE_LIMIT_MYR } from "../src/formation-fate.ts";
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

    // F10b: a world's story. The card opens on the heaviest lasting world; any track can be chosen.
    const first = storyFor(c, null)!;
    assert.equal(first, [...lasting].sort((a, b) => b.m_end - a.m_end)[0]!.id);
    assert.equal(storyFor(c, lasting[0]!.address!), lasting[0]!.id, "a selected world opens its own story");
    for (const t of c.tracks) {
      const st = worldStory(src.run, c, t.id)!;
      assert.ok(st, t.id);
      const shares = st.material.reduce((a, m) => a + m.share, 0);
      assert.ok(st.material.length === 0 || Math.abs(shares - 1) < 1e-9, `${t.id}: material shares sum to one (${shares})`);
      assert.ok(st.lines.length >= 1 && st.lines.every((l, i) => i === 0 || l.t >= st.lines[i - 1]!.t), `${t.id}: its story in time order`);
      assert.ok(!/NaN|undefined/.test(JSON.stringify(st)), JSON.stringify(st).slice(0, 300));
    }
    // A world that swallowed another carries its share "from" that body, and its impact is drawn, not calculated.
    const target = c.tracks.find((t) => t.end.how === "merged")!;
    const host = worldStory(src.run, c, target.end.how === "merged" ? target.end.into : "")!;
    assert.ok(host.material.some((m) => m.kind === "impactor" && m.key === target.id), "the merged body's share is named");
    assert.ok(host.sure.some((r) => r.label.startsWith("Impact") && r.value === "drawn"));
    assert.ok(host.lines.some((l) => l.violent), "its impact is in its story");
    // Every lasting world's story says where it is now (what is still to come follows it).
    for (const t of lasting) assert.ok(worldStory(src.run, c, t.id)!.lines.some((l) => /^now at /.test(l.text) && l.t === c.now));
    assert.deepEqual(howWorkedOut(src.run).map((r) => r.value), ["tabulated", "calculated, stepped in time", "drawn from calibrated odds", "calculated", "not yet"]);
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

test("the star's end: a Sun-like star swells, swallows its near worlds, widens the rest, and the chart ends there", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, packs);
    // u:kestrel/sys:5 grows a 0.74 Sun star with six worlds; u:kestrel/sys:3 a red dwarf that outlives the chart.
    const red = await s.runFormation({ seed: "first light", system: "u:kestrel/sys:3" });
    assert.ok(red.ok);
    const redPlan = fatePlan(red.result, formationChart(red.result));
    assert.equal(redPlan.kind, "outlives");
    assert.match(redPlan.kind === "outlives" ? redPlan.words : "", /trillion years/);

    const r = await s.runFormation({ seed: "first light", system: "u:kestrel/sys:5" });
    assert.ok(r.ok);
    const c = formationChart(r.result, { seed: "first light" });
    const plan = fatePlan(r.result, c);
    assert.equal(plan.kind, "ahead");
    if (plan.kind !== "ahead") return;
    const lasting = c.tracks.filter((t) => t.end.how === "lasts");
    assert.equal(plan.spec.planets.length, lasting.length, "every world that lasts is aged");
    assert.ok(plan.target_myr > plan.end_myr && plan.end_myr < FATE_LIMIT_MYR);
    const h = await s.stateHash();
    const la = await s.lookAhead(plan.spec, plan.target_myr);
    assert.ok(la.ok, JSON.stringify(la));
    if (!la.ok) return;
    assert.ok(await s.discardBranch(la.result.branch));
    assert.equal(await s.stateHash(), h, "looking at the star's end writes nothing");

    const f = chartFate(c, plan, la.result);
    assert.ok(f.ms_end > c.now && f.ms_end < f.end && f.end <= f.until, `${f.ms_end} ${f.end} ${f.until}`);
    assert.equal(f.remnant, "white dwarf");
    assert.ok(f.star.some((x) => x.r_au > 0.5), "the giant reaches out past half an AU");
    const gone = f.worlds.filter((w) => w.engulfed !== undefined);
    const kept = f.worlds.filter((w) => w.engulfed === undefined);
    assert.ok(gone.length > 0 && kept.length > 0, `some swallowed (${gone.length}), some left (${kept.length})`);
    // Swallowed worlds are the near ones, each where the star's surface reached it.
    assert.ok(Math.max(...gone.map((w) => w.pts[0]!.a)) < Math.min(...kept.map((w) => w.pts[0]!.a)));
    for (const w of gone) {
      const surf = f.star.filter((x) => x.t <= w.engulfed!).pop()!;
      assert.ok(surf.r_au >= w.pts[w.pts.length - 1]!.a * 0.5, `${w.label} is swallowed where the star is (${surf.r_au} vs ${w.pts.at(-1)!.a})`);
    }
    // The survivors' orbits widen as the star loses mass: a(after) / a(before) is about M(before) / M(after).
    const m0 = f.star[0]!.m;
    const m1 = f.star.at(-1)!.m;
    for (const w of kept) {
      const ratio = w.pts.at(-1)!.a / w.pts[0]!.a;
      assert.ok(Math.abs(ratio - m0 / m1) < 0.15 * (m0 / m1), `${w.label} widens ${ratio} for a mass ratio ${m0 / m1}`);
    }
    assert.ok(!/NaN|undefined/.test(JSON.stringify(f)), f.text);

    // The chart runs on to the end; swallowed tracks stop there; the story says so.
    const g = withFate(c, f);
    assert.ok(g.t_bottom >= f.until && g.t_bottom >= c.t_bottom);
    assert.equal(g.tracks.filter((t) => t.end.how === "engulfed").length, gone.length);
    for (const t of g.tracks) for (let i = 1; i < t.segs.length; i++) assert.ok(t.segs[i]!.t0 >= t.segs[i - 1]!.t0, `${t.id} runs down the page`);
    for (const t of g.tracks.filter((x) => x.end.how === "lasts")) assert.ok(Math.abs(t.segs.at(-1)!.t1 - g.t_bottom) < 1e-6 * g.t_bottom);
    assert.ok(g.events.some((e) => e.kind === "engulfed"));
    const st = worldStory(r.result, g, gone[0]!.id)!;
    assert.ok(st.lines.some((l) => /^now at /.test(l.text)), "a world to be swallowed is still there now");
    assert.ok(st.lines.at(-1)!.violent && /swallowed/.test(st.lines.at(-1)!.text), JSON.stringify(st.lines.at(-1)));
    const kt = worldStory(r.result, g, kept[0]!.id)!;
    assert.match(kt.lines.at(-1)!.text, /white dwarf; its orbit has widened/);
    assert.equal(storyFor(g, null), [...g.tracks].filter((t) => t.end.how === "lasts" || t.end.how === "engulfed").sort((a, b) => b.m_end - a.m_end)[0]!.id);
  } finally {
    await k.close();
  }
});
