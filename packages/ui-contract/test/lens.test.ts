// U9 acceptance in Node: lenses. The shipped defaults are valid; every player
// data view and every fog-of-war audience is a lens; a lens saved, exported in
// a pack and reloaded is identical; card rules order and hide fields without
// touching them; honesty stamps cannot be hidden on the GM view but can be on
// player views (the kernel's scene through a data view without show_honesty).
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { enterFacts, observeOf, type AuthoredFacts } from "../src/data-views.ts";
import { addSeat, defaultPlan, fogViews } from "../src/seats.ts";
import type { PackDoc } from "../src/physics.ts";
import { SHIPPED_LENSES, allLenses, cardFacts, cardFor, checkLens, copyLens, defaultLens, gmStamps, hasLayer, lensAudience, lensFits, lensPack, lensesFromPack, playerLenses, saved, type Lens } from "../src/lens.ts";
import { nodeKernel } from "./node-kernel.ts";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const base: PackDoc = json("packs/base-physics/pack.json");
const playerData: PackDoc = json("packages/shell/src/documents/kestrel.player-data.pack.json");
const facts: AuthoredFacts = json("packages/shell/src/documents/kestrel.player-facts.json");
const setup = observeOf([playerData])!.setup;

test("the shipped lenses are valid and distinct", () => {
  assert.deepEqual(
    SHIPPED_LENSES.map((l) => l.id),
    ["basic", "advanced", "building", "running"],
  );
  for (const l of SHIPPED_LENSES) assert.deepEqual(checkLens(l), [], l.id);
  assert.equal(defaultLens("basic"), "basic");
  assert.equal(defaultLens("capable"), "advanced");
  for (const l of SHIPPED_LENSES) assert.ok(lensFits(l, "basic"), `${l.id} runs on a phone`);
});

test("every player data view and every fog audience is a lens", () => {
  let plan = defaultPlan();
  const fog = fogViews(setup)!;
  plan = addSeat(plan, { label: "Empire: Vesh", fog });
  const ls = playerLenses(setup, plan);
  for (const v of setup.views) assert.ok(ls.some((l) => l.data === v.id), `a lens through ${v.id}`);
  assert.ok(ls.some((l) => l.view_as === "everyone" && !l.data));
  const vesh = ls.find((l) => l.label === "As Empire: Vesh sees it")!;
  assert.equal(lensAudience(vesh), vesh.view_as);
  for (const l of ls) assert.deepEqual(checkLens(l), [], l.id);
  // Shipped first, then player lenses; nothing listed twice.
  const all = allLenses([], [], setup, plan);
  assert.equal(new Set(all.map((l) => l.id)).size, all.length);
  assert.equal(all[0]!.id, "basic");
});

test("a lens saved, exported in a pack and reloaded is identical", () => {
  const mine = copyLens(SHIPPED_LENSES[2]!, "Political map", ["basic"]);
  const edited: Lens = { ...mine, layers: ["orbits", "labels", "known"], tools: ["undo"], cards: [{ subject_kind: "planet", fields: ["surface_k", "a_au"], density: "compact" }, { subject_kind: "*", density: "line" }] };
  assert.equal(edited.id, "mine.political-map");
  assert.deepEqual(checkLens(edited), []);
  const pack = lensPack([edited], { id: "mine.lenses" });
  const back = lensesFromPack(JSON.parse(JSON.stringify(pack)));
  assert.deepEqual(back.problems, []);
  assert.equal(back.lenses.length, 1);
  assert.deepEqual(saved(back.lenses[0]!), saved(edited));
  assert.equal(JSON.stringify(saved(back.lenses[0]!)), JSON.stringify(saved(edited)));
  // A pack's lens joins the switcher; a bad one is reported, not half loaded.
  assert.ok(allLenses([], [pack], setup).some((l) => l.id === edited.id && l.origin === "pack"));
  const bad = lensesFromPack({ provides: { other: { lenses: [{ ...saved(edited), layers: ["weather"] }] } } });
  assert.equal(bad.lenses.length, 0);
  assert.match(bad.problems[0]!, /unknown layer weather/);
});

test("card rules order and hide fields; hidden fields are untouched", () => {
  const facts = [{ field: "name" }, { field: "a_au" }, { field: "mass_earth" }, { field: "surface_k" }, { field: "albedo" }];
  const basic = SHIPPED_LENSES[0]!;
  assert.deepEqual(
    cardFacts(cardFor(basic, "planet"), facts).map((f) => f.field),
    ["a_au", "mass_earth", "surface_k"],
  );
  assert.deepEqual(cardFacts(cardFor(SHIPPED_LENSES[1]!, "planet"), facts), facts);
  assert.equal(facts.length, 5);
  assert.equal(cardFor(basic, "star").fields![0], "mass_sun");
});

test("honesty stamps always show on the GM view, whatever the lens", () => {
  const stamps = ["resolved", "authored", "fictional", "unsurveyed"];
  const quiet: Lens = { ...SHIPPED_LENSES[3]! };
  assert.ok(!hasLayer(quiet, "stamps"));
  assert.deepEqual(gmStamps(quiet, stamps), ["authored", "fictional"]);
  assert.deepEqual(gmStamps(SHIPPED_LENSES[2]!, stamps), stamps);
});

test("honesty marks can be hidden on player views (data views without show_honesty)", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "kestrel reach campaign" }, [base, playerData]);
    assert.equal(await enterFacts(s, facts), null);
    const scanned = setup.views.find((v) => v.id === "scanned")!;
    assert.notEqual(scanned.show_honesty, true);
    const r = await s.scene(facts.scope, { audience: "everyone", data_view: "scanned" });
    assert.ok(r.ok);
    // The recorded facts are authored by the GM, yet the players' scene carries no marks.
    assert.ok(r.scene.subjects.length > 1);
    for (const sub of r.scene.subjects) for (const f of sub.facts) assert.equal(f.marks, undefined, `${sub.address} ${f.field}`);
    // The same view with show_honesty on does carry them: the switch is the data view's, not the lens's.
    const loud = { ...setup, views: setup.views.map((v) => (v.id === "scanned" ? { ...v, show_honesty: true } : v)) };
    const r2 = await s.scene(facts.scope, { audience: "everyone", data_view: "scanned", observe: loud });
    assert.ok(r2.ok);
    assert.ok(r2.scene.subjects.some((sub) => sub.facts.some((f) => f.marks?.includes("authored"))));
  } finally {
    await k.close();
  }
});
