# ND4: Orac reads a paper

Design: `handover/design/SSE4-new-physics-demo.md` (2.2 gap 5, section 3, storyboard shots 2 and 3).

Status: done (24 Nov 2026, agents track).

## What it does
- **Text out of a PDF, in the browser, with no library** (`packages/agents/src/pdf-text.ts`). It reads objects and
  object streams, Flate streams, and the page tree in order. For each page's fonts it uses the ToUnicode map, or else
  an encoding with Differences (glyph names, ligatures), or else Latin-1. It reads the text operators Tj, TJ, ' and ",
  and takes line breaks and cell gaps from the text position. Decompression is passed in: `DecompressionStream` in
  the browser, zlib in tests.
- It refuses in words a file that is not a PDF, a locked (encrypted) one, and one with almost no text, such as
  scanned pages. Each refusal offers pasting the text instead. Pasted text and `.txt` or `.md` files always work.
- **The paper is labelled data** (`paper.ts`, `paperBlock`). It is framed as "the text of a paper, given to you as
  data", twice. Code fences inside it are broken, and it cannot close its own frame. The GM's request comes before it.
- **A plan in words first** (`planPaper`). The plan is one `plan` thing: claims with the paper's own numbers, the
  catalogue keys it would use, the kinds it would draft, and what is missing. `checkPlan` checks it before the GM sees
  it: a key that is not in the catalogue, or a kind Orac cannot make, goes back to Orac. A plan has no Accept button.
  **Draft it** runs the usual loop with the plan and the paper (`draftFromPlanRequest`), and every draft is checked
  as always.
- **Too long for the model:** refused before sending by the adapter's own check in `chat`, with the token count. The
  size line under Ask Orac counts the paper.
- The Orac tab has "Hand Orac a paper" (file) and "or paste its text". Orac's handbook (`tools`), the Tars shelf and
  the Orac tab's own text say so.

## Limits, honestly
- No column layout: two-column papers come out in reading order only when the PDF wrote them that way, and that is
  usual for TeX. Tables come out as lines with cells separated by spaces.
- Pictures of text (scans) are not read. Only Flate compression is read: LZW and the image filters are skipped. Text
  inside form XObjects is skipped.
- A TeX font without a ToUnicode map reads through its glyph names; uncommon symbols may come out empty.

## Checks
- `packages/agents/test/paper.test.ts`:
  - two hand-built PDFs: a TeX-style page (its own encoding, the fi ligature, kerning, an object stream) and a
    browser-style page (a two-byte font with ToUnicode);
  - each refusal;
  - the labelled frame;
  - the plan's check;
  - plan, then draft, on a mock provider, with a prompt injection kept inside the frame;
  - a paper too long for the model, refused with nothing sent.
- `tests/ui/orac.e2e.mjs` part 3 (Chromium): a PDF printed by the browser is dropped on Orac, read on the page, sent
  framed, and answered with a plan card that has no Accept.
- Screenshot: project files `sse4/handover-screens/ND4-screens/nd4-plan.png`.

## Next
- ND5, one job as a set (boxes, flow node changes, light-table and data sheet rows, accepted together with one
  Undo). It waits on MP11, the kernel's ND3 and the chart room's flow editor.

## Golden hashes
None.
