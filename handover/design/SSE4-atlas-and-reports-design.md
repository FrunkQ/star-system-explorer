# SSE4: the Atlas, its editions, print, and the four starter interfaces (track AT)

Status: **ACCEPTED by Alex, 24 Nov 2026 20:14 UTC** ("Yes to all - notes below", all 16 questions of section 14), with seven notes of his own that evening and the next morning, recorded in **section 14A, which wins over anything earlier in this note that it contradicts**. Design only; nothing here is built. Packages for builders are in section 13.

Mockup, clickable: `/mnt/project-files/sse4/handover-screens/AT-screens/atlas-mockup.html` (open in a browser; the Edition switch at the top changes the page; the browser's own print shows the print styling). Screenshots beside it: `desk-survey.png`, `desk-guide.png`, `desk-co.png`, `phone-company.png`, and `print-company.pdf`. The mockup is one world's page in three editions. The Guide as Alex wants it, a cheap plastic gadget with a cover to press past, is `guide-mockup.html` (`guide-1-cover.png`, `guide-2-page.png`, and two phone shots). A page of a paper edition caught mid-curl is `desk-survey-page-curl-1.png` (and `desk-co-page-curl.png`); press Next in the atlas mockup to see it move. A second mockup, `starters-mockup.html` (screenshot `starters-four.png`), shows the four starter interfaces of section 11A side by side. Both are pictures of the idea, not specifications of pixels.

Alex's brief, in his words (24 Nov, 19:52 and 19:54 UTC):

> "One thing that I think we have not included so far, that's in SSE3 and all the previous ones, is the whole idea of textual reports. That can be printed or visualized in a kind of star atlas. We had the guide as the one full of jokes. I'd love to preserve that take too, so there is a player view, which is a traditional star guide like that allows them to step through, obviously only seeing the information we let them see. But further to that is we can use that same information, the same blackouts and hidden data to have a cool company report. Which can be themed to their organization. They can throw a logo in the top, or until then we can give them some crap ASCII art based on their company name."

> "and an interactive star atlas as a 'view' (The Guide being a version of that with 'free added jokes') but some may be dry and corporate - perhaps the 'print' function is actually UNDER these - i.e. Print will constrain to what is on that page and be printed in the style of that interactive guide. These must be customisable or buildable - we perhaps don't need the UI but the guide to let others (or Orac) design them."

And at 20:04 and 20:05, after seeing the first draft of this note:

> "i think we need something space trucker/basic 'the greenscreen and wireframe' as one, a slick corporate (all interactive 3d slick), a slick diagram orientated 2d LCARS type, and 'The Guide' - its own special thing - the versions you show above - we keep them all - having this type of text interface with different flavours is magical... but we still need the more 'app type tools' above too - that combine all the UI elements we already have (have the controls on each turn on and offable and have them all as reusable components)... Orac can deal with the details... you just provide him the tools (and write a guide for how someone could put one together by hand). In SSE3 i let people invent infinite varieties with loads of sliders and tools. scrub that shit - get with the times.... we provide these as 4 solid starters - they can create their own using orac. we still have ALL the controls and options and shaders"

> "we are combining a few things here - your ideas and mine - i think they work well together - without complicating things. Allow the user to have some limited editing - company name - colour pallets (even if just green to amber screen)"

**Picking this up cold:** read sections 1 and 2, look at the three screenshots, then section 13 (packages). Section 12 is the authoring guide Alex asked for and is written to stand alone.

## 1. The short answer

1. **The Atlas is a view.** One new view module, `sse.view.atlas`: a paged, readable book of a place (a sector, a system, a world, a ship, a voyage), stepped through with contents and next and previous. It is interactive: press a world's name and you are on its page.
2. **An edition is how the Atlas reads and looks.** Survey atlas (traditional and plain), The Guide (the same pages with "free added jokes"), Company (dry, corporate, on the organisation's letterhead). An edition is a data document. It decides words, order, look, margin notes, letterhead and how a gap is shown. **It never decides a fact.**
3. **One set of facts.** Every edition draws the audience's own scene, the same one the chart and the card draw (`sse-observe`, the player scene). So the Guide and the Company report cannot disagree on a number, and a player's Atlas can only hold what that role knows. There is no report generator and no second copy.
4. **Print is under the view.** "Print this page" prints the page you are on, in the edition you are reading. "Print…" offers this page, this chapter or the whole book, still in that edition. There is no separate report screen and no theme picker in a print dialog.
5. **Gaps are shown by rule, never by the secret.** What the GM alone knows leaves nothing at all. What the audience knows it has not learnt yet (a line the template always has, above their rung) is shown the edition's way: "not yet surveyed" in the Survey atlas, a joke in the Guide, a black bar in the Company report. A bar is drawn from the template and the rung, so a bar never reveals that a value exists.
6. **Jokes are margin notes.** They sit in the margin under "The Guide says", they are chosen by rules that read only what the audience knows, any number in one is the real number, and the same world always gets the same joke on screen and on paper.
7. **A logo, or bad ASCII art.** The Company edition takes the role's brand (already built: `SeatBrand`). With no logo, a small generator draws the initials in block letters made of themselves, in a box that is one dash short, with "(logo pending. do not print.)" under it.
8. **Editions are buildable without a UI.** Section 12 is the format and the how-to. Orac drafts an edition as a change request the GM accepts, like any other document it writes.
9. **The main aim is a toolkit** (Alex, 20:08: "the toolkit that can be used to actually build a trillion cool interfaces"): parts, controls, looks, palettes, deliveries, editions and kits, each a small checked piece of data, with four starters as proof. **Four starter interfaces, no slider editor.** The Atlas is the text side. Beside it a role gets a whole "app type" interface from one of four solid starters: **Hauler** (green screen and wireframe), **Corporate** (slick, interactive 3D), **Deck** (slick 2D diagrams and panels) and **The Guide** (its own special thing). A starter is one data file, an **interface kit**, that picks reusable parts, switches each part's controls on or off, and names a look, a view style and an Atlas edition. SSE3's preset editor with its hundred sliders is not rebuilt. Every option and shader still exists as a field in the data; Orac makes new kits from a sentence; section 12.7 says how to make one by hand (section 11A).
10. **Three things anyone can change without Orac:** the company name, the logo, and the palette (green to amber, and the like).

## 2. What SSE3 did, and what changes

Read from Alex's PC, `C:\Development\star-system-explorer-v2\star-system-generator\src` (the V3 source is not in this project's repositories).

| SSE3 | Where | In SSE4 |
|---|---|---|
| **Paper report**: a dialog (GM or Player, three themes: Retro line printer, Corporate / Industrial, Standard clean, "include constructs"), which opened `/report` in a new window. Sections 01 System overview, 02 Celestial survey, 03 Artificial traffic and infrastructure. Header with DATE and CLEARANCE ("ULTRAVIOLET (GM)" or "PUBLIC"), a TOP SECRET stamp on the corporate GM copy. A4 print styles. | `lib/reports/ReportDocument.svelte` (about 110 KB, one hand-written page), `openReport.ts`, `ReportConfigModal.svelte`, `report-styles.css` | Kept as content: the three sections become the Atlas's system chapter, and clearance, date and stamp become furniture of the Company edition. **Dropped:** the separate window (it failed silently when the browser blocked the popup, their bug B113), the separate theme list, the wall-clock date (the report's date is the campaign's date), and the hand-written page. |
| **The Guide document**: a block model (`DocBlock`: heading, text, key and value rows, tags, lists, image, body disc, schematic) built once and drawn in twelve "document styles" (guide, report, brochure, terminal, amber, blueprint, holotable, hazard, alert, cleanroom, neon, greyscale). One builder fed the document and the side panel. | `lib/catalogue/document/guideDocument.ts`, `blocks.ts`, `documentStyles.ts`, `starmapDocument.ts` | **This is the part to keep and do better.** SSE3 already had "one content model, many looks"; it stopped short of words, order and jokes being data. The Atlas view is that block model on SSE4's scene, and an edition is the style plus everything SSE3 hard-coded. The twelve palettes come back as looks (the Looks and themes already built), not as a second list. |
| **The jokes**: 131 one-liners in a text file Alex curated (`Mostly_harmless.txt`), shown as random margin banners when a preset had `guideTips` on. Picked with `Math.random()`, not tied to any fact, so a black hole joke could land on a garden world and the joke changed on every redraw. | `lib/catalogue/guideNotes.ts`, `infoCard.ts` | Kept, as pack data with a rule for when each fits (section 6). Chosen by seed, so it is stable and prints as seen. |
| **The Guide preset**: cover "DON'T PANIC!", label "Megadodo Publications", rainbow schematic, friendly and oversized. Five other presets (Datapad, Console, CRT Terminal, Holo Table, Projection). | `lib/player/presets.ts` | The Guide is an edition. The cover words are a question (section 14, question 5). The other presets are looks and layouts, already covered by the Playbook and Switchboard design. |
| **Branding**: a faction name and logo kept on the GM's machine and broadcast; starter logos shipped, including a Weyland-Yutani one. | `lib/catalogue/branding.ts`, `presets.ts` | Brand is already in SSE4 on the role (`ui-contract/src/brand.ts`, V3 gap 25). The starter logos are **not** carried over (trademarks). The ASCII fallback replaces them. |
| **Hold-screen quotes**: 100 or so real quotations about space, shown between screens. | `lib/catalogue/spaceQuotes.ts`, `space-quotes.txt` | Not part of this note. Holding cards are pack data in the Playbook design (its section 6.3); the quotations can join that list. |
| **Player redaction**: data arrived already cut (`computePlayerSnapshot`); the page never saw a secret. No visible redaction at all. | `ReportDocument.svelte` header comment | Same guarantee, from the record's need-to-know rule. Visible redaction is new (section 5). |

**Was it in the gaps audit?** Yes, as item 24 of `SSE4-v3-gaps.md` ("Print a report, and save a player-safe copy"). Alex settled it on 23 Nov as "report screens built for the purpose (a GM briefing, a player survey sheet, a world dossier), shipped as screen documents and printable; the lens is the filter". It is marked "Planned, not started", stage 5. `docs/v3-parity.md` says the same in one line ("Reports as screens"). What the audit did not have: the Atlas as a thing you step through, the Guide's jokes, the company edition, the logo and ASCII art, and visible redactions. This note fills item 24 in and widens it. One shipped screen document already points the way: `players.survey.screen.json` ("Tarn survey report", layout `page`, theme `paper`).

## 3. Words

| Word | Means |
|---|---|
| **Atlas** | The view: a book of a place, read a page at a time. Never "report generator". |
| **Page** | One subject's entry: a world, a star, a station, a sector's index, a voyage. A page is what "Print this page" prints. A long page may take several sheets of paper. |
| **Chapter** | A page and the pages under it: a system with its worlds, a world with its moons. |
| **Edition** | A data document that says how the Atlas reads and looks: Survey atlas, The Guide, Company, or one a pack or the GM adds. |
| **Template** | The part of an edition that lists a page's sections and lines for one kind of subject. |
| **Margin note** | A line in the margin that is opinion, not fact: a joke, a warning in voice, a GM's aside. |
| **Letterhead** | The top of a page in an edition that has one: logo or ASCII art, name, document number, date, clearance. |
| **Gap** | A line the template has and this audience has not learnt. |

"Report" stays as an ordinary word for a printed Atlas chapter ("print the Kestrel report"), and the Company edition calls its pages reports. There is no second thing called a report.

## 4. The Atlas view

### 4.1 Where it is

- **A rail entry, "Atlas"**, in the Understand group at the sector, system and world scales (a `rail` entry in the Atlas's screen document, never a tab). It opens on the place the place line names.
- **On any object's press-and-hold menu:** "Open in Atlas" (one entry in the shared action list, `ui-contract/actions.ts`).
- **On the players' page** it is a layout like any other ("Atlas"), so a role can have it as its screen or as one box beside the chart. The Playbook's rule holds: a layout holds bookmarks, never the GM's knowledge.
- **On the Switchboard** an Atlas page is a source. Sending "Tarn, Company edition" to Bridge is a line like any other, and a preset can hold it.

### 4.2 What is on the screen

As in the mockup:

- **The place line** says where you are, with "Atlas" after it.
- **Edition**, a switch of the campaign's editions. Changing it re-dresses the same page in place. The choice is kept per role (the GM gives a role its edition on the role card, beside its look; "players may change it" is the same switch that governs looks).
- **Seen by** (GM only): GM, or any role. It is the existing "See as" lens under its Atlas name. A player has no such switch; they are always themselves.
- **Contents** down the left: the chapter's pages with page numbers, built from the audience's own knowledge (a world they have not found has no line and no number). On a phone it folds into a "Contents" button.
- **Step:** previous and next page under the contents, left and right arrow keys, swipe on touch. Next walks the book in contents order.
- **The page** itself. Names of other things on a page are links to their pages. A figure can be pressed for its Why, as everywhere (section 8).
- **Print this page** and **Print…** (section 9).

Who steers a sent Atlas (the GM turns the page, or the players read freely) follows the Playbook design's section 5.3 ("Theirs" or "Read only") with no new control.

### 4.3 The pages of the book

The shipped templates, one per kind of subject. Each is a list of sections and lines in the edition's data; these are the defaults every edition starts from.

| Page | Sections | Notes |
|---|---|---|
| **Sector** | chart plate; the systems in a table (name, star, worlds known, a line each); known links and routes; who holds what ("according to" this audience) | the index of the whole atlas |
| **System** | overview (stars, a schematic plate, zones); the worlds in orbit order, one line each; traffic and stations; hazards and approach | SSE3's sections 01 and 03 |
| **Star** | what it is, light, activity, age and fate | |
| **World** | summary prose; the world; air and sea; surface and hazards; life; people and places; getting there | the mockup's page; SSE3's section 02 entry, grown |
| **Moons, belts, small bodies** | a table under their parent, each with a page of its own when it has enough to say | |
| **Station, ship, construct** | what it is, where, what it carries, who runs it | ship pages lean on the manifest thread's facts |
| **Voyage** | route, legs, dates, fuel, hazards | later (AT12); from the voyages design |
| **How we know** | the notes for every marked figure in the chapter | section 8 |

A page's prose is the plain text layer that already exists (`sentence()` in ui-contract, used by `PlayerText.svelte`), which says what is known and says plainly what is not. Where a thing has an authored description with its own "Known to" eye (V3 gap 23, Minerva's Describe), the page prints it after the plain summary.

Plates (pictures) are the views that already exist, drawn at a fixed size: the chart grammar's symbols and system chart for a schematic, the globe for a world where the device can draw it, the flat chart symbol where it cannot. An edition chooses the plate style (chart, True, wireframe, none). Nothing is painted for the Atlas.

### 4.4 Any date

The Atlas reads the campaign's now. Because every reveal is a dated `observe.grant` event, the same page can be asked for "as the Bridge crew knew it on day 150": move the ruler and the page follows. Nothing is stored for this. A printed page says the date it speaks for in its footer.

## 5. Gaps, blackouts and redactions

Alex wants "the same blackouts and hidden data" to make a cool company report. The secrets design has a hard rule, "nothing leaks by absence": a player's screen never shows a gap, a count or a blank line where a secret is. Both hold, because there are two different things:

| What it is | Example | In every edition |
|---|---|---|
| **GM only** (closed eye): the audience does not know it exists | a hidden base; a kept-back line on a world | **Nothing.** No bar, no blank, no count, no page number skipped. |
| **A cover story** ("Told instead") | the listening post reads "ice mining station" | Printed as an ordinary fact. No mark. |
| **A gap**: a line this page's template always carries, which the audience's rung does not yet reach | Population, on a world scanned from long range | Shown the edition's way (below). |
| **Heard of** (half-open eye): known to exist, details held | a station with a name and a place | Its page exists and is mostly gaps. |

**The rule that keeps this honest: a gap is drawn from the template and the audience's rung, never from the value.** "Population" is a line on every world page. A crew at Long range sees a gap there on every world, whether the world has ten million people or none. So a bar tells them only what the pack's public ladder already says ("population needs Surveyed"). Bars are a fixed length for the same reason. The wire test of the secrets design (SV1) is extended: the bytes sent to a seat for a page are the same whether or not a kept-back value exists behind a gap.

How an edition shows a gap is one field, `gaps`:

- `plain`: quiet words from the vocabulary ("not yet surveyed"). The Survey atlas.
- `voice`: the edition's own words ("nobody has looked yet"). The Guide.
- `bar`: a black bar. The Company edition, with one line under the section: "Lines marked with a bar are above this document's clearance or await survey."
- `omit`: the line is left out. For a travel brochure, or any edition that should read as complete.

**A deliberate tease (question 3).** Sometimes the GM wants the table to see that something is being kept from them. The "Known to" popup gains one choice on a kept-back fact: **"Show as withheld"**. That fact then becomes a gap for that audience (bar, words or voice, by edition) on the Atlas, and "withheld" on the card. It is off by default, it is the GM's explicit act, and it is the only way a bar can mean a real secret. Underneath it is one more precision in the data view beside exact, rounded, band and told: `withheld`. The value is still never sent.

**On the GM's own copy** (Seen by: GM) nothing is barred. What the table does not know is in pencil, as on the chart, and that prints: dotted underline, quieter ink. A GM briefing is therefore simply the Atlas seen by the GM, with GM notes printed in their section.

## 6. Jokes: the Guide's margin notes

### 6.1 What is kept

Alex's list, his voice, and the idea that the Guide is the friendly one that is "mostly accurate". What changes is that a joke now belongs to something.

### 6.2 A margin note is data

```json
{
  "id": "guide.sea.liquid-market",
  "about": "world",
  "when": { "fact": "sea.cover", "at_least": 0.7 },
  "text": "With {sea.cover} of the surface under water, the local property market is best described as \"liquid\".",
  "weight": 2
}
```

- `about`: the kind of page it may sit on (world, star, system, sector, station, ship, any).
- `when`: a condition on the subject's facts, in the same small condition language the class atlas and data views use (a fact at least, at most, between, equal to, present; a tag present; all of, any of). Absent means "fits anywhere".
- `text`: the words. A `{fact}` slot is filled by the figures door (`say()` in `ui-contract/figures.ts`) with the audience's own value at the audience's own precision. **A margin note may not contain a typed number about the subject.** If it needs one, it uses a slot. That is the whole honesty rule for figures, and a check enforces it when a pack loads.
- `weight`: how keen it is to be picked.

### 6.3 The rules

1. **In the margin, signed.** A margin note is drawn in the margin (under the section on a phone), in the edition's margin hand, headed by the edition's signature ("The Guide says"). It is never a row in a facts table and never part of the summary prose. The Guide's footer says so once: "Margin notes are the Guide's opinions and are not facts."
2. **It reads only what the audience knows.** A note's `when` is tested against the audience's scene, never the truth. A joke about the ammonia sea cannot appear until the crew knows there is one. A joke can never be a leak.
3. **Cover stories are facts to the audience,** so a joke may be about one. That is correct: the Guide has been fooled too.
4. **The same joke for the same page.** The pick is seeded from the subject's address and the edition's id (`pick(seed, fitting notes, weights)`), with no clock and no `Math.random`. Every player sees the same note, the print matches the screen, and it stays put between sessions. When knowledge grows, more notes fit and the pick may change; that is a reveal doing its job.
5. **At most two to a page** by default (`margin.max`), so the page stays a reference.
6. **Honesty marks are untouched.** Authored, impossible and fictional are stamps about physics, on the GM's view. A margin note is neither a fact nor physics, so it carries no stamp and changes none. On the GM's view a margin note shows its id on hover and "Not this one" re-picks for that page (stored as a small exclusion in the campaign, so it is still deterministic).

### 6.4 Who writes them

- **Alex's list** ships as the base Guide's notes. Of the 131, most fit anywhere or fit a kind of place, and the builder adds an `about` and a `when` to each as a first pass (ship notes to ship pages, "local sun" notes to star pages, gravity notes to a gravity band). About 15 new notes with slots are worth writing so the Guide visibly reacts to the real world (sea cover, day length, gravity, air, temperature), like the two in the mockup.
- **The GM** writes a "Guide's note" on any thing: an authored note with a "Known to" eye that an edition's margin shows first, ahead of any picked one. A GM's note is free text and may say anything; it is theirs.
- **Packs and Explorers** carry note sets (`sse.margin-notes/1`), so a community Guide or a setting's own house humour is a small free pack.
- **Minerva**, with the GM's own key, drafts a note for this world in the edition's voice from the audience's facts. The draft is an authored note the GM accepts or declines; her existing check (every number against her facts, nothing kept back) applies. **Nothing is generated live on a player's screen**, so the Atlas needs no key, works offline and stays deterministic.

### 6.5 The voice is more than the margin

An edition's `voice` re-words furniture, never facts:

- **Section titles and line labels:** "Air and sea" becomes "Breathing and swimming". This is a vocabulary overlay on top of `vocab: plain`.
- **Gap words:** "nobody has looked yet".
- **A verdict:** a stamp read off real facts through a small table in the edition, for example hazard facts to "Mostly survivable", "Bring a mask", "Do not". It is a label read off results, in the spirit of "processes, not classes", and its Why names the facts it came from.
- **A subtitle** per page from the same kind of table ("third rock, the damp one").

### 6.6 The borrowed lines (question 5)

**Superseded 25 Nov: Alex keeps every line and the Megadodo cover in the Guide itself (section 14A, note 4). The text below is kept as the record of what was asked.**

36 of the 131 lines use names or near-quotations from Douglas Adams' books (Zaphod Beeblebrox, Vogons, the Pan Galactic Gargle Blaster, the Sirius Cybernetics Corporation, "Mostly harmless", "almost, but not quite, entirely unlike tea", the whale and the petunias, 42), and the SSE3 Guide's cover reads "DON'T PANIC!" over "Megadodo Publications". SSE3 is Alex's own site; SSE4 is open source under Apache-2.0 with packs that others redistribute. Recommendation: the base Guide ships the 95 lines that are Alex's own jokes in that spirit, plus the new ones; the 36 and the cover go into a free, separate, clearly unofficial fan pack, the same footing as named-setting packs. The Guide loses nothing a table would miss, and the base pack stays clean. This is Alex's call.

## 7. Letterhead, logo and the ASCII art

### 7.1 Whose letterhead

The Company edition has no name of its own. It takes the **brand of the role** reading it (`SeatBrand`: a name up to 40 characters and an optional small PNG, JPEG or WebP logo, already built and already on the role card under More). So the Bridge crew's copy is on Halden-Voss paper and the rival crew's is on theirs, from one edition. With no brand set, it uses the campaign's name.

Brand gains two optional fields: `accent` (one colour for rules and stamps) and `motto` (one line under the name).

### 7.2 The furniture

Document number (made from the brand's initials, the system and the page: `HV-KES-03/TARN`), issue date in the campaign's calendar, clearance (the role's name, or "GM EYES ONLY" on the GM's copy), page n of m, a distribution line, and one stamp ("COMPANY CONFIDENTIAL") in the brand's accent. Each is a slot in the edition's letterhead template (section 12.4), so a navy, a church or a pirate brotherhood words them differently.

### 7.3 The ASCII art

Until a logo is added, the letterhead draws one from the name. Alex asked for "crap ASCII art", so it is designed to be endearingly bad, and exactly the same every time for the same name:

```
+----------------------------------+
|  H   H  V   V  DDD    SSS        |
|  H   H  V   V  D  D  S           |
|  HHHHH  V   V  D  D   SS         |
|  H   H   V V   D  D     S        |
|  H   H    V    DDD   SSS    =>   |
+---------------------------------+
 HALDEN-VOSS DEEP SURVEY *est 1098*
   (logo pending. do not print.)
```

- **Letters made of themselves**, from one small 5 by 5 block font kept as data in the repo (our own; no FIGlet font files, whose licences vary).
- **Initials** of the name's words, up to four; a one-word name is spelt out up to eight letters.
- **Three flaws, chosen by a seed from the name:** the box's bottom edge is one dash short or long; one flourish from a small list (`=>` a rocket, `(o)` a planet, `*` a star, `~~` a comet) sits slightly off; and one apology line from a list ("(logo pending. do not print.)", "(placeholder. ask marketing.)", "(drawn by the intern.)").
- **"est" and a year** from the campaign's calendar at the campaign's start.
- It is plain text, so it prints anywhere, pastes into a chat, and reads aloud as its `aria-label` ("Halden-Voss Deep Survey, placeholder logo drawn in text").

The same fallback serves anywhere a brand has no logo (the join screen, a holding card), so it is one mechanism (question 9).

## 8. Every number keeps its Why

- **On screen:** pressing a figure on an Atlas page opens its Why, the same popup as on a card, at the reader's depth (Curious, Science, Engine). The Atlas adds nothing here; it must simply not lose it.
- **For a player, Why starts with how they know.** The first line of a player's Why is the source of their knowledge, from the grant that gave it: "Orbital survey by this crew, day 212" or "Bought chart, day 190" (a Hand over). The physics chain follows only as far as their knowledge reaches.
- **On paper:** an edition may mark figures with small numbers and print a **"How we know"** page at the end of the chapter: for each mark, the source line and one line of reasoning with its citation where the reader's depth shows it. The Survey atlas has this on; the Guide has it off; the Company edition prints a "Source:" line under each section instead (as in the mockup). It is one field, `sources: endnotes | section | off`.
- **Rounding** is the audience's precision from the data view, through the figures door. An edition cannot change it.

## 9. Print and export

Print is an output of the view, in the edition showing.

- **Print this page** prints the page on screen. **Print…** asks only: this page, this chapter (the page and those under it), or the whole atlas; paper A4 or Letter (guessed from the device, changeable); and "Save ink" (on by default). Then the browser's own print dialog, which is also how a PDF is made ("Save as PDF"). No PDF library is shipped (question 7).
- **In place, never a popup.** The page prints from a print-only rendering in the same window. SSE3's report window was blocked by browsers without a word; if printing cannot start, the Atlas says so in a sentence.
- **Each edition has a paper form.** An edition's look names its print tokens. "Save ink" swaps a dark edition (the Guide's night-blue) for its paper form: white ground, dark ink, the accent kept for rules and stamps. With "Save ink" off it prints as shown.
- **Sheet furniture** from the edition: running head, footer with the edition's name, "as known to" the audience, the campaign date, and sheet numbers. A section does not split from its heading; a facts table does not split a row; plates stay whole.
- **Plates print as vector** where they are chart drawings (SVG), and as a still from the renderer where they are globes.
- **What prints is what that audience holds.** A player printing from their own page gets their own knowledge, by construction (question 10). The GM printing "Seen by: Bridge crew" gets the same sheet the crew would.
- **Other outputs, same page model:** "Copy as text" (plain text or Markdown, for a chat, a forum or a virtual tabletop; the letterhead's ASCII art comes along), and "Save page" as one self-contained HTML file. "Save as seen by…" for a whole spoiler-free campaign file stays as settled in gap 24 and is not this track.
- **The same in, the same out.** For one campaign, date, audience, edition and page, the page's block list is identical on every device. That list is what the tests compare (a UI fixture, not a kernel golden).

## 10. How it is built: one page model

```
audience's scene (sse-observe)  ─┐
template (from the edition)      ├─>  page builder  ─>  blocks  ─>  drawn on screen
margin notes, voice, letterhead ─┘                             └─>  drawn for paper / as text
```

- **The page builder** is a pure function in `ui-contract`: scene, subject, edition, date in; a list of blocks out. Blocks are a small fixed set: `letterhead`, `title`, `prose`, `facts` (rows of label and figure, each row with a state: known, gap, withheld, pencil), `table`, `plate`, `margin`, `stamp`, `sources`, `rule`, `break`. This is SSE3's `DocBlock` idea, kept.
- **The Atlas view** draws blocks with the shared tokens and the edition's look. It formats no figure itself (the figures ledger rule).
- **No kernel work for the first packages.** The scene already carries each fact with its precision. Two asks arrive later, both already implied by the secrets design: the rung landmark and field states for an audience (`known_to`, SV1) to tell a gap from an absent line with certainty, and the source of a grant for "how we know" (section 8). Until SV1, a gap is "the template has the line and the scene has no fact for it", which is correct for generated facts.
- **No new place for state.** The role's edition is a field on the role beside its look. The campaign's editions are documents in its pack stack. "Not this one" exclusions and GM's notes are campaign data.

## 11. The shipped editions

| | Survey atlas | The Guide | Company |
|---|---|---|---|
| For | the traditional star guide; the default | the friendly one, with jokes | an organisation's own paper |
| Look | chart room paper and ink, serif names, mono figures | night blue, gold and teal, rainbow chart plate | mono type, grey section bars, the brand's accent |
| Voice | plain | the Guide's (titles, gaps, verdict, subtitle) | formal: numbered sections, capitals |
| Margin notes | none (GM's notes only) | on, two to a page | none |
| Gaps | plain words | in voice | black bars |
| Letterhead | a quiet running head | a masthead | the role's brand, logo or ASCII art, document number, clearance, stamp |
| Sources | endnotes, "How we know" | off | a Source line per section |
| Plates | chart style | chart style, colourful | wireframe figures, numbered |

A fourth, later: a **Brochure** (warm, `gaps: omit`, a tourist board's voice), which SSE3 had as a document style. The Traveller pack may bring an edition of its own in its service's style, without any logo (question 11).

## 11A. Four starter interfaces, and the parts they are made of

Alex, 24 Nov 20:04: the text Atlas "with different flavours is magical", and all its editions stay; but players also need "the more app type tools" that combine the UI elements already built, "have the controls on each turn on and offable and have them all as reusable components". Four solid starters ship. Nobody gets SSE3's wall of sliders. Orac makes the rest.

### 11A.1 What is already there

Almost all of it. This section mostly names and joins:

| Exists on main | Is |
|---|---|
| Player views: chart, card, readout (gauge, light), text, table (`packages/views/player-stack`), the system in 3D, the landing view | the parts |
| Screen documents `sse.screen/1` ("Bridge", "Bridge in 3D", "Landing", "Tarn survey report") | layouts of parts |
| Themes `sse.theme/1`: Chart, Paper, **Phosphor** (green screen), **Deck panels**, **Clean glass**, **Field guide** | the four looks, already drawn as our own designs with generic names |
| View controls (View style, Seen from, Scale, HUD, Band) declared by each view (`ViewModule.controls`) | the pattern for on-and-off controls |
| Style documents, filter chains, signal faults (the Looks work, Playbook design 7.1 and 7.4) | "all the controls and options and shaders" |
| Orac's targets for screens, styles, symbologies and data views (`orac-looks.ts`) | Orac's tools, minus two |
| Brand on the role (`SeatBrand`) | the company name and logo |

What is missing is one document that bundles these into "an interface", a uniform way to switch a part's controls, and the four bundles themselves.

### 11A.2 The interface kit

An **interface kit** (`sse.kit/1`) is everything a role's screens need, chosen once:

- **Layouts:** one per situation (sector, system, world, ship, landing, holding), each an ordinary screen document made of parts.
- **Look:** a theme with its filters and signal fault.
- **View style** its diagrams start in: Chart 2D, Chart 3D, True or Wireframe.
- **Atlas edition**, and optionally the look that edition wears inside this kit.
- **Palettes:** a short list of named colour sets the look may wear (11A.5).
- **Words:** vocabulary (plain, physics, story), join words, holding cards.
- **Needs:** the lowest device tier it runs on and what it falls back to.

On the role card it is one line, above Look: **"Interface: Hauler ▾"**. Choosing a kit sets the role's layout, look and edition together. Each can still be changed singly afterwards, as the Playbook design allows; the kit is the starting set, not a lock.

### 11A.3 The four starters

As in `starters-four.png`. All four show the same facts to the same crew.

| | Hauler | Corporate | Deck | The Guide |
|---|---|---|---|---|
| Alex's words | "space trucker/basic, the greenscreen and wireframe" | "slick corporate, all interactive 3D" | "slick diagram orientated 2D LCARS type" | "its own special thing" |
| Look (exists) | Phosphor | Clean glass | Deck panels | Field guide |
| Diagrams | Wireframe | Chart 3D, True where the device can | Chart 2D, schematic | Chart 2D, colourful |
| Made of | one wireframe body or chart, a text report beside it, a command line of keys along the bottom | a full-screen 3D stage with floating cards and a few pills | a diagram in the middle, a column of big buttons, gauges and readouts | the Atlas page as the main panel, with the chart as its picture |
| Controls on | almost none: next, previous, print, three keys | View style, Seen from, turn and zoom, cards open from things | the button column, band, HUD | contents, step, print |
| Atlas edition | Company, wearing phosphor | Survey atlas, wearing glass | Survey atlas, wearing deck panels | The Guide |
| Palettes | green, amber, white | blue, graphite, the brand's accent | amber and violet, teal and blue, red alert | cream, night |
| Runs on | anything, down to the text tier | a capable device; falls back to Chart 3D and says so | anything with a canvas | anything |

- **Hauler** is the basic one and the safe default for an old laptop or a phone: mostly text, one line drawing, almost no controls.
- **Corporate** is the showpiece. It leans on the renderer's True views and the HUD switch, so it is as good as they are on the day.
- **Deck** is the diagram lover's console. "LCARS type" names the genre Alex means; the shipped look is **our own design in that spirit under a generic name**, as `skins/player/themes.ts` already rules ("none copies a franchise's artwork, fonts, colours or logos"). The name LCARS and its exact shapes are not used.
- **The Guide** is not one more skin over the same console. It is book first: the role's screen is the Atlas in the Guide edition and the chart is an illustration in it.

A fifth kit, plain **Chart** (the GM's chart room look, today's default Bridge), stays as the neutral default for a campaign that has chosen nothing.

### 11A.4 Parts, and controls that switch on and off

A **part** is a view module used in a layout. The rule that makes parts reusable:

1. **Every part declares its controls** as a list with an id, plain words and a default (`controls` on the view module; the five view controls are already declared this way). For the Atlas part: `contents`, `step`, `edition`, `print`, `links`. For a chart: `view_style`, `seen_from`, `scale`, `hud`, `band`, `turn`, `select`, `menu`. For a card: `open`, `why`, `depth`. For a table: `sort`, `filter`.
2. **A layout switches them per use:** `"controls": { "edition": false, "print": true }`, or `"controls": "none"` for a pure display, or `"all"`. Unstated controls take the part's default.
3. **A part never draws a control it has not declared, and never draws one that is off.** One shared component draws each kind of control (as `ViewControls` does now), so they look and behave the same in every kit, wearing the kit's look.
4. **Off means off for the player.** The GM steering that screen from the Switchboard still can (who steers, Playbook 5.3).
5. **Controls follow the style** (Alex, 20:06: "buttons and interface elements need to follow style"). A control is declared as a kind (button, switch, choice, step, dial, tab, link), never as a drawing. Each look carries a **control set**: how each kind is drawn, lit, pressed and focused in that look. So Print is `[P]RINT` on a command line in Hauler, a rounded coloured cap in Deck, a glass pill in Corporate and an inked button in the Guide, from one declaration. A look without a control set of its own takes its parent's, so a new palette or a small variant costs nothing. The chart room's own buttons never appear on a styled player screen. Touch size, focus ring and reduced motion are fixed minimums no look can go under. The mockup's edition switch and Print buttons are drawn in the GM's chart room style because that bar is the GM's; on a player's screen they wear the kit's look, as in `starters-four.png`.
6. **A part with every control off is a picture.** That is what a wall screen or a stream output wants, and it is the same part.

The parts list for 4.0, all existing or already planned by their owners: chart (sector, system), system in 3D, body globe, wireframe body, card, readout, text, table, **Atlas page** (this track), clock and date, place line and picker, route and voyage, sky from here, hazard map, "Newly learnt", read-aloud words, holding card. A new view any thread builds becomes a part by declaring its controls; nothing else is needed.

### 11A.4a How a screen arrives and leaves

Alex, 24 Nov 20:08: "Dont forget to reuse some of the stuff we had around transitions... greenscreens building up rather than just flicking - the slow warm up and the disappear to a dot." SSE3 had five (`wipe`, `fade`, `crt_collapse`, `scanline`, `static_dissolve`, with durations of 0.7 to 2 seconds, in `lib/player/presets.ts`). The Playbook design already names the SSE4 mechanism, **screen delivery**, as part of a look (Fade, Scene wipes, Power on, Static burst, Typewriter, Scan in; its section 5.2 and package PB10). The toolkit uses that one mechanism and fills it in:

**Five moments, each with its own delivery in the look:**

| Moment | When | Hauler's choice |
|---|---|---|
| **Power on** | a screen joins, or a hold is released | Warm up: black, a faint glow that swells and steadies over about two seconds, then Build up |
| **Arrive** | a new picture is sent | Build up |
| **Turn** | the player steps to the next Atlas page or selects another world | Type on, short |
| **Leave** | the picture is replaced | none; the next one builds over it |
| **Power off** | a hold, Screens off, or the session ends | Collapse to a dot: the picture squeezes to a bright line, the line to a dot, the dot fades |

**The deliveries in the toolkit** (each a named, parameterised piece the renderer's looks own): Fade; Wipe (a direction); Build up (the frame, then lines, then labels, then figures, top to bottom, as a terminal draws); Type on (text arrives a character at a time at a set rate); Scan in (a bright line sweeps down, leaving the picture); Warm up; Collapse to a dot; Static burst; Dissolve in blocks; Slide; Page turn (the Guide); Iris. Each takes a duration and a few settings, as fields.

**Three scales, so a screen builds up and does not just flick:** a delivery may act on the whole screen, on each part in turn (the kit gives the order: chart, then report, then status line), or inside a part (a table fills row by row, a wireframe draws ring by ring, text types on). Parts declare which they can do; a part that cannot simply fades.

**The rules that keep it pleasant:**

1. **It never blocks.** Everything under a delivery is live from the first frame; a tap or key finishes it at once. The interface style's limit of 120 to 240 ms is for feedback to a touch and still holds for every control. A delivery is a change of scene and may run longer.
2. **Long the first time, short after.** Power on and the first Arrive of a send play in full. Turn is always the short form (under about 400 ms), so stepping through an Atlas never becomes a wait.
3. **Reduced motion turns every delivery into a plain fade,** whatever the kit says, and nothing flashes more than three times a second (both already ruled for Looks).
4. **It is the look's, so it follows the kit.** Corporate eases cards up into place; Deck lights its panels in sequence; the Guide turns a page. A kit names them under `delivery`.
5. **Print ignores all of it.**

The other starters' choices: Corporate uses Fade with Slide for cards and Iris for power; Deck uses panel-by-panel Build up and Scan in; the Guide uses Page turn and Fade.

### 11A.5 What a person can change without Orac

Alex, 20:05: "Allow the user to have some limited editing - company name - colour pallets (even if just green to amber screen)". Exactly three things, all on the role card, all one press or one field:

1. **Company name** (the brand name, already there). It appears on the letterhead, the top bar and the join screen.
2. **Logo** (already there), with the ASCII art until one is added.
3. **Palette**, a small row of swatches from the kit's own list: Hauler's green, amber, white. A palette is a named set of colour tokens checked for contrast when the kit loads, so no choice can make a screen unreadable. There is no colour picker and no slider.

Everything else (which parts, where, which controls, filters, fonts, signal fault, words) is the kit's data, changed by asking Orac or by hand (12.7). The player's own device settings (text size, reduced motion, colour-blind patterns) always win, as ruled.

### 11A.6 "Scrub the sliders": what that means for screens already built

- SSE3's `PlayerPresetEditor` (about 110 KB of sliders, pickers and toggles) is **not** carried over in any form.
- SSE4 already has two GM screens in that family, built before this ruling: the **Screens composer** (`views/screen-composer`) and **Styles** (`views/styles`) in Workshop, and the Playbook design plans a four-row **look card**. Recommendation (question 15): build no more of that kind. Keep the look card to what is used at the table (pick a look, pick a palette, the interference switch and its live dial); keep the composer and Styles as they are, at Engine depth, as a plain view of the documents for people who want to see and nudge the data; put no new work into them. The front door for making or changing an interface is Orac, and the layout editor of Playbook 7.4 (drag boxes, three kinds of input) stays because it is arranging, not slider tuning.

### 11A.6a Why this is not as easy as it sounds

Alex, 20:06: "we hide the old complexity and reoffer it back using a properly architected and structured system and an LLM to put it together for their use case - seems too easy". The idea is sound and most of the structure exists. Four things make or break it, and each has a package:

1. **The structure must be complete before Orac is any use.** Orac can only switch what is declared. A part with an undeclared control, or a look with a half-drawn control set, is a hole Orac cannot fill. AT13 and the control sets in AT15 are the real work.
2. **Every check must run without Orac.** A kit made by hand, by a pack or by an LLM goes through the same loader. Nothing trusts the model.
3. **People without a key still need enough.** Orac is bring-your-own-key. That is why there are four good starters, three plain edits, and a hand guide. Nobody is stuck with nothing because they have no key.
4. **Four starters done well is a lot of drawing.** Each needs its control set and five layouts, on desktop and phone, in both light conditions where it has them. It is the largest package here.

### 11A.7 What Orac is given

Orac already drafts screens and styles as change requests. For kits it needs:

- **A catalogue entry per part:** what it shows, its params, its controls with defaults, the tiers it runs on, its cost. (`paramLines` and `lookEntries` in `orac-looks.ts` are the pattern; controls are the new column.)
- **Catalogue entries for** looks, palettes, deliveries (11A.4a), filters and shaders with their settings, signal faults, view styles, Atlas editions, margin note sets, and the four starters as worked examples.
- **Three new targets** beside `screenTarget` and `styleTarget`: a kit, an edition, a margin note set. Each uses the same checker the loader uses and gives new documents a `mine.` id, so a shipped starter is never overwritten.
- **The checks:** every part, control, look, palette and edition named exists; the kit's cost fits its stated tier, with a fallback named when it does not; contrast for every palette; the edition checks of 12.6. A kit, like an edition, cannot state a fact or widen what a role knows.
- **A preview:** the GM's current place drawn in the new kit beside the role's current one, as the role would see it. Accept gives it to the role; decline leaves nothing behind.
- **The handbook entry** (`guide:interfaces` in `orac-handbook.json`): sections 11A and 12 of this note, shortened.

Asks to support from day one: "Hauler but amber, with the hazard light always on", "a salvage crew's console: wireframe, no 3D, a cargo table", "Corporate for a church: gold on black, formal words", "take the print button off the wall screen".

## 12. Building your own: the authoring guide

Written for a GM, a pack author, or Orac. No editor is needed: an edition is one JSON document in a pack or a campaign, and so is a whole interface (12.7). Sections 12.1 to 12.6 and 12.8 are about Atlas editions; 12.7 is about interface kits.

### 12.1 The promise

**An edition changes how the Atlas reads and looks. It cannot change what is true, what is known, or how precisely.** Everything below is checked when the edition loads, and an edition that fails a check is refused with a sentence saying why.

An edition **may** choose: which sections and lines a page has and in what order; what they are called; the look; the plate style; whether there are margin notes and which sets; how a gap is shown; the letterhead and its words; stamps; where sources are printed.

An edition **may not**: state a fact (no typed values about a subject); name a field outside the catalogue of facts; change a figure's precision or unit system (those are the audience's and the reader's); show anything the audience's scene does not hold; run script; load anything from the web; carry a font or picture bigger than the pack limits.

### 12.2 The smallest edition

Start from a shipped one and change what you want. Everything not stated comes from `extends`.

```json
{
  "schema": "sse.edition/1",
  "id": "my.edition.navy",
  "name": "Fleet Survey Office",
  "description": "Dry naval survey sheets with a crest and bars.",
  "extends": "sse.edition.company",
  "look": "sse.look.blueprint",
  "voice": {
    "labels": { "section.people": "Ports and garrisons", "section.travel": "Passage" },
    "gap": "NOT HELD AT THIS STATION"
  },
  "gaps": "bar",
  "letterhead": {
    "lines": ["{brand.name}", "SURVEY SHEET {doc.number}", "CORRECTED TO {date}", "ISSUED TO: {audience}"],
    "stamp": "RESTRICTED"
  }
}
```

### 12.3 Every field

| Field | What it does | Default |
|---|---|---|
| `schema`, `id`, `name`, `description` | identity; `description` is one line people and Orac choose by | required |
| `extends` | the edition this one starts from | `sse.edition.survey` |
| `look` | a look id (palette, type, filters); its print tokens are used on paper | the parent's |
| `plates` | `chart`, `true`, `wireframe`, `none`; and `colourful: true` for the rainbow chart | `chart` |
| `voice.labels` | new words for section titles and line labels, by key | plain vocabulary |
| `voice.gap` | the words for a gap when `gaps` is `voice` | "not yet surveyed" |
| `voice.subtitle`, `voice.verdict` | tables: a condition on facts gives a phrase (first match wins) | none |
| `gaps` | `plain`, `voice`, `bar`, `omit` | `plain` |
| `margin` | `{ "sets": [ids of margin note sets], "max": 2, "signature": "The Guide says" }` | no margin notes |
| `letterhead` | `{ "use": "brand" or "masthead" or "none", "lines": [...], "stamp": "..." }` with slots (12.4) | `none` |
| `sources` | `endnotes`, `section`, `off` | `endnotes` |
| `templates` | per page kind, the list of sections; each section a key, and a list of lines (fact fields) or one block (`prose`, `plate`, `table`) | the shipped templates of section 4.3 |
| `print` | `{ "running_head": "...", "footer": "...", "save_ink_look": look id }` with slots | from the look |

A template section, in full:

```json
{ "key": "section.air", "lines": ["air.mix", "air.pressure", "sea.liquid", "sea.cover", "interior.summary", "resources.summary"] }
```

A line names a fact field from the catalogue (the data sheet and the vocabulary list every field with its plain label and its rung). A line whose fact does not apply to the subject at all (a sea on an airless rock) is left out; a line that applies and is not known is a gap.

### 12.4 Slots

Usable in `letterhead.lines`, `print.running_head`, `print.footer` and margin note text:

| Slot | Gives |
|---|---|
| `{brand.name}`, `{brand.motto}`, `{brand.logo}` | the role's brand; `{brand.logo}` is the picture or the ASCII art |
| `{audience}` | the role's name, or the GM |
| `{date}` | the campaign date the page speaks for, in the campaign's calendar |
| `{place}`, `{chapter}` | the page's subject and its chapter |
| `{doc.number}` | the document number (section 7.2) |
| `{page}`, `{pages}` | sheet numbers |
| `{fact field}` (margin notes, subtitles and verdicts only) | the audience's own figure for that field, through the figures door |

### 12.5 A margin note set

```json
{
  "schema": "sse.margin-notes/1",
  "id": "my.notes.navy",
  "name": "Wardroom remarks",
  "notes": [
    { "id": "navy.any.1", "about": "any", "text": "Charted by committee. Navigate accordingly." },
    { "id": "navy.lowg.1", "about": "world", "when": { "fact": "gravity.surface", "at_most": 0.2 },
      "text": "At {gravity.surface}, shore leave is best taken tethered." }
  ]
}
```

Rules: no digits about the subject outside a slot (a check looks for numbers next to units and refuses them); a slot's field must appear in the note's `when` or be a field every such page has; keep a note under 200 characters so it fits a margin; write for the table you would read it aloud to.

### 12.6 What the loader checks

1. The schema, and that `extends` exists and does not loop.
2. Every field key in a template, a `when`, a slot or a voice table is in the fact catalogue.
3. No typed figure about a subject in any text.
4. Looks, note sets and plate styles named exist in the pack stack.
5. Sizes: text lengths, a page's line count (a page of 200 lines is refused with advice to split it), pictures within pack limits.
6. A **test render**: the edition is drawn for the fixture world as the GM and as a seat with partial knowledge, and the two block lists are compared with the Survey atlas's. Every figure must match; nothing may appear for the seat that the Survey atlas does not show them.

### 12.7 A whole interface, by hand: the kit

An interface kit is one JSON document. The quickest way is to copy a starter and change it.

```json
{
  "schema": "sse.kit/1",
  "id": "mine.kit.salvage",
  "name": "Salvage console",
  "description": "A tired amber terminal for a salvage crew: wireframe, no 3D, cargo always in view.",
  "extends": "sse.kit.hauler",
  "look": "phosphor",
  "palettes": ["amber", "green"],
  "view_style": "wireframe",
  "vocab": "plain",
  "atlas": { "edition": "sse.edition.company", "look": "phosphor" },
  "delivery": {
    "power_on": { "use": "warm-up", "seconds": 2.0, "then": "build-up" },
    "arrive": { "use": "build-up", "by": "part", "order": ["a", "b", "c", "d"] },
    "turn": { "use": "type-on", "chars_per_second": 400 },
    "power_off": { "use": "collapse-to-dot", "seconds": 1.2 }
  },
  "needs": { "tier": "canvas", "falls_back_to": "sse.kit.hauler" },
  "layouts": {
    "system": {
      "arrangement": "picture-and-side",
      "items": [
        { "view": "sse.view.chart", "area": "a", "style": "wireframe",
          "controls": { "select": true, "menu": false, "view_style": false, "seen_from": false, "scale": false } },
        { "view": "sse.view.atlas", "area": "b", "params": { "page": "selected" },
          "controls": { "step": true, "print": true, "contents": false, "edition": false } },
        { "view": "sse.view.table", "area": "c", "params": { "of": "cargo" }, "controls": "none" },
        { "view": "sse.view.readout", "area": "d", "params": { "style": "light", "tag": "hazard", "label": "Hazard" } }
      ]
    },
    "world": { "extends": "system" },
    "holding": { "card": "Signal lost" }
  }
}
```

Step by step:

1. **Pick the starter nearest to what you want** and put its id in `extends`. Anything you leave out comes from it.
2. **Choose the look and palettes.** A look is a theme id; the palettes listed are the swatches the role card will offer, first one the default. To add a palette of your own, add it to the look's `palettes` with its colour tokens; it is refused if text contrast falls below the standard.
3. **Choose the view style** diagrams start in, and the vocabulary.
4. **Choose the Atlas edition,** and a look for it to wear inside this kit if it should not wear its own. **Choose how screens arrive and leave** (`delivery`, section 11A.4a): one named delivery for each of the five moments; leave one out and the look's own is used.
5. **Lay out each situation.** `arrangement` is one of the layout editor's five (One picture, Picture and side panels, Two by two, Report page, Blank), with areas lettered `a`, `b`, `c`. Each item is a part: its `view`, its `area`, its `params`, and its `controls`.
6. **Switch controls.** List only the ones you want different from the part's default. `"none"` makes the part a picture. The catalogue (Reference, and Orac) lists every part's controls.
7. **Say what it needs.** `needs.tier` is the weakest device tier the kit is meant for (`text`, `canvas`, `gpu-basic`, `gpu-high`); `falls_back_to` is the kit a weaker device gets, with a line saying so.
8. **Load it.** Put the file in a campaign's own pack (or drop it on the Packs screen). The loader runs the checks of 11A.7 and says in a sentence what is wrong if anything is. Then choose it on a role card.

What a kit may not do is what an edition may not do (12.1), plus: it may not turn on a control a part has not declared, and it may not name a part the role's audience is not allowed (a GM-only screen is never a part).

### 12.8 What Orac needs to draft an edition safely

- **Its reading list:** this section, the `sse.edition/1` and `sse.margin-notes/1` schemas, the fact catalogue with plain labels and rungs, the list of looks, and the shipped editions as examples. All are data it can already be given the way it is given screens and looks (`orac-looks.ts` is the pattern).
- **Its output is a change request:** a new or edited edition document, validated by the loader's six checks before the GM ever sees it, shown as a side-by-side preview on the page the GM is on ("Fleet Survey Office" beside "Company"). Accept or decline, as with every Orac change.
- **Its limits are the edition's limits.** Because an edition cannot state a fact, Orac cannot put one in by mistake. Margin notes it drafts go through the same no-typed-figures check; a note for one specific world is an authored GM's note that the GM accepts, with its "Known to" eye starting at GM only.
- **Useful asks to support from day one:** "make me a dry navy edition in blueprint", "the Guide but for a pirate crew", "put hazards first and drop the life section", "write five margin notes for cold ocean worlds".
- **It does not** edit the facts, the data view, or who knows what from an edition request. If the GM asks for "a report that shows the hidden base", Orac says that is a "Known to" change and offers that change request instead.

## 13. Packages

Proposed owner for most: **a new Opus builder thread, "Build star atlas and editions"**, owning `packages/views/atlas/`, `packages/ui-contract/src/atlas/` (page model, edition and margin note documents) and `packages/shell/src/atlas/`, with a status file `handover/status/atlas.md` (question 12). Seams with other threads are named per package and agreed by message.

| Id | What | Owner | Check |
|---|---|---|---|
| AT1 | The page model: blocks, the page builder as a pure function over the audience's scene, the shipped templates for sector, system, star and world, in the Survey atlas edition only. `sse.edition/1` with `extends`. | atlas | the same scene, edition and date give the same block list; a seat's list holds nothing its scene lacks; no figure formatted outside the figures door |
| AT2 | The Atlas view on screen: page, contents, step, links between pages, phone layout, both chart modes; its screen document with a `rail` entry; "Open in Atlas" in the action list. | atlas; navigation thread for the action entry | desktop and phone screenshots; keyboard alone; reduced motion |
| AT3 | Editions as data: look, voice labels, gap styles, plates, sources; the loader's checks 1 to 5; the Edition switch; the role's edition on the role card. | atlas; Run desk thread for the role card field | switching edition changes no figure (compared block by block) |
| AT4 | Print under the view: print this page, chapter, whole atlas; A4 and Letter; Save ink; sheet furniture; in place with a spoken failure. Copy as text, Save page. | atlas | a printed PDF per shipped edition in the screens folder; a blocked print says so |
| AT5 | Gaps and redaction: gap states from template and rung; bars at fixed length; "Show as withheld" (`withheld` precision). | atlas for drawing; kernel thread for `withheld` beside SV5's `told`; chart room for the popup choice | the bytes sent to a seat are identical with and without a kept-back value behind a gap |
| AT6 | Letterhead: brand `accent` and `motto`; the ASCII art generator and its 5 by 5 font; document numbers, clearance, stamp. The Company edition. | atlas; Run desk thread for the two brand fields | the same name always draws the same art; a logo replaces it |
| AT7 | Margin notes: `sse.margin-notes/1`, conditions on the audience's scene, slots, the seeded pick, "Not this one", GM's Guide's notes. The Guide edition with Alex's list given `about` and `when`, plus the new slotted notes. Verdict and subtitle tables. | atlas; Alex reads the note set before it ships | a note never fires on a fact the seat lacks; the same page gives the same note on two devices |
| AT8 | Why on the Atlas: figures open their Why; the player's source line; "How we know" endnotes and Source lines. | atlas; kernel thread for the grant's source (with SV1) | every marked figure has a note; a seat's note names only its own knowledge |
| AT9 | The Atlas on the players' side and the Switchboard: an Atlas layout, an Atlas page as a source and in a preset, who steers. | Run desk thread | a sent page arrives in the role's edition |
| AT10 | The GM's copy: pencil and ink on the page and in print, GM notes section. | atlas, using the chart room's pencil styles (SV2) | what the table does not know prints dotted |
| AT11 | Orac and Minerva: Orac drafts and edits editions and note sets as change requests with the test render (check 6) and side-by-side preview; Minerva drafts a Guide's note as an authored note. The authoring guide (section 12) published in Reference. | Build Orac, Minerva and Tars | Orac's edition passes the loader before it is shown; Minerva states no kept-back fact |
| AT13 | Controls that switch: every player part declares its controls; a layout item's `controls` turns each on or off; one shared drawing of each kind of control; the parts catalogue (with controls) in Reference and for Orac. | Run desk thread (owns the player parts), with each view's owner declaring its own | a part with `"none"` draws no control; an off control cannot be reached by keyboard either |
| AT14 | `sse.kit/1`, its loader and checks; "Interface" on the role card; palettes as named token sets with a contrast check and swatches on the role card. | atlas for the document and checks; Run desk thread for the role card | choosing a kit sets layout, look and edition in one press; every shipped palette passes contrast |
| AT15 | The four starters as kits: Hauler, Corporate, Deck, The Guide, each with its control set (every kind of control drawn in that look) and its layouts for sector, system, world, landing and holding, on the existing Phosphor, Clean glass, Deck panels and Field guide themes. | atlas, with the renderer thread for Corporate's True stage and the wireframe body | desktop and phone screenshots of all four at the same place; each falls back as it says on a weak device |
| AT15a | Deliveries for the starters: the five moments in a look, the three scales (screen, part, inside a part), Warm up, Build up, Type on, Collapse to a dot and Page turn added to the Playbook's list; each part declares which it can do. | the renderer's looks work under PB10 (Run desk and renderer threads), with atlas for the Atlas part's Type on | Hauler warms up, builds up and collapses to a dot; a key press finishes any delivery; reduced motion gives a fade |
| AT16 | Orac makes interfaces: kit, edition and note set targets, catalogue entries for parts, controls, palettes, filters and shaders, side-by-side preview, the `guide:interfaces` handbook entry. | Build Orac, Minerva and Tars | "Hauler but amber" gives a kit that passes every check before the GM sees it |
| AT12 | Later: voyage and ship pages (travel and manifest threads); a Brochure edition; a "what is new since day N" bulletin; editions and note sets shared on Explorers; a Traveller pack edition. | as named | |

**Order for the starters:** AT13, then AT14, then AT15 with AT6 and AT7 beside it (Hauler needs the Company edition, The Guide needs the Guide edition), then AT16. Hauler first: it needs the least and proves the kit.

**Order for the Atlas:** AT1, AT2, AT3, AT4 give a plain, printable Atlas and need nothing that is not on main today, so they can start at once. AT6 and AT7 are the two Alex asked for by name and follow directly. AT5 and AT8 wait on the kernel's SV1 and SV5 for their full form but have an honest first cut without them. AT9 follows the Run desk's layouts package (PB13). In the coming-together order this is stage 5 (run a game), as gap 24 already was; AT1 to AT4 fit stage 2 if a builder is free.

## 14. Questions for Alex

Each with a recommendation, so "yes to 1, 2 and 4" works.

1. **The Atlas is one view, and Survey atlas, The Guide and Company are editions of it; "edition" is the word.** Recommend yes.
2. **Gaps are drawn from the template and the audience's rung, never from the secret; what only the GM knows leaves nothing at all, in every edition.** Recommend yes. It is what lets the Company report have bars without breaking "nothing leaks by absence".
3. **"Show as withheld": a GM may mark one kept-back fact so the table sees a bar (or "withheld") there, off by default.** Recommend yes. It is the deliberate tease, and the only bar that means a real secret.
4. **Jokes are margin notes with a rule for when they fit, reading only what the audience knows, with real figures through slots, and the same joke always on the same page.** Recommend yes.
5. **The 36 lines with Douglas Adams' names and phrases, and the "DON'T PANIC!" and "Megadodo Publications" cover, go in a free unofficial fan pack; the base Guide ships your other 95 plus new ones.** Recommend yes. I am not a lawyer; the point is to keep the Apache-2.0 base pack clean. The SSE3 starter logos (Weyland-Yutani and the rest) are likewise not carried over.
6. **The base edition keeps the name "The Guide",** with the strap line "friendly, illustrated, mostly accurate". Recommend yes; say if you want another name.
7. **Print and PDF go through the browser's own print for 4.0, in place, with no PDF library.** Recommend yes.
8. **Minerva drafts Guide's notes only as notes the GM accepts; nothing is generated live on a player's screen.** Recommend yes.
9. **The ASCII art is the fallback wherever a brand has no logo (letterhead, join screen, holding cards).** Recommend yes.
10. **Players may print and copy their own Atlas from their own page.** Recommend yes; it can only hold what they know.
11. **A Traveller pack edition in 4.0** (a survey service's sheet, no logo, by the Traveller pack thread after AT6). Recommend yes, small, and last.
12. **A new builder thread "Build star atlas and editions" on Opus takes AT1 to AT8 and AT10, starting with AT1 to AT4 now.** Recommend yes. On the new account this is the first thread to open for this track.

13. **Four starter interfaces as "interface kits": Hauler, Corporate, Deck and The Guide, with plain Chart kept as the neutral default.** Recommend yes. Say if you want other names; "Hauler" is my word for your space trucker.
14. **The only edits without Orac are company name, logo and a palette picked from the kit's swatches (Hauler: green, amber, white). No colour picker, no sliders.** Recommend yes.
15. **Scrub the sliders: SSE3's preset editor is not rebuilt; the Workshop's Screens composer and Styles screens stay as they are at Engine depth with no new work; the Playbook's look card is cut down to look, palette and the interference switch and dial.** Recommend yes. This trims a card you accepted earlier today, so it needs your own word.
16. **Deck is our own design in the spirit of that genre, under a generic name, never the LCARS name or its exact shapes.** Recommend yes; it is the rule the shipped themes already follow.

## 14A. Alex's answers and notes (24 Nov, 20:14 to 20:16 UTC)

"Yes to all" to questions 1 to 16. His notes, in his words, and what each changes:

1. **"and these are autobookmarked of course."** Every Atlas page is a bookmark without anyone making one. A page already has everything a bookmark holds (the place, the date, the view, the edition), so each page a GM or a role opens is offered in the Bookmarks list under "Atlas pages" and on the Switchboard as a source, newest first, ready to send or to keep in a preset. Kits and editions the campaign uses are listed the same way. Nothing has to be saved by hand. (Package AT9.)
2. **"I got permission from the artist for those logos... if they don't give a logo we do a crap ascii one and if they do supply one we use it... and we have the above as 'built in'."** This changes the last sentence of question 5: SSE3's starter logos **are** carried over as a built-in set a GM can pick for a brand (`static/images/logo/` in the SSE3 source: SSE, Interspan, Kelido, Nexum, Terra, TSEC). The order of use is: the logo the GM supplies, else a built-in one they picked, else the ASCII art. The artist is credited in the pack's attributions. The set also holds a Weyland-Yutani logo, a film studio's name and mark. **Settled by Alex, 25 Nov 01:41 UTC: "no need for a WY logo", it "can wait for the game pack".** It is not in the built-in set; if a game pack for that setting is ever made, the logo is that pack's business.
3. **"The original desk-co.png desk-survey.png... all valid too as more serious variations of 'The Guide'."** The Survey atlas and Company editions stay exactly as drawn. They and the Guide are one family: the same Atlas, from serious to silly.
4. **"'The Guide' ... will always be a special place - make it look like you think it would look like including a cover to click past each time... DON'T PANIC :)"** and **"the guide is NOT paper - it is cheesy electronics built to a low cost."** So:
   - The Guide is a **gadget**: a chunky cream plastic case, a slightly garish screen with faint scan lines, a row of big coloured hardware buttons under it (Home, Back, a large green DON'T PANIC, Next, Print), a battery mark that is never quite full, and the line "made cheaply, with love". As in `guide-mockup.html`.
   - **A cover every time.** Opening the Guide shows the cover, "DON'T PANIC" in large friendly letters, and one press anywhere goes past it. It shows once per opening of the Guide (not on every page), and the green button brings it back. It is a holding card that belongs to the kit, so it needs no new mechanism.
   - **It does not turn pages.** Its pages change the way a cheap screen redraws: a quick stepped wipe. No paper effects anywhere in the Guide. Its look is a new one, "Guide gadget", not the warm-paper "Field guide" theme now on main; that theme stays as an ordinary look for anyone who wants it. Its palettes are night (the default) and a brighter day screen, not cream.
   - **The cover words and the borrowed lines. Question 5 is reversed.** Alex, 25 Nov 00:57 UTC, in the builder's thread: "keep them all everyone is fine with a douglas adams joke. the whole point is that this IS the guide". So all 131 of his lines, the 36 with Douglas Adams' names and phrases included, are in the Guide's own note set; the cover reads "DON'T PANIC" with "Megadodo Publications" under it, as in SSE3; and there is no fan pack for them. Section 6.6 and the recommendation in question 5 no longer apply. (Built that way in PR 481.)
   - **Figures in jokes.** Alex, 25 Nov 00:43, same thread: a joke may type its own percentages when they plainly cannot be true (they add up to over 100%, or one is over 100%), "then you know you can rely on what's left". Every other figure in a margin note still comes from a slot. This amends rule 6.2's "may not contain a typed number". He also asked for the one-liners to be sorted by use case and matched to the scene's tags, with top and bottom banners on screens; the builder's package log has the detail.
5. **"those reports that look like paper... can you have the pages flick? replicate the paper experience :)"** Yes. Editions whose look is paper (Survey atlas and Company in their own looks) turn the page when you step. **It curls, it does not swing** (Alex, 25 Nov 01:41: "have the paper curl as you turn it - less mechanical - like you are leafing through it"): the bottom outer corner lifts first and the sheet peels back along a slanted fold that straightens as it crosses the page, showing the pale back of the paper with a soft highlight along the curl and a shadow on the page beneath, which is already there to be read. The fold wobbles slightly, as a hand does. About 0.7 seconds; going back lays the sheet down again in reverse. The builder's recipe is in the mockup's `turn()` function: clip the page along the fold, draw the lifted part mirrored across it, shade it across the fold, and cast one blurred shadow. A drag or swipe carries the page with the finger and lets go either way. Stepping fast skips straight to the last page asked for. A deck of sheets is hinted by a thin stacked edge on the side you are turning towards. It is the "Turn" moment of section 11A.4a with the delivery "Page turn", so the same edition wearing phosphor in the Hauler kit types on and does not flick. Reduced motion gives a plain change. The mockup's Back and Next show it.

7. **"those 'paper' based ones will be printable too - a print view showing all the pages - so you can tick untick the ones you want to print is probably a nice touch and a black and white toner friendly version."** (25 Nov, 01:43.) This replaces the small "Print…" dialog of section 9 with a **print view** (mockup `print-mockup.html`, screenshot `print-view.png`):
   - **Every page of the book as a small sheet,** in order, each with a tick box, its page number and name, and how many sheets of paper it takes. Ticked sheets are bright; unticked ones fade back. All are ticked on opening.
   - **Quick picks** above the sheets: All, None, Only the page I was on, This chapter. A running count beside Print: "6 of 9 pages, 7 sheets".
   - **Ink: As shown, or Black and white.** Black and white is a true toner-friendly form, not a grey copy of the colours: white ground, black ink only, no tints or filled panels, plates as line drawings with hatching where colour carried meaning (the Paper theme on main already works this way: "one ink, so colour collapses to patterns"), black bars stay black, stamps become outlined boxes, and the logo is printed in one ink (the ASCII art already is). The thumbnails redraw in the chosen ink, so what you see is what the printer gets. This is the "Save ink" switch of section 9 under a clearer name, and it is offered for every edition; a dark edition printed "As shown" warns once how much ink that is.
   - **Paper: A4 or Letter,** then Print, which hands the ticked pages to the browser's own print (and so to PDF).
   - "Print this page" stays as the one-press shortcut and skips this view. The Guide and the phosphor Hauler pages are printable through the same view; they simply have no page curl.
   - It works from the page model alone (the thumbnails are the same blocks drawn small), so it needs no new data. (Package AT4.)
6. **"the edition at the top is fine for perusing as the GM - in reality - the bookmark of these eg: Survey Atlas is just added to the video routing and can be fired at any end point and used by that user."** (20:17.) So the Edition switch in the mockup's top bar is the GM's own, for browsing and choosing. A player's screen has no Edition switch. An Atlas in an edition ("Survey atlas: Kestrel", "The Guide: Tarn") is a bookmark, made automatically (note 1), and on the Switchboard it is a source like any other: fire it at Everyone, a role or one screen, and whoever is at that screen reads and steps through it themselves. This replaces "the choice is kept per role" and "players may change it" in section 4.2: the edition travels with what was sent, not with the role. A kit still names the edition its own Atlas part opens in.

Changes to earlier sections that follow from these: in 11A.3 the Guide's look is "Guide gadget" and its palettes night and day; in 11A.4a the Guide uses a stepped wipe, and Page turn belongs to the paper editions; in section 7 the logo order is supplied, built-in, ASCII; AT6 gains the built-in logo set, AT7 and AT15 the gadget look and cover, AT4 and AT15a the page turn.

## 15. Not in 4.0

- An editor screen for editions or interface kits, and any slider panel for looks (the authoring guide and Orac are the way in, as Alex said).
- Knowledge that ages or goes stale by itself ("last surveyed 40 years ago" as a warning). It arrives with 4.1's living things and will use the same source lines.
- Newspapers, bulletins and in-universe publications that write themselves.
- Per-player secrets inside one crew's Atlas (one role, one knowledge, as ruled).
- Paid editions on Explorers.

## 16. Picking this up without the chat

- **The words, so they are not mixed up:** a *part* is a view used on a player's screen; a *layout* arranges parts on one screen; a *look* is colours, type and filters; an *edition* is how the Atlas reads; an *interface kit* bundles layouts, a look, palettes, a view style and an edition for a role. Bookmarks, presets, roles and screens are the Playbook design's words and are unchanged.
- **Nothing is built.** The only existing pieces are the ones this leans on: the player scene and `sentence()` (ui-contract), `PlayerText`, `PlayerCard` and `PlayerTable` (`packages/views/player-stack`), the `page` layout and `paper` theme, `players.survey.screen.json`, `SeatBrand` (`ui-contract/src/brand.ts`), looks, the figures door, and the action list.
- **Depends on, by track:** SV (secrets: `known_to`, pencil and ink, `told`), PB (roles, layouts, sources and presets; brand on the role card), the agents track (Orac change requests, Minerva's fact check), NV (the object menu).
- **SSE3's source** for anyone who wants to read the originals is on Alex's PC under `C:\Development\star-system-explorer-v2\star-system-generator\src\lib\` in `reports/`, `catalogue/` and `catalogue/document/`; the joke list is `C:\Development\star-system-explorer-v2\Mostly_harmless.txt` (also `catalogue/guideNotes.ts`). They are not in this project's repositories, so the builder of AT7 needs that file copied into the pack by Alex's say-so.
- **The mockup's names** (Kestrel, Tarn, Hollin Flats) are the repo's fixture; "Halden-Voss Deep Survey" is invented for the mockup. The two jokes in the mockup are new, written to show slots, and are not from Alex's list.
- **XO-style rule:** AT is a track code for threads and files only, never shown in the product.
