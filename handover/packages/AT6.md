# AT6: letterhead, the placeholder logo, the built-in logos

Track AT (atlas design section 7, 12.4, 14A note 2). Built 24 Nov 2026.

## What it is
- `ui-contract/src/atlas/letterhead.ts`: `withLetterhead(page, edition, ctx)` puts a `letterhead` block on top of a
  page when the edition's `letterhead.use` is not `none`. The lines are the edition's template with slots filled
  (`{brand.name}`, `{doc.number}`, `{date}`, `{audience}` and the rest of `EDITION_SLOTS`); the stamp likewise. It
  adds paper, never a fact: the blocks below are the page's own (tested).
- The mark, in order: the brand's own logo (a data URI), else a built-in one the GM picked, else the placeholder
  drawn in text. With no brand at all, the name is the edition's `brand.fallback` label ("Independent Survey Office").
- Document number `docNumber(brand, place, page, title)`: initials, place, page, subject (`KEL-KES-02/TARN`).
  Clearance is "GM EYES ONLY" on the GM's copy, else the audience's name.
- `ui-contract/src/atlas/ascii.ts`: `asciiLogo(name, est)` in our own 5 by 5 block font (`blockfont.json`, A to Z
  and 0 to 9), each letter drawn with itself. Initials of up to four words, or one word spelt to eight letters. Three
  flaws from an FNV seed of the name (no clock, no Math.random): the bottom edge one dash out, a flourish
  (`=>`, `(o)`, `*`, `~~`) nudged off line, and an apology line. It has an `aria-label`. "est" is the campaign
  calendar's year at the campaign's start.
- `ui-contract/src/atlas/logos.json`: SSE3's starter logos, rescaled to at most 400 px wide so each passes
  `checkBrand`: SSE (Alex's own), Interspan, Kelido, Nexum, Terra and TSEC (World Zero, CC BY 4.0, credited in NOTICE
  and per logo). Weyland-Yutani is left out (open with Alex). They are light marks for a dark ground, so `mono` draws
  them as a luminance mask in the page's ink or the brand's accent, on any look.
- Brand `accent` (#rrggbb only; anything else is ignored) colours the rule and the stamp; `motto` sits under the
  name. Both fields are added to `SeatBrand` by the Run desk thread (its PB6 to PB8 PR). The Atlas reads them as
  optional.
- On screen the GM browsing an edition with a letterhead has a **Letterhead** picker (drawn in text, or a built-in
  logo). On a player's screen the role's own brand comes through `AtlasHost.brand(audience)` with AT9.
- Copy as text and Save page carry the letterhead (the art as a code block in Markdown).

## Checks
- `node --test packages/ui-contract/test/atlas.test.ts`: the same name draws the same art; only the letters
  themselves; edge, flourish and apology present; the letterhead's lines, number, stamp, accent and motto; a logo
  replaces the art; Survey has no letterhead; a bad accent is dropped; every built-in logo passes `checkBrand`, is
  credited, and none is Weyland-Yutani.
- `tests/ui/atlas.e2e.mjs`: Company shows the drawn logo and "CLEARANCE: GM EYES ONLY"; picking Kelido replaces it
  and the number reads KEL-KES-02/TARN; going back draws the same art; the Guide has no letterhead.
- Screenshots and PDFs: project files `sse4/handover-screens/AT6-screens/`.
