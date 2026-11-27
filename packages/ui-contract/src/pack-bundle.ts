// One file for a pack that is kept as a folder (CM0, compound ships design section 13). A store pack such as Sol
// today is written as a folder: `pack.json` names its data files under `provides.other.files` (a name for each, a
// trailing `/` for a folder of them). The one door every pack comes in by (Open, a dropped file, an Explorers link,
// then the stack) takes one `sse.pack/1` document, so the folder is bundled into one: every file the pack names is
// held under `provides.other.contents`, by its path in the folder. Nothing else changes, so a screen reads a file
// the same way whether the pack is a folder on a server or a document in the stack: `heldFiles(pack)("crews.json")`.
// No imports: the bundling script (scripts/bundle-pack.mjs) runs this file directly under node.

type Json = null | boolean | number | string | Json[] | { [k: string]: Json };

/** Where a pack's held files are kept inside it. */
export const CONTENTS = "contents";

function isObj(x: unknown): x is Record<string, unknown> {
  return !!x && typeof x === "object" && !Array.isArray(x);
}

/** A path inside a pack folder: relative, forward slashes, no way out of the folder. */
export function safePackPath(p: string): boolean {
  return p.length > 0 && !p.startsWith("/") && !p.includes("\\") && !p.split("/").some((s) => s === ".." || s === ".");
}

/** The files a folder pack names (`provides.other.files`), as paths; a folder is given with its trailing `/`. */
export function namedFiles(pack: unknown): string[] {
  const files = isObj(pack) && isObj(pack.provides) && isObj(pack.provides.other) ? pack.provides.other.files : undefined;
  if (!isObj(files)) return [];
  return [...new Set(Object.values(files).filter((v): v is string => typeof v === "string"))].sort();
}

export interface Bundled {
  pack: Json;
  /** Paths held, sorted. */
  paths: string[];
  /** Anything named that could not be held, in words. */
  problems: string[];
}

/**
 * One pack document from a folder pack. `read(path)` gives a file's JSON (null when it is missing or not JSON);
 * `list(dir)` gives the paths of the JSON files under a named folder, at any depth. The pack's own words, credits and
 * everything else are kept as they are; only `provides.other.contents` is added.
 */
export function bundlePack(pack: unknown, read: (path: string) => Json | null, list: (dir: string) => string[]): Bundled {
  const problems: string[] = [];
  if (!isObj(pack) || pack.format !== "sse.pack/1") return { pack: null, paths: [], problems: ["pack.json is not an sse.pack/1 document"] };
  const named = namedFiles(pack);
  if (!named.length) problems.push("pack.json names no files under provides.other.files");
  const held: Record<string, Json> = {};
  for (const n of named) {
    if (!safePackPath(n.replace(/\/$/, ""))) {
      problems.push(`${n}: a path must stay inside the pack folder`);
      continue;
    }
    const paths = n.endsWith("/") ? list(n) : [n];
    if (n.endsWith("/") && !paths.length) problems.push(`${n}: the folder holds no files`);
    for (const p of paths) {
      const doc = read(p);
      if (doc === null) problems.push(`${p}: missing, or not JSON`);
      else held[p] = doc;
    }
  }
  const paths = Object.keys(held).sort();
  const contents: Record<string, Json> = {};
  for (const p of paths) contents[p] = held[p]!;
  const provides = pack.provides as Record<string, unknown>;
  const other = { ...(provides.other as Record<string, unknown>), [CONTENTS]: contents };
  return { pack: { ...pack, provides: { ...provides, other } } as Json, paths, problems };
}

/** A reader over the files a pack holds, or null when it holds none (a folder pack not yet bundled, or any other pack). */
export function heldFiles(pack: unknown): ((path: string) => Json | null) | null {
  const c = isObj(pack) && isObj(pack.provides) && isObj(pack.provides.other) ? pack.provides.other[CONTENTS] : undefined;
  if (!isObj(c)) return null;
  return (path: string) => (Object.prototype.hasOwnProperty.call(c, path) ? (c[path] as Json) : null);
}
