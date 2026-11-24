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

## Not yet
- The page turn for paper editions (14A note 5) goes through the look transitions thread's `resolveDelivery` and
  `playDelivery` (PR 483). It is not on main yet; the call is one line in `Atlas.svelte` on `article[data-atlas-page]`
  when the page changes, and the Guide (not paper) gets its stepped wipe in AT7.
- Running head and letterhead on sheets come with AT6 (`print.running_head`, `print.footer` and letterhead slots).

## Checks
- `node --test packages/ui-contract/test/atlas.test.ts`: Copy as text carries every figure and the fixed bar;
  Save page is escaped, has no script and loads nothing.
- `tests/ui/atlas.e2e.mjs`: with the browser's print stood in for, Print this page prints only Tarn with the screen's
  figures in each edition; the whole atlas prints every page on Letter; the print pages go afterwards; a print that
  does not start is said; Copy as Markdown and Save page carry the same words. With `--screens` (or `--pdfs`) it
  writes a PDF per edition and the whole atlas.
- Screenshots and PDFs: project files `sse4/handover-screens/AT4-screens/`.
