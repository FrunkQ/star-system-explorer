// U0 rule, as a check: the GM shell and every view reach the kernel only
// through the client API (packages/ui-contract: src/index.ts for everything,
// src/host-browser.ts for starting a session). No file under the checked
// folders may import the kernel client, the protocol types or the wasm
// glue, or name a kernel request.
import { readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join, relative, resolve, sep } from "node:path";

export const CHECKED = ["packages/shell/src", "packages/views", "packages/chart-grammar/src", "packages/skins", "packages/renderer/src", "packages/sound/src"];
export const CLIENT_API = ["packages/ui-contract/src/index.ts", "packages/ui-contract/src/host-browser.ts"];
// three.js is the decided renderer (SSE4-surfaces-and-rendering.md, accepted 18 Nov).
const PACKAGES_OK = [/^svelte(\/|$)/, /^@fontsource\//, /^three(\/|$)/];
const FORBIDDEN_TEXT = [/\bcallRaw\b/, /\bKernelClient\b/, /["']?op["']?\s*:\s*["'](set_authored|open_universe|load_stack|commit|branch|derive|resolve)["']/, /sse_wasm/];

export interface Breach {
  file: string;
  line: number;
  why: string;
}

const SPEC = /(?:import|export)\s[^'"]*?from\s*["']([^"']+)["']|import\s*\(\s*["']([^"']+)["']\s*\)|import\s+["']([^"']+)["']|new\s+URL\s*\(\s*["']([^"']+)["']/g;

/** Check one file's text. `file` is repo-relative with forward slashes. */
export function checkSource(root: string, file: string, text: string): Breach[] {
  const out: Breach[] = [];
  const lineOf = (i: number) => text.slice(0, i).split("\n").length;
  for (const m of text.matchAll(SPEC)) {
    const spec = (m[1] ?? m[2] ?? m[3] ?? m[4])!;
    const line = lineOf(m.index!);
    if (spec.startsWith(".")) {
      const target = relative(root, resolve(root, dirname(file), spec)).split(sep).join("/");
      const inChecked = CHECKED.some((c) => target === c || target.startsWith(c + "/"));
      const isApi = CLIENT_API.includes(target);
      if (!inChecked && !isApi) out.push({ file, line, why: `imports ${target}, outside the client API` });
    } else if (!PACKAGES_OK.some((r) => r.test(spec))) {
      out.push({ file, line, why: `imports package ${spec}; only svelte, fonts and three are allowed here` });
    }
  }
  text.split("\n").forEach((l, i) => {
    for (const r of FORBIDDEN_TEXT) if (r.test(l)) out.push({ file, line: i + 1, why: `names a kernel internal (${r.source})` });
  });
  return out;
}

function walk(dir: string): string[] {
  return readdirSync(dir).flatMap((n) => {
    if (n === "node_modules" || n === "dist" || n === "public") return [];
    const p = join(dir, n);
    return statSync(p).isDirectory() ? walk(p) : /\.(ts|js|mjs|svelte)$/.test(n) && !n.endsWith(".d.ts") ? [p] : [];
  });
}

export function checkRepo(root: string): { files: number; breaches: Breach[] } {
  const files = CHECKED.flatMap((c) => walk(join(root, c))).filter((f) => !f.includes(`${sep}test${sep}`) && !f.endsWith("vite.config.ts"));
  const breaches = files.flatMap((f) => checkSource(root, relative(root, f).split(sep).join("/"), readFileSync(f, "utf8")));
  return { files: files.length, breaches };
}
