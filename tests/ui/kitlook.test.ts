// The look a role's kit puts on its screens (AT15 part 2): every starter kit in every one of its palettes keeps its
// text readable, the palette's own tokens are exact, and a one-colour look turns wholly to the palette's colour.
import { test } from "node:test";
import assert from "node:assert/strict";
import { kitLookOf, mixHex, turnColours, turnBetween } from "../../packages/shell/src/atlas/kitlook.ts";
import { contrastOf, resolveKit, SHIPPED_PALETTES, STARTER_KITS } from "../../packages/ui-contract/src/index.ts";

const tokenOf = (css: string, k: string) => {
  const all = [...css.matchAll(new RegExp(`--sse-${k}:\\s*([^;]+);`, "g"))];
  return all.at(-1)?.[1]!.trim();
};

test("each starter kit wears each of its palettes readably", () => {
  for (const k of STARTER_KITS) {
    const kit = resolveKit(k.id);
    for (const id of (kit.palettes ?? []).map((p) => (typeof p === "string" ? p : p.id))) {
      const l = kitLookOf(k.id, id);
      assert.ok(l, `${k.id} ${id}: a look`);
      assert.equal(l.palette, id);
      const p = SHIPPED_PALETTES.find((x) => x.id === id)!;
      for (const [t, v] of Object.entries(p.tokens)) assert.equal(tokenOf(l.css, t)?.toUpperCase(), v.toUpperCase(), `${k.id} ${id}: ${t} is the palette's`);
      const ground = tokenOf(l.css, "ground")!;
      for (const t of ["ink", "ink-muted", "accent-ink"]) assert.ok(contrastOf(tokenOf(l.css, t)!, ground)! >= 4.5, `${k.id} ${id}: ${t} reads on the ground`);
      assert.ok(contrastOf(tokenOf(l.css, "on-ink")!, tokenOf(l.css, "ink")!)! >= 4.5, `${k.id} ${id}: a pressed control's words read`);
      for (const t of ["title-ink", "label-ink", "margin-ink"]) {
        const v = tokenOf(l.css, t);
        if (v) assert.ok(contrastOf(v, ground)! >= 4.5, `${k.id} ${id}: the Atlas's ${t} reads on the ground`);
      }
      assert.equal(l.doc.mode, contrastOf(ground, "#000000")! > contrastOf(ground, "#FFFFFF")! ? "light" : "dark", `${k.id} ${id}: the mode follows the ground`);
    }
  }
});

test("a palette asked for that the kit lacks gives the kit's first; no kit, no look", () => {
  assert.equal(kitLookOf("sse.kit.hauler", "glass-blue")?.palette, "phosphor-green");
  assert.equal(kitLookOf("sse.kit.hauler")?.palette, "phosphor-green");
  assert.equal(kitLookOf(undefined), null);
  assert.equal(kitLookOf("no.such.kit"), null);
});

test("a one-colour look turns wholly: amber phosphor has no green left in it", () => {
  const l = kitLookOf("sse.kit.hauler", "phosphor-amber")!;
  for (const [k, v] of Object.entries(l.doc.tokens)) {
    if (typeof v !== "string") continue;
    for (const m of v.matchAll(/#([0-9a-f]{6})/gi)) {
      const n = parseInt(m[1]!, 16);
      const [r, g, b] = [(n >> 16) & 255, (n >> 8) & 255, n & 255];
      assert.ok(!(g > r + 40 && g > b + 40), `${k} is still green (${m[0]})`);
    }
  }
});

test("colour turning: rgba and hex inside a value; mixing", () => {
  const t = turnBetween("#FF0000", "#0000FF")!;
  assert.equal(turnColours("0 0 6px rgba(255, 0, 0, 0.5)", t), "0 0 6px rgba(0, 0, 255, 0.5)");
  assert.equal(turnColours("#FF0000 and #ff0000", t), "#0000FF and #0000FF");
  assert.equal(mixHex("#000000", "#FFFFFF", 0.5), "#808080");
});
