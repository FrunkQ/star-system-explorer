// A2 part 5 acceptance: Orac creates a missing substance as data sheet rows.
// The kernel's data sheet checks refuse bad rows in its own words; a real
// substance already on the sheet is pointed to; invented matter goes into a
// pack marked fictional; nothing is stacked until accept. The handbook and
// the sheet are in Orac's catalogue.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Session } from "../../ui-contract/src/session.ts";
import { gmSeat } from "../../ui-contract/src/seat.ts";
import type { PackDoc } from "../../ui-contract/src/index.ts";
import { nodeKernel } from "../../ui-contract/test/node-kernel.ts";
import { plainWorkspace } from "../../ui-contract/test/workspace.ts";
import { asker, emptyAgents, type AgentsState } from "../src/agents.ts";
import { runOrac } from "../src/orac.ts";
import { FICTIONAL_REFERENCE_PACK, REFERENCE_PACK, handbookEntries, sheetEntries, substanceTarget } from "../src/orac-sheet.ts";
// @ts-expect-error plain JS test helper
import { mockProvider } from "./mock-provider.mjs";

const root = new URL("../../../", import.meta.url);
const json = (p: string) => JSON.parse(readFileSync(new URL(p, root), "utf8"));
const base: PackDoc = json("packs/base-physics/pack.json");
const sheet: PackDoc = json("packs/base-reference/pack.json");
const scene = json("packages/shell/src/documents/kestrel.scene.json");
const fence = (o: unknown) => "```json\n" + JSON.stringify(o) + "\n```";
const made = { kind: "", doc: null };

// Neon is on the sheet; krypton difluoride is not. Values from the NIST WebBook entry (unverified here).
const kf2 = (formula: Record<string, number>, src = "nist-webbook") => ({
  substances: [{ id: "krf2", formula, names: { physics: "krypton difluoride", plain: "krypton difluoride", story: "krypton difluoride" }, molar_mass: { v: 121.795, unit: "g/mol", src, check: "unverified" } }],
});

test("substances: the data sheet's checks refuse bad rows; accept stacks the campaign's reference pack", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), scene.universe, [base, sheet], k.scratch);
    const t = substanceTarget(s, plainWorkspace(scene.now_yr));
    const stack0 = s.stack().hash;
    assert.match((await t.check({ substances: [{ id: "ne" }] }, made)).problems[0]!, /^ne is already on the data sheet \(neon\): use it/);
    const unknownElement = await t.check(kf2({ kr: 1, fx: 2 }), made);
    assert.ok(unknownElement.problems.some((p) => /formula names element fx/.test(p)), unknownElement.problems.join("; "));
    const wrongMass = await t.check(kf2({ kr: 1, f: 3 }), made);
    assert.ok(wrongMass.problems.some((p) => /molar mass .* disagrees with its formula/.test(p)), wrongMass.problems.join("; "));
    const noSource = await t.check(kf2({ kr: 1, f: 2 }, "nobody2026"), made);
    assert.ok(noSource.problems.some((p) => /nobody2026/.test(p)), noSource.problems.join("; "));
    const good = await t.check(kf2({ kr: 1, f: 2 }), made);
    assert.deepEqual(good.problems, []);
    assert.equal(s.stack().hash, stack0, "checking stacks nothing");
    assert.match(t.summary(good.value!), /^krypton difluoride \(KrF2, 1 property\)\. Accept adds it to your setting's data sheet/);
    assert.equal(await t.accept(good.value!), null);
    assert.ok(s.stack().added.some((p) => p.id === REFERENCE_PACK && !p.fictional));
    const r = await s.readMatter({ table: "substances", id: "krf2" });
    assert.ok(r.ok, "the kernel now has the row");
    assert.ok(sheetEntries(s).some((e) => e.key === "substance:krf2"));
    assert.match((await t.check(kf2({ kr: 1, f: 2 }), made)).problems[0]!, /^krf2 is already on the data sheet/);
  } finally {
    await k.close();
  }
});

test("substances: invented matter goes into a pack marked fictional, with its own source row", async () => {
  const k = nodeKernel();
  try {
    const s = await Session.open(k.client, gmSeat(), scene.universe, [base, sheet], k.scratch);
    const t = substanceTarget(s, plainWorkspace(scene.now_yr));
    const doc = {
      fictional: true,
      sources: [{ id: "mine.invented", text: "Invented for this setting: not real matter", status: "authored" }],
      substances: [{ id: "mine.lumenite", formula: { si: 1, o: 2, ne: 1 }, names: { physics: "lumenite", plain: "lumenite", story: "lumenite" }, phases: { density_solid: { v: 2900, unit: "kg m-3", src: "mine.invented", check: "authored" } } }],
    };
    const good = await t.check(doc, made);
    assert.deepEqual(good.problems, []);
    assert.match(t.summary(good.value!), /Marked fictional/);
    assert.equal(await t.accept(good.value!), null);
    const p = s.stack().added.find((x) => x.id === FICTIONAL_REFERENCE_PACK);
    assert.equal(p?.fictional, true);
  } finally {
    await k.close();
  }
});

test("Orac drafts a missing gas: the kernel's refusal goes back, the fix is offered; the handbook is in the catalogue", async () => {
  const k = nodeKernel();
  const p = await mockProvider({ replies: (m: { content: string }[]) => fence({ note: "Krypton difluoride.", make: [{ kind: "substance", title: "Krypton difluoride", doc: kf2(/disagrees/.test(m.at(-1)!.content) ? { kr: 1, f: 2 } : { kr: 1, f: 3 }) }] }) });
  try {
    const s = await Session.open(k.client, gmSeat(), scene.universe, [base, sheet], k.scratch);
    const st: AgentsState = { ...emptyAgents(), providers: [{ id: "x", kind: "other", label: "X", base: p.base }], links: { building: { provider: "x", model: "m" } } };
    const guide = handbookEntries();
    assert.ok(["method", "blocks", "packs", "data-sheet", "honesty", "secrecy", "tools"].every((id) => guide.some((e) => e.key === `guide:${id}`)));
    const r = await runOrac({ request: "Add krypton difluoride for this world.", targets: [substanceTarget(s, plainWorkspace(scene.now_yr))], catalogue: [...guide, ...sheetEntries(s)], ask: asker(st, "building") });
    assert.equal(r.ok, true, r.message);
    assert.match(r.rounds[0]!.problems[0]!, /disagrees with its formula/);
    assert.equal(r.passed.length, 1);
    assert.ok(!s.stack().added.some((x) => x.id === REFERENCE_PACK), "nothing stacked before accept");
  } finally {
    await p.close();
    await k.close();
  }
});
