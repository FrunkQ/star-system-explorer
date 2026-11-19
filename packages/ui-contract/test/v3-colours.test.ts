// V12 with V3's colours: palette overrides (V3 localStorage) land on the
// campaign's copy of the symbology; a save's tag categories become tag
// colours; per-object colours are listed by class (authored or derived).
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { deflateRawSync } from "node:zlib";
import { markerFor, checkSymbology, type SymbologyDoc } from "../src/symbology.ts";
import { coloursFromV3Save, objectColourSummary, parseV3Palette, symbologyFromV3Palette, v3SaveText, withV3TagColours, campaignCopy, type V3TokenMap } from "../src/v3-colours.ts";

const doc = (n: string) => JSON.parse(readFileSync(new URL(`../../shell/src/documents/${n}`, import.meta.url), "utf8"));
const REG: SymbologyDoc[] = ["symbology.diagrammatic.json", "symbology.colour-coded.json", "symbology.mixed.json"].map(doc);
const MAP = doc("v3-colour-tokens.json") as V3TokenMap;
const FULL = { supports: { colours: "full" as const, patterns: true, shapes: true } };

test("the token map covers V3's body and star tokens and every base class once", () => {
  const v3 = JSON.parse(readFileSync(new URL("../../../docs/v3-colours.json", import.meta.url), "utf8")) as { bodies: { token: string }[]; stars: { token: string | null; key: string }[] };
  for (const b of v3.bodies) assert.ok(MAP.tokens[b.token], b.token);
  for (const s of v3.stars) if (s.token?.startsWith("--star-") && s.key !== "red-giant") assert.ok(MAP.tokens[s.token], s.token);
  const classes = Object.values(MAP.tokens).flatMap((t) => t.classes ?? []);
  assert.equal(new Set(classes).size, classes.length, "a class is in two V3 groups");
});

test("a pasted V3 palette recolours the groups V3 used each token for", () => {
  const pasted = JSON.stringify({ "--body-gas-giant": "#112233", "--body-habitable": "#00f", "--star-k": "#445566", "--body-belt": "#777777", "--bg": "#000000", "--zone-habitable": "#00ff00", "--body-ice-giant": "var(--x)" });
  const parsed = parseV3Palette(pasted);
  assert.equal(typeof parsed, "object");
  const r = symbologyFromV3Palette(parsed as Record<string, string>, MAP, "sse.symbology.mixed", REG);
  assert.equal(checkSymbology(r.doc, REG), null);
  assert.equal(r.doc.id, "campaign.symbology");
  assert.equal(r.doc.inherits, "sse.symbology.mixed");
  assert.deepEqual(r.applied.map((a) => a.token), ["--body-belt", "--body-gas-giant", "--body-habitable", "--star-k"]);
  assert.deepEqual(r.skipped.map((a) => a.token), ["--bg", "--body-ice-giant", "--zone-habitable"]);
  assert.match(r.skipped.find((x) => x.token === "--bg")!.note, /theme/);
  const m = markerFor(r.doc.id, [...REG, r.doc], FULL);
  assert.equal(m.mark({ kind: "planet", classes: ["planet/hot-jupiter"], tags: [] }).colour, "#112233");
  assert.equal(m.mark({ kind: "planet", classes: ["planet/ocean"], tags: ["habitability/earth-like"] }).colour, "#0000ff");
  assert.equal(m.mark({ kind: "planet", classes: ["planet/ice-giant"], tags: [] }).colour, "#add8e6", "an unreadable override leaves V3's colour");
  assert.equal(m.mark({ kind: "star", classes: [], tags: ["spectral:K"] }).colour, "#445566");
  assert.equal(m.mark({ kind: "belt", classes: [], tags: [] }).colour, "#777777");
  // The mix keeps its shading: only colours changed.
  assert.equal(m.mark({ kind: "planet", classes: ["planet/ocean"], tags: [] }).fill_pattern, "band");
});

test("a palette onto the diagrammatic grammar starts from V3's colours, not from nothing", () => {
  const r = symbologyFromV3Palette({ "--body-terrestrial": "#aa5500" }, MAP, "sse.symbology.diagrammatic", REG);
  const m = markerFor(r.doc.id, [...REG, r.doc], FULL);
  assert.equal(m.mark({ kind: "planet", classes: ["planet/desert"], tags: [] }).colour, "#aa5500");
  assert.equal(m.mark({ kind: "planet", classes: ["planet/gas-giant"], tags: [] }).colour, "#cc0000");
});

test("palette text is read from the bare map or a localStorage dump, and refused politely otherwise", () => {
  assert.deepEqual(parseV3Palette('{"body-belt":"#888888"}'), { "--body-belt": "#888888" });
  assert.deepEqual(parseV3Palette(JSON.stringify({ "sse-palette-overrides": JSON.stringify({ "--star-g": "#ffffff" }), "sse-skin": "nebula" })), { "--star-g": "#ffffff" });
  assert.equal(typeof parseV3Palette("not json"), "string");
  assert.equal(typeof parseV3Palette("[1,2]"), "string");
  assert.equal(typeof parseV3Palette("{}"), "string");
});

const SAVE = {
  bundleFormat: "sse-starmap/1",
  id: "sm1",
  name: "Test map",
  coiCategories: [
    { id: "owner", label: "Owner", color: "#123456", textColor: "#FFFFFF", tags: [{ key: "owner/guild", label: "Guild" }] },
    { id: "status", label: "Status", tags: [] },
  ],
  systems: [
    {
      id: "s1",
      name: "Alpha",
      system: {
        nodes: [
          { id: "b1", name: "Alpha b", parentId: "a", apparentColorHex: "#a0b0c0", hydrosphere: { layers: [{ colorHex: "#2244aa" }] }, biosphere: { colorHex: "#33aa33", lightHex: "#55cc55" } },
          { id: "c1", name: "Station One", parentId: "b1", icon_color: "#ffcc00", engines: [{ exhaust_color_hex: "#ff00ff" }] },
        ],
      },
    },
  ],
};

test("a V3 save gives its tag category colours and lists each object's colours by where they go", () => {
  const c = coloursFromV3Save(SAVE);
  assert.deepEqual(c.tag_colours, { owner: { label: "Owner", color: "#123456", textColor: "#ffffff" } });
  const byField = c.objects.map((o) => `${o.name}:${o.field}:${o.class}`).sort();
  assert.deepEqual(byField, ["Alpha b:apparentColorHex:derived", "Alpha b:biosphere.colorHex:authored", "Alpha b:biosphere.lightHex:authored", "Alpha b:hydrosphere.layers[0].colorHex:authored", "Station One:engines[0].exhaust_color_hex:authored", "Station One:icon_color:authored"]);
  const lines = objectColourSummary(c.objects);
  assert.match(lines[0]!, /^5 authored object colours/);
  assert.match(lines[1]!, /^1 derived true colour .*recomputes/);
  const d = withV3TagColours(campaignCopy("sse.symbology.colour-coded", REG), c);
  assert.equal(checkSymbology(d, REG), null);
  assert.equal(d.tag_colours!.owner!.color, "#123456");
  // Tag colours never change a mark.
  const a = markerFor("sse.symbology.colour-coded", REG, FULL).mark({ kind: "planet", classes: ["planet/ocean"], tags: ["owner/guild"] });
  const b = markerFor(d.id, [...REG, d], FULL).mark({ kind: "planet", classes: ["planet/ocean"], tags: ["owner/guild"] });
  assert.deepEqual(a, b);
});

function zip(name: string, text: string): Uint8Array {
  const data = Buffer.from(text, "utf8");
  const comp = deflateRawSync(data);
  const n = Buffer.from(name);
  const local = Buffer.alloc(30);
  local.writeUInt32LE(0x04034b50, 0);
  local.writeUInt16LE(8, 8);
  local.writeUInt32LE(comp.length, 18);
  local.writeUInt32LE(data.length, 22);
  local.writeUInt16LE(n.length, 26);
  const cen = Buffer.alloc(46);
  cen.writeUInt32LE(0x02014b50, 0);
  cen.writeUInt16LE(8, 10);
  cen.writeUInt32LE(comp.length, 20);
  cen.writeUInt32LE(data.length, 24);
  cen.writeUInt16LE(n.length, 28);
  cen.writeUInt32LE(0, 42);
  const body = Buffer.concat([local, n, comp]);
  const end = Buffer.alloc(22);
  end.writeUInt32LE(0x06054b50, 0);
  end.writeUInt16LE(1, 8);
  end.writeUInt16LE(1, 10);
  end.writeUInt32LE(cen.length + n.length, 12);
  end.writeUInt32LE(body.length, 16);
  return new Uint8Array(Buffer.concat([body, cen, n, end]));
}

test("the save is read from a .sse.zip (starmap.json) or plain JSON", async () => {
  const text = JSON.stringify(SAVE);
  assert.equal(await v3SaveText(zip("starmap.json", text)), text);
  assert.equal(await v3SaveText(new TextEncoder().encode(text)), text);
  await assert.rejects(v3SaveText(zip("readme.txt", "hi")), /no starmap.json/);
});
