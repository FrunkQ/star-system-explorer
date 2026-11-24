# AT7: the Guide, margin notes, the gadget and its cover

Track AT (atlas design section 6, 12.5, 14A note 4). Built 24 Nov 2026.

## What it is
- `ui-contract/src/atlas/margin.ts`: `sse.margin-notes/1` note sets. A note has an `about` (a page kind, `ship` or
  `any`), an optional `when` (a fact at least, at most, between, equal to or present; a tag; all; any; numbers SI)
  and words with `{field}` slots. `meets` tests the audience's scene, never the truth; a band meets a bound only
  when all of it does. `fillNote` says each slot through the figures door; an unknown slot means the note does not
  fit. `pickNotes` is a weighted pick seeded by the page address and edition id (mulberry32 from an FNV seed; no
  clock, no Math.random), at most `margin.max`. `withMargin` adds a `margin` block and the line "Margin notes are
  the Guide's opinions and are not facts." `checkNoteSet` refuses with sentences: no typed figure outside a slot, a
  slot's field must be tested by the note's `when`, catalogued fields, ids unique, under 200 characters.
- **Alex's list** (SSE3 `src/lib/catalogue/guideNotes.ts` in FrunkQ/star-system-generator, from his
  Mostly_harmless.txt): 131 lines.
  - 91 are in the base set `ui-contract/src/atlas/notes/guide.notes.json` as `guide.alex.NNN` (NNN is his line
    number), each given an `about` and a `when` as a first pass. Lines about inhabitants need settlements; wildlife
    needs thick air; sea jokes need a sea; ship notes wait for ship pages.
  - 35 use Douglas Adams' names or phrases and are in the unofficial fan pack `store-packs/guide-fan-notes/` (notes
    and an edition, "The Guide (classic)", extending the Guide with "Megadodo Publications" on its cover). It is not
    built into SSE4.
  - 5 type a figure about the subject, which the honesty rule refuses, so they are **held for Alex** (he rewrites with
    a slot, or says otherwise; nobody else rewrites his jokes):
  - line 7: "The local ecosystem is 90% water, 8% carbon, and 2% misplaced optimism."
  - line 28: "The propulsion system is operating at 99% efficiency and 1% utter confusion."
  - line 83: "The shields are currently at 42% efficiency. The ship's computer considers this a good omen. It isn't."
  - line 96: "The local atmosphere is 12% oxygen, 78% nitrogen, and 10% pure spite."
  - line 117: "Warning: The local civilization recently discovered the concept of marketing. The third planet is currently offering 20% off all gravity until Friday."
  - 15 new notes with slots (`guide.new.*`, weight 2) react to the real world: sea cover, year length, temperature,
    air pressure, star brightness, mass, settlements, core and orbit. Alex reads the set before it ships (design 13).
- Editions gain `cover` (big words, a title with `{place}`, a strap, a publisher) and `device` (case words,
  hardware keys, how a page changes). The loader checks their words like the rest.
- The Guide is a **device**: `look: guide-gadget` (night-blue screen, gold and teal, a cream case, big coloured
  keys) lives in `shell/src/atlas/guide-gadget.theme.ts` with `ATLAS_LOOK_EXTRAS` (case, keys, cover, title, label
  and margin colours) until AT14 and AT15 move looks into skins. Key text passes 4:1 on every key.
  - **The cover** shows once each time the Guide is opened; a press anywhere, Enter or an arrow goes past it. The big
    key (its label is the cover's big words, "DON'T PANIC") brings it back. Home goes to the top of the book.
  - Pages change with a **stepped wipe** (seven steps, about a quarter of a second); reduced motion gives a plain
    change. No paper effects in the Guide.
  - Margin notes sit beside the page (under it on a phone), signed "The Guide says". On the GM's own view each has
    "Not this one", which turns it away for this sitting and the pick moves on, still deterministic.

## Not yet
- "Not this one" is kept for the sitting; keeping it with the campaign, and the GM's own "Guide's notes" with a
  "Known to" eye, need a campaign field (with SV's notes work).
- Verdict and subtitle tables (design 6.5): next in this track.
- Minerva's drafted notes are AT11 (Orac thread).
- The fan pack's loading into the stack (note sets and editions from packs) arrives with the pack loader step.

## Checks
- `node --test packages/ui-contract/test/atlas.test.ts`: notes fit by what the audience knows; a slot is the
  figures door's text; the same page gives the same notes; a note never reads an unknown fact; bands meet bounds only
  wholly; "Not this one"; Survey has no margin; both sets load; no borrowed names in the base set; every line of
  Alex's list is placed or held; the fan edition puts the publisher on its cover and the base Guide does not; the
  loader's refusals.
- `tests/ui/atlas.e2e.mjs`: the Guide opens on its cover (DON'T PANIC, the place, no publisher, focus on the cover);
  Enter opens; one or two signed notes and the opinions line; Next and Back keys; the same notes on return; the cover
  key brings the cover back; the Guide fits a phone.
- Screenshots: project files `sse4/handover-screens/AT7-screens/`.
