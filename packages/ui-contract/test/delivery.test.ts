// Screen delivery (atlas design 11A.4a, AT15a): the five moments, the short form, reduced motion, the checks.
import { test } from "node:test";
import assert from "node:assert/strict";
import { DELIVERIES, DELIVERY_KINDS, MOMENTS, STARTER_DELIVERIES, TURN_MAX_MS, checkLookDelivery, planMs, resolveDelivery } from "../src/delivery.ts";

test("Hauler warms up then builds up on power on, builds up on arrival, collapses to a dot on power off", () => {
  const h = STARTER_DELIVERIES.hauler;
  const on = resolveDelivery(h, "power_on", { first: true });
  assert.equal(on.kind, "warm_up");
  assert.equal(on.ms, 2000);
  assert.equal(on.then?.kind, "build_up");
  assert.equal(on.then?.scale, "inside");
  assert.equal(planMs(on), 3400);
  assert.equal(resolveDelivery(h, "arrive", { first: true }).kind, "build_up");
  const off = resolveDelivery(h, "power_off");
  assert.equal(off.kind, "collapse_dot");
  assert.equal(off.exit, true);
  assert.equal(resolveDelivery(h, "leave").kind, "none");
});

test("long the first time, short after; Turn is always short and never over the cap", () => {
  const h = STARTER_DELIVERIES.hauler;
  assert.equal(resolveDelivery(h, "arrive", { first: true }).ms, DELIVERIES.build_up.ms);
  const later = resolveDelivery(h, "arrive", { first: false });
  assert.equal(later.ms, DELIVERIES.build_up.short_ms);
  assert.equal(later.then, undefined);
  assert.ok(resolveDelivery(h, "turn", { first: true }).ms <= TURN_MAX_MS);
  assert.ok(resolveDelivery({ turn: { kind: "fade", short_ms: 3000 } }, "turn").ms <= TURN_MAX_MS);
});

test("reduced motion makes every delivery a plain fade; none stays none", () => {
  for (const name of Object.keys(STARTER_DELIVERIES) as (keyof typeof STARTER_DELIVERIES)[]) {
    for (const m of MOMENTS) {
      const p = resolveDelivery(STARTER_DELIVERIES[name], m, { first: true, reducedMotion: true });
      assert.ok(p.kind === "fade" || p.kind === "none", `${name} ${m}: ${p.kind}`);
      assert.ok(p.ms <= 150);
      assert.equal(p.then, undefined);
    }
  }
});

test("a moment left out arrives by fade and leaves with nothing; a wrong-way delivery falls back", () => {
  assert.equal(resolveDelivery(undefined, "arrive").kind, "fade");
  assert.equal(resolveDelivery(undefined, "power_off").kind, "none");
  assert.equal(resolveDelivery({ power_off: { kind: "warm_up" } }, "power_off").kind, "none");
  assert.equal(resolveDelivery({ arrive: { kind: "collapse_dot" } }, "arrive").kind, "fade");
});

test("stepping back turns the page the other way", () => {
  assert.equal(resolveDelivery(STARTER_DELIVERIES.paper, "turn").kind, "page_turn");
  const fwd = resolveDelivery({ turn: { kind: "wipe", direction: "right" } }, "turn");
  const back = resolveDelivery({ turn: { kind: "wipe", direction: "right" } }, "turn", { back: true });
  assert.equal(fwd.direction, "right");
  assert.equal(back.direction, "left");
});

test("every starter passes the checks; the checks say what is wrong in words", () => {
  for (const [name, look] of Object.entries(STARTER_DELIVERIES)) assert.deepEqual(checkLookDelivery(look), [], name);
  const errs = checkLookDelivery({ arrive: { kind: "sparkle" }, power_off: { kind: "type_on" }, turn: { kind: "fade", ms: 99999, wobble: 1 }, later: { kind: "fade" } });
  assert.ok(errs.some((e) => /"sparkle" is not a delivery/.test(e)));
  assert.ok(errs.some((e) => /Type on only plays as a picture arrives/.test(e)));
  assert.ok(errs.some((e) => /turn\.ms must be 0 to/.test(e)));
  assert.ok(errs.some((e) => /unknown setting "wobble"/.test(e)));
  assert.ok(errs.some((e) => /"later" is not a moment/.test(e)));
});

test("every kind is catalogued with words, an explanation and at least one scale", () => {
  for (const k of DELIVERY_KINDS) {
    const d = DELIVERIES[k];
    assert.equal(d.kind, k);
    assert.ok(d.words && d.says && d.scales.length);
    assert.ok(d.short_ms <= d.ms || d.kind === "page_turn");
  }
});
