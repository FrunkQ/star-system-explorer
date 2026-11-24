// SV2: the "Known to" logic, against the fixture the kernel (SV1) tests with too.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { checkIntent, intentToRequest, type Intent } from "../src/intent.ts";
import { decide, gmSeat, tableSeat } from "../src/seat.ts";
import { eyesShow, fieldLandmark, hasException, lineChange, holdReveal, knownOverall, pressedLandmark, secrecyRelevant, tableOf, type KnownTo } from "../src/known.ts";

const fx = JSON.parse(readFileSync(new URL("../../../tests/fixtures/sv/known-to.json", import.meta.url), "utf8")) as {
  answers: { ask: { address: string; field?: string }; value: KnownTo }[];
  changes: { intent: Intent; request: Record<string, unknown> }[];
};
const plain = (x: unknown) => JSON.parse(JSON.stringify(x, (_k, v) => (typeof v === "bigint" ? Number(v) : v)));
const tarn = fx.answers[0]!.value;
const base = fx.answers[2]!.value;

test("the table's landmark for a thing and for one fact, with its exception", () => {
  const t = tableOf(tarn)!;
  assert.equal(t.audience, "everyone");
  assert.equal(t.landmark, "known");
  assert.equal(fieldLandmark(t, "population"), "gm_only", "kept back");
  assert.equal(fieldLandmark(t, "mass"), "known", "a fact follows its thing");
  assert.equal(knownOverall(base), "gm_only");
  assert.equal(tableOf({ address: "u:x", at_s: 0, audiences: [] }), null);
});

test("one press reveals, a press on an open eye hides", () => {
  assert.equal(pressedLandmark("gm_only"), "known");
  assert.equal(pressedLandmark("heard_of"), "known");
  assert.equal(pressedLandmark("known"), "gm_only");
});

test("eyes show on Build and Run once secrecy matters, never on Explore", () => {
  const plainTarn: KnownTo = { ...tarn, audiences: tarn.audiences.map((a) => ({ ...a, exceptions: [] })) };
  assert.equal(secrecyRelevant(plainTarn, false), false, "all known and not shared: nothing to show");
  assert.equal(secrecyRelevant(plainTarn, true), true, "shared");
  assert.equal(secrecyRelevant(tarn, false), true, "an exception");
  assert.equal(secrecyRelevant(base, false), true, "a secret");
  assert.equal(eyesShow("explore", true), false);
  assert.equal(eyesShow("build", true), true);
  assert.equal(eyesShow("run", true), true);
  assert.equal(eyesShow("run", false), false);
});

test("set_known: each fixture change checks and becomes the fixture's request; players never send it", () => {
  for (const c of fx.changes) {
    assert.equal(checkIntent(c.intent), null, JSON.stringify(c.intent));
    assert.deepEqual(plain(intentToRequest(c.intent)), c.request);
    assert.equal(decide(gmSeat(), c.intent).action, "apply");
    assert.equal(decide({ ...tableSeat(), may_send: ["set_known"] }, c.intent).action, "refuse");
  }
  const one = fx.changes[0]!.intent as Extract<Intent, { kind: "set_known" }>;
  assert.match(checkIntent({ ...one, landmark: null }) ?? "", /only a fact's exception/);
  assert.match(checkIntent({ ...one, scope: { list: [] } }) ?? "", /names addresses/);
  assert.deepEqual(plain(intentToRequest({ ...one, from_yr: 1 })).from_s, plain(intentToRequest({ ...one, at_yr: 1 })).at_s, "from a later date");
});

test("the two-second hold: Undo before it ends sends nothing; after, it is too late; no players, no hold", () => {
  let due: (() => void) | null = null;
  const timer = { set: (f: () => void) => ((due = f), 1), clear: () => (due = null) };
  let sent = 0;
  const a = holdReveal(() => sent++, { hold: true, timer });
  assert.equal(sent, 0);
  assert.equal(a.undo(), true);
  assert.equal(due, null);
  assert.equal(sent, 0, "never sent");
  const b = holdReveal(() => sent++, { hold: true, timer });
  (due as unknown as () => void)();
  assert.equal(sent, 1);
  assert.equal(b.undo(), false, "already sent");
  const c = holdReveal(() => sent++, { hold: false, timer });
  assert.equal(sent, 2, "sent at once");
  assert.equal(c.sent, true);
  const d = holdReveal(() => sent++, { hold: true, timer });
  d.flush();
  assert.equal(sent, 3, "flush sends now");
  d.flush();
  assert.equal(sent, 3, "once");
});

test("a line: withheld past its rung is pencil; a press makes an exception, and pressing it back follows the thing again", () => {
  const t = { ...tableOf(tarn)!, withheld: ["population", "atmosphere.pressure"] };
  assert.equal(fieldLandmark(t, "atmosphere.pressure"), "gm_only", "past its rung");
  assert.equal(hasException(t, "population"), true);
  assert.equal(hasException(t, "atmosphere.pressure"), false);
  assert.equal(lineChange(t, "population", "known"), "known", "still withheld by the exception's own list: an exception");
  const kept = { ...t, withheld: [] };
  assert.equal(lineChange(kept, "population", "known"), null, "back to its thing: drop the exception");
  assert.equal(lineChange(kept, "mass", "gm_only"), "gm_only", "keep one fact back");
  assert.equal(lineChange(t, "atmosphere.pressure", "known"), "known", "show one fact early");
});

test("set_known and undo_known carry exact seconds when given (a year in floating point is only good to half a minute)", () => {
  const at_s = "145480536000000017";
  const set: Intent = { kind: "set_known", address: "u:k/sys:1/body:3", audience: "everyone", landmark: "known", scope: "this", at_yr: 4.61e9, at_s };
  assert.equal(checkIntent(set), null);
  assert.equal((intentToRequest(set) as { at_s: bigint }).at_s, 145480536000000017n);
  const undo: Intent = { kind: "undo_known", undo: "abc", at_yr: 4.61e9, at_s };
  assert.equal((intentToRequest(undo) as { at_s: bigint }).at_s, 145480536000000017n);
  assert.match(checkIntent({ ...set, at_s: "4.6e9" }) ?? "", /whole seconds/);
});
