// The sector desk and world sheet on the chart room (X3a, X3c): a reading
// becomes hex map fields the Traveller pack's glyphs and sifters read, the
// desk opens on the one subsector in use, and rail entries that need a code
// sheet show only while a pack gives one.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { mapFields, stackProvides, type CodeReading, type SectorMap } from "../../packages/ui-contract/src/sector.ts";
import { railOf, checkScreen } from "../../packages/ui-contract/src/screen.ts";
import { blocksInUse, exportName, hexConfigOf, mapWorlds, openingBlock, rollEveryWorld, rollTallyWords } from "../../packages/views/sector/src/desk.ts";
import { marksFor } from "../../packages/views/hex-map/src/glyphs.ts";
import { sift } from "../../packages/views/hex-map/src/sift.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const json = (p: string) => JSON.parse(readFileSync(root + p, "utf8"));
const traveller = json("packs/traveller/pack.json");

const reading = (seg: Record<string, [string, number | null]>, groups: Record<string, string[]>): CodeReading => ({
  sheet: "traveller.world",
  lines: { uwp: "A867974-C" },
  segments: Object.entries(seg).map(([id, [symbol, value]]) => ({ id, symbol, value, why: { kind: "classified" } })),
  groups: Object.entries(groups).map(([id, tags]) => ({ id, tags: tags.map((t) => [t, {}] as [string, unknown]) })),
  prepared: [],
});

const kessa = reading({ starport: ["A", null], size: ["8", 8], hydrographics: ["7", 7], gas_giants: ["2", 2], atmosphere: ["6", 6], law: ["4", 4] }, { bases: ["N", "S"], zone: ["A"], trade: ["Ri"] });
const rubble = reading({ starport: ["E", null], size: ["0", 0], hydrographics: ["0", 0], gas_giants: ["0", 0], atmosphere: ["B", 11], law: ["2", 2] }, { bases: [], zone: [], trade: [] });

const map: SectorMap = {
  type: "sector_mapped",
  sheet: "traveller.world",
  worlds: [
    { system: "u:k/sys:0101", body: "u:k/sys:0101/body:3", hex: "0101", name: "Kessa", sector: "Lantern Fall", reading: kessa },
    { system: "u:k/sys:0203", body: "u:k/sys:0203/body:1", hex: "0203", name: "Rubble", sector: "Lantern Fall", reading: rubble },
  ],
  skipped: [],
};

test("a reading becomes the fields the pack's glyphs draw", () => {
  const f = mapFields(kessa, "Kessa");
  assert.equal(f.starport, "A");
  assert.equal(f.size, 8);
  assert.deepEqual(f.bases, ["N", "S"]);
  assert.equal(f.zone_first, "A");
  assert.equal(f.uwp, "A867974-C");
  const cfg = hexConfigOf([traveller])!;
  const ms = marksFor({ hex: "0101", name: "Kessa", fields: f }, cfg.glyphs);
  assert.equal(ms.find((m) => m.at === "top")?.text, "A");
  assert.equal(ms.find((m) => m.at === "ring")?.tone, "caution");
  assert.equal(ms.find((m) => m.at === "top-left")?.mark, "star");
  const belt = marksFor({ hex: "0203", name: "Rubble", fields: mapFields(rubble, "Rubble") }, cfg.glyphs);
  assert.equal(belt.find((m) => m.at === "centre")?.mark, "belt");
  assert.equal(belt.find((m) => m.at === "ring"), undefined, "no zone, no ring");
});

test("the pack's sifters read the same fields", () => {
  const cfg = hexConfigOf([traveller])!;
  const ws = mapWorlds(map);
  const by = (id: string) => cfg.sifters!.find((s) => s.id === id)!;
  assert.deepEqual(sift(by("refuel"), ws), ["0101"]);
  assert.deepEqual(sift(by("red-candidates"), ws), ["0203"], "unzoned, corrosive air");
});

test("the desk opens on the one block in use and names its export", () => {
  const cfg = hexConfigOf([traveller])!;
  const ws = mapWorlds(map);
  assert.deepEqual(blocksInUse(cfg, ws), [{ index: 0, name: "A", worlds: 2 }]);
  assert.equal(openingBlock(cfg, ws), 0);
  const far = [...ws, { hex: "3240", name: "Edge", fields: {} }];
  assert.equal(openingBlock(cfg, far), undefined, "two blocks: the whole sheet");
  assert.equal(exportName(map, "tab"), "Lantern-Fall.tab");
  assert.equal(exportName(null, "sec"), "sector.sec");
  assert.equal(hexConfigOf([{ provides: { other: {} } }]), null);
});

test("rail entries that need code sheets show only while a pack gives them", () => {
  const screens = { sector: json("packages/shell/src/documents/sector.gm.json"), codes: json("packages/shell/src/documents/codes.gm.json"), body: json("packages/shell/src/documents/body.gm.json") };
  const lines = (scale: "galaxy" | "world", packs: unknown[]) => railOf(screens, scale, (n) => stackProvides(packs, n)).flatMap((g) => g.lines.map((l) => l.screen));
  assert.deepEqual(lines("world", []), ["body"]);
  assert.deepEqual(lines("world", [traveller]), ["body", "codes"]);
  assert.deepEqual(lines("galaxy", []), []);
  assert.deepEqual(lines("galaxy", [traveller]), ["sector"]);
  // Without a predicate every entry shows, as before.
  assert.equal(railOf(screens, "galaxy").length, 1);
  assert.equal(stackProvides([{ provides: { other: { code_sheets: [] } } }], "code_sheets"), false);
});

test("a rail entry's needs must be a list of pack keys", () => {
  const doc = json("packages/shell/src/documents/sector.gm.json");
  const bad = { ...doc, rail: [{ ...doc.rail[0], needs: "code_sheets" }] };
  const r = checkScreen(bad, []);
  assert.equal(r.ok, false);
  assert.ok(!r.ok && r.errors.some((e) => e.includes("needs must be a list")), JSON.stringify(r));
});

test("a sector-wide roll writes the unset codes, keeps hand-set ones, and goes on past a refusal", async () => {
  const asks: string[] = [];
  const roll = async (a: { program: string; body: string; apply?: boolean }) => {
    asks.push(`${a.program}:${a.body}:${a.apply}`);
    if (a.body === "bad") return { ok: false as const, code: "unknown_body", message: "no such body" };
    const steps = a.body === "imported" ? [{ how: "kept" }, { how: "kept" }, { how: "skipped" }] : [{ how: "rolled" }, { how: "zero" }, { how: "skipped" }];
    return { ok: true as const, result: { steps } };
  };
  const t = await rollEveryWorld(roll, "traveller.social", ["fresh", "imported", "bad"], 0n);
  assert.deepEqual(asks, ["traveller.social:fresh:true", "traveller.social:imported:true", "traveller.social:bad:true"]);
  assert.equal(t.worlds, 2);
  assert.equal(t.written, 2);
  assert.equal(t.kept, 2);
  assert.equal(t.refused.length, 1);
  const words = rollTallyWords(t);
  assert.match(words, /^2 worlds rolled: 2 codes written; 2 codes already set kept; 1 world refused \(no such body\)\. Each written code/);
  assert.equal(rollTallyWords({ worlds: 1, written: 0, kept: 3, refused: [] }), "1 world rolled: 0 codes written; 3 codes already set kept.");
});
