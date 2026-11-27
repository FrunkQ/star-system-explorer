#!/usr/bin/env node
// Bundle a store pack kept as a folder into the one file the Open door takes (CM0). The rule is
// packages/ui-contract/src/pack-bundle.ts: every file pack.json names under provides.other.files is held under
// provides.other.contents by its path. Writes nothing into the repo unless told to: the file is what goes on
// Explorers, built fresh from the folder each time.
// Usage: node scripts/bundle-pack.mjs <pack folder> <out.json>
//   e.g. node scripts/bundle-pack.mjs store-packs/sol-today ../sol-today-0.1.0.json
import { readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { join, relative, sep } from "node:path";
import { bundlePack } from "../packages/ui-contract/src/pack-bundle.ts";

const [folder, out] = process.argv.slice(2);
if (!folder || !out) {
  console.error("usage: node scripts/bundle-pack.mjs <pack folder> <out.json>");
  process.exit(2);
}
const readJson = (p) => {
  try {
    return JSON.parse(readFileSync(join(folder, p), "utf8"));
  } catch {
    return null;
  }
};
const walk = (dir) =>
  readdirSync(join(folder, dir)).flatMap((n) => {
    const full = join(folder, dir, n);
    return statSync(full).isDirectory() ? walk(relative(folder, full)) : n.endsWith(".json") ? [relative(folder, full).split(sep).join("/")] : [];
  });
const r = bundlePack(readJson("pack.json"), readJson, (d) => {
  try {
    return walk(d.replace(/\/$/, ""));
  } catch {
    return [];
  }
});
if (r.problems.length) {
  console.error(`not bundled:\n  ${r.problems.join("\n  ")}`);
  process.exit(1);
}
const text = JSON.stringify(r.pack);
writeFileSync(out, text);
console.log(`${out}: ${r.paths.length} files held, ${(text.length / 1e6).toFixed(2)} MB`);
