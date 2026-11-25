# Status: postcards and hold backdrops (track PC)

Owner thread: "Design saved views and hold backdrops" (Fable, design only, 25 Nov 2026) for the note. Builder thread: "Build postcards and hold backdrops" (Opus, opened 25 Nov 01:28 UTC), owning PC1, PC3, PC4 and PC6; from here on that thread edits this file.
Design: `design/SSE4-postcards-design.md`, **ACCEPTED by Alex 25 Nov 2026 01:27 UTC** ("yes to all... they look great and are a great promo tool"). Read its sections 1, 3 and 12A first; 12A (his rulings in his own words) wins where it differs. Mockup in project files at `sse4/handover-screens/PC-screens/`.

PC1, PC3 and PC4 part 1 are built and on main (PR 509, merged 25 Nov 10:21 UTC; logs `packages/PC1.md`, `packages/PC3.md`). The postcards browser test runs in CI (`tests/ui/postcards.e2e.mjs`, Alex's yes). PC is a code only, never shown in the product. For 4.0.

| Package | State | Log |
|---|---|---|
| PC1 contract: `pose` on the bookmark, `sse.postcard/1` | done 25 Nov (ui-contract pose.ts, postcard.ts; the globe keeps its camera) | `packages/PC1.md` |
| PC2 a still of any True view from a pose; the reason line per pass | next; for the renderer thread (agreed, design section 14) | |
| PC3 the card (front, back), making one, the wall | done 25 Nov for the globe (Keep as postcard, the wall, Send); other views draw when PC2 lands; the bookmark row's "Make a postcard" is with the Run desk | `packages/PC3.md` |
| PC4 opening one from outside: the visit | done 25 Nov. Part 1: paste, link, file open a read-only visit; packs and newer-physics lines; Keep this system. Part 2: a sent postcard's world opens on a visiting kernel and its front is drawn live, matching the sender's picture. An address postcard (none are made yet) still shows its stamp | `packages/PC3.md` |
| PC5 grounds on holding and transmission cards, the Quotes kind | done 25 Nov by the Run desk thread (PR 563); "Make a postcard" on a Bookmarks row opens it on the wall (visits.own) | `packages/PC5.md` |
| PC6 the base set of about 24 and its check | queued; with the trailer thread | |
| PC7 the quote set | queued; for the Atlas thread. Quote rights answered yes: real scientists' and astronauts' words in the base app, film, song and game quotes in the free fan pack | |
| PC8 Explorers: shelf with an always-free selection, stamps, the link page, private sends with a message (sender sets its life, a year at most), Discord webhook, "Offer for the shipped set" | no owner and not started, on purpose: waiting for an Explorers owner thread and Alex's own yes on storage | |

Seams agreed by message on 25 Nov with the Run desk, renderer, Atlas and trailer threads: design note section 14.
