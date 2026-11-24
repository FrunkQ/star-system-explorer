---
name: sse4-atlas-and-editions
description: SSE4 track AT, the Atlas view with data-defined editions (Survey atlas, The Guide with jokes, Company report) and print under the view; proposed 24 Nov 2026, awaiting Alex's answers
metadata:
  type: project
  modified: 2026-11-24T20:00:10.351Z
---

Alex, 24 Nov 2026 19:52 and 19:54 UTC: SSE3's textual reports and star atlas must come to SSE4. The star atlas is an interactive VIEW players step through, seeing only what they are allowed; "The Guide" is a version of it with "free added jokes"; others may be dry and corporate (a company report with the organisation's logo, or "crap ASCII art" from its name until one is added). Print sits UNDER the view: it prints the page showing, in that edition's style. Editions must be customisable or buildable; no editor UI needed, but a written guide so others or Orac can design them.

Design note (PROPOSED, 12 questions in its section 14 await Alex): `/mnt/project-files/sse4/SSE4-atlas-and-reports-design.md`; mockup and screenshots `/mnt/project-files/sse4/handover-screens/AT-screens/`. Not yet in `handover/design/` (goes there by PR when he agrees). Fills in V3 gap 24 ([[sse4-v3-gaps]]).

The proposal in brief: one view `sse.view.atlas`; an "edition" (`sse.edition/1`) changes words, order, look, margin notes, letterhead and how a gap shows, never a fact. Gaps come from the template and the audience's rung, never from the secret, so black bars are safe beside "nothing leaks by absence" ([[sse4-secrets-and-visibility]]); a GM-only thing leaves nothing; optional "Show as withheld" tease. Jokes are margin notes (`sse.margin-notes/1`) with conditions on the audience's facts, figures only through slots, seeded pick. Company edition uses the role's brand (`SeatBrand`); ASCII fallback from a 5x5 block font. Packages AT1 to AT12; proposed new Opus builder thread "Build star atlas and editions".

SSE3 facts found: source is only on Alex's PC (`C:\Development\star-system-explorer-v2\star-system-generator\src\lib\reports`, `catalogue`); the joke list is `Mostly_harmless.txt` there (131 lines, 36 using Douglas Adams' names or phrases; my recommendation is those go to an unofficial fan pack, Alex to decide).

**How to apply:** check whether Alex has answered before building; never design a separate report generator or a print-time theme picker.

Alex 24 Nov 20:04 to 20:07 (added to the note as section 11A and 12.7): besides the text Atlas (all editions kept, "magical"), players get "app type" interfaces from FOUR solid starters: space trucker green screen and wireframe ("Hauler", my name), slick corporate interactive 3D, slick 2D diagram "LCARS type" ("Deck", own design, never the LCARS name), and The Guide as its own special thing. Built from reusable parts whose controls switch on and off; buttons and interface elements must follow the style. "Scrub" SSE3's infinite sliders: no slider editor; Orac assembles new ones from a structured system; a hand guide exists; the only direct edits are company name, logo and palette (e.g. green to amber). Proposed document `sse.kit/1` (interface kit); questions 13 to 16 added (16 total).
Alex 20:08: the main aim is "the toolkit that can be used to actually build a trillion cool interfaces"; reuse SSE3's transitions: green screens building up rather than flicking, the slow warm up, the disappear to a dot. In the note as section 11A.4a (five moments: power on, arrive, turn, leave, power off; uses the Playbook's "screen delivery" in a look, PB10).
