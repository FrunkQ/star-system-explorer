// V3 gap item 30 (Alex, 23 Nov 2026): no third "clarity" skin; every theme is checked instead. Colours that must be
// told apart stay apart as people with protanopia, deuteranopia and tritanopia see them (Machado et al. 2009), and
// as everyone sees them. ΔE is CIE76: about 2 is just noticeable; 10 is plainly a different colour.
import { test } from "node:test";
import assert from "node:assert/strict";
import { resolveTheme, leastApart, apart, contrast } from "../../chart-room/src/index.ts";
import { ALL_THEMES } from "../src/index.ts";

const ORIGINS = ["big-bang", "cosmic-rays", "low-mass-stars", "massive-stars", "white-dwarfs", "neutron-star-mergers", "human-made"].map((o) => `origin-${o}`);

// Pairs a reader tells apart by colour alone: the accent (what is chosen) from the ink and the zone labels, and the
// field's lines out from in (Mag). A one-colour theme tells the accent from the ink by pattern and weight instead.
const MUST_DIFFER: [string, string][] = [
  ["accent", "ink"],
  ["accent", "ink-muted"],
  ["accent", "zone-ink"],
  ["field-out", "field-in"],
];

test("the model: a red and a green that differ only in hue merge for deuteranopia", () => {
  assert.ok(apart("#D2462E", "#2F9E3A", "typical") > 40);
  assert.ok(apart("#CC6633", "#669933", "deuteranopia") < apart("#CC6633", "#669933", "typical") / 2);
  assert.ok(apart("#000000", "#FFFFFF", "tritanopia") > 99);
});

test("every theme keeps what must differ apart for every colour vision", () => {
  for (const t of ALL_THEMES) {
    const r = resolveTheme(t.id, ALL_THEMES);
    for (const [a, b] of MUST_DIFFER) {
      if (t.supports.colours === 1 && a === "accent") continue;
      const d = leastApart(String(r[a]), String(r[b]), String(r.ground));
      assert.ok(d.delta >= 10, `${t.id}: ${a} and ${b} are ΔE ${d.delta.toFixed(1)} apart for ${d.vision}`);
    }
  }
});

test("every theme's element origins stay apart for every colour vision", () => {
  for (const t of ALL_THEMES) {
    const r = resolveTheme(t.id, ALL_THEMES);
    for (let i = 0; i < ORIGINS.length; i++)
      for (let j = i + 1; j < ORIGINS.length; j++) {
        const d = leastApart(String(r[ORIGINS[i]!]), String(r[ORIGINS[j]!]));
        assert.ok(d.delta >= 10, `${t.id}: ${ORIGINS[i]} and ${ORIGINS[j]} are ΔE ${d.delta.toFixed(1)} apart for ${d.vision}`);
      }
  }
});
