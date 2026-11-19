import { test } from "node:test";
import assert from "node:assert/strict";
import { TOKENS } from "../src/schema.ts";
import { THEMES } from "../src/themes.ts";
import { resolveTheme, themeCss } from "../src/resolve.ts";
import { contrast } from "../src/contrast.ts";

test("both modes define every token, and nothing else", () => {
  for (const t of THEMES) {
    const r = resolveTheme(t.id);
    for (const s of TOKENS) assert.ok(s.name in r, `${t.id} lacks ${s.name}`);
    assert.doesNotThrow(() => themeCss(t.id));
  }
});

test("night chart keeps chart room's structure: it changes only colours and shadow", () => {
  const night = THEMES.find((t) => t.id === "night-chart")!;
  const kinds = new Map(TOKENS.map((s) => [s.name, s.type]));
  for (const k of Object.keys(night.tokens)) assert.ok(["colour", "shadow"].includes(kinds.get(k)!), `night chart overrides ${k}`);
});

test("text tokens reach WCAG AA (4.5:1) and line tokens 3:1 on their grounds, in both modes", () => {
  const rows: string[] = [];
  for (const t of THEMES) {
    const r = resolveTheme(t.id);
    for (const s of TOKENS) {
      for (const g of s.text_on ?? []) {
        const c = contrast(String(r[s.name]), String(r[g]));
        rows.push(`${t.id} ${s.name} on ${g}: ${c.toFixed(2)}`);
        assert.ok(c >= 4.5, `${t.id}: ${s.name} on ${g} is ${c.toFixed(2)}:1`);
      }
      for (const g of s.line_on ?? []) {
        const c = contrast(String(r[s.name]), String(r[g]));
        rows.push(`${t.id} ${s.name} line on ${g}: ${c.toFixed(2)}`);
        assert.ok(c >= 3, `${t.id}: ${s.name} on ${g} is ${c.toFixed(2)}:1`);
      }
    }
  }
  if (process.env.SSE_PRINT) console.log(rows.join("\n"));
});

test("inheritance loops and unknown tokens are rejected", () => {
  const a = { ...THEMES[0]!, id: "a", extends: "b" };
  const b = { ...THEMES[0]!, id: "b", extends: "a" };
  assert.throws(() => resolveTheme("a", [a, b]), /loop/);
  const odd = { ...THEMES[0]!, id: "odd", tokens: { ...THEMES[0]!.tokens, sparkle: 1 } };
  assert.throws(() => themeCss("odd", [odd]), /unknown token/);
});
