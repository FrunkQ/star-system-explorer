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
  - All 131 are in the Guide's set `ui-contract/src/atlas/notes/guide.notes.json` as `guide.alex.NNN` (NNN is his
    line number), each given an `about` and a `when` as a first pass. Lines about inhabitants need settlements; wildlife
    needs thick air; sea jokes need a sea; ship notes wait for ship pages.
  - 36 use Douglas Adams' names or phrases. They were first split into an unofficial fan pack; Alex, 25 Nov 00:57:
    "keep them all everyone is fine with a douglas adams joke. the whole point is that this IS the guide". So they are
    in the Guide's own set, the fan pack is gone, and the Guide's cover carries "Megadodo Publications" as in SSE3.
    This overrides design 6.6 and 14A note 4 on this point.
  - 5 typed a figure about the subject. Alex, 25 Nov: "let them through as obvious jokes BUT have them as the
    proportion that adds up over 100% - then you know you can rely on what's left." They are back with their
    numbers made plainly impossible: lines 7, 28 and 96 now total 110% (12% misplaced optimism, 11% utter confusion,
    20% pure spite), line 117 offers 120% off all gravity, and line 83 has shields at 142%. The loader's rule
    (`obviousJoke` in `margin.ts`): a note may type percentages only when they total over 100% or one is over 100%;
    every other figure still goes through a slot.
  - 15 new notes with slots (`guide.new.*`, weight 2) react to the real world: sea cover, year length, temperature,
    air pressure, star brightness, mass, settlements, core and orbit. Alex reads the set before it ships (design 13).
- **Verdict and subtitle phrase** (design 6.5, `ui-contract/src/atlas/voice.ts`): tables of rules in the edition
  (`verdict`, `subtitles`), in the notes' condition language, first rule met wins, nothing said when none is. Each
  carries its Why (the facts read, said by the figures door); on screen the stamp opens to show it. The Guide's:
  Do not, Bring a suit, Mostly survivable, Bring a mask; the damp, big, chilly, toasty, airless, dim and bright one.
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
- Minerva's drafted notes are AT11 (Orac thread).
- Note sets and editions from packs (`atlasDocsOf`) reach the view with the pack loader step.

## Seen, for another thread
- The Guide vocabulary's year sentence reads "Birthdays come round every about 214 days" (the figures door's "about"
  after "every"); `vocab.guide.json` in `shell/src/documents` is not this thread's.

## Checks
- `node --test packages/ui-contract/test/atlas.test.ts`: notes fit by what the audience knows; a slot is the
  figures door's text; the same page gives the same notes; a note never reads an unknown fact; bands meet bounds only
  wholly; "Not this one"; Survey has no margin; the set loads with all 131 of Alex's lines; the Guide's cover has
  DON'T PANIC and Megadodo Publications; the loader's refusals.
- `tests/ui/atlas.e2e.mjs`: the Guide opens on its cover (DON'T PANIC, the place, Megadodo Publications, focus on the cover);
  Enter opens; one or two signed notes and the opinions line; Next and Back keys; the same notes on return; the cover
  key brings the cover back; the Guide fits a phone.
- Screenshots: project files `sse4/handover-screens/AT7-screens/`.

## The one-liners at the top and bottom of screens (Alex, 25 Nov)
SSE3 showed the same list as random "guide tips" banners at the top and bottom of screens when a preset turned them
on. Alex: categorise them "by use case to use more appropriately - i.e. make them tag sensitive to match the scene
and create more to fill out the punchlines."
- Every line has `uses` (`NOTE_USES`): `page` (the Atlas margin), `arrival`, `travel`, `danger`, `settled`, `empty`,
  `waiting`, `ship`. Left out, a ship line is for ships and travel and any other for the page. Alex's lines were
  sorted by hand; the margin only draws on `page` lines.
- 42 new lines (`guide.line.*`, written 25 Nov, not Alex's and not Douglas Adams') fill the thin cases: twelve
  waiting lines, eight travel, six arrival, four empty, six danger (each on a fact: heat, cold, thin or crushing air,
  listed hazards), four settled, two ship. Every use case has at least six lines (a test holds this).
- `pickBanners(scene, address | null, situation, sets, opts)`: two different lines whose use fits the situation
  (`SITUATION_USES`: sector, system, world, ship, landing, holding), whose kind fits the subject, and whose
  condition the audience's facts and tags meet. A holding screen (no subject) shows only lines needing no fact.
  Seeded by situation, subject and an optional seed, never the clock.
- The Guide edition's device shows them in strips at the top and bottom of its screen (`device.banners`): waiting
  lines on the cover, the page's own on a page. An interface kit names them with `banners: {at, sets}` (the Guide
  starter: both, the Guide's set); wearing them on other screens comes with AT15 and the Run desk.
- Screenshots: `sse4/handover-screens/AT7b-screens/`.
