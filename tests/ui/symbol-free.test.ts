// V12 acceptance: no view contains encoding code. A view asks the symbology
// for a mark (ctx.mark) or a legend (legendFor) and draws what comes back;
// it never decides what a class, kind or tag looks like: no shape or pattern
// names in its script, no reading of class tags, no colour per class.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { BODY_SHAPES, PATTERNS } from "../../packages/ui-contract/src/symbology.ts";

const root = new URL("../../", import.meta.url).pathname;
function files(dir: string): string[] {
  return readdirSync(dir).flatMap((n) => {
    const p = join(dir, n);
    if (statSync(p).isDirectory()) return n === "node_modules" || n === "test" ? [] : files(p);
    return /\.(svelte|ts)$/.test(n) ? [p] : [];
  });
}

const NAMES = [...BODY_SHAPES.filter((s) => s !== "disc"), ...PATTERNS.filter((p) => p !== "solid")];

/** Encoding decisions in one source file's script. */
export function encodingLeaks(src: string): string[] {
  const script = (/<script/.test(src) ? [...src.matchAll(/<script[^>]*>([\s\S]*?)<\/script>/g)].map((m) => m[1]!).join("\n") : src).replace(/\/\/.*$/gm, "").replace(/\/\*[\s\S]*?\*\//g, "");
  const markup = src.replace(/<script[^>]*>[\s\S]*?<\/script>/g, "").replace(/<style[^>]*>[\s\S]*?<\/style>/g, "");
  const out: string[] = [];
  for (const n of NAMES) if (new RegExp(`["'\`]${n}["'\`]`).test(script)) out.push(`decides a look: "${n}"`);
  if (/["'`]class:/.test(script) || /["'`]class:/.test(markup)) out.push("reads class tags");
  if (/\.(shape|fill_pattern|colour)\s*===?\s*["'`]/.test(script + markup)) out.push("branches on a mark's look");
  return out;
}

test("the checker catches encoding in a view", () => {
  assert.equal(encodingLeaks(`<script>const look = s.tags.includes("class:planet/ocean") ? "band" : "solid";</script>`).length, 2);
  assert.equal(encodingLeaks(`<script>if (m.shape === "star") r = 9;</script>`).length, 1);
  assert.deepEqual(encodingLeaks(`<script>const m = ctx.mark(s);</script><Mark mark={m} />`), []);
});

test("no view contains encoding code", () => {
  const bad: string[] = [];
  for (const f of files(join(root, "packages/views"))) for (const l of encodingLeaks(readFileSync(f, "utf8"))) bad.push(`${f.slice(root.length)}: ${l}`);
  assert.deepEqual(bad, []);
});
