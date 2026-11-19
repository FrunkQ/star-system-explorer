// V2: every shipped player theme defines every token (through `extends`),
// reaches the contrast the schema asks for, and keeps to its declared
// capabilities; filter chains turn into overlays and honour the viewer.
import { test } from "node:test";
import assert from "node:assert/strict";
import { TOKENS, resolveTheme, themeCss, contrast } from "../../chart-room/src/index.ts";
import { ALL_THEMES, PLAYER_THEMES, filterLayers } from "../src/index.ts";

test("six player themes with generic names, each complete", () => {
  assert.deepEqual(PLAYER_THEMES.map((t) => t.id), ["chart", "paper", "deck", "phosphor", "glass", "guide"]);
  for (const t of PLAYER_THEMES) {
    const r = resolveTheme(t.id, ALL_THEMES);
    for (const s of TOKENS) assert.ok(s.name in r, `${t.id} lacks ${s.name}`);
    assert.doesNotThrow(() => themeCss(t.id, ALL_THEMES));
    for (const s of TOKENS.filter((x) => x.type === "keyword")) assert.ok(s.options!.includes(String(r[s.name])), `${t.id}: ${s.name}=${r[s.name]}`);
  }
});

test("text reaches WCAG AA and lines 3:1 in every player theme", () => {
  for (const t of PLAYER_THEMES) {
    const r = resolveTheme(t.id, ALL_THEMES);
    for (const s of TOKENS) {
      for (const g of s.text_on ?? []) {
        const c = contrast(String(r[s.name]), String(r[g]));
        assert.ok(c >= 4.5, `${t.id}: ${s.name} on ${g} is ${c.toFixed(2)}:1`);
      }
      for (const g of s.line_on ?? []) {
        const c = contrast(String(r[s.name]), String(r[g]));
        assert.ok(c >= 3, `${t.id}: ${s.name} line on ${g} is ${c.toFixed(2)}:1`);
      }
    }
  }
});

test("a one-colour theme draws with one hue", () => {
  for (const t of PLAYER_THEMES.filter((x) => x.supports.colours === 1)) {
    const r = resolveTheme(t.id, ALL_THEMES);
    const hues = new Set(
      TOKENS.filter((s) => s.type === "colour" && /^#/.test(String(r[s.name])))
        .map((s) => String(r[s.name]))
        .filter((c) => {
          const n = parseInt(c.slice(1), 16);
          const [R, G, B] = [(n >> 16) & 255, (n >> 8) & 255, n & 255];
          return Math.max(R, G, B) - Math.min(R, G, B) > 24; // chromatic, not grey
        })
        .map((c) => {
          const n = parseInt(c.slice(1), 16);
          const [R, G, B] = [(n >> 16) & 255, (n >> 8) & 255, n & 255];
          return R >= G && R >= B ? "r" : G >= B ? "g" : "b";
        }),
    );
    assert.ok(hues.size <= 1, `${t.id} uses hues ${[...hues]}`);
  }
});

test("filter chains become overlays; motion and effects follow the viewer", () => {
  const phosphor = PLAYER_THEMES.find((t) => t.id === "phosphor")!;
  const all = filterLayers(phosphor.filters);
  assert.deepEqual(all.map((l) => l.kind), ["glow", "scanlines", "vignette", "flicker"]);
  assert.match(all[0]!.root, /drop-shadow/);
  assert.ok(all.find((l) => l.kind === "flicker")!.moves);
  assert.deepEqual(filterLayers(phosphor.filters, { reducedMotion: true }).map((l) => l.kind), ["glow", "scanlines", "vignette"]);
  assert.deepEqual(filterLayers(phosphor.filters, { off: true }), []);
  assert.throws(() => filterLayers([{ kind: "tint", colour: "red", strength: 1 }]), /RRGGBB/);
  assert.equal(filterLayers([{ kind: "grain", strength: 9 }])[0]!.overlay.includes("opacity: 0.5"), true, "strength is clamped to 1");
});
