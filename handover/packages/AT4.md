# AT4: print under the view

Track AT (atlas design section 9, 14A note 1: "print UNDER these"). Built 24 Nov 2026.

## What it is
- **Print this page** and **Print…** sit in the Atlas's bar. Print… opens a small panel: what to print (this page,
  this chapter, the whole atlas, from `atlasContents` and `chapterOf`), paper (A4, or Letter where the device's
  language is en-US, en-CA, es-MX or fr-CA; changeable), **Save ink** (on by default).
- Printing is in place, in this window: `AtlasPrint.svelte` mounts the pages asked for at the end of `<body>` with an
  `@page` rule (size, margins, "page / pages" in the bottom margin), and `@media print` hides everything else. Each
  sheet keeps sections, rows and figures whole. The pages are built by the same `buildPage` as the screen, in the
  edition showing, so a printed figure is the page's figure.
- If printing does not start (no `beforeprint`, or `window.print` throws), the Atlas says so in a sentence and points
  to the browser's own Print, Copy as text or Save page.
- **Save ink**: `AtlasHost.paperCss(look, save_ink_look)` from the shell. A dark look prints in the edition's
  `print.save_ink_look`, else `paper`; a light look prints as it is.
- **Sheet furniture**: every page's footer reads "edition · book · as known to <audience>, <date>" with its page
  number; the date is the ruler's time in the campaign's calendar (`dateWords`, pack calendar or default).
- **Copy as text** and **Copy as Markdown** (`pageText` in `ui-contract/src/atlas/text.ts`): the page's own words and
  figures; a bar is a fixed-length run of blocks, whatever is behind it. **Save page** (`atlasHtml`): the pages as one
  self-contained HTML file, escaped, no script, nothing loaded from the web, printable on its own.

## The print view and black and white (AT4b; atlas design 14A note 7, Alex 25 Nov)
- "Print…" opens `AtlasPrintView.svelte`, a dialog over the Atlas: every page of the book as a small sheet in order,
  each with a tick box, "k. Title" and its sheet count; all ticked on opening, unticked sheets fade. Quick picks All,
  None, Only the page I was on, This chapter. A running count "N of M pages, S sheets": sheets are each page drawn at
  the printable width of the chosen paper (A4 178 mm, Letter 184 mm, @page margins 16/16/18 mm) and measured against
  its printable height. Ink: As shown or Black and white; the thumbnails redraw in the chosen ink. Paper: A4 or
  Letter. Copy as text, Copy as Markdown and Save as a web page act on the ticked pages.
- Black and white is a toner form, not a grey copy: the shell gives `tonerCss()` (the paper look's fonts, every
  colour black on white, in `packages/shell/src/atlas/screens.atlas.ts`), and `.ink-bw` in `AtlasPage.svelte`
  takes away tints, fills and shadows, keeps bars solid, outlines stamps, and draws logos in one ink. The view names
  no colour (the theme-free test).
- A dark edition left on "As shown" says once, in the view, that it covers every sheet in ink.
- "Print this page" stays a one-press shortcut that skips the view: as shown for a light edition, its toner form for
  a dark one.

## Not yet
- Done 25 Nov: the page turn (14A note 5) through AT15a. `host.lookDelivery(look)` gives the edition's look's
  deliveries; `resolveDelivery(…, "turn", {back})` picks the plan. Paper (`page_turn`) steps through `curlTurn` on
  `article[data-atlas-page]`, so the old page peels over the new one; other looks step and `playDelivery` (the Guide
  gadget now carries `STARTER_DELIVERIES.guide`, a stepped wipe, in place of the Atlas's own wipe CSS). A drag on the
  page is AT15a's `use:pageTurn` (with `sheet` on paper, the curl follows the finger); the Atlas's own swipe code is
  gone. The e2e checks the curl's copy appears and goes; screenshot `AT4b-screens/atlas-page-curl-desk.png`.
- Running head and letterhead on sheets come with AT6 (`print.running_head`, `print.footer` and letterhead slots).

## Checks
- `node --test packages/ui-contract/test/atlas.test.ts`: Copy as text carries every figure and the fixed bar;
  Save page is escaped, has no script and loads nothing.
- `tests/ui/atlas.e2e.mjs`: with the browser's print stood in for, Print this page prints only Tarn with the screen's
  figures in each edition; the whole atlas prints every page on Letter; the print pages go afterwards; a print that
  does not start is said; Copy as Markdown and Save page carry the same words. With `--screens` (or `--pdfs`) it
  writes a PDF per edition and the whole atlas.
- Screenshots and PDFs: project files `sse4/handover-screens/AT4-screens/`; the print view in `AT4b-screens/`.
- The e2e also opens the print view: every page ticked, None then two by hand (only those print), black and white
  redraws the thumbnails and prints with `data-atlas-ink="bw"`, Only the page I was on, Close.
