# SSE4 naming: name schemes, name layers and language packs

Status: **accepted for building 22 Nov 2026** (Alex, 16:26: "just implement - not much to review - we can tweak later - just as long as they are available from store and can be included in the world/universe packs"). Design thread "Design naming and language packs". Track **NM**. Nothing here is built yet. Section 9 records the choices made as decisions; all can be changed later.

It builds on the handoff's section 5.1 (anything addressable can be renamed, a rename is a dated authored event, an object may hold several names), the surface design's section 4 (feature names, per audience), the night sky design's section 9 (catalogue designations from "the campaign's naming scheme"), the matter-pack design's sections 7A.7 and 7B (the Reference screen's "This setting" pages: Units and Calendars), the tags design (a label never feeds physics; universe packs) and the handoff's section 7 (packs are data only).

## 1. What Alex asked for (22 Nov, 16:23)

- "At the moment, it all seems very English, not alien, and not scientific."
- "Under references a bunch of language / naming packs, some lookup tables or cleverer items to bring syllables together."
- "Multiple ways of naming objects and let the GM choose a default for their universe, but also a default for each system and or planet as they get down to."
- Four kinds named: "scientific designations and formal catalog names"; "English-sounding names"; "international English, so it looks like it's been renamed by a whole bunch of nations on Earth"; and "straight up aliens that will have their own language and their own pronunciation".
- "This is not a fixed thing. This is again something that can be built. Probably these are simple text files that can be dragged in and created ... similar to a block and available off explorers."
- 16:25: "The user can always manually rename everything... and respect that HD102 I, HD102 II, etc would also be 'Mystar I' and 'Mystar II'... so it would be recursive where appropriate." (Section 4.5.)
- 16:26: the two conditions that matter: naming packs are "available from store" (Explorers) and "can be included in the world/universe packs". (Section 4.13.)

## 2. The short answer

Yes, and it is one mechanism with three parts.

1. **A name scheme** is a small piece of pack data that makes names. It is built from five plain makers (a list, a sound set, a number, a pattern and a mix). A catalogue style, an English style, a many-nations style and an alien language are all name schemes; none is a special case in code.
2. **Name layers.** Every object can carry several names at once: its designation, its common name, and what each culture calls it. Each name is a fact with a date and an audience, so a player's chart shows only the names that player's side knows. This already exists in the design for features; it becomes the rule everywhere.
3. **Naming settings that inherit downward.** The universe has a default scheme per layer. A galaxy region, a system, a world or one kind of feature on a world can set its own, and everything beneath follows the nearest setting above it.

A scheme is also where a language's **sounds, spelling, script and pronunciation** live, and optionally a small **word list with meanings**, so an alien name can mean "red mountain" and say so. Every name answers "why is it called that?" on a card, as every number answers "Why?".

A scheme is a text file. It can be dragged in, written by hand, made by Orac from a sentence ("a guttural insectoid species") and shared on Explorers.

## 3. What exists today

Read from the repo at main `6fb131e` and the design files.

| Thing | State today |
|---|---|
| **Surface features** | `sse-surface/src/named.rs` (S7). One `NamingSettings` in `packs/base-surface`: about thirty syllables (ka, lo, ri, an, tel, mar, ven ...), two to three joined at random, capitalised, dropped into a pattern per kind ("Mount {}", "{} Planitia", "Port {}"). This is the source of "it all seems very English". Uniqueness is a used-set inside one world, in generation order. Renames, per-audience names, the kept original and "restore" are built. |
| **Stars, planets, moons** | No generated names. A grown system shows as `Seed "<seed>"`; bodies carry formation ids (Aa, Ab) or their address tail. V3 imports keep the names in the save. Sol's names come from `base-sol`. |
| **Catalogued sky objects** | Designed, not a general mechanism: night sky Q6 ruled "a pack naming scheme, defaulting to the campaign's survey name plus a running number"; real-sky campaigns use real names. |
| **Biomes and guilds** | Structural names first ("closed tall canopy"), Earth words where the numbers match, "packs can add vocabulary" (evolution design 9.4). |
| **The pack format** | `provides` already lists `vocab` and `catalogues`, and the universe layer is described as holding "names". No shape has been given to any of them. |
| **Calendars** | Month and day names are typed by hand in the calendar data. |
| **V3** | The SSE4 documents record nothing about how V3 made names: `docs/v3-parity.md` has no row for it. NM0 looks, on Alex's machine, so that anything V3 users rely on is known. It is not copied. |

So the rules about names are settled and partly built (rename anywhere, several names, per audience, original kept, authored wins). What is missing is everything that makes the names: one generator, thirty syllables, features only.

## 4. The proposal

### 4.1 A name scheme, and the five makers

A scheme is a document with an id, a label, a short "about", and some of the pieces below. Each piece is a plain table a person can read.

| Maker | What it does | Used for |
|---|---|---|
| **List** | Picks a row from a table. A row holds the name and, optionally, its native script, pronunciation, meaning, language and source. Rows can carry a theme ("rivers", "astronomers", "sea birds"). | Real-language names, hand-written name lists, themed sets. |
| **Sounds** | Builds a word from sound classes (consonants, vowels, endings), syllable shapes ("CV", "CVC"), a length range, and rules: weights so common sounds are common, sequences that never occur, and replacements where sounds meet. | Invented languages. This is the "bring syllables together" piece, with real phonotactics behind it instead of a bag of syllables. |
| **Number** | Makes a designation from counters and facts: a survey prefix and a running number, a letter from b in order, a Roman numeral, a component letter (A, B, Aa), a coordinate string from the object's sky position. | Catalogue and scientific names. Unique by construction. |
| **Pattern** | A form per kind of object that joins the others with fixed text: `{word}`, `Mount {word}`, `{parent} {roman}`, `{list:rivers} Vallis`, `{means:red}{means:mountain}`. Several forms per kind with weights. | Every scheme. This is what `NamingSettings.patterns` does today. |
| **Mix** | Chooses among other schemes by weight, with a **clump**: the choice is made once per system (or world, or region) and held beneath it, with a small share of strays. | Many nations. A system surveyed by one expedition is named in one tradition, the way real naming is clumpy. |

Two further pieces are optional and arrive later (NM5):

- **Learned.** A scheme may hold a list of example words and say "make more that sound like these". The loader turns the examples into a letter-sequence table (a Markov chain of order two or three, whole-number counts only), and names are drawn from it. Useful when you can write fifty words in a language but not its rules.
- **Meanings.** A scheme may hold a small word list keyed by shared meaning ids (colours, sizes, landforms, numbers, sky words: about two hundred, published once by `base-names`). A pattern can then ask for a meaning instead of a random word, and may pick the meaning from the object's own facts. A red world's range in the Vesh language is "Khazrak", and its card says: *khaz* red, *rak* mountain. This is a lexicon and a joining rule, not a grammar. Translating sentences is out of scope.

One general rule covers them all: a scheme names anything that has a **kind** (star, planet, moon, belt, peak, sea, settlement, ship, month ...). The engine does not know the kinds. A scheme lists the kinds it has forms for and a default form for the rest, so naming a new sort of thing later needs a new call site and perhaps new forms, never new engine code.

### 4.2 Name layers: several names at once

An object's names are records, each with: the text, its layer, the scheme that made it (or "authored"), a from-date, who gave it, and which audiences know it. This is `NameRecord` in today's feature code, made general.

Three layers ship, and a pack may declare more:

| Layer | What it is | Default source |
|---|---|---|
| **Designation** | The formal, unique handle: "KS 4127 c", "Tarn II". | The campaign's designation scheme. Always present, so nothing is ever nameless and two things never share a handle. |
| **Common** | The name people use: "Tarn". | The nearest naming setting above the object (4.3). |
| **Local** | What a particular culture or species calls it. One per culture that knows the place. | That culture's scheme, from its universe pack or the life suite's species record. Exists from the date the culture got there. |

What follows from treating a name as a fact:

- **Players see the names their side knows.** A name is disclosed like any fact. An unvisited world shows the party's own survey designation; the locals' name appears when the party learns it. Two empires may letter the same planets differently, because each has its own designation layer from its own discoveries.
- **Which layer a chart shows is a view setting**, like units and calendars: "common, else designation" by default, settable per lens and per player view ("designation only" gives a hard-science chart; "local" gives the chart a native would draw).
- **Names have dates.** The time ruler shows the name in use at the date being looked at. A colony renames its world; the old name stays in the history.
- **Authored names always win.** Typing a name writes an authored record in that layer. The generated name is kept as the original and can be restored. Authored names are events on the address, so regrowing or re-resolving the object never touches them.
- **The find tool searches every name the seat knows.**

### 4.3 Choosing: one default, overridden as you go down

A naming setting says, for a layer, "names here come from this scheme". Settings hang on addresses and inherit down the address tree: universe, galaxy region, system, world, and optionally one kind on a world ("this world's craters come from the Astronomers list").

- The campaign has a default for Designation and for Common. That is the whole setup for most GMs.
- Any system or world card has a small "Names here" control showing what it inherits and letting the GM pick another scheme. This is Alex's "default for each system and or planet as they get down to".
- A polity or species that carries a scheme supplies the Local layer for places inside its territory or range, with no setting needed.
- A setting is an authored event with a date. It is setup, so it sits with the pack stack's rule (handoff section 7): working screens select from what is stacked.

Changing a setting changes generated names beneath it that are not yet **fixed** (4.4). For fixed names the GM is asked once: leave them, or rename them from the new scheme as of a date. The second choice writes ordinary dated renames, so it reads in the history as "the Concord renamed these worlds in 2291", and can be undone.

### 4.4 Determinism: the same name on every machine, and names that stay put

1. **A generated name is a keyed draw.** It depends only on the universe seed, the object's address, the layer, and the scheme's content hash. It never depends on the order things were looked at, on the device, or on any other pack. Naming draws use their own stream, so adding or changing a scheme cannot disturb a physics draw, and no physics draw can disturb a name.
2. **A campaign pins its packs by hash** (existing rule), so a name changes only when the GM changes the scheme on purpose. Updating some other pack changes no name. The check for this is an acceptance test of NM1.
3. **Names become fixed by the rule that already makes things permanent.** "Looking promotes" (galaxy design): selecting, visiting, cataloguing or routing to an object stores its drawn description. Its names are stored with it. From then on they are read, not redrawn, so even a later version of the same scheme cannot change a name anyone has used. Before that, a name is a pure function and costs nothing to keep.
4. **No clashes, without depending on order.**
   - Designations are unique by construction.
   - Among siblings made together (the planets of a system, the peaks of a world), names are settled in address order: a clash or a near-clash (two names one letter apart, which sound alike at the table) moves the later sibling to its next draw. The set of siblings comes from the parent's resolve, so this is the same everywhere.
   - Across the campaign, a clash is checked when a name is fixed, against the names already fixed in the same scope. Fixing is a chronicle event, and the chronicle is the order, so replay gives the same result.
   - After a set number of tries (64, as today) the object keeps only its designation, and the name card says why.
5. **No LLM in the path.** Orac writes a scheme, which is authored pack data with a hash. Names then come from the data.

### 4.5 Names that follow their parent

Alex's rule: if the star HD102 has planets "HD102 I" and "HD102 II" and the GM renames the star "Mystar", the planets become "Mystar I" and "Mystar II", and so on down through the moons.

- **A name made from a form that mentions its parent is kept as the form, not as finished text.** "{parent} {roman}" is stored as "my parent's name, then II". The text is worked out when shown, from the parent's name as it stands in the same layer, for that audience, at that date.
- So a rename anywhere flows down to every name beneath that follows it, **recursively**, at once and with nothing rewritten. "Mystar II" has moons "Mystar II a" and "Mystar II b" without anyone touching them.
- **A name the GM types is plain text and stays put.** Rename "Mystar II" to "Haven" and it stays "Haven" whatever happens to the star. Its own moons, which still follow, become "Haven a" and "Haven b".
- It works across layers the same way. In the Designation layer the planets read "HD102 I"; in the Common layer, where the star is "Mystar", they read "Mystar I". Where the parent has no name in a layer, the form uses the parent's shown name.
- The name card of a typed name offers "follow the parent again", which is the existing "restore original".
- "Where appropriate" is the scheme's choice per kind: planets, moons, belts and component stars follow by default in the Catalogue scheme; a peak or a town normally has its own word and does not. A form may also use `{parent.short}` or `{system}`.
- Fixing a name (4.4) stores its form and its own drawn parts, never the parent's text, so a fixed name still follows.
- History is kept: because each rename is dated, the ruler shows "HD102 II" before the renaming and "Mystar II" after.

### 4.6 Names are labels and never feed physics

- A scheme **may read** facts as words: kind, class label, tags, colour, position, the parent's name, the discovery record. That is how a descriptive name knows the world is red.
- **Nothing may read a name.** Names live in their own records, are not quantities, and cannot be wired into a flow; the loader refuses a flow input of name type.
- Acceptance, as for lenses: switch every scheme in a campaign and the physical state hash is unchanged.

### 4.7 Pronunciation, scripts and romanised forms

A name has up to four faces, all made together:

| Face | Example | Where it comes from |
|---|---|---|
| Spelling (romanised) | Khazrak | Each sound's spelling in the scheme. This is what charts show by default. |
| Say it | KHAZ-rak | Each sound's plain hint ("kh as in loch") and the scheme's stress rule. |
| IPA | /ˈxaz.rak/ | Each sound's IPA symbol. Science and Engine depth. |
| Native script | 天宫 for Tiangong | A list row's native form, or each sound's glyph where the script is in Unicode. |

- Real-language list rows carry their native script and an accepted romanised form (for Chinese, pinyin, with tone marks kept in the data and optional on the chart).
- A view chooses which face to show; a player view for an alien seat could show native script with the spelling beneath.
- **Invented scripts** (glyphs that are not in Unicode) are left for later (NM11), as stroke data in the pack, never as a font file.
- **Hearing a name** is also later. Browser speech does not take IPA reliably and differs by machine. A small synthesiser on the sound track could do it properly (decision 9).

### 4.8 The four schemes Alex named, and how each is built

All ship in one pack, `base-names`, beside an avoid list (4.9).

**A. Catalogue (designations).** Number and Pattern makers only, following real practice (sources in section 8, to be checked in NM0):
- Stars: survey prefix and number, or a coordinate name from sky position. Components of a multiple star take A, B, Aa, Ab, which matches the ids formation already uses.
- Planets: star name plus a Roman numeral by orbit ("HD102 I", "HD102 II"), the long-standing science-fiction and Traveller habit and the default (decision 6). The number is fixed once issued, so a later capture takes the next one.
- A second form in the same scheme follows real practice: a lower-case letter from b ("HD102 b"). Real practice letters by discovery order; where a survey record exists, that audience's layer letters by its own discoveries, and otherwise by orbit. The GM picks one form.
- Moons: a lower-case letter after a numeral ("HD102 II a"), a Roman numeral after a letter. Small bodies: number plus provisional code. Galaxies, clusters, nebulae and streams: prefix and number or coordinates.
- All of these follow their parent's name (4.5).
- Surface features: the IAU's descriptor terms, which today's patterns already half use (Mons, Planitia, Chasma, Rupes), with a themed List per world and kind, as the IAU does (one world's craters after one theme).
- Real stars keep their real names and designations from whichever sky catalogue pack is stacked (night sky ruling). Gaia designations appear only when the optional Gaia pack is stacked.

**B. English-sounding.** Built from real English place-name elements with their meanings (ash, kel, brook, ford, ton, wick, mere), joined by the rules English place names follow, plus short Lists (surnames of long-dead astronomers and explorers, plain nature words). "Ashford", "Kelmere", "Mount Harrow". It reads as English because it is built the way English names are, and the card can say what the parts mean.

**C. Many nations.** A Mix of per-language schemes, clumped by system. The rules that keep it respectful and plausible:
1. **Real languages use real words.** Each language's scheme is a List of real, checked words and names with meaning, native script, romanised form and source. No real language is ever made by joining syllables: that is exactly how caricature happens.
2. **Follow real naming practice.** The world already names things in space in many languages: the IAU's public naming of exoplanets by country, its approved star names (Arabic, Chinese, Hindi, Polynesian and others), and the names space agencies give their craft (Tiangong, Chandrayaan, Hayabusa). The lists draw on that kind of word: nature, sky, rivers, mountains, virtues, long-dead scientists and explorers, figures from old stories.
3. **Adopt the IAU's exclusions**: no living people, no political, military or religious figures of living significance, no commercial names.
4. **Where a language builds place names from parts** (a "new river", a "-pur", a "-yama"), that may be added per language only against a cited grammar or with a named reviewer who speaks it. Lists come first.
5. **Every row has a source and the pack invites correction** through the Explorers report route. The pack says plainly that it is a first pass by non-speakers until a row is reviewed, and marks reviewed languages.
6. **The mix of nations is data**, with presets ("today's spacefaring agencies", "by number of speakers", "even") and editable weights, because who went to the stars is the GM's setting, not ours to decide.

First languages: English, Mandarin, Hindi, Spanish, Arabic, Russian, Japanese, Swahili, Portuguese, French. More are added as lists, with no code.

**D. Alien languages.** Sounds, Pattern and (later) Meanings. Two original examples ship so the range is clear: one soft and flowing, one guttural with clicks and doubled stops, each with pronunciation hints. These are what Orac's scheme-maker copies the shape of. Invented languages from named settings (Tolkien's, Klingon and the like) are other people's work: per the standing ruling they are fan-made, free and unofficial on Explorers, never built in and never paid.

### 4.9 Avoiding rude words and real brands

- `base-names` carries an **avoid list**: offensive words in the major languages, slurs, and a short list of well-known brand names. A generated name is compared after folding case, stripping accents and mapping look-alike letters, as a whole word and as a part above a set length. A hit moves to the next draw.
- It applies to **generated names only**. The GM may type anything.
- The GM can add words. A universe pack can add its own list (a setting where "Kor" is a swear word).
- When a pack is loaded, its lists are run against the avoid list and the count is shown ("3 rows match your avoid list") before the GM accepts it.
- Honest limit: no list catches every word in every language or every trade mark. The promise is best effort plus a one-click rename, not a guarantee. It goes in the pack's "about".

### 4.10 Safe to load from strangers

Packs are data only and never contain code (handoff section 7). Naming adds text from strangers that will be drawn on screen and shown to players, so:

| Risk | Rule |
|---|---|
| A file built to hang the app | Size caps (a scheme under 1 MB, a list under 50,000 rows, a name under 64 characters). No regular expressions in schemes: "never occurs" rules are plain sound sequences. Bounded tries. The scheme declares a cost class like any operation. |
| Text that breaks or disguises the page | Names are always drawn as text, never as markup. Unicode is normalised (NFC); control and direction-override characters are refused; at most two combining marks per letter; an allow-list of character classes. |
| Look-alike names passing for something else | The fold used by the avoid list is also used for clash checks, so a look-alike of an existing name counts as a clash. |
| Fonts and images | None in schemes. Unicode text only in the first version. |
| A pack posing as a real catalogue | Every name's card shows the pack and layer it came from. Only the real-sky catalogue packs may mark a name "real". |
| Offensive content | The avoid-list count at load, and the Explorers site's existing report route. |

Hostile fixtures for each row are part of NM1's acceptance.

### 4.11 The file: plain text you can write by hand

Inside a pack a scheme is JSON like all pack data (`provides.names`). Alex asked for simple text files, so there is also a **name sheet**: a plain text form of one scheme that the app turns into a pack entry when dropped in, and writes back out without loss.

```
name:    Vesh
about:   Hive-builders of the Kestrel reach. Guttural, clicks, doubled stops.
sounds:
  C = k kh g gh q r z tk x
  V = a i u aa
  E = k z kh rr
shape:   CV CVE V CVC          # syllable shapes
length:  2-3
never:   aa a, kk, x x
say:     kh "ch as in loch", x "a tongue click", rr "rolled r", aa "long a"
stress:  first
means:   red = khaz, mountain = rak, water = tiz, home = uq, great = gaar
forms:
  planet     = {word}
  moon       = {word}-{parent.short}
  peak       = {means:colour?}{means:mountain}
  settlement = {word} {means:home}
```

A plain list (one name per line, optional tab-separated meaning) is also a valid name sheet: the simplest possible scheme. Canonical form is the JSON; the sheet is a second way in and out.

### 4.12 Where it lives in the interface

- **A Names page on the Reference screen**, in the "This setting" group beside Units and Calendars: all three are how this campaign writes things down, none touches physics, and each shows the pack it came from. No new tab and no new rail entry; it is a page of the Reference screen.
- **If the class atlas joins Reference as a "Names and classes" group** (open with Alex now), the Names page sits in that group beside the atlas, since both are "what we call things". **If it does not**, Names stays under "This setting". The page, its data and its packages are the same either way; only the chip it sits under moves. Nothing in NM1 to NM5 depends on the answer.
- **The page** has three parts: the campaign's defaults (Designation, Common, and a line per culture); the schemes on the stack, each with its pack; and a live sample for the chosen scheme (twelve names for a chosen kind, with "say it" beneath, and a button for another twelve, which simply shows other addresses). Depth does its usual work: Curious picks and listens, Science shows IPA and meanings, Engine opens the makers for editing.
- **Making one**: "New scheme" (a short form, or start from a copy), "Ask Orac", or drag a file onto the page.
- **On a system or world card**: a quiet "Names here" line showing what is inherited, which opens the same picker.
- **On any name**: click to type a new one, as now. A small mark opens the **name card**: every name the object has, by layer, with its say-it line, meaning, scheme and pack, who knows it, and "restore original". That is the "Why this name?" answer.
- **Phone and Night chart**: the page is a chip in the Reference row; the sample and the name card are sheets from the foot; colours come from the tokens only. Honesty marks follow the standing rule: "authored" on GM view only.
- A mockup (NM6a) is drawn alongside the kernel work. It guides NM6 and does not hold anything up.

### 4.13 Orac, and Explorers

- **Orac** gets one tool: make or change a name scheme. "Make me a naming scheme for a guttural insectoid species" returns a scheme through the same loader and checks as a dropped file, shows the sample panel, and saves to the campaign pack on the GM's yes. Minerva may suggest single names; accepting one is an ordinary authored rename.
- **Available from the store.** Explorers gets a Names shelf: a scheme travels alone as a small pack of kind `names`, found, previewed (its sample names) and added from the library like any pack.
- **Included in world and universe packs.** `provides.names` is an ordinary part of any pack, so a universe pack carries its own schemes, its naming settings (which scheme for which region, system or world) and any names its author typed. Stacking the pack is all the GM does; the setting's aliens arrive speaking their own language. A world or system shared on its own carries the schemes its names use, so it reads the same wherever it is opened.
- Standing rules apply unchanged: named-setting language packs are free fan work; paid packs must be original; real-language packs carry their sources.

### 4.14 What is included now and what is left for later

One engine, by kind. The difference is only which call sites are wired first.

| In the first version | Later, same engine, no engine change |
|---|---|
| Stars and systems, planets, moons, belts | Ships and stations |
| Surface features and regions (replacing today's generator) | People and crews |
| Settlements and bases from the built layer and universe packs | Species and guilds (structural names stay first; the scheme gives the common name) |
| Catalogue designations for sky objects (night sky Y7) | Polities and factions |
| Galaxies, clusters, nebulae, streams (designations) | Months and days for "make a calendar from this world" |
| | Events and epochs; invented scripts; hearing a name |

## 5. What changes in what is built

- `NamingSettings` in `base-surface` becomes the first scheme read through the new engine; the old shape still loads (a syllable list is a Sounds maker with one class). The default for features changes to the campaign's Common scheme, so **feature-name golden hashes change on purpose**, logged in the NM4 package log.
- The per-world used-set is replaced by the sibling rule (4.4), which no longer depends on generation order.
- `NameRecord`, `Rename` and the audience field move from the surface crate to the general name records, with the surface register reading them.
- Grown systems stop showing `Seed "x"` as their only name: they get a designation and a common name. The seed stays on the card.
- Night sky Y7 takes its designations from the Catalogue scheme instead of its own counter.

## 6. Standing rules this design keeps

One general mechanism (five makers, kinds as plain words, no specials for aliens or catalogues). Deterministic from seed, address and pinned pack. Labels never feed physics. Authored always wins and survives regeneration. Config for Orac first, people tweak. Packs are data and safe to load. Named-setting packs free and unofficial. Gaia only in its optional pack. Phone and Night chart from the start. No new tab. Do not copy V3.

## 7. Build packages

Owners are the existing threads; the coordinator may move them. Each is one branch and one pull request with an evidence packet.

| Id | Package | Owner | Needs | Acceptance |
|---|---|---|---|---|
| NM0 | **Sources and audit.** Check every source in section 8 and the licence of each name list (IAU lists, USGS gazetteer, any place-name source). Read what V3 does for names on AC-ProArt and add the row to `docs/v3-parity.md`. Work out clash odds for the shipped schemes at 10^4 and 10^6 names. | Build the master matter data sheet (citation checks) | none | each source confirmed or replaced, with licence; V3 row written; clash table in the package log |
| NM1 | **Engine.** Crate `sse-names`: scheme shape in `provides.names`, loader and checks, List, Sounds, Number, Pattern and Mix, keyed draws on their own stream, the fold, the avoid check, text safety and limits, cost class. | Build kernel resolve and rules | K2, K5 | 10,000 names identical natively and in the browser harness; changing an unrelated pack changes no name; adding or switching schemes leaves the physical state hash unchanged; every hostile fixture of 4.10 refused with a readable reason; old `NamingSettings` still loads |
| NM2 | **Name layers and settings.** General name records (layer, scheme, date, by, audiences, faces); settings on addresses with inheritance; fixed on promote; sibling and campaign clash rules; "rename from new scheme as of a date"; API calls to read names, set a setting and rename. | Build kernel resolve and rules | NM1, K9 | a three-seat fixture shows each seat only the names it knows; names read back the same after replay; an authored name survives regrowing its system; a setting on a world changes only unfixed names beneath it; renaming a star renames its following planets and their moons in every layer with no new events on them, while a typed child name stays and its own moons follow it |
| NM3a | **`base-names`: Catalogue scheme** and the avoid list. | Build the master matter data sheet | NM0, NM1 | Sol's bodies get designations that match real practice on a fixture; a 200-body system has no duplicate designation |
| NM3b | **English-sounding scheme.** | Build the master matter data sheet | NM1 | 1,000 samples: none on the avoid list, every part has a meaning on the card |
| NM3c | **Many nations.** Per-language lists with meaning, script, romanised form and source; presets for the mix; the reviewed mark. | Build the master matter data sheet | NM0, NM1 | every row has a source; a system's names come from one tradition with the set share of strays; pack "about" states review status per language |
| NM3d | **Two invented languages** with pronunciation. | Design thread writes them, matter thread loads them | NM1 | each sound has spelling, say-it and IPA; samples read as two clearly different languages |
| NM4 | **Call sites.** Surface features through the engine (golden change logged); stars, planets, moons and belts named on resolve; settlements from the built layer; Y7 designations. | Build SSE4 proven core (surface, formation), Build kernel resolve and rules (resolve), Build light and nebulae or the night sky owner (Y7) | NM2, NM3a | a grown system shows designation and common name; feature names come from the campaign scheme; changing the world's setting re-names unfixed features only |
| NM5 | **Learned and Meanings.** The example-word maker; shared meaning ids in `base-names`; descriptive forms that read facts. | Build kernel resolve and rules | NM1 | learned tables identical on every device; a red world's descriptive name uses the scheme's word for red and the card shows the parts |
| NM6a | **Mockup**: Names page, picker on a card, name card; desktop, phone, Night chart. | Design thread (Fable) | none | boards published; used by NM6, tweaked later |
| NM6 | **Names page on the Reference screen**, "Names here" on cards, the name card, drag-in of a name sheet, layer choice in lenses and player views. | Rebuild the chart room's top level | NM2, NM6a, MP10 | screenshots at desktop and phone in both modes; keyboard operable; a dropped sheet appears, samples, and can be set as default; a player view shows only known names |
| NM7 | **Name sheet** text form: read, write, round trip. | Build kernel resolve and rules | NM1 | every shipped scheme round-trips without loss; a bare list of names loads |
| NM8 | **Orac's scheme tool.** | Build Orac, Minerva and Tars | NM1, NM7 | "a guttural insectoid species" yields a scheme that passes the loader, with samples; nothing from the model reaches a name except through the saved pack |
| NM9 | **Explorers Names shelf**; pack kind `names`. | Integrate Explorers site with SSE4 | NM7, TG15 | a scheme uploads, lists, previews its samples, downloads and loads; a universe pack and a single shared system each carry their schemes and open with the same names; fan and paid rules enforced. Anything touching the site's database needs Alex's own yes first |
| NM10 | **Traveller pack naming style** (hex-number designations and the pack's own forms; no names copied from published sectors). | Build Traveller pack | NM3a | a subsector fixture is named in the pack's style |
| NM11 | **Later:** ships, people, species, polities, months, events; invented scripts as stroke data; hearing a name. One small package each when its owner reaches it. | various | NM2 | per package |

Order: NM0 and NM1 first; NM2 and NM3a next; then NM4 so the change is visible; NM3b to NM3d, NM6a and NM7 in parallel; NM6, NM5, NM8, NM9, NM10 after.

## 8. Sources to check in NM0

Listed from memory, as the standing rule requires them to be checked before anything relies on them.

- IAU, "Naming of astronomical objects" and "Naming of exoplanets" (public pages); IAU Working Group on Star Names, list of approved star names; IAU NameExoWorlds campaigns (names proposed by country, with meanings).
- Hessman and colleagues 2010, "On the naming convention used for multiple star systems and extrasolar planets", arXiv:1012.0707 (the b, c, d convention). Washington Multiplicity Catalog for component letters.
- USGS and IAU, Gazetteer of Planetary Nomenclature (planetarynames.wr.usgs.gov): descriptor terms, themes per body, and the rules on who may not be commemorated.
- Minor Planet Center, provisional designation rules.
- Unicode Standard Annex 15 (normalisation), Annex 9 (direction), Technical Standard 39 (look-alike characters).
- For the method behind Sounds (classes, shapes, weights): Rosenfelder, *The Language Construction Kit* (2010), as a guide to practice, not a source of data.
- English place-name elements: the English Place-Name Society's published element lists, licence to be checked.

No code is taken from any existing name generator; the engine is written fresh.

## 9. Decisions (22 Nov 2026)

Alex's word was to build it and tweak later, so these are made here and recorded as decisions. Each is data or a default, so each can be changed later without rework.

1. **Three layers on every object** (Designation, Common, Local); packs may add more.
2. **Settings inherit down the address tree** (universe, region, system, world, kind on a world); a polity or species brings its own scheme for the Local layer.
3. **Names follow their parent where the scheme says so**, recursively, and a typed name stays put (Alex, 16:25; section 4.5).
4. **Everything can be renamed by hand, always** (Alex, 16:25). A typed name wins in its layer and survives regeneration; the original can be restored.
5. **Canonical form is JSON in a pack; the plain-text name sheet is a second way in and out.** A bare list of names, one per line, also counts.
6. **Planet designations: Roman numerals by orbit** ("HD102 I", "HD102 II") as the default form, since that is the form Alex used, with real-practice letters from b ("HD102 b") as the second form in the same scheme. Moons take lower-case letters after a numeral and numerals after a letter.
7. **Many nations uses real words from lists only**, the IAU's exclusions, a source on every row, the ten starting languages of 4.8C, shipped marked "first pass, corrections welcome" with a reviewed mark per language.
8. **Many nations weights**: three presets (today's spacefaring agencies, by number of speakers, even), editable, default "even", so the pack makes no claim about who inherits the stars.
9. **Hearing a name spoken: later.** "Say it" and IPA now.
10. **Invented scripts: later**, as stroke data and never as font files. Real scripts work from the first version because they are ordinary text.
11. **An avoid list ships**, applied to generated names only, GM-editable, stated as best effort.
12. **Meanings** (names that mean something and read the world's facts) are in the first version, after the basics (NM5).
13. **Changing a scheme where names are in use asks once**: "leave them" (default) or "rename as of a date".
14. **The Names page sits under "This setting"** beside Units and Calendars. If the class atlas joins Reference as "Names and classes", the page moves there with it. No build work depends on which.
15. **Available from the store and inside world and universe packs** (Alex, 16:26; section 4.13, NM9).

Nothing here is hard to undo. The one thing that needs Alex's own yes later is any change to the Explorers site's database in NM9, under the standing rule for that site.
