# MP10 part 2: the Reference screen's Elements page

Matter-pack design 8.1 (periodic table, origin lens, origin stories) and 8.6 (phone and Night chart), built to the
mockup's Elements boards. UI track. Follows MP10 part 1 (packages/MP10a.md).

## What it does

- **Elements** under Nature on the Reference screen (`packages/views/reference/src/Elements.svelte`).
- Read through `read_matter`: `readElementsPage` in `ui-contract/src/matter.ts` reads the listing (ids and the origin
  legend) and each element's row. Nothing is typed; the 118 positions come from `tablePosition(z)`, which is layout.
- **The table, coloured by where each element was made**, the default lens. Each cell's bar is filled in proportion
  to its `origin.shares`, in the sheet's legend (`origin_legend`), drawn with seven new theme tokens
  `origin-big-bang` … `origin-human-made`, defined in every chart-room and player theme. The mockup's colours; Night
  chart lightens massive stars, mergers and the grey, as section 8.6 says. One legend for the whole app: the
  composition bar and resource cards will use the same tokens. Elements past 118, or rows a pack marks, go in a
  campaign row with dashed (fictional) or dotted (predicted) borders.
- **The card**: number and symbol, name, any pack mark, the shares as a bar and in words ("Merging neutron stars, about
  94 parts in 100"), the origin story (the plain wording at Curious, the physics at Science and Engine), "How sure is
  this?" from `origin.argued`, and at Science the atomic weight, the birth-cloud abundance and the shares' caveat.
  "Why?" links the row's sources; Engine adds the id, the packs and the check.
- **Phone**: the table becomes a small coloured map with no letters, with a strip of seven full cells beneath it
  around the chosen element; the card rises from the foot.
- Escape closes a card wherever focus is (window capture), so the shell's Escape (go up a step) only acts with no card
  open. Part 1's Constants card now does the same.

## Checks

- `matter.test.ts`: 118 elements in 118 distinct cells, the legend in order, gold's shares and words, every element
  with a plain story.
- `reference.e2e.mjs` (desk and phone): 118 cells, seven distinct legend colours, gold's card at Curious (plain story,
  no figures) and Science (physics story, weight 196.966 57, sources), the phone's strip, no overflow, iron in Night
  chart. Screens in `/mnt/project-files/sse4/MP10b-screens/`.
- ui:check, ui:test, full ui:e2e; the theme tests pass with the new tokens.

## Not yet

- The other lenses (how common, where in this world, where it condenses, worth mining) need a selected system and the
  composition op (MP6); "For this star" (mergers that seeded it) needs the galaxy ledger (MP7).
- Substances and the phase diagram (part 3), Mixtures, Calendars, editing (MP9).
