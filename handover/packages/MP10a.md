# MP10 part 1: the Reference screen's Constants and Units pages

Matter-pack design section 8 (the Reference screen) and 8.6 (phone and Night chart), built to the mockup
"SSE4 Reference screen mockup" (Constants, Units and calendars, and the phone boards). UI track. DM1's request
(handover/packages/DM1.md): the theoretical mark and its note at every depth, on a phone and in Night chart.

## What it does

- A **Reference** entry on the Workshop rail (`packages/shell/src/documents/reference.gm.json`, order 7), with no tab.
  View `sse.view.reference` (`packages/views/reference`).
- Everything is read through the kernel's `read_matter` (new `ViewSession.readMatter`), so an edit in a pack shows as
  loaded. `ui-contract/src/matter.ts` holds the reading and the wording (`readConstantsPage`, `readRow`,
  `readSource`, `formatValue`, `formatUnit`, `evalFormula`, `unitFactor`, `siUnits`, `rowNote`, `sourceName`,
  `dimensionless`). Nothing is typed: values, units, sources, marks and group wording all come from the sheet.
- **Constants**: the two numbers no change of units can move (the fine-structure constant, and gravity's coupling
  G mₚ²/ħc worked out from the sheet's rows) at the top; then the sheet's groups in order (nature, conventions,
  factors, cosmos, then any a pack adds), each with its `about`. Each row: name, value (all the sheet's digits at
  Science and Engine, four figures at Curious; worked-out rows to ten figures), unit with superscript powers, and how
  it is known (exact, ± uncertainty, worked out, or the source). The cosmos group's cited notes follow its rows.
- **Theoretical rows** (dark matter, dark energy, w0 and wa, the dark matter particle rows) carry a "Theoretical"
  stamp and their `mark_note` under the row at every depth, on a phone and in Night chart.
- **A card** for every row: the full value, the mark and its note, the row's note, how it is known (worked out from a
  formula, exact by definition, or measured with its uncertainty), and "Why this number?" with each source linked by
  DOI or URL. Engine adds the id, the packs that set it, the blocks that read or skip it, and the check status.
  Escape closes the card and gives focus back to its row; on a phone the card is a sheet from the foot.
- **Units** (under "This setting"): every unit on the sheet, by what it measures, each against that measure's SI
  unit; a factor that rests on constants is worked out from the sheet's values (a parsec is 3.085 677 581 × 10¹⁶ m).
- Phone: page chips scroll sideways, rows become two lines and are at least 44 px tall.

## Checks

- `packages/ui-contract/test/matter.test.ts`: formatting, formulas, source names, and the pages read through a real
  kernel (group order, every theoretical row with a note, the parsec, the two pure numbers).
- `tests/ui/reference.e2e.mjs` (desk and phone): the groups and their about lines, the pure numbers, theoretical
  stamps and notes at Curious, Science and Engine, ids only at Engine, a card with a DOI link, Escape and focus, the
  Omega_c card, the units page, Night chart, no phone overflow, 44 px rows, the state hash unchanged, no errors.
- ui:check, ui:test, full ui:e2e. Screens in `/mnt/project-files/sse4/MP10a-screens/`.

## Not yet

- The cosmos notes' sources show as "Clowe 2006" until `read_matter` reads the sources table by id (asked of the
  master data sheet thread on 22 Nov; the page already looks each up and links it once the kernel answers).
- Elements (the periodic table by origin), Substances with the phase diagram, Mixtures, Calendars, unit schemes,
  editing (MP9) and the "what if G changed" reach panel follow in later parts.
