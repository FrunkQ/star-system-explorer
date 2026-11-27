// CM0: a pack kept as a folder (Sol today) becomes the one file the Open door takes, and that file goes into a
// campaign the way any pack does: the file is recognised as a pack, passes the pack check, and the browser kernel
// stacks it with its calendar. The bundling rule is pack-bundle.ts; scripts/bundle-pack.mjs is the same rule on disk.
import { test } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, readdirSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, relative, sep } from "node:path";
import { bundlePack, checkPackDoc, heldFiles, namedFiles, safePackPath, sniffFile, type Json } from "../src/index.ts";
import { Session } from "../src/session.ts";
import { gmSeat } from "../src/seat.ts";
import { nodeKernel } from "./node-kernel.ts";

const ROOT = new URL("../../../", import.meta.url).pathname;
const PACK = join(ROOT, "store-packs/sol-today");
const readJson = (p: string): Json | null => {
  try {
    return JSON.parse(readFileSync(join(PACK, p), "utf8")) as Json;
  } catch {
    return null;
  }
};
const walk = (dir: string): string[] =>
  readdirSync(join(PACK, dir)).flatMap((n) => {
    const full = join(PACK, dir, n);
    return statSync(full).isDirectory() ? walk(relative(PACK, full)) : n.endsWith(".json") ? [relative(PACK, full).split(sep).join("/")] : [];
  });
const bundled = () => bundlePack(readJson("pack.json"), readJson, (d) => walk(d.replace(/\/$/, "")));

test("Sol today bundles into one pack holding every file it names, and nothing else changes", () => {
  const folder = readJson("pack.json") as { provides: { other: Record<string, Json> } };
  const r = bundled();
  assert.deepEqual(r.problems, []);
  // Every log in the folder, plus the eight named files besides the logs folder.
  const logs = walk("logs");
  assert.ok(logs.length > 1200, `${logs.length} logs`);
  assert.equal(r.paths.length, logs.length + namedFiles(folder).filter((n) => !n.endsWith("/")).length);
  const held = heldFiles(r.pack)!;
  assert.ok(held, "the bundle holds its files");
  for (const p of ["crews.json", "in-memory.json", "memorials.json", "catalogue.json", "logs/index.json", "logs/apollo-11.json", "logs/departed/" + walk("logs/departed")[0]!.split("/").pop()]) {
    assert.deepEqual(held(p), readJson(p), p);
  }
  assert.equal(held("nope.json"), null);
  assert.equal(held("toString"), null, "only held paths, not the object's own keys");
  // The pack's words, credits and calendar are its own, untouched.
  const other = (r.pack as { provides: { other: Record<string, Json> } }).provides.other;
  for (const k of Object.keys(folder.provides.other)) assert.deepEqual(other[k], folder.provides.other[k], k);
  assert.equal(heldFiles(folder), null, "the folder's pack.json holds nothing");
});

test("a folder pack that names a missing file or a path out of its folder is not bundled", () => {
  const pack = { format: "sse.pack/1", id: "x.y", version: "1", kind: ["universe"], kernel_range: "^0.1", licence: "CC0-1.0", provides: { other: { files: { a: "a.json", b: "../b.json", c: "logs/" } } } };
  const r = bundlePack(pack, (p) => (p === "a.json" ? { ok: true } : null), () => []);
  assert.deepEqual(r.problems, ["../b.json: a path must stay inside the pack folder", "logs/: the folder holds no files"]);
  assert.ok(!safePackPath("/etc/x") && !safePackPath("a/./b") && safePackPath("logs/departed/x.json"));
});

test("the script writes the same one file, and the Open door reads it as a pack that stacks with its calendar", async () => {
  const dir = mkdtempSync(join(tmpdir(), "sse-bundle-"));
  try {
    const out = join(dir, "sol-today.json");
    execFileSync(process.execPath, [join(ROOT, "scripts/bundle-pack.mjs"), PACK, out], { stdio: "pipe" });
    const bytes = readFileSync(out);
    const text = bytes.toString("utf8");
    assert.equal(text, JSON.stringify(bundled().pack), "script and rule agree, byte for byte");
    const sniffed = sniffFile("sol-today-0.1.0.json", text, new Uint8Array(bytes));
    assert.equal(sniffed.kind, "pack");
    assert.equal(checkPackDoc(sniffed.doc, ["sse.base-physics"]), null);

    const base = JSON.parse(readFileSync(join(ROOT, "packs/base-physics/pack.json"), "utf8"));
    const k = nodeKernel();
    try {
      const s = await Session.open(k.client, gmSeat(), { id: "kestrel", seed: "k" }, [base]);
      const r = await s.send({ kind: "stack_packs", packs: [sniffed.doc as never] });
      assert.equal(r.status, "applied", JSON.stringify(r));
      assert.ok(s.stack().added.some((p) => p.id === "sse.sol-today"));
      const c = await s.calendar();
      assert.ok(c.ok && c.calendar, JSON.stringify(c));
      assert.equal(c.calendar!.format, "{mday} {month} {year}", "the campaign's dates are the pack's");
    } finally {
      await k.close();
    }
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
