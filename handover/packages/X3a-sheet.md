# X3a world data sheet: the world codes view

Status: view module done (19 Nov 2026), Traveller pack track. Design: handoff 7A part 1 (every digit answers why); reads the kernel `codes` op (X3a part 2b). Not yet: a GM screen for it (see Limits).

## What was built (`packages/views/world-codes`, `sse.view.world-codes`)
- `sheet.ts`, pure: the `codes` response typed as the kernel sends it (sse-codes `Reading`, `SegmentReading`, `Why` in its six kinds, `GroupReading` with `[code, TagState]` pairs, sse-sift `Label`). The input is `{response, sheet, vocabulary?, classes?, names?, selected?, title?}`; `inputFromPacks(packs, response)` fills it from a stack: the sheet the response names (`provides.other.code_sheets`), the merged `provides.other.vocabulary` tables (later packs win per entry), class names from `provides.other.classes[].names`. Nothing names a game.
  - `whySentence`: classified (class name as "physics (plain)", fit, modifiers, kept by stickiness, defaulted facts); unclassified "?" with the nearest class and the change it needs; authored "set by hand (field); physics says X, <class>", with "too" when they agree and a disagreement sentence when not; leading (the fact as m × 10^n); counted (N members, named); no system.
  - `disagrees` (as `SegmentReading::disagrees`), `lineRows` (layout order), `segmentRows` (sheet order), `chipGroups` (sheet group order, empty groups left out, words and a tooltip with the rule and evidence), `bodyRows` (list mode), `textSheet` / `textList` (text tier).
  - Words for a group: the vocabulary table named by the group's prefix less its dot (`traveller.trade`), else the sheet's namespace and the group id (`traveller.bases` for the prefix `traveller.base.`).
- `WorldCodes.svelte` and `index.svelte.ts`: one body's sheet (code lines large; a table of digit, code, why, with a "set" mark on hand-set digits and a dashed "≠ X" mark and tinted row where physics disagrees; rule groups as chip buttons that open their words and evidence), or a table of every body (name as a button that opens its sheet, one column per layout, one per non-empty group, a "≠ N" count). Text tier: the same as sentences in a list. Narrow screens stack the why under the digit. Colours only from theme tokens (`--sse-ink`, `--sse-ink-muted`, `--sse-line`, `--sse-rule`, `--sse-accent`, `--sse-alert`, `--sse-focus`, `--sse-ground*`). All controls are native buttons and a labelled select; no a11y warnings. Params: `mode` (auto, sheet, list), `tier` (canvas, text). An optional `onpick(address)` prop reports the body opened.

## Checks (`node --test tests/ui/world-codes.test.ts`, in `npm run ui:test`; `npm run ui:check` clean, 0 errors, 0 warnings)
- The kernel's own toy answer (contract golden, the three-body `codes` case) through the toy pack's sheet (`tests/fixtures/packs/codes-toy.json`): lines, segment rows, the counted why with names, body 2's hand-set L against physics S flagged and worded, chips with no words, list rows with the disagreement count.
- An unknown sheet gives no input; no id picks the first sheet; a named one is found across packs.
- A hand-built Earth-like reading through the Traveller pack's sheet: UWP before PBG, sheet order, every why kind (authored with and without physics, classified with defaulted and stickiness, unclassified, leading 8.1 × 10^9, counted, no system).
- Chips in sheet group order with the pack's words (Hi high population, N naval base), a group without words, empty zone group left out; a later pack's words win; the text tier for one body and for the list.

## Limits
- Not on a GM screen yet: it registers through `packages/shell/src/screens.more.ts` once that hook lands; the screen will call `codes` and `inputFromPacks` with the session's stack and body names.
- Body names are an input; the `codes` response carries addresses only.
- Layout labels are the layout id in capitals (UWP, PBG); the sheet has no label field for layouts.
- `prepared` flags (the prepare rules' evidence) are not shown yet; a "why" drill-down from a classified digit into its axes and near misses is left for the explain lens.
- No browser test yet (no e2e for this view); only the pure half is tested.
