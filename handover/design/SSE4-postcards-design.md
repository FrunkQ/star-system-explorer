# SSE4: postcards and hold backdrops (track PC)

Status: **PROPOSED, 25 Nov 2026; the idea and the name are welcomed by Alex (01:08 UTC: "postcards - amazing idea"), with three rulings in section 12A, which wins where it differs from the rest. Not yet accepted in full: questions 2 to 7, 9, 11 and 12 of section 12 and new questions 13 and 14 are still open, and no builder starts until he answers them.** Design and mockup only. PC is a code only and is never shown in the product.

Mockup, clickable: project files `sse4/handover-screens/PC-screens/postcards-mockup.html`, with desktop and phone screenshots beside it and its source in `PC-screens/source/`. The pictures in the mockup are painted stand-ins, not SSE4 output; each says so.

Alex's brief, in his words (25 Nov 00:51): "space quotes - an option on the hold screen on transmissions - original purpose to a beautiful backdrop - which we can now create dynamically - a hires still of a library of cool preset screenshots - we should have that as a savable item - peeps can send their faves and the world is created enough to show them - and then they can see why it looks pretty."

Read with: `SSE4-playbook-and-switchboard-design.md` (bookmarks in 4A, holding in 6, pictures in 4.7), `SSE4-atlas-and-reports-design.md` (margin notes), `SSE4-view-styles.md`, `SSE4-secrets-and-visibility-design.md`.

## 1. The short answer

1. **A postcard is a bookmark that can travel.** A bookmark already says where, when, which instrument and how it is looked at. A postcard adds the three things a bookmark lacks to stand alone: the exact camera, a fixed date, and enough of the world to draw it somewhere else. It is a few kilobytes of plain data. It is never a picture file.
2. **Every screen draws its own picture from it.** The front of the card is a live, high-resolution still made by the renderer's still tier (`photograph`, R3) on whatever device opens it. So it is sharp on a 4K wall, right on a phone, and can be redrawn in another band.
3. **The back of the card says why it looks like that.** Where and when it was taken, and three to five reasons read from the same facts the renderer drew from ("The sea is green because..."), each one a press away from its physics block. "Go there" opens the place itself.
4. **"Created enough to show them" is one rule with three cases.** A place in the generated universe travels as its address alone. A place someone authored travels with a clip of the bodies in frame (the existing `sseClip: 2`). A place in the campaign you are in carries nothing: the world is already there.
5. **A hold can have a ground.** A holding card is words over a ground. Today the ground is blank. A ground may now be a postcard, or a set of postcards taken in turn. "Quotes" is one more kind of holding card beside Blank, Plain and Jokes. A quote over a postcard is SSE3's hold screen, drawn live. The transmission card takes a ground the same way.
6. **A set ships, and people send theirs.** The base presentation pack carries a first set of about 24. A postcard goes to a friend as a link, a file or the clipboard, and to everyone through a Postcards shelf on Explorers (that part needs Alex's own yes, section 9).
7. **No new mechanism.** It reuses the bookmark, the clip, the still tier, the holding cards file, the Atlas margin-note format, the role's knowledge and the role's look.

## 2. Words

| Word | Means |
|---|---|
| **Postcard** | A bookmark with a camera, a fixed date and its world, able to be opened anywhere. Proposed name (question 1) |
| **Front** | The picture, drawn live |
| **Back** | Where, when, how it was taken, why it looks like that, who made it |
| **Stamp** | A small kept picture (about 640 px wide) shown at once while the real one is drawn, and on a list |
| **Ground** | What a holding or transmission card's words sit over: nothing, a postcard, or a set of postcards |
| **Set** | A named list of postcards in a pack (`sse.postcards/1`) |

## 3. What a postcard holds

```jsonc
{ "schema": "sse.postcard/1", "id": "pc-…", "name": "Vault's rings from the night side",
  "caption": "Backlit rings, nine hours before the equinox.",        // optional, the maker's words
  // The bookmark, unchanged (ui-contract bookmark.ts), with two rules: at_yr is never null, instrument is one that can draw a still.
  "bookmark": { "schema": "sse.bookmark/1", "scope": "u:…/kestrel", "at_yr": 4.6100213e9, "instrument": "system",
                "view": { "style": "true", "seen": "free", "scale": "real", "band": "visible", "hud": false }, "selected": "u:…/kestrel/iv" },
  // New: the camera. Absent, the view's fixed "Seen from" framing is used.
  // Stored the way the renderer builds its camera (renderer thread, section 14): the eye is a direction in the body's frame and a distance.
  "pose": { "about": "u:…/kestrel/iv", "eye_dir": [x, y, z], "eye_m": 4.1e8, "up": [x, y, z], "fov_deg": 28,
            "exposure": { "mode": "long", "ev": 0 },                 // "long" | "eye"; ev is 0 until exposure is metered (VP4)
            "keep": [0.18, 0.22, 0.80, 0.74] },                      // the part of the frame that must stay in view, any screen shape
  // New: how to get the world.
  "world": { "how": "address", "root": "u:<id>", "packs": [ { "id": "base-physics", "range": "^4.0" } ], "made_with": "4.0.0" }
        // or { "how": "clip", "clip": { "sseClip": 2, … }, "packs": […], "made_with": "…" }   an authored place
        // or { "how": "here" }                                                                  kept inside its own campaign
  "credit": [ "Alex" ],                                              // oldest first, as a clip's creditChain
  "stamp": "data:image/webp;base64,…" }                              // optional, small; checked like picture.ts
```

- **`pose` goes on the plain bookmark as an optional field**, beside `view` and `selected`. Today a bookmark made in Free view does not come back to the same camera (`ViewChoice` has no pose). That is a gap in bookmarks. `bookmarksFrom` reads it leniently: a malformed pose is dropped, never the bookmark.
- **One store (agreed with the Run desk thread).** Inside a campaign there is no separate postcard object: a bookmark with a pose and a fixed date is a postcard, kept in `workspace.bookmarks`, and the existing `show: { bookmark }` source carries it (`showOf` carries the pose). `sse.postcard/1` is only the envelope a postcard travels in: the bookmark plus its world, credit and stamp.
- **`keep` is how one postcard fits a wall TV and a phone.** The camera's direction is fixed; the field of view opens until the kept part fits the screen's shape. It also tells a quote where the quiet side of the picture is.
- **Nothing about players** is in it: no role, no look, no knowledge. That stays the Switchboard's.
- **Presentation only.** Never read by physics, never in the state hash.

## 4. "The world is created enough to show them"

| The place is | It travels as | Opening it |
|---|---|---|
| in the generated universe, untouched | its address, the universe's root and the packs it was made under | the kernel resolves that one system (or galaxy, or patch of web) at that date. Nothing else is made |
| authored or edited (a campaign world, a hand-built system) | a clip of the bodies in frame and what they orbit, as authored facts | the clip is laid into a small visiting universe of its own; physics works the rest out again, as a pasted clip does |
| in the campaign that is open | nothing (`here`) | drawn from the campaign, through the role's knowledge |

Rules that keep this honest:

1. **A postcard carries what its maker's seat could see.** A player's postcard holds only what their role knows. A GM making one to send chooses "as the table sees it" (the default once a campaign has roles) or "as I see it". Notes, secrets, Known-to records, factions and anything outside the frame never go. Before it leaves, "What goes with it" lists the bodies and the count of facts in plain words.
2. **Opening one never changes your campaign.** It opens as a **visit**: read-only, marked "Visiting: a postcard from Alex", with the ordinary place line, cards and Why? working. "Keep this system" is the ordinary paste of the clip, on purpose, with its credit chain.
3. **Missing packs and newer physics are said, not hidden.** Without a pack it needs, the card draws with what is there and the back says "Drawn without the Kestrel Reach pack, so it may differ." If the kernel is newer than `made_with` and the place resolves differently, the back says so and the stamp shows how it looked when taken. Then and now side by side is a small physics lesson, not a fault.
4. **"The same picture" means the same world, the same camera and the same light.** The kernel is deterministic, so the world is identical. Graphics cards differ by a hair in the last bit, so the note never promises identical pixels.
5. **A view that cannot draw True on this device** shows the stamp, enlarged and marked "kept picture", with the reason, in the same way view styles already speak for a weak device.
6. **Date precision.** `at_yr` as a 64-bit number is good to about a minute at the universe's present age: enough for a still. Finer (a cloud deck at one second) is the kernel thread's call if a view ever needs it.

## 5. The card on screen

**Front.** The picture, edge to edge. The stamp shows at once, the first drawn frame replaces it, and it sharpens as the still tier refines (the progress is a thin line at the foot that goes when done). Three quiet controls at the corner: **Turn over**, **Go there**, **Send**. Band can be changed on a card whose view has other bands: the same camera in infrared is one press.

**Back.** Laid out like the back of a postcard, in the chart room's type:

- left: the caption, then **Why it looks like this**, three to five lines, each ending "why?" into the physics as everywhere else;
- right: where (place line), when (date, and the local time of day where that means something), how (band, scale, field of view, exposure in plain words: "A long exposure. To the eye the rings would be fainter and the night side black"; where the sky, the glare or the long-exposure lift make the picture a composite, the back says so), who (credit chain), and what it travels with (address or clip, packs);
- foot: **Go there**, **Keep** (to this campaign's postcards), **Send**, **Save picture**.

**Where the reasons come from.** Every True recipe pass already draws from kernel facts (air from its composition, sea from the liquid's absorption, rings from their particles, the glow from the eye's glare equation). The ask of the renderer is that a pass which contributes to the frame also hands back one line: what it drew, the facts it read, and the block they came from. The back shows the lines for the passes that cover the most of the picture. Later, pressing a spot on the front answers "what am I looking at here?" with that spot's line. This is the "clever user can test it" touch: change the liquid, and the reason and the sea change together.

**Honesty.** A postcard of an impossible or fictional thing carries the honesty mark on its back for the GM, as on the chart. A postcard taken at Readable scale says "sizes exaggerated" on its back; the shipped set is taken at Real.

## 6. Making and keeping one

- **One place to make it: the Bookmarks button.** "+ Bookmark where I am" stays. A bookmark's row gains **Make a postcard**. In True the existing "Take photograph" result gains **Keep as postcard** beside Save picture. No other entry.
- **Kept with** the campaign's bookmarks (one store, section 3), and on the device outside one. Exported with the campaign. The Bookmarks list marks the ones that are postcards with a small stamp.
- **The wall.** A screen document `postcards` with its own `rail` entry (never a tab), under Look at every scale: this campaign's postcards, then each set from the packs, then Explorers. A row of stamps; press one and it opens from where it sat. The Bookmarks list's foot links to it.
- **Framing help, small.** While "Make a postcard" is open the stage shows the `keep` box as four corner ticks that can be dragged, and a rule-of-thirds grid at HUD on. No filters, no depth of field, no lens flare: nothing is added that the physics did not put there (the eye's glare is physics and stays).

## 7. Sending

A postcard is small text, so it goes the ways a clip goes:

- **Copy**: to the clipboard; pasted into SSE4 anywhere it opens as a visit.
- **A link**: `v4.starsystemx.com/#postcard=…` when it travels as an address (about 300 characters). Opens the card with nothing installed.
- **A file**: `.ssepostcard.json`, for one that carries a clip or a stamp.
- **Explorers**: section 9.

"The GM streams only data" holds without a special case: a postcard fired at a screen is data, and that screen draws it.

## 8. Holds and transmissions with a ground

Extends section 6 of the Playbook design; the Run desk thread owns it.

1. **Kinds of card:** Blank, Plain, Jokes, and now **Quotes**.
2. **Ground:** None (today), **A postcard**, or **Postcards in turn** (from the universe's default set first, another set, or the campaign's own; never the same twice running; changes by a slow fade every few minutes, and not at all under reduced motion).
3. **On the line card**, under the kind: one row "Ground" with the three choices and a strip of stamps. A click on a line still uses the kind and ground used last, so the common case stays one press.
4. **What a held screen shows:** the picture full screen; the card's words in the display face on the picture's quiet side over a soft shade so they always read; for a quote, who said it beneath. One piece of words only: a waiting line from the Atlas banner picker appears at the foot only with Blank and Plain cards, never under a quote or a joke (Alex, section 12A point 5).
5. **Still nothing behind the card.** A hold stays a send: the screen gets the postcard and the words and nothing of what it was showing.
6. **Knowledge.** A postcard from a set is somewhere else entirely, so there is nothing to hide. A campaign postcard (`here`) is drawn through the role's knowledge like everything else; if the role does not know the place, that screen falls back to the next one in turn, or to no ground.
7. **Look.** Drawn through the role's look, with the same "look off for this picture" switch as a Play picture (Playbook 4.7). A green terminal showing a green postcard is right; a wall TV in the plain look gets the photograph.
8. **Transmission cards** ("Incoming transmission…") take a ground by the same field in the same file.
9. **OBS for free:** a Stream role on hold with Postcards in turn is a "back soon" screen.
10. **Data (as the Run desk thread asked):** the ground sits on the held card (`HoldCard` in board.ts), not on the hold source: `ground: "none" | { bookmark: id } | { postcard: {…} } | { set: "base" | "campaign" }`. A hold already wins over every layer as a card on the wire, so a card with a ground reaches the screens with no new layering rule. A card in `sse.holding-cards/1` gains `kind: "quote"` and `by`; `by` is not part of a card's identity (cards are de-duplicated by kind plus words). `CardDeck`'s fallback becomes "any other kind, then Stand by".

**The quotes.** One more note set in the Atlas margin format (`ui-contract/src/atlas/margin.ts`, arriving with PR 481), each line with `uses: ["waiting"]` and two new optional fields the Atlas thread adds: `by` (who said it) and `source` (a book, a mission, a year). The loader treats `source` as a citation, not a fact, so a year in it does not trip the figure check. The hold screen asks `pickBanners` with the situation "holding". SSE3's 90 are the source list. Which go in the base app is the rights question the Atlas thread already put to Alex; it is repeated as question 7 so one answer serves both.

## 9. The shipped set, and people's favourites

- **The base set:** about 24, in the base presentation pack as `sse.postcards/1`, all travelling as addresses or as clips of the shipped systems (Solar System, Kestrel, the showcase worlds). Chosen with the trailer thread from its lists (project files `sse4/trailer/STORYBOARDS.md` section H, beauty shots H1 to H4, and `SHOTLIST.md`, the 25 Nov GPU takes; strongest so far: shot 26 the galaxy in True edge-on, shot 25 the Solar System in True, shot 06 a generated system's tilted orbits), so a place worth filming is also a postcard, and the trailer rig can film from a postcard's pose. Spread on purpose across scales (surface, world, rings, star, system, galaxy, web) and bands (a few infrared and X-ray).
- **A check, not a golden:** a test opens every shipped postcard, resolves its world and confirms it draws on the canvas tier. A postcard that stops resolving fails the build with its name.
- **Explorers, a Postcards shelf (needs Alex's own yes: it adds a kind and stores a small picture).** A postcard publishes like a clip: the site stores and serves the bytes as they are. The one new thing is the stamp, a picture made in the publisher's browser, which goes through the same moderation gate as every file. Captions and names are text under the existing report and takedown routes. The shelf shows stamps; opening one in SSE4 draws it live.
- **"Send it in":** on a published postcard, "Offer for the shipped set". Alex picks; a chosen one joins the next release's base set with its credit.
- **Rights are simple here** because a postcard is a view of a world, and worlds on Explorers already carry their attestation. A postcard of a paid or fan pack's world needs that pack to open, and says so.

## 10. Seams (asked by message 25 Nov; answers go in section 14)

| With | The seam |
|---|---|
| Run desk (owns bookmark.ts, board.ts, holding cards) | optional `pose` on `sse.bookmark/1`; `ground` on the held card; the Quotes kind; "Make a postcard" on a bookmark row; presets use the existing `show: { bookmark }` |
| Renderer | `photograph` for every True view from a pose, at a given size and shape, refining; `exposure_ev` and `fov_deg` in the pose; the per-pass reason line; the closer camera in True uses the same pose shape |
| Atlas | the quote set as a margin-note set with `uses: waiting`; the banner picker on hold screens |
| Explorers | a `postcard` kind and shelf, stamp storage and moderation, "Offer for the shipped set" (all waiting on Alex) |
| Trailer | one list of beauty shots for both; clips filmed from a postcard's pose |
| Chart room | the Bookmarks list's row action and foot link; the wall's rail entry |
| Kernel | none new: resolve at an address and date, and clip paste, exist |

## 11. Packages (proposed; for a builder thread "Build postcards and hold backdrops", Opus)

| Pkg | What | Owner | Needs |
|---|---|---|---|
| **PC1** | Contract: `pose` on the bookmark; `sse.postcard/1` (ui-contract postcard.ts): read, check, make from where I am, pack for sending through a seat's knowledge, link, file and clipboard forms | new thread, with Run desk for bookmark.ts | nothing |
| **PC2** | A still of any True view from a pose at a size and shape; the reason line per pass | renderer | R3 |
| **PC3** | The card: front, back, Turn over, Save picture; "Make a postcard" and "Keep as postcard"; the wall with its rail entry | new thread | PC1; PC2 for views beyond the globe |
| **PC4** | Opening one from outside: the visit, Go there, Keep this system, missing-pack and newer-physics lines | new thread | PC1 |
| **PC5** | Grounds on holding and transmission cards; the Quotes kind; the line card's Ground row | Run desk | PB5, PC1 |
| **PC6** | The base set of about 24 and its check | new thread with trailer | PC2, PC3 |
| **PC7** | The quote set | Atlas | question 7 |
| **PC8** | Explorers: kind, shelf with an always-free selection, stamp, offer; the link page with its preview picture; private sends with a message; the Discord webhook | Explorers | Alex's yes |
| later | a postcard on the start sheet; a poster-size picture through the Atlas print path; a "living postcard" that lets time run slowly; press a spot for its reason | | |

PC1, PC3 for the globe, and PC5 can start on main today. Only PC2 and PC8 wait on anyone.

## 12. Questions for Alex

1. **The name "postcard".** It has a front and a back, and it is something you send. Recommend: yes. (Others considered: plate, snapshot, vista.)
2. **A postcard is a bookmark plus camera, date and world, and the camera goes onto plain bookmarks too.** Recommend: yes.
3. **Authored places travel as a clip of the bodies in frame, through what the maker's seat can see; a GM chooses "as the table sees it" (default) or "as I see it".** Recommend: yes.
4. **Opening someone's postcard is a read-only visit; taking the system into your campaign is a separate, deliberate "Keep this system".** Recommend: yes.
5. **The back gives reasons read from the renderer's own passes, each with its why?** Recommend: yes, starting with the globe and system views.
6. **Hold and transmission cards take a ground (None, A postcard, Postcards in turn), and Quotes is a fourth kind of card.** Recommend: yes.
7. **Quotes and rights (the Atlas thread's question, once for both):** real scientists' and astronauts' words with their names in the base app; film, song and game quotes in the free fan pack. Recommend: yes.
8. **Postcards carry a small stamp picture (about 640 px).** It lets a list and a weak phone show something at once, and shows "as taken" when physics has moved on. Recommend: yes.
9. **In 4.0:** PC1 to PC7. Recommend: yes; it is small and it makes the first screen a newcomer sees at a table beautiful.
10. **Explorers gets a Postcards shelf with moderated stamps and "Offer for the shipped set"** (your own yes needed: a new kind and stored pictures). Recommend: yes, after PC1 to PC6 work without it.
11. **No filters or effects on a postcard beyond the role's look: framing, band, exposure and date only.** Recommend: yes ("lean into the physics, never fake it").
12. **A postcard on the start sheet as its backdrop, changing each visit.** Recommend: later, once the set exists.

## 12A. Alex's rulings, 25 Nov 2026 01:08 UTC

His words: "postcards - amazing idea - always a selection free in the shop - it will tell the user packs it needs to download to replicate - so really can only show content in game or on the store".

1. **The name is postcard** (question 1: yes).
2. **There is always a free selection of postcards in the shop** (Explorers). This answers question 10 in direction: the shelf is wanted, and it is never empty and never paid. The Explorers work itself (a new kind, stored stamps) is still built by the Explorers owner as its own package, PC8.
3. **A postcard tells the user which packs it needs to download to replicate it.** On the back, "Travels as" becomes a list of the packs needed, each with its state on this device (have it, free to get, paid) and a "Get" press that goes to that pack in the shop. A postcard of a paid pack's world can sit in the free selection as that pack's shop window: the stamp shows the picture, and the live still and "Go there" need the pack.
4. **So a postcard on the shop shows only content that is in the game or on the store.** A published postcard travels as an address plus named packs, never as a carried clip: everything it shows must come from the base game or from something published on Explorers (a pack, or a published campaign or system). Someone whose picture is of their own authored world publishes that world first, and the postcard names it. The site refuses a postcard whose packs or world it does not hold, and says which.

5. **A quote stands alone** (01:10 UTC: "Quote does not need a joke below as well"). A held screen carries one piece of words. Over a Quote card there is no waiting line or joke at the foot: only the quote, who said it, and the quiet labels (the role, the place). The Atlas banner picker's waiting line is used only with Blank and Plain cards, and a Joke card is its own words with nothing added. This replaces section 8 point 4's "top and bottom lines" for quotes and jokes.
6. **A universe has a default set** (01:10 UTC: "a universe will have a default set"). A universe pack carries its own `sse.postcards/1` set (and may carry its own quote set). In a campaign in that universe, "Postcards in turn" and the wall open on that universe's set first, so a setting's hold screens show its own places with no setting up. The base app's set of about 24 is the default set of the base universe, and the fallback for a universe pack that brings none. The GM can still choose another set or the campaign's own.

7. **Every band, and Composite** (01:15 UTC: "i love the way you can pick IR - a wavelength - and even better when we have others and composite views too"). The front's band switch is the standard Band control: every band the view can draw (Visible, Infrared, Ultraviolet, X-ray, Radio, Mag), then Composite, with the unavailable ones quiet and saying why, as everywhere. A postcard is kept in one band (or one Composite mix, stored as the mix), and whoever opens it may look in any other. The reasons on the back are the reasons for the band showing.
8. **Send to anyone, with a personal message** (01:15: "being able to send to anyone with a personal message would be cool"). Send gains "Write on the back": a short message and who it is from, shown on the right-hand side of the back, where a postcard's message goes. The message box opens already holding **"Wish you were here!"** (Alex, 01:15: 'default message "Wish you were here!" :)'), ready to send as it is or to type over; a universe pack may give its own default line. The message belongs to that sending, not to the postcard: it travels in the link or file, is never published to the shop, and is plain text only (no links made live, no markup).
9. **A postcard link works for someone without SSE4, and takes them there** (01:15: "if one is sent to someone without SSE it can take them there - a great tool to build interest"). The link opens a page that needs nothing installed: the stamp at once, the message, the caption, and "See it live", which opens SSE4 in the browser on that postcard as a visit (the alpha notice and the way to V3 for real use stay as they are on the opening page). Where the device cannot draw True the stamp stays, with the reason. This is why the stamp is kept (question 8 is in effect answered yes).
10. **The Discord sharing channel** (01:15: "We can have them come through the sharing channel on the discord too"). A postcard link pasted into Discord unfurls as a picture card: the page carries the stamp as its preview picture, with the name and caption. Posting newly published shop postcards into the channel automatically is a webhook on the Explorers side, part of PC8 and needing Alex to supply the channel's webhook address as a secret.

Two things follow for the builder. A link that shows a stamp and a message needs somewhere to keep them, since a web address cannot hold a picture: a shop postcard already has its page; a private send stores a small sealed copy (stamp, message, the postcard data) on Explorers behind an unguessable address, with a lifetime (question 14). And a private send is the first user-written text and picture that is not moderated before someone sees it, so it carries a Report link and is never listed or searchable.

14. **New question: private sends kept on Explorers behind an unguessable link, for how long?** Recommend: one year from the last time it was opened, then removed; the sender can remove it at once from "Sent" on the wall. Needs your own yes with PC8, since it stores user content.

What this changes above: section 4's "clip" case and section 7's file with a clip are **not for the shop**. Whether a clip-carrying postcard is still allowed directly between friends (copy, file) is new question 13. Section 9's moderation gets simpler: the only new thing the site stores is the stamp.

13. **New question: may a postcard still carry a clip of an unpublished authored world when sent directly to a friend (copy or file, never the shop)?** Recommend: yes. It is the same act as pasting a clip to a friend today, through what the maker's seat can see, and it keeps "send me that view" working for a GM's private world. If no, every postcard is an address plus packs and `world.how: "clip"` is dropped.

## 13. Standing rulings, checked

One general mechanism (bookmark, clip, still tier, holding cards, margin notes reused; nothing special-cased). Don't copy V3 (static images become live stills that explain themselves). GM streams only data. Honesty marks on the GM's view only. Sound off. Reduced motion honoured. No new tab. Tags never feed physics; nothing here feeds physics. Figures on the back go through `figures.ts`.

## 14. Seam answers (25 Nov 2026)

- **Run desk** ("Build Run desk playbook and players' side"): yes to all four, with changes now folded into sections 3, 6 and 8. Pose optional on the bookmark and read leniently; the ground on the held card, not the hold source; the Quotes kind with the `CardDeck` fallback and identity notes; one store, so no `show: { postcard }`. It will own PC5 once Alex accepts.
- **Atlas** ("Build star atlas and editions"): a margin note has no speaker or source today; it adds `by` and `source` in PC7 and will own PC7. margin.ts reaches main with PR 481; `uses`, `pickBanners` (situation "holding") and the waiting lines follow in its next PR.
- **Trailer** ("Capture footage for SSE4 trailers"): no separate stills list; the candidates are in the two files named in section 9. Happy for postcards to be picked from them and for the rig to film from a postcard's pose later.
- **Renderer** ("Build photoreal renderer"): sound, and nothing in it is costly. (1) A still of any True view is the same `gpu.draw` with its `finish` (VP1, PR 488) into an offscreen target of the asked size, read back once; refinement uses the tiers thread's accumulate on FinishFrame (jittered frames averaged before glare), so a postcard matches the live view exactly, and `photograph` becomes a thin caller of that, not a second renderer. (2) The pose is stored as the camera is built: frame body's address, eye as a body-frame direction plus distance from centre in metres, up, `fov_deg`, the keep box; style, band, seen and the date come from the bookmark. The close-up camera fits it (distance down to a grazing height, plus a ground point the detail ladder reads). There is no metered exposure yet: True is fixed at 1 (white in the star's daylight, fudge row VP1) with a choice of long or eye, so the pose stores `exposure: { mode, ev: 0 }` and `ev` becomes the metered value when VP4 meters it. (3) Reason lines are cheap: `trueLook` (true-body.ts) is the one door every True face comes through and already collects what a face could not show; it is extended to a list per body of pass, facts read and block id, with coverage from the body's projected disc. A pixel pick is one extra id draw at the click. The sky, the glare and the long-exposure lift give their own lines, so the back can say which parts are a composite.
- **Explorers**: not asked; nothing there moves before Alex's yes to question 10.

## 15. Picking this up without the chat

Read sections 1, 3 and 4, then open the mockup. The mockup shows: a held wall screen with a quote over a postcard, light and through a green look; the same on a phone; the card's front and back; the wall; the Switchboard line card with its Ground row; and "What goes with it". It does not show the visit, the Explorers shelf or the framing ticks. If Alex has answered, his answers are in `handover/decisions.md` and this note's status line; if he has not, nothing is to be built.
