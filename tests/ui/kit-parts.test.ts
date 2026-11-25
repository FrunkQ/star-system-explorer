// The four starter kits against the players' parts as they really declare their controls (AT13, AT14): a starter
// may name only a real part and only a control that part declares. The ui-contract test uses a copy; this reads the
// declarations themselves, so a part that renames a control turns this red.
import { test } from "node:test";
import assert from "node:assert/strict";
import { checkKit, partsOf, STARTER_KITS, SHIPPED_EDITIONS } from "../../packages/ui-contract/src/atlas/index.ts";
import { CARD_CONTROLS, CHART_CONTROLS, TABLE_CONTROLS } from "../../packages/views/player-stack/src/schemas.ts";
import { ATLAS_CONTROLS } from "../../packages/views/atlas/src/controls.ts";

const parts = partsOf([
  { id: "sse.view.atlas", switchable: ATLAS_CONTROLS },
  { id: "sse.view.chart", switchable: CHART_CONTROLS },
  { id: "sse.view.card", switchable: CARD_CONTROLS },
  { id: "sse.view.table", switchable: TABLE_CONTROLS },
  { id: "sse.view.system-3d", switchable: [{ id: "turn", words: "Turn", kind: "dial", default: true }] },
  { id: "sse.view.text" },
  { id: "sse.view.readout" },
]);

test("every starter kit names real parts and only the controls they declare", () => {
  const stack = { parts, editions: SHIPPED_EDITIONS.map((e) => e.id), noteSets: ["sse.notes.guide"] };
  for (const k of STARTER_KITS) assert.deepEqual(checkKit(k, stack), [], k.id);
  assert.ok(!ATLAS_CONTROLS.some((c) => c.id === "edition"), "no Edition switch on a player's screen (14A note 6)");
});
