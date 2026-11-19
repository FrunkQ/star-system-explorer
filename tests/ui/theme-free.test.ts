// V2 acceptance: no view contains theme-specific code. Views draw only with
// theme tokens (`var(--sse-…)`) and frame parts; they never name a colour, a
// font or a theme, and never ask which theme is on.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { ALL_THEMES } from "../../packages/skins/player/src/index.ts";

const root = new URL("../../", import.meta.url).pathname;
function files(dir: string): string[] {
  return readdirSync(dir).flatMap((n) => {
    const p = join(dir, n);
    if (statSync(p).isDirectory()) return n === "node_modules" || n === "test" ? [] : files(p);
    return /\.(svelte|ts)$/.test(n) ? [p] : [];
  });
}

/** Theme-specific code in one source file: literal colours, fonts, theme names, theme checks. */
export function themeLeaks(src: string): string[] {
  const out: string[] = [];
  const css = [...src.matchAll(/<style[^>]*>([\s\S]*?)<\/style>/g)].map((m) => m[1]!).join("\n");
  const code = src.replace(/<style[^>]*>[\s\S]*?<\/style>/g, "").replace(/\/\/.*$/gm, "").replace(/<!--[\s\S]*?-->/g, "");
  for (const m of (css + "\n" + code).matchAll(/#[0-9a-fA-F]{6}\b|#[0-9a-fA-F]{3}\b(?![\w-])|\brgba?\(\s*\d/g)) out.push(`literal colour ${m[0]}`);
  for (const m of css.matchAll(/font-family:([^;]+);/g)) if (!/^(var\(|inherit)/.test(m[1]!.trim())) out.push(`literal font ${m[1]!.trim()}`);
  // Theme names in script (markup class names like "chart" are not theme checks).
  const script = /<script/.test(src) ? [...src.matchAll(/<script[^>]*>([\s\S]*?)<\/script>/g)].map((m) => m[1]!).join("\n") : src;
  for (const t of ALL_THEMES) if (new RegExp(`["'\`]${t.id}["'\`]`).test(script.replace(/\/\/.*$/gm, ""))) out.push(`names theme ${t.id}`);
  if (/dataset\.theme|data-theme/.test(code)) out.push("asks which theme is on");
  return out;
}

test("the checker catches leaks", () => {
  assert.equal(themeLeaks(`<style>.a { color: #ff0000; font-family: Georgia; }</style>`).length, 2);
  assert.equal(themeLeaks(`<script>if (document.documentElement.dataset.theme === "phosphor") x();</script>`).length, 2);
  assert.deepEqual(themeLeaks(`<style>.a { color: var(--sse-ink); font-family: var(--sse-title-font); }</style>`), []);
});

test("no view contains theme-specific code", () => {
  const bad: string[] = [];
  for (const f of files(join(root, "packages/views"))) for (const l of themeLeaks(readFileSync(f, "utf8"))) bad.push(`${f.slice(root.length)}: ${l}`);
  assert.deepEqual(bad, []);
});
