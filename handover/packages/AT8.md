# AT8: every number keeps its Why (atlas design section 8)

## What it is

- **On screen** a figure on an Atlas page is a button. Pressing it opens its Why under the line. The Why starts with
  "How we know", which is how the audience knows the figure. On the GM's seat, where the card line has a derivation,
  the kernel's whole chain follows: the same `Explain` popup as the body card (imported from `views/body-card`). A
  second press, or Close, shuts it. It shuts when the page changes.
- **How we know, per audience.**
  - **For the GM:** where the value came from, read off the card line's `source`. The wordings are "Worked out from
    the physics", "Set by the GM", "From a catalogue", "Resolved for this universe" and "From a sample".
  - **For the table:** the rung it was given at, from the kernel's `known_to`, as in "Orbital survey, given to the
    table". With no rung it reads "Given to the table".
  - **Reasoning:** the kernel explanation's lens at the reader's depth, first sentence only, then its first reference
    (`showReference`).
  - **Citations** print at Science and Engine and not at Curious.
  - **What a seat's note can say.** It is the method, never an input value, and it is made only for a figure already
    on that audience's page.
- **On paper** the edition's `sources` field decides (`withSources` in `ui-contract/src/atlas/sources.ts`, pure):
  - `endnotes` (Survey atlas): each figure gets a small number (`FactRow.mark`, from 1, in reading order). A "How we
    know" block ends the page body, before the margin.
  - `section` (Company): a "Source: …" line goes under each facts section, each source said once.
  - `off` (the Guide): nothing is added.
  - Copy as text puts `[1]` by the figure and the notes under HOW WE KNOW. Markdown uses footnotes (`[^1]`). Save as
    a web page uses `<sup class="mark">` and an ordered list.
- **A first cut on purpose: notes end each page, not each chapter.** A page then prints whole with its own notes, and
  the numbers stay local. A chapter-end "How we know" page can come later as one more block over the same notes.
- **Card lines.** The GM's system scene names lines in display units (`year_d`, `surface_k`) and the player scene in
  SI. `CARD_FIELDS` and `cardFieldsOf` in the catalogue say which card line carries a figure's derivation. This is not
  a conversion.
- **Reading.** The page in front is read as it changes: one `known_to` per page for a table audience, and one
  `explain` per derived figure, cached by depth, page and field. "Print…" reads every page first, so the print view's
  thumbnails carry their notes.

## Waits on

- **The grant's own source and day** ("Orbital survey by this crew, day 212", "Bought chart, day 190") wait on the
  kernel. `known_to` gives the rung, not who granted it or when. When the grant carries its source, only `howOf` in
  `Atlas.svelte` changes.
- **A player's Why:** a player seat has no kernel, so its Why is "How we know" alone.

## Checks

- `packages/ui-contract/test/atlas.test.ts`, three new tests:
  - Endnotes mark every figure and nothing else. Every mark has a note. Only the page's own figures are asked about.
    Values are unchanged. Curious drops the citation. The notes come before the margin.
  - Company has Source lines and the Guide has none. Text, Markdown and HTML carry the notes.
  - `firstSentence` and `cardFieldsOf`.
- `tests/ui/atlas.e2e.mjs`:
  - The Survey page's marks equal its notes.
  - Pressing a figure opens the Why with the GM's source; pressing again closes it.
  - The printed Survey page carries How we know. The printed Company page has Source lines.
  - The table's Source lines say "given to the table", exactly where the table has figures. Kestrel's table knows no
    figure of Tarn yet, so the unit tests carry that case.
  - The edition comparisons read figures without their marks.
- Screenshots: project files `sse4/handover-screens/AT8-screens/`.
