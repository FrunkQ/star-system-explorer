// The editing grammar's check on parts (editing design section 3B; the Profile from the ring and belt editing design,
// question 4): every editing component is one of the grammar's parts, and every handle in the handle sheet opens a
// part the grammar has. A new component in packages/views/editing/src fails here until it is named under its part in
// packages/ui-contract/src/grammar.ts, so a new part is added on purpose, never by drift. The figures check is
// figures-ledger.test.ts; this one is about which parts exist.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readdirSync } from "node:fs";
import { GRAMMAR_PARTS, partOfScale } from "../../packages/ui-contract/src/grammar.ts";
import { HANDLES } from "../../packages/ui-contract/src/handles.ts";

const EDITING = new URL("../../packages/views/editing/src/", import.meta.url);
const components = readdirSync(EDITING).filter((n) => n.endsWith(".svelte")).sort();

test("every editing component is one of the grammar's parts, and only one", () => {
  const claimed = new Map<string, string[]>();
  for (const p of GRAMMAR_PARTS) for (const f of p.files) claimed.set(f, [...(claimed.get(f) ?? []), p.id]);
  for (const c of components) {
    const by = claimed.get(c) ?? [];
    assert.equal(by.length, 1, `${c} is ${by.length ? `claimed by ${by.join(" and ")}` : "not a part of the editing grammar: name it under its part in ui-contract/src/grammar.ts"}`);
  }
  for (const f of claimed.keys()) assert.ok(components.includes(f), `grammar.ts names ${f}, which is not in views/editing`);
});

test("the grammar has fifteen parts, the Profile among them", () => {
  assert.equal(GRAMMAR_PARTS.length, 15);
  assert.equal(new Set(GRAMMAR_PARTS.map((p) => p.id)).size, 15, "ids are unique");
  const profile = GRAMMAR_PARTS.find((p) => p.id === "profile");
  assert.deepEqual(profile?.files, ["Profile.svelte"]);
  assert.equal(partOfScale("profile")?.id, "profile");
  assert.equal(partOfScale("log")?.id, "strip");
  assert.equal(partOfScale("dial"), null);
});

test("every handle, and every part of every box, opens a part the grammar has", () => {
  for (const h of HANDLES) {
    assert.ok(partOfScale(h.part.scale), `${h.id}: scale "${h.part.scale}" is no part of the grammar`);
    for (const p of h.box?.parts ?? []) assert.ok(partOfScale(p.part.scale), `${h.id}.${p.id}: scale "${p.part.scale}" is no part of the grammar`);
  }
});
