// J5 parity: the chart room's approval rule and the kernel's intent registry
// give the same action for the same seat and intent. The kernel half reads
// the same file (crates/sse-trigger/src/intent_tests.rs).
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { decide, type Seat } from "../src/seat.ts";
import type { Intent } from "../src/intent.ts";

const cases = JSON.parse(readFileSync(new URL("../../../tests/fixtures/intents/decide-cases.json", import.meta.url), "utf8")) as {
  cases: { name: string; seat: Seat; intent: Intent; action: string }[];
};

test("seat.ts decide matches the shared approval cases", () => {
  assert.ok(cases.cases.length >= 10);
  for (const c of cases.cases) assert.equal(decide(c.seat, c.intent).action, c.action, c.name);
});
