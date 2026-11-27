# SSE4 store packs: one way to make, find, stack, update and publish a pack

Status: **ACCEPTED by Alex 27 Nov 2026 12:02 to 12:05 UTC** ("Otherwise yes to all"; question 10 answered 12:04; question 1 answered 12:05, changed from the recommendation: kind is a label read from the contents, not a permission that refuses). His rulings are in section 13A and win where the text differs. Nothing here is built. Written by the thread "Design store packs" (Fable, design only) at Alex's request of 27 Nov 11:47 UTC: "a short design effort to rationalise 'store packs' - how they categorise, manifest and appear, layer and can load prerequisites securely (advantage of OUR store)... labelling the old v3 ones. How do we 'save them' and upload them - a smooth workflow as connected." Track code **SP**, a code only, never shown in the product.

Read with: `SSE4-explorers-paid-packs.md` (money, accepted 21 Nov; nothing here changes it), `SSE4-example-packs.md` section 2 (shelves), `SSE4-compound-ships-design.md` section 8B.3 (parts packs, CM18), `SSE4-real-catalogues-design.md` sections 5.2 to 5.3A (downloaded data, detail expansion packs, RW8), `SSE4-edits-reach-design.md` (block edits). On the Explorers side: `docs/decisions.md` D-98 and D-99 in `FrunkQ/starsystemx-creator-hub`.

## 1. The short answer

1. **The hard parts exist.** One format (`sse.pack/1`), one door (Open, drop, an Explorers link), a store that keeps each published version unchanged for ever, gives each pack id to its first publisher and serves the exact bytes with their fingerprint. This note adds no second format and no second door.
2. **Six things are loose, and each gets one rule:**
   - A pack's `kind` means nothing today (the shipped ship list is filed as `base-physics`; Sol today and Paper spacecraft are both `universe`). **Kind becomes an honest label**, read from what the pack contains, so the user always knows what it will touch. The shelf is worked out the same way.
   - Title, maker and credits are written three different ways. **One `about` block.**
   - A pack can say what it needs, and nothing fetches it. **One "this needs" sheet**, filled by the store, checked by fingerprint.
   - Flows need an explicit `replaces` to override; data (engines, parts, ships) silently lets the later pack win. **One layering rule for everything.**
   - An installed pack cannot be updated at all (the app refuses "already in the stack"), which is why a kept campaign never gets a newer Sol. **Campaigns pin exact versions; updates are offered, previewed and never applied unasked.**
   - There is no way to make a pack, or publish one, from inside the app. **"Make a pack" and "Publish" on the Packs screen**, using the device pairing the store already has.
3. **Secure means five plain things** (section 6): a pack is data and never code; its bytes are proved by fingerprint; only our store is fetched from automatically; its card says truthfully what it touches, whatever its author wrote; and every change it would make is shown before it is added.
4. **V3 gets a label, a door and no conversion of rules** (section 10): every V3 item on the store says V3 and opens in SSE4 through the importer; V3 custom definitions do not come across and the report says so.

## 2. What exists today, and what is new

| Need | Today | New in this note |
|---|---|---|
| Format and checking | `sse.pack/1`; `crates/sse-pack` resolves a stack: ids, versions, kernel range, `requires` present lower in the stack, one publisher per quantity, explicit `replaces` for flows and blocks, reproduction checks run on load | kind as a label worked out from contents; the `about` block; one layering rule for data |
| A pack kept as a folder | `store-packs/<name>/`, made one file by `scripts/bundle-pack.mjs` (CM0) | one script that also checks and previews the listing (SP9) |
| The door | Open, drop and `?explorers=` take a pack, show how many values it would change, and add it (`open-file.ts`, U5) | the "this needs" sheet; update in place |
| The store | Explorers `v4_items`: shelves, a "contains" filter read from the pack, one owner per id, versions never change, bytes stored by sha256 and served with `x-sse-sha256`, fan-work field, holds and reports (D-98, D-99) | a resolve route; V3 marks; a draft state (needs Alex, question 6) |
| Fetch and verify | `fetchExplorers`: only `explorers.starsystemx.com/api/v4/<slug>`, listing sha, header sha and our own digest must agree | the same check on every prerequisite |
| Where packs live in the browser | inside the campaign save; over 256 K characters, beside it in IndexedDB `sse-packs` by sha256 (CM0) | that store becomes the device's **library**; the campaign holds a pin list |
| Defaults fetched from the store | `stack.json` `from_explorers` (PR 712, open): Sol today into fresh campaigns | the same list as the general "starter set" (section 8.3) |
| Publishing | a web form at `/upload/v4`; device pairing exists (`/api/device/start`, `poll`) and its token identifies a creator to publish as, but SSE4 never uses it | Publish from the app |
| Money | designed and part built (PP1 to PP3), behind the off switch | nothing; section 11 only says how the rules surface in the app |

Three findings worth knowing:

- **F1.** Nothing checks `kind`. No code in the kernel reads `PackKind` beyond parsing it.
- **F2.** A campaign published to Explorers still carries whole copies of every added pack (`stack.added`); the site strips the GM record and undo history, not the packs. The paid-packs ruling 9 says a shared campaign names packs and does not carry them. Not built (it is PK1's neighbour). Section 7.2 settles it.
- **F3.** Sol today's `about` uses `label`; the store reads `about.title`; Traveller's `about` is a plain string. So titles on the store fall back to the id unless typed in by hand.

## 3. Kind is a label read from the contents; so is the shelf

**Ruled by Alex, 27 Nov 12:05 UTC:** "label the pack kind - physics may travel with other stuff... user just has to know what it will impact - so sneaking physics in is fine - it will ALSO be categorised as physics." This replaces the first draft, which had the kernel refuse content outside a pack's kind.

The six kinds stay as they are in the format. What changes is that a pack's kinds are **worked out from what it carries**, by the kernel when the stack loads and by the store when it is listed, and nothing is refused for its kind. A ship pack that also carries a physics change is a universe pack **and** a physics pack: it shows both lines on its card, sits on both shelves, and its physics is marked like any other. What the author wrote in `kind` is a hint; where it leaves something out, the worked-out kinds win and the card says so.

The table reads as "carrying this makes a pack that kind":

| Kind | Words on the card | Carrying any of these makes it this kind |
|---|---|---|
| `presentation`, `suite` | "Changes how things look. Changes no numbers." | looks, themes, view styles, overlays, tours, vocabulary, name schemes, class labels |
| `universe` | "Adds things. Changes no physics." | parts, ships, stations, engines as data, settings, templates, dressing rules, tags, catalogues, calendars, units |
| `game-system` | "Reads game codes off the physics. Never feeds it." | code sheets, rules, sifters, difficulty |
| `physics` | "Changes physics. Everything it touches is marked." | flows, boxes, block edits, quantity kinds, reference data; each override still names what it replaces |
| `base-physics` | (ours) | the shipped base; the store takes this kind only from the app's own account |

A pack of more than one kind shows every line that applies. The shipped packs' written kinds are wrong today (`base-ships` says `base-physics`); because kinds are now worked out, nothing has to be refiled and no golden moves. Their written kinds are corrected whenever a pack is next edited for another reason.

**Shelves** are for people and are read from the contents, as the store already does for tag sets. The author may name a shelf in `about.shelf`; that is its main shelf if the contents support it, and it is also listed on every other shelf its contents give it:

| Shelf | Holds |
|---|---|
| Campaigns and worlds | campaign files, systems, clips |
| Settings | `universe` packs with places, factions, templates, dressing; catalogues and detail expansion packs (RW8) sit here too |
| **Constructs and parts** (new) | every kind of construct and what it is made of: ships, stations, land structures, and their parts and engines (section 3.1). Named by Alex, 27 Nov 11:57 to 11:58 UTC: "ships and parts makes sense... and constructs and constituent parts", then "'ships' is too narrow" |
| Game systems | `game-system` |
| Physics | `physics`, including new reference data (materials, colours, constants), which changes numbers and so is physics. Alex, 27 Nov 11:58 UTC: "they can travel together as a physics pack". A pack of reference data alone sits here too, found by the contains filter |
| Looks, names and tours | `presentation`, `suite` |
| Fan-made | anything with a named setting, apart from everything with a price (existing ruling) |
| V3 library | section 10 |

The small things (calendars, unit schemes, ship styles, overlays) keep the "contains" filter and get no shelf, as ruled on 23 Nov.

**No shelf for real data (Alex, 27 Nov 11:57 UTC):** "probably not necessary as it is just another campaign world and with the star catalogue importer many will have real data." A world or pack holding real data sits on the shelf its contents put it on. Real data is found with the contains filter ("real data", read from `downloaded_data`) and is told apart by the Free only mark (section 11).

### 3.1 What the ship builder needs (from the ship builder thread, 27 Nov)

- **Parts packs and ship packs are both `universe` packs on Constructs and parts.** A ship pack `requires` the parts packs its ships are built from, so adding a ship pack brings its parts (section 5). A ship never carries a copy of another pack's parts.
- **The port standard is a versioned requirement like any other.** The connection-point sizes ship in `sse.base-parts`; a parts pack writes `requires: sse.base-parts ^1`. No special manifest field: a new port standard is a new major version of that pack.
- **Per part: a licence and an era.** `licence` on a part overrides the pack's; a part resting on downloaded data carries the non-commercial flag (section 11). Each part carries an **era id**.
- **Eras are pack data** (from the ship builder thread, CM14a built; Alex, 27 Nov: "lets not do them by age but by eras... we can have a few future eras", "they can vary in length"). This replaces the first draft's first-flight year and fixed 25-year bands. An era has an id, a name, optional from and to years (any length), a tech level and a future flag. The shipped eras are in `sse.base-parts`; a parts pack may add its own. Era lists from several packs merge by id: the same id with the same content is one era, a new id is added, and changing an era another pack defined takes the usual explicit `replaces` (section 7.1).
- **The default catalogue** is the shipped `sse.base-ships` plus whatever ship packs are in the campaign; each ship says which pack it came from. Sol today's real craft arrive this way.
- **Names.** Nothing we ship carries a named ship from film, TV or games. For what users upload, see the ruling of 12:04 in section 13A: free fan content rests on the disclaimer already in place; anything for sale is scanned (SP13). The app does not check names.
- **Saving a built ship** is section 9: "Add to my ship pack" from the builder, then Publish.

## 4. The manifest

Nothing is removed. One block is added at the top level, left out when empty so existing packs keep their hashes until they are edited:

```json
"about": {
  "title": "Sol today",
  "summary": "Real spaceflight from 1957 to now.",
  "maker": { "name": "FrunkQ", "handle": "frunkq" },
  "shelf": "constructs-and-parts",
  "credits": [ { "text": "GCAT, Jonathan C. McDowell", "url": "https://planet4589.org/space/gcat/", "licence": "CC-BY-4.0" } ],
  "notice": { "text": "...", "source": "..." },
  "fan_setting": null,
  "downloaded_data": [ "gcat" ],
  "changes": "0.2.0: 41 new launches; Starship flight 9 corrected."
}
```

- `provides.other.about`, `credit`, `credits` and `notice` are read as before and folded into this block when a pack is next saved by the app. Readers accept both for good.
- `requires` stays `{id, range}` and gains an optional `suggests` list of the same shape ("looks better with"). A suggestion is offered on the "this needs" sheet, unticked.
- `downloaded_data` names the catalogues whose values the pack holds. It is what makes the never-sold rule checkable (section 11), and the app fills it in, the author does not.
- `changes` is one line a GM can read, shown on the update offer.

**Two fingerprints, both kept, neither inside the file.** The store's sha256 of the bytes proves the download. The kernel's content hash (canonical CBOR) proves the meaning and is what a campaign pins. The pack card shows the first eight characters of each.

## 5. Prerequisites: the "this needs" sheet

Today a pack that needs another is refused with "requires X, which is not lower in the stack". Instead:

1. The app sends the store the pack's `requires` (and, for a campaign, its pin list): `GET /api/v4/resolve?need=acme.parts@^1.2,...`. Public, no account, cached at the edge.
2. The store answers a **plan**: for each need, the highest public version that fits, with its sha256, size, licence, maker, whether it is free, paid or already bought, and its own needs, followed all the way down. A need nobody has published is said by name.
3. The app shows one sheet before anything is fetched: "Kestrel yards needs 3 more packs, 5.2 MB", each with maker, licence and price mark, and the total. One press adds them all.
4. Each file is fetched and checked exactly as today (listing sha, header sha, our own digest), placed lower in the stack than the pack that needs it, and the whole stack goes through the kernel's existing preview ("would change N values") before Add.

Suggest, never refuse: a need that is paid and not bought, or missing, does not block the sheet. The pack is kept in the library, the sheet says what is missing and where to get it, and it can be added the moment the need is met. A campaign opened without a pack it pins opens with the values it adopted, and says what the missing pack would add (paid packs ruling 9).

A file opened from disc with no connection works the same way from the library: needs already on the device are found by id and hash; others are named.

## 6. What "secure" means here

This is the advantage of our own store, said as rules a builder can test.

1. **A pack is data, never code.** JSON only. Formulas are the kernel's own formula blocks, run by the kernel within its limits. No script, no HTML, no address the app fetches on a pack's behalf. Pictures and shapes are data inside the pack, of listed types. A link in a credit is text until the user presses it.
2. **Integrity.** Three-way sha256 on every fetch (exists). The campaign pins the kernel content hash, so a pack that changed is seen even if it arrives by another road.
3. **Provenance.** Only `explorers.starsystemx.com` is fetched from without the user choosing a file (exists). A pack id has one owner; a published version never changes. The card says where a pack came from: "From Explorers, published by (handle)", or "From a file: nobody vouches for it". A file is still opened.
4. **An honest label.** A pack's kinds are worked out from its contents by the kernel (section 3), not taken from what its author wrote, so a file from anywhere says truthfully what it will touch. Physics inside any pack is named as physics on the card and on the shelf.
5. **Its own names only.** Everything a pack defines has an id under the pack's own id. Touching another pack's thing takes an explicit `replaces` naming it (section 7.1).
6. **Shown first.** Every value a pack would change is listed before Add (exists), and marked after.
7. **Limits.** The store's size limit (50 MB, 200 MB for supporters, existing gates); the app shows the size before fetching; at most 32 packs and 8 levels of needs in one plan; nothing is fetched without a press, except the app's own starter set (section 8.3).
8. **Withdrawn packs.** When the store holds or removes a pack, the resolve route says so and the app marks it "withdrawn from Explorers" on the next check. It keeps working: there is no remote off switch, as ruled for paid packs.

Later, and only if wanted (question 8): the store signs a small listing receipt (id, version, hashes, maker, date) with the key PP4 already plans, so a pack passed hand to hand can still say who published it when offline.

## 7. Layering

### 7.1 One rule for everything a pack provides

Stack order, bottom to top: the shipped base, then installed packs (a pack always above what it requires; otherwise in the order added, which the GM can change on the Packs screen), then the campaign's own pack holding the GM's edits.

- A pack defines things only under its own id (`acme.yards/kestrel`). Two packs cannot collide by accident.
- To change another pack's thing, a pack names it: `replaces: { thing, pack, range }`. This is today's rule for flows and blocks, now also for engines, parts, ships, templates and the rest. Today's "later pack silently wins" for engines goes.
- If two packs both replace the same thing, the stack still loads: the higher one wins, the Packs screen shows the clash with both names, and the GM's choice is written as a `replaces` in the campaign's own pack, so it is saved and in plain sight.
- **One-at-a-time things** (calendar, unit scheme, vocabulary, theme): any number may be installed; the campaign says which is in force, by default the highest in the stack. Today the first calendar found wins.
- Every value, part and ship can say which pack gave it and what it replaced.

Moving existing packs to own-id names is done with warnings first, the shipped packs fixed, then refused (SP2). Old ids are kept as aliases so saved campaigns still open.

### 7.2 The library, the campaign, and what a file carries

- **The library** is the packs on this device (the IndexedDB store that exists), by hash. Several versions of one pack can sit there.
- **A campaign holds a pin list**: id, version and content hash for each pack, in order. It is the campaign's recipe with the seed: same seed and same pins, same universe.
- **Save campaign to a file** carries the packs whole, as now. It is the GM's own backup and must open offline anywhere.
- **Share or Publish a campaign** writes pins only for any pack that is on the store, and carries whole only packs that are not (the campaign's own pack, an unpublished pack of the GM's). This closes finding F2 and means a paid or downloaded-data pack is never re-published inside someone's campaign.

## 8. Updates

### 8.1 A published version never changes; a campaign never changes unasked

- When online, on opening a campaign and at most once a day, the app asks the resolve route about the campaign's pins. One request, no account.
- Newer versions show as a quiet count on the Packs entry, with each pack's `changes` line.
- **Update** runs the same preview as adding ("would change N values, such as..."), then swaps the pin. The old version stays in the library until no campaign pins it, so Undo and old saves work.
- Version numbers mean: patch, fixes with no ids added or removed; minor, adds; major, removes or renames ids. Before a major update is applied the app lists every placed thing that points at an id that is going, and keeps those things with their adopted values.
- Removing a pack lists what rests on it first: placed things, and packs that require it.

### 8.2 The Sol case

PR 707 names an old Sol by a special check. With pins it is the general case: Sol's pack has a newer version, the campaign is offered it with the count of values it would change, and the GM chooses. The special check can go when SP5 lands.

### 8.3 The starter set

`stack.json` `from_explorers` becomes a list of pins fetched into a fresh campaign: today Sol today. "A smaller Sol in the app with an expansion on the store" (Alex, 27 Nov) is then a change to this list and to what ships in `packs/`, not new machinery. The rule for what ships inside the app: what the first picture needs with no connection. Everything else is a store pack. Automated browsers fetch nothing, as now.

## 9. Appearing, making, saving and publishing

### 9.1 The Packs screen (the Workshop entry that exists)

Three parts, no new tab:

- **In this campaign**: the stack in order, each pack's card (title, maker, version, what it touches in words, licence, size, marks, needs, what it changed), update marks, drag to reorder, remove.
- **On this device**: the library, including "yours, not published".
- **On Explorers**: the shelves, read from the public `/api/v4`, with the contains filter. Add is one press and goes through the "this needs" sheet. A priced pack shows "Buy on Explorers" (PK2).

Marks on a card are the badge set of section 9.5, the same in the app and on the store.

### 9.2 Make a pack

From anywhere a thing is selected (a ship in the builder, parts, a system, a name scheme, a look, the campaign's physics edits): **Add to a pack**, into a new pack or one of yours.

- Id is filled in as `<your Explorers handle>.<name>` when connected, `local.<name>` when not (renamed on first publish).
- Needs are worked out, not typed: a ship built from Acme's parts makes the pack require `acme.parts`. Another pack's content is never copied in.
- Kind and shelf are worked out from what was added. Licence is picked from a short list, with a recommended default (question 5).
- Catalogued values are written by reference and `downloaded_data` filled in (PK1).
- **Save** keeps it in the library and offers the one file. Saving again while unpublished overwrites the draft; the version number only moves on publish, and the app raises it for you.

### 9.3 Publish, when connected

- **Connect to Explorers** once per device: the app shows a short code, the user approves it in the browser (device pairing, exists). The token can publish as that creator and nothing else.
- **Publish** shows the listing exactly as the store will show it, runs the store's own checks in the app first (the same rules, from shared test files, so the app and the site cannot disagree), then asks for the rights statement, the fan-setting field and, for sellers when the switch is on, a price with any refusal named.
- The upload goes to the existing `/api/v4/upload`. The answer is the listing's link, kept on the pack's card.
- **Publish an update** is the same button: the id is yours, the version is raised, `changes` is asked for.
- Campaigns, systems and single ships use the same button from Share.
- Not connected: the same sheet ends in "Save the file" and a link to the web upload page.

### 9.4 Our own packs

`store-packs/<name>/` stays the source. One script replaces the bundle step: `scripts/pack.mjs check|bundle|listing <folder>` gives the file, both fingerprints and the listing preview, and CI runs `check` on every folder and confirms the starter set's pins match. Publishing stays Alex's own press on his account; nothing in CI or in an agent publishes.

### 9.5 Badges: one set, in the app and on the store

Alex, 27 Nov 12:34 UTC: "probably need a badge update on the store now we have more types of content." The store has a badge component and one fan-made badge today. This is the whole set. Every badge is one or two words with a line of plain words behind it, worked out from the item itself and never typed by its maker, in this order on a card:

| Group | Badges | Shown when |
|---|---|---|
| Which app | **V3**, **V4** | always, one of the two; after the upgrade pass (SP12) an item with both files shows both |
| What it is | **Campaign**, **System**, **Clip**, **Setting**, **Constructs**, **Parts**, **Game system**, **Physics**, **Looks** | one for each thing it carries, so a pack can show several. **Physics** shows on anything that carries physics or reference data, whatever else it is (the "also physics" of section 3), and is the one badge in a warning colour |
| Where it stands | **Fan-made** (names the setting), **Fictional physics**, **Real data** | fan-setting field filled; `fictional` set; `downloaded_data` not empty |
| Money | **Free**, a price, **Free only** | every item shows one. Free only is on anything that can never be sold (real data), and its line says why |
| State | **Needs N packs**, **Update**, **Withdrawn**, **Yours**, **Draft** | Update and Yours in the app only; Draft only to its maker |

Smaller contents (calendars, ship styles, overlays and the rest) stay on the contains line under the badges, not as badges. One shared table of badge names, lines and rules is used by the app and the store, so they cannot drift (with SP1's table).

## 10. V3

- **On the store.** Every V3 map, system and clip shows a **V3** mark, every V4 item a **V4** mark (the API already says `generation`). V3 items get one more button beside their existing ones: "Open in SSE4", which hands the file to SSE4's Open door, where the V3 importer that exists reads it. Nothing published is altered, moved or re-saved.
- **In the app.** What came in says so: "Imported from V3: (title) by (maker)", with the importer's report of what was kept as authored and what was worked out again. The `sse.sse3-classic` labels pack is offered on the "this needs" sheet so worlds keep the class names their authors knew.
- **V3 custom definitions** (`rulePackOverrides`: custom gases, body types, tech) are not converted by the importer as a file opens. They are listed in the report as kept and not used.
- **The upgrade pass (Alex, 27 Nov 12:02 UTC).** "Once we get everything sorted out and settled we will do an upgrade of all the V3 stuff (and hand fix if needed) and upgrade them for the users as there are not many. The store will be able to serve the right version to the right app (v3 will probably move to legacy.starsystemx.com)." So: when V4 is settled, every V3 item on the store is upgraded once by us, fixed by hand where the importer falls short (custom definitions included), and kept **beside** its V3 original as the same listing. The store serves the V3 file to V3 and the V4 file to SSE4. The V3 bytes are never altered. Each upgraded item is checked against its original before it is shown, and the pass is run by Alex or on his go, as with anything that touches published worlds (SP12).
- **Publishing the V4 version** is the owner's choice, through 9.3, as a new V4 item whose credit chain points at the V3 original. The V3 original stays.
- Universe Sandbox and SpaceEngine files keep going through the converter, as now.

## 11. Free, paid, fan-made and never sold, as the app shows them

No new rule but one line; this is where the accepted ones surface.

**The store says it prefers free (Alex, 27 Nov 12:02 UTC).** The upload page and the Publish sheet carry one message: "We suggest store uploads are provided free. There is a mechanism for charging; contact us if you are interested." This fits the accepted rule that selling is by invitation at first. Wording is Alex's to change.

| Mark | Rule | Where it bites |
|---|---|---|
| Free | always free, never ranked below paid | nowhere |
| Paid | original work only; first version seen by a moderator | Buy on Explorers; a receipt on the card |
| Fan-made | never a price; own shelf | the price box is not offered |
| Free only | holds downloaded data (`downloaded_data` not empty, or any part flagged non-commercial) | the price box is not offered, and names the data |

A paid pack may **require** a Free only pack and may not contain its data; it may not require a fan-made pack. The Make a pack sheet says which mark a pack will get, and why, before anything is uploaded.

## 12. Packages

Small, in order. Owners are proposed. Explorers packages add only; any migration is shown to Alex and run by him.

| Id | Package | Repo | Needs | Checks |
|---|---|---|---|---|
| **SP1** | Kind as a label: the kernel and the store work a pack's kinds out from its contents (one shared table and test files); the lines on the card; a pack on every shelf its kinds give it. Nothing refused | SSE4 kernel, UI, Explorers | none | a pack written as `universe` that carries a flow shows as physics too, on both shelves; no golden moves |
| **SP2** | One layering rule: own-id names and explicit `replaces` for data; clashes shown; one-at-a-time things chosen by the campaign; old ids as aliases. Warnings first | SSE4 kernel, UI | SP1 | two packs defining one engine id: warned, then refused; an old save opens |
| **SP3** | The `about` block, `suggests`, `changes`; readers take old and new; store reads it (fixes F3) | SSE4, Explorers | none | Sol today shows its title on the store untyped |
| **SP4** | Resolve route on Explorers and the "this needs" sheet in the app, with limits | Explorers, SSE4 UI | SP3 | a ship pack brings its parts pack; a wrong sha stops one file and says which |
| **SP5** | Library and pins: the campaign holds pins, the library holds bytes; update offer, preview, swap, old version kept; removal lists dependants. Retires the Sol special case | SSE4 UI | SP4 | a kept campaign is offered the newer Sol and can undo it |
| **SP6** | Packs screen in three parts, with the badge set of 9.5; shelves read in the app (takes in PK2) | SSE4 UI | SP3 | press budgets unchanged |
| **SP7** | Make a pack: add things to a pack, needs worked out, kind and shelf worked out, save to library and file. Includes "Add to my ship pack" from the builder | SSE4 UI | SP2, CM14 for the builder button | a ship saved, removed and re-added from its pack is the same ship by hash |
| **SP8** | Connect and Publish from the app: pairing, shared check files, listing preview, upload, update. Draft state if Alex says yes | SSE4 UI, Explorers | SP7 | round trip on a test account; nothing reaches a public shelf in tests |
| **SP9** | `scripts/pack.mjs` and the CI check for `store-packs/` and starter pins | SSE4 | SP3 | a folder with a missing file fails CI |
| **SP10** | V3 marks and "Open in SSE4" on V3 items; import label and report in the app | Explorers, SSE4 UI | none | a V3 map opens from its page; the original's bytes unchanged |
| **SP11** | Shared campaigns carry pins, not packs (F2) | SSE4, Explorers | SP5, PK1 | a published campaign holds no copy of a store pack |
| **SP12** | The V3 upgrade pass (section 10): a V4 file kept beside each V3 original, the store serving each app its own; hand fixes; `legacy.starsystemx.com` for V3. Only when V4 is settled, and only on Alex's go | Explorers, converter | SP10, a settled V4 | every V3 original byte for byte unchanged; each upgrade compared with its original |
| **SP13** | The sale scanner: a pack given a price is read for names of known settings, ships, brands and publishers, and for text and pictures matching known sources; findings are listed for the first-sale moderator. Free and fan packs are not scanned | Explorers | PP5, PP6 | a priced pack with a known screen-ship name reaches the moderator with the name shown; a free one is untouched |
| **SP14** | Badges on the store (9.5): the full set on shelf cards, item pages and the V3 library, from the shared table; the public API returns them so the app shows the same | Explorers | SP1, SP3 | a ship pack carrying a flow shows Constructs, Parts and Physics on its card; every V3 map shows V3 |

The "we suggest free" message (section 11) goes in with SP8 on the app side and is a one-line change on the store's upload page.

Already owned elsewhere and unchanged: PP1 to PP8 (money), PK1 (catalogued values by reference), CM16 to CM18 (part editor and parts packs), RW8 (detail expansion packs; its "pack kind" becomes an ordinary `universe` pack found by the real data filter, and its byte-range reads stay its own).

## 13A. Alex's rulings, 27 Nov 2026

- 11:57 UTC: no Real data shelf; "it is just another campaign world and with the star catalogue importer many will have real data".
- 11:57 to 11:58 UTC: the new shelf holds "constructs and constituent parts"; "'ships' is too narrow". Named Constructs and parts.
- 11:58 UTC: new reference data "can travel together as a physics pack".
- 12:02 UTC: the store carries the "we suggest free" message (section 11).
- 12:02 UTC, question 7 amended: V3 items will be upgraded by us in one pass once V4 is settled, hand fixed where needed; the store serves the right version to the right app; V3 probably moves to legacy.starsystemx.com (section 10, SP12).
- 12:02 UTC: "Otherwise yes to all": questions 3, 4, 5, 6, 8, 9 and 11 stand as recommended. Question 6 still needs its one database change shown to him and run by him.
- 12:04 UTC, question 10 answered: "we will need to build a scanner on the store for copyright content IF the pack is being sold. Gonna turn a blind eye to fan content for now - we have the disclaimer for that in place." So free fan content is left to the existing disclaimer, with no name checking; a pack offered **for sale** is scanned by the store for other people's names, brands and content before its first sale, and what the scanner finds goes to the moderator who already looks at every first paid version (paid packs section 7.4). The scanner informs a person; the person decides (SP13).
- 12:04 UTC, the same point said outright: "the store CAN list TV/film ones - fan content... we will deal with issues as they arrive. If they charge - different case." So users may publish film and TV ships and settings free, as fan content; the earlier "no named ships from film, TV or games" ruling is read as covering what we ourselves ship and feature (the thread's reading).
- 12:05 UTC, question 1 answered, not as recommended: kind is a label, physics may travel with other things and the pack is also categorised as physics; the user only has to know what it will impact (section 3). No pack is refused for its kind and no golden moves.
- 12:34 UTC: "probably need a badge update on the store now we have more types of content." The badge set is section 9.5; the store half is SP14.

## 13. Questions as put

1. **Kind as a permission, enforced by the kernel, with shipped packs refiled (one planned move of the core golden)?** Recommended: yes. *Answered 12:05 UTC, not as recommended: a label, nothing refused, no golden move (section 13A).*
2. **One new shelf, Constructs and parts, worked out from contents?** *Answered by Alex, 27 Nov 11:57 UTC: yes, and it holds all constructs and their constituent parts. Real data as a shelf was dropped the same minute; it is a filter and a mark.*
3. **One layering rule: own-id names, explicit `replaces` for data as for flows, the GM's choice saved when two packs clash?** Recommended: yes, warnings for one release before refusing.
4. **Campaigns pin exact versions; updates are offered and previewed, never applied unasked?** Recommended: yes. It is what keeps two devices on the same universe.
5. **Default licence offered in Make a pack.** Recommended: CC BY 4.0 (others may build on it with credit), with CC BY-NC and "all rights reserved, for sale" one press away.
6. **A draft state on Explorers ("only I can see this") so Publish can be tried before it is public.** Needs one added value in a database check, run by you. Recommended: yes.
7. **V3 custom definitions do not convert; worlds do, and the report says what was left.** Recommended: yes.
8. **A signed listing receipt on free packs too, so a file passed around still says who published it.** Recommended: later, with PP4's key; not needed for 4.0.
9. **A saved campaign file carries its packs whole; a shared or published one carries pins.** Recommended: yes.
10. **Screen-ship names: warn in the app, state it in the publish statement, moderators hold; no automatic name list.** Recommended: yes.
11. **Who builds.** Recommended: SP1 and SP2 to the kernel thread; SP3 to SP9 and SP11 to one new Opus thread "Build store packs"; the Explorers halves to "Integrate Explorers site with SSE4"; SP10 to the Explorers thread with the editing thread's Open door.
