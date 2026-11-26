# SSE4: real catalogues, the real Milky Way, and the Detail control (track RW)

Status: **PROPOSED, 26 Nov 2026. Not accepted.** Design only; nothing here is built by this note. The questions for Alex are in section 12, each with a recommendation. RW is a code only, never shown in the product.

Written by the thread "Design the real star catalogue import" (Fable, design only) from Alex's messages of 26 Nov 2026, 17:25 to 17:29 UTC:

- "the star catalogue importer to bring in REAL stuff. How do we do that? We need a default 'milky way' setup into which someone can import a 'section of it'. eg: user picks a star and 20ly around - the catalogue fetches and you place it 'in the right place' - our default milky way (real) will start with earth now."
- "We need a gloriously overmodelled earth system with EVERY known rock catalogued with a master slider for the user to use to say 'don't show below this size'... so we can have the classic 9 bodies and moons as the default but if you wanna tip that slider up (and your machine can cope) you will get 1000s of asteroids trojans, etc. Maybe we can do more than a mass filter. Again general systems for general purpose."
- "the detail slider will be a permanent fixture to scale data to machine capability - we can find the set points for our data - also doing this gives us an excellent data set to compare 'fake solar systems' - do they have as much interest and setup?"
- "we have a set point at 'the stuff people expect to see in a solar system sim'... and see what peeps do when they realise they have added 10000 asteroids and hundreds of trojans and a dozen planet X candidates we keep finding out in the dark. So the 'size' makes sense."
- "we don't even READ the data we won't use so always have them indexable by size to make it fast."

Read with: `SSE4-night-sky-design.md` section 8 (the real-sky pack, accepted 21 Nov), `SSE4-design-handoff.md` section 10 (real data fetched on demand), `SSE4-galaxy-design.md` section 10 (authored galaxies), `SSE4-grand-zoom-design.md` section 6 (the tie-up of places), `SSE4-conserved-chain-design.md` sections 3, 10 and 11 and `SSE4-conserved-chain-splits.md` sections 4 to 6A, `SSE4-explorers-paid-packs.md` section 8 (Gaia and money).

**Figures.** The paper search is out until 1 Dec 2026 and the cloud cannot reach any catalogue host (section 2.5). Every count, size and licence line marked **[unverified]** is from memory and must be read at its source before a builder relies on it. Package RW1 does that.

---

## 1. The short answer

1. **Most of the thinking for real stars is already accepted and none of it is built.** The night sky note chose the catalogues, the "pins claim first, the fill is thinned by the catalogue's completeness" rule and a real-sky pack (stage Y8). The galaxy note says a universe pack can pin a galaxy. V3 has a working region importer (SIMBAD, the NASA Exoplanet Archive, Gaia), copied into the converter. What is missing is the mechanism that joins them, and that is what this note designs.
2. **One mechanism does all three of Alex's asks: a catalogue of measured things that claim seeded slots.** A real star, a real asteroid, a real galaxy and a Traveller world are the same kind of record: an entry with measured facts, a place in the catalogue's own frame, and an amount. Placing it means it takes over the generated slot that fits it, and that slot's share of the budget. It replaces; it never adds. Section 3.
3. **The real Milky Way is a universe pack on the ordinary machinery.** Its home galaxy is chosen from the cosmic web like any other (GZ12), and the pack then states what is measured about it: the Sun's place, the arms (Reid 2019), the neighbours. The Solar System is its home system, at today's date. Section 4.
4. **The importer is "pick a star, pick a radius, preview, apply".** The result is kept with the campaign, so it never needs the network again and every device agrees. Section 5.
5. **Every population is a list ranked by size, biggest first, and the Detail control is a cut on that list.** That holds for a real belt, a generated belt, the stars near home and the galaxies nearby. Data below the cut is never read, because the files are stored in size order. Sections 6 and 7.
6. **The fully catalogued Solar System becomes the yardstick for generated systems.** One census read works on both, and the differences become a list of processes the generator lacks. Section 9.

---

## 2. What exists today (checked on main at 7e01502, 26 Nov 2026)

### 2.1 A default real Milky Way that starts at Earth, now

| Piece | State |
|---|---|
| The catalogued Solar System, `packs/base-sol` (SOL1 to SOL5) | **Built.** 36 bodies: the Sun, eight planets, Pluto, Ceres and 25 moons, each value cited. 12 regions: the main belt, Kuiper belt and Oort cloud as edges only, and nine rings. "Open the Solar System" on the start sheet reads it |
| Where the bodies are today | **Not in base-sol.** It carries size, mass, semi-major axis, eccentricity and inclination, and no node, perihelion or position at a date. A separate Sol fixture in the nav crate has full J2000 elements. So "Earth, now" is not yet true of the pack people open |
| Amounts for the belts | **Missing.** The three belts are edges with no mass. That breaks "no join passes a description without an amount" for the real system |
| The Solar System's tilt in the galaxy | **Built** (`system.frame.*` in base-sol, measured) |
| The Solar System's place in the galaxy | **Not built.** Opened from the start sheet it has no star slot and no cell (grand zoom note 6.2). GZ13 gives every system a slot; not started |
| A home galaxy chosen from the web | GZ12, in the kernel's queue, reported as landing today. "Find a home" already has a cited "Like the Milky Way" wish (halo mass, a quiet history, a bound big neighbour) |
| Measured arms, Local Arm included (Reid 2019) | Work in progress on branch `bvlyq1`, no PR; last in the agreed golden order |
| A galaxy pinned by a pack (galaxy design 10) | **Designed, not built.** No pack states measured facts about a galaxy today |
| A Milky Way universe pack | **Does not exist.** The three shipped universe packs are fictional |

### 2.2 A star catalogue importer

| Piece | State |
|---|---|
| The design: which catalogues, pins and thinning, a real-sky pack, tiles through our own Worker | **Accepted 21 Nov** (night sky note section 8, stage Y8). Not started |
| Y8's foundations: the count tree, `sky_from_here`, the glow map, promotion that keeps a star's description | **Built** (Y1, Y2, Y3, Y7a) |
| Pins in the star field | **Built for drawn stars only.** `promote` makes a drawn star permanent. There is no way to pin a star that came from outside with a measured mass, place and velocity |
| V3's real-sky importer (3,094 lines, copied into the converter at `src/lib/vendor/import/realsky/`) | **Working in V3.** It queries SIMBAD, the NASA Exoplanet Archive and Gaia live, and ships a snapshot to 12.7 pc. It writes V3 systems on a flat map, with no galaxy underneath |
| A data service | **Not built** (build plan X5). The site Worker has one fetch route, `/tmap/*` for Traveller sectors, cached a day. There is no storage bucket |
| The Traveller sector reader (`sse-sector-io`, op `import_sector`) | **Built.** It is the nearest thing to a general importer: preview first, "never invent", every unplaced token a warning. Its worlds are not yet tied to stars under their hexes |
| The tracked set and the neighbourhood stream (chain note 10 and 11) | Accepted, waits on CC6. Held until then: the Local stars rung and anything fixed to particular generated stars |

### 2.3 The overmodelled Solar System and a detail slider

| Piece | State |
|---|---|
| Every known small body | **None.** Ceres is the only asteroid in base-sol. The store pack `sol-today` holds 21 memorial asteroids with orbits from JPL Horizons |
| Generated belts as ranked lists | **Built.** `Cloud::resolve(seed, k)` gives the k-th largest member of any belt, cloud or ring; `sse-small` says what it is. This is already "indexable by size" |
| Members on screen | **Four per belt.** System True draws the 1st, 10th, 100th and 1000th largest (SB3b, a registered stand-in). Nothing draws a crowd |
| The roster (chain note 6A) | **Proven in a scratch prototype, not merged.** Named bodies down to 1 km; a read costs about 2.5 microseconds; 7.7 MB of memory |
| A detail slider or any size filter | **Does not exist.** The words are taken twice already and mean other things: detail levels (Glance, Orbit, Survey, Site) are how deeply one object is worked out, and the quality tiers (Light, Full, Ultra) are how finely a frame is drawn. Neither chooses how many objects there are |
| Machine capability | **Built.** The speed probe places a device on a tier by measured speed, with a heat watch and a governor (VP4) |
| Filters | **Half built.** Sifters and find-by-tag exist for resolved objects. Nothing filters a population |

### 2.4 What generation makes that the real system has

Read from the code, not run: formation makes belts, debris from shattered worlds, bared cores, trojan swarms, scattered discs, Oort clouds, captured and irregular moons, and resonant chains of planets. I found no code for collisional families, gaps cleared by a giant's resonances (Kirkwood gaps), the Hilda group, centaurs as a population, or near-star-crossing bodies as a group. Section 9 turns this into a measured list.

### 2.5 The network

From the cloud build machine every catalogue host refuses the connection: JPL's small-body database and Horizons, the Minor Planet Center, CDS (SIMBAD, VizieR), the NASA Exoplanet Archive, ESA's Gaia archive and Codeberg (tried 26 Nov, all refused by the proxy). So a builder in the cloud cannot download a catalogue. Catalogue data has to be built on Alex's PC, or fetched through the built-in browser there as the data sheet thread does, and uploaded.

---

## 3. The one mechanism: a catalogue of claims

### 3.1 What a catalogue is

A **catalogue** is data, in a pack or fetched. It is the same thing whether it lists real stars, real asteroids, real galaxies, a fan setting's invented stars or a Traveller sector.

| Part | What it says |
|---|---|
| **Level** | What kind of slot its entries claim: galaxy, star, planet or moon, small body |
| **Frame and anchor** | The frame its positions are in, and the one entry that everything else is placed from. For real stars the anchor is the Sun |
| **Date** | When the positions were true. Entries carry velocities or orbits so they can be moved to the campaign's date |
| **Entries, sorted by size, largest first** | Each has a stable id, names, a position or an orbit, a size, and whatever else was measured, every value with its source and a grade (measured, estimated, convention) |
| **Completeness** | How much of what exists the catalogue holds, as a function of size and distance. Outside its completeness the seed fills in |
| **Terms** | Licence, credit line, version. Shown wherever the data is |

Two rules make it general.

- **Size is the first key, always.** "Size" is the level's natural measure: mass for galaxies and stars, diameter for worlds and small bodies. A catalogue file is stored in size order with an index of where each size step starts, so a reader takes a prefix of the file and stops. Data below the cut is never fetched, parsed or held in memory. This is Alex's 17:29 rule and it is part of the format, not a courtesy of the reader.
- **Never invent.** The reader keeps what was written and warns about what it cannot place, as the sector reader does. Anything the catalogue does not measure is worked out afterwards, around the measured facts, and tagged as fill (already ruled, handoff section 10).

### 3.2 What a claim is

Every level already has seeded slots with shares: haloes in the web, star slots in a cell's count tree, ranks in a belt. **Placing a catalogue entry means it claims the slot that fits it best, and the measured facts replace the drawn ones.** The slot's share of the parent's budget is spent once, by the entry. This is the pin of the design handoff ("pins claim from the budget first"), widened so that a pin can come from outside.

| Level | The slot | What "fits best" means |
|---|---|---|
| Galaxy | A catalogued halo of the web, or a satellite in the home halo's tree | Nearest in mass and distance from home |
| Star | A slot in the count tree of the 10 pc cell that holds its measured place | Same luminosity class in that cell; the entry keeps its own exact position |
| Planet or moon | A pin in the system's formation request (the route the Traveller pack already uses to grow a whole system round a coded world) | The measured orbit and mass win; formation fills the rest round them |
| Small body | A rank in its belt's size list | Its own size decides its rank |

This is the same rule GZ13 already uses for imported systems (by map position, or the best-fitting drawn star). It is now one rule at every level.

### 3.3 No double counting, and no description without an amount

**Every entry must carry an amount before it can claim.** A measured mass is used. Where there is none, the mass is worked out from what was measured by the kernel's own physics and graded "estimated": a star's mass from its brightness and colour through the star model, an asteroid's from its diameter and a density for its makeup. Its six element groups come from a measured metal content where there is one, and otherwise from its parent's stock at that place. An entry with nothing to estimate from is refused with a reason and listed in the preview.

**Then the books, level by level:**

- **Stars.** A cell's count tree gives a count for each luminosity class. Catalogue stars claim first. The fill is what remains, never less than zero, and it is thinned by the catalogue's completeness, so inside 25 pc of the Sun almost nothing is fill and at the catalogue's edge the fill takes over smoothly (night sky note 8.4). Where the real sky holds more stars of a class than the cell's seeded count (the real neighbourhood is lumpy), the measured stars stand and the excess is taken from the remainder of the wider block the cell was split from. So the books close at the block, not the cell. This matches what the chain note already accepts for the home bubble: it is a window on the ring's stock and mass closes on average.
- **Small bodies.** A belt has a mass by element group. Catalogued members claim from it, biggest first. The seeded tail is the remainder, and it only supplies sizes below the catalogue's completeness limit. Where a real belt's members would add up to more than the stated belt mass, the belt mass is wrong and is corrected at its source; nothing is tuned.
- **Galaxies.** A named neighbour takes a halo or satellite that the web already made. Its measured mass replaces the drawn one and the difference is recorded on the group's line.

**Three tests at every claim join**, as the standing rule asks, each first run without the fix and seen to fail: closure (claims plus remainder equal the parent's share, at the level named above), indifference (the same import gives the same universe whatever was looked at first, on any device), and no stand-in place (every claimed entry has a real slot; none sits at "the Sun's radius, azimuth 0").

### 3.4 How it sits with the tracked set and the campaign date

The chain note anchors the star field at the campaign's date: the stars in the home bubble (10 pc by default) are promoted to the tracked set and are the same stars at every date. With a real catalogue stacked, **the tracked set is the catalogue's stars in the bubble plus whatever fill survives thinning.** A measured star's position, proper motion and line-of-sight speed become the closed-form orbit that `stars.rs` already gives every permanent star, so it moves correctly at every date. Where the line-of-sight speed was never measured (common for faint stars) it is drawn from the ring's spread of speeds at that place and tagged fill.

An import wider than the bubble makes more pins, at the same small cost per star as a tracked star (its record and an orbit, no system formed until someone goes there, as ruled: cataloguing does not form the system).

Stream arrivals that would overlap the anchor date are already dropped in favour of the tracked set, so a real star is never met twice.

**Order with the chain.** Claims on star slots change which stars exist near home, so the star half of this track lands after CC6 (the one change to the star field) and GZ13 (slots). Everything inside a system is on the chain note's "safe to carry on" list and can start now.

### 3.5 Looking, deeds and travel

Stacking a catalogue pack is part of the universe's recipe, like the seed: two devices with the same seed and the same pack versions have the same universe. An import fetched live is a **deed**: its entries are written to the campaign record with their source and version, they travel with the campaign, and a postcard from inside an imported region names the catalogue it needs or carries the few entries on its own chain. Nothing about looking changes.

---

## 4. The real Milky Way

### 4.1 What it is

A universe pack, working name "The real sky". It holds no engine code and no special case. It states:

1. **A home wish**, the cited "Like the Milky Way" wish that exists today. When the universe opens, GZ12 picks the web's best-fitting halo as home. So the real galaxy sits inside the web like any other, with a real halo, a merger tree and a budget.
2. **Measured facts about that galaxy, as claims.** The disc's scale lengths, the bar, the arms as a measured list (Reid et al. 2019, the Local Arm included, from the Local Arm thread's work), the Sun's distance from the centre and height above the plane. The measured structure replaces the drawn structure; the halo's mass budget is what the web gave.
3. **A home system: the Solar System**, claiming a star slot at the Sun's measured place. The azimuth is fixed by the measured arms, so the Local Arm, Sagittarius and Perseus lie where they are measured from here. Base-sol's measured tilt already says how the planets' plane sits in the galaxy.
4. **A date: now.** The campaign's defined date is the day the universe is made, read once from the device and stored. After that it is a number like any other, so the kernel stays free of the clock.
5. **Which catalogues apply and at which versions** (section 5), and the named neighbours: the Magellanic Clouds, Sagittarius, Andromeda, Triangulum and the Local Group's dwarfs, as galaxy-level claims on the satellites and neighbours the web already has.

A fictional universe pack is the same five things with invented data. Nothing in the kernel knows the word "Milky Way".

### 4.2 The honest limits

- **The real galaxy sits in a seeded universe.** Beyond the Local Group the web is generated from the seed. Virgo is not where Virgo is. Reproducing the real large-scale structure would need a constrained reconstruction of the local universe, which is research code and too far. The fidelity label says "measured" out to the Local Group and "generated" beyond.
- **The neighbours' histories are the web's, not the real ones.** Andromeda claims the bound big neighbour the wish asked for. Its mass and place become the measured ones; its merger history stays as generated unless the pack authors events (galaxy design 10 allows it: the ancient merger, Sagittarius falling in).
- **The galaxy's own past is a reconstruction.** Measured structure is today's. The rings' history of star formation and chemistry runs as modelled, and after the CC3 retune it is checked against the measured Milky Way.
- **A real star's birth is not known.** Its age comes from the catalogue where measured and otherwise from the ring's history at its place, tagged fill.

### 4.3 Is it the default?

The night sky note ruled fictional galaxies the default. Alex's message now says "our default milky way (real) will start with earth now". I read that as: the first thing a new user gets is the real sky at Earth today, and a grown universe is the other door. That reverses one line of an accepted note, so it is question 1.

The start sheet's "Open the Solar System" door becomes the way into the real sky pack: the same door, now with a galaxy round it. No new door is added.

---

## 5. The importer

### 5.1 What the GM does

On the Build desk, from a star's own menu or the Local stars view: **"Bring in real stars round here"**. Pick the centre (a star by name; the lookup is SIMBAD's), pick a radius (Alex's example, 20 light years), and read the preview before anything changes:

- how many stars, brown dwarfs, known planets and multiple systems;
- how complete the catalogue is out to that radius, and how many more stars the fill will add;
- anything refused, with the reason;
- the credit line and terms;
- the download size.

Apply writes the claims. Nothing here is a new screen: it is an action with a preview, the same shape as the sector import.

### 5.2 Any centre, any galaxy

- **Any real centre.** A sphere round Tau Ceti or the Pleiades is the same query. Completeness is measured from the Sun, so a region far from home is thinner in real stars and richer in fill, and the preview says so.
- **Any galaxy.** A catalogue is placed by its anchor. In the real sky pack the anchor (the Sun) sits at the Sun's slot. In a fictional galaxy the GM picks which star slot the anchor lands on, and the real neighbourhood arrives round it, turned to that galaxy's plane. The books work the same way: the entries claim that galaxy's slots.
- **Any catalogue.** A fan setting's star list, with invented stars, is a catalogue with a different anchor. A Traveller sector is a catalogue whose frame is a hex grid. Both can be laid into the real galaxy at a chosen place. This closes the Traveller thread's open item "imported worlds tied to the stars under their hexes" by the general rule instead of a special one.

### 5.3 Which catalogues

This keeps the accepted list (night sky 8.3) and the ruling that Gaia-derived data lives only in an optional pack. Three ways for data to arrive, one reader for all three.

| Way | What | Why |
|---|---|---|
| **Shipped in the app** | The Solar System down to the "Expected" stop (section 7). A small set of bright and named stars built only from sources with open terms: the Hipparcos and Yale Bright Star catalogues and the IAU's list of star names **[terms and sizes unverified; about 9,000 stars to the naked-eye limit, a few hundred kilobytes, estimate]** | The real sky works offline on day one, and the app stays Apache-2.0 with no non-commercial data in it |
| **The free real-sky pack**, fetched once from SSE4's own storage | The complete nearby sample: CNS5 (5,931 objects within 25 pc **[count from the night sky note, checked there 21 Nov]**). Doubles from the Washington Double Star Catalog. Known planets from the NASA Exoplanet Archive. Nearby galaxies and named deep-sky objects. Optionally the Gaia Catalogue of Nearby Stars to 100 pc (331,312 objects) as a second, larger download | Complete where a campaign lives. Carries its licences. Never inside a paid pack (paid packs note, ruled) |
| **Live, on request** | Anything further or fainter, through the site Worker to SIMBAD and the Gaia archive, the route V3's importer already uses. Cached by region and version | No limit on reach; nothing downloaded that was not asked for |

Alex's 20 light year example is about 6 pc. The whole 10 pc sample is a few hundred systems **[about 340 systems and 540 objects, from memory, unverified]**, a download of tens of kilobytes. A 100 pc import is about 330,000 objects and a few megabytes.

**Terms, as far as they are known.** Gaia data is CC BY-NC 3.0 IGO (read at ESA's page on 21 Nov by the night sky thread). CNS5 and AT-HYG are built partly from Gaia and are treated as non-commercial until read at source. JPL and NASA data is not copyrighted **[unverified at source]**; the Minor Planet Center's terms need reading. RW1 reads every one at its source before any data is bundled, and nothing ships on my memory of a licence.

**Versions.** Each catalogue's version is pinned in the campaign, as the handoff ruled. Moving to a newer release is a deliberate act with a "what changed" list.

### 5.4 The data service

A storage bucket (Cloudflare R2) behind new Worker routes, `/cat/*`, serving catalogue files and caching live queries. Files are immutable and named by catalogue and version. Each is in size order with a small index, so a device asks for a byte range and stops (section 3.1). Star files are also cut by region (HEALPix cells by distance shell, the grid `sse-sky` already has), so a 20 light year import reads one small tile.

Build scripts live in the repo and run on Alex's PC, because the cloud cannot reach the sources (section 2.5). Creating the bucket is Alex's to do in Cloudflare (question 8).

Offline, a device uses what it has cached, and the edge of the real data is shown, as ruled.

---

## 6. Populations: everything is a ranked list

A **population** is any crowd of like things under one parent: a belt's members, a planet's moons, the trojans of a giant, the comets of a cloud, the stars of a cell, the satellites of a galaxy. The design gives every population one shape:

- **members ranked by size, biggest first**;
- rank k is either **catalogued** (measured or authored, a claim) or **seeded** (worked out from the parent's seed and its rank, as `Cloud::resolve` does today);
- above the catalogue's completeness limit every rank is catalogued, below it the seeded tail carries on, and the join is thinned so nothing is counted twice;
- a **table read** returns a block of ranks at once as plain columns (orbit, size, kind, a few flags), a few dozen bytes a member, without resolving any of them. Opening a member resolves it properly, as now ("best calc is no calc").

For a generated system this is what exists, plus the table read. For the real Solar System the top of every list is the catalogue. The same code draws both, filters both and counts both. That is the general system Alex asked for: the real system is not a special case, it is a system whose lists happen to be measured near the top.

### 6.1 The Solar System, every known body

Sources and rough counts, all **[unverified, from memory]**:

| Population | Source | Known today |
|---|---|---|
| Planets and dwarf planets | NSSDCA, as base-sol now | 8, and 5 recognised dwarf planets |
| Moons | JPL's satellite tables | over 400, most of them small irregular moons of Jupiter and Saturn |
| Main belt, Hildas, Jupiter trojans, near-Earth objects, centaurs | Minor Planet Center orbit file, JPL small-body database | about 1.4 to 1.5 million bodies in all; about 15,000 Jupiter trojans; about 40,000 near-Earth objects |
| Beyond Neptune, the distant detached bodies included | the same | about 5,000 |
| Comets | the same | about 4,000 to 5,000 |
| Hypothetical bodies (a "Planet Nine") | published candidate orbits | a handful; off by default, tagged hypothetical, switched on by the GM (the star census note's rule for hypotheticals) |

**Sizes.** Only a minority of small bodies have a measured diameter (roughly one in ten, mostly from infrared surveys **[unverified]**). The rest have a brightness. The standard relation gives a diameter from brightness and albedo; where the albedo is unmeasured it comes from the body's type or its region's mean. Those sizes are graded "estimated" and the card says so. Masses come from diameter and a density for the makeup, also estimated. Without this the slider would hide nine tenths of the catalogue, so estimated sizes take part (question 7).

**Size on disc.** About 30 bytes a body packed (six orbit numbers, size, kind, flags), so the whole catalogue is about 45 MB before compression **[estimate]**. Because it is in size order, the app ships only the top, and each further stop is its own range of the file:

| Down to | Bodies, about **[unverified]** | Download, about |
|---|---|---|
| the Expected stop | 50 to 100 | in the app |
| 100 km | 1,000 | 30 kB |
| 30 km | 5,000 | 150 kB |
| 10 km | 15,000 to 20,000 | 0.5 MB |
| 3 km | 150,000 | 4 MB |
| 1 km | 800,000 | 25 MB |
| everything known | 1.5 million | 45 MB |

Alex's "10000 asteroids and hundreds of trojans" is the 10 km stop. RW2 measures the real numbers and they replace this table.

**What base-sol itself must gain** (it is today's gap, section 2.1): full orbits at a stated date for every body, so that today's positions are right; mass by element group for each belt, so the real belts have amounts; and the kinds of group (trojans, Hildas, centaurs, families) as populations with their own lists.

**Motion and its honest limit.** A catalogue orbit is true at its date. Moved forward as a plain ellipse it stays good for decades and drifts over centuries as the planets tug it. The view says "as catalogued" near the date and "plausible" far from it. Over millions of years the real small bodies are today's snapshot and their past is the reconstructed history the chain note already plans for imported systems (CC12).

### 6.2 Other real systems

A real star with known planets gets them as pins in its formation request; formation fills the rest and tags it fill. This is the route X3b built for Traveller worlds ("codes win, impossible values marked with reasons"). No new mechanism.

---

## 7. The Detail control

### 7.1 What it is

**One control, on every view, that sets the smallest thing shown.** It is a permanent fixture of the frame (Alex, 17:26), a sixth view control beside View style, Seen from, Scale, HUD and Band, carried by the same `ViewControls`, the same for every seat, and remembered per device as the aids are.

It reads in the level's own measure and always says what it costs:

> Detail: down to 10 km · 17,204 bodies

| Level | "Size" means | Low end | High end |
|---|---|---|---|
| Cosmic web | halo mass | the great clusters | every catalogued halo |
| Galaxies nearby | galaxy mass | the big neighbours | every dwarf |
| Local stars | star mass | the stars the eye would see | every red and brown dwarf |
| System | diameter | the Expected stop | every known or seeded body |
| A world | diameter of its moons and ring members | the major moons | every moonlet |

It is a different thing from the two controls whose names are near it, and the note for builders is: Detail chooses **how many**; the detail levels (Glance to Site) choose **how deeply one thing is worked out**; Quality chooses **how finely it is drawn**. The word shown to the user should be settled once by the three-desks and chart room threads; "Detail" is the working name.

### 7.2 Set points

The slider moves continuously in size, with named stops it snaps to. The stops are found from our own data (Alex, 17:26), not chosen by taste: RW2 reads the size lists of the real system and of a thousand generated ones and puts a stop wherever the count crosses a round number a device tier can hold.

One stop is named and is the default everywhere: **Expected**, "the stuff people expect to see in a solar system sim" (Alex, 17:28). For a system it is the star, the planets, the dwarf planets and the major moons. Its rule has to work on a generated system too, so it is a rule and not a list:

- everything above a size at which a body pulls itself round (a label read off the result, about 400 km for ice and more for rock **[unverified]**), and
- **notables, whatever their size** (below).

### 7.3 Always shown

Some things pass the cut whatever the slider says, because hiding them would be a lie or a nuisance:

- anything the GM or players named, pinned, visited, built on or put in the record;
- anything with a construct on or round it;
- notables: in a generated system, what events made (a bared core, a captured moon); in a catalogue, entries flagged notable by the pack (Phobos and Deimos, Halley's comet, anything a spacecraft has visited);
- the subject of the view and whatever is selected.

### 7.4 The machine

The speed probe already places a device on a tier. It gains one more measurement, how many moving points the device can draw at a steady frame rate, and that sets a **ceiling mark** on the slider. Below the mark the slider is free. Above it the user may still go, and is told what to expect ("this device may slow down: 800,000 bodies"), because the interface suggests and never refuses. The governor may step the cut back to the mark if the frame rate collapses, and says that it did. On the text tier the control reads as a count and a list.

Expected first-guess ceilings, to be replaced by RW2's measurements: about 20,000 points on a phone and a few hundred thousand to over a million on a desktop GPU. The night sky note's star budgets (20,000 and 200,000) are the same idea and move under this one control.

### 7.5 Nothing below the cut is read

The chain from slider to screen never touches what it will not show:

1. The slider's size becomes a rank through the population's index (a handful of numbers per population).
2. Only that prefix of each file is fetched, by byte range, and kept as flat columns in memory. A member is never an object until someone opens it.
3. The columns go to the GPU once. Each point's place on its orbit is worked out in the shader from its orbit numbers and the clock, so a million bodies move with no work per body on the processor.
4. A press finds the nearest point and asks the kernel to resolve that one member in full.

For a seeded population step 2 is replaced by the table read, which works out the same columns from the seed, biggest first, and stops at the cut.

The shader's orbit sum is single precision and for display only: the kernel remains the truth for any body opened, and the stand-in goes in the fudge register.

---

## 8. Filters: more than size

Alex: "Maybe we can do more than a mass filter." Size stays the master control because it is the one that bounds the machine. **Filters choose which, the slider chooses how many.** A filter is a sifter over the columns every population already carries, shown as chips beside the Detail control, each a general rule and none special to the Solar System:

| Filter | Reads | Example |
|---|---|---|
| Group | where and how it orbits: near the star, a belt, sharing a planet's orbit, crossing a planet's path, beyond the planets, a comet | "only the trojans" |
| Family | which break-up it came from | the pieces of one shattered world |
| Makeup | metal, rock, carbon, ice (the type is a label read off this) | "metal-rich bodies over 1 km", for a prospector |
| Orbit | size, stretch and tilt of the orbit | everything that comes inside the home world's path |
| Hazard | how close its path comes to a chosen world | what could hit us |
| Known | discovered by a date, or known to an audience | the sky as of 1850; what the players' sensors have found |
| Visited | has a flight log or a construct | everything a probe has been to |
| Interesting | the chain note's "What is interesting here" list | the rare things first |

"Known by a date" is worth a word. A catalogue entry carries its discovery date, so a campaign set in 1900 can show players the 450 asteroids known then, while the GM sees all of them. It is the secrets design's "Known to" eye applied to a catalogue, with no new machinery (question 9).

---

## 9. The Solar System as the yardstick

Alex, 17:26: the catalogued system is "an excellent data set to compare 'fake solar systems' - do they have as much interest and setup?"

### 9.1 One census for any system

A kernel read, `census`, returns the same table for a catalogued system and a generated one:

- bodies per size step, by group (planets, moons per planet, each belt, trojans, comets);
- mass per population by element group;
- structure: resonant pairs and chains, gaps, families, rings, captured and irregular moons, pairs and contact binaries;
- notables, each with its reason.

Because it reads ranked lists, the census of a million-body belt costs the index, not the members.

### 9.2 What it is used for

- **For the GM**, on any system's card on the Explore desk: "Against the Solar System", the two censuses side by side in plain words. "Three times the belt mass. No moons larger than 500 km. A debris family the Solar System has no match for."
- **For the builders**, a report over a thousand seeded Sun-like systems against the real one: where the generator's systems are routinely poorer, and what they never make at all. From the code read in section 2.4 I expect the first list to say: no collisional families, no resonance gaps in belts, no Hildas, no centaurs, too few irregular moons. Each line names a missing process, and that list is the generator's backlog.
- **For the slider**, the same counts are where the set points come from.

### 9.3 Two cautions, which go in the report itself

- **The real catalogue is what has been found, not what is there.** Small and distant bodies are badly undercounted. The comparison is made only above each population's completeness limit.
- **One system is not a target.** The Solar System may be unusual. Alex's standing rule applies: fix the cause, do not tune numbers to pass a check. The yardstick finds missing physics; it is never a set of numbers to match.

---

## 10. Speed

| Thing | Cost | State of the figure |
|---|---|---|
| Tracked set, 10 pc bubble | 12 ms | measured (CC1) |
| A 25 pc import: about 6,000 pins | under 0.2 s if a pin costs what a tracked star does | estimate; RW2 times it |
| A 100 pc import: about 330,000 pins | tens of seconds and about 0.75 GB of save at today's 2.3 kB a star, so it needs a compact pin record (a few dozen bytes) or it is kept as a table and pinned only when opened | estimate. I recommend the table: a catalogue star far from home stays a row until someone looks |
| Roster read | 2.5 microseconds | measured (CC4) |
| Table read of 100,000 seeded members | about 0.1 to 0.3 s once, then cached | estimate from the prototype's 64 ms for one class |
| Drawing 1 million moving points | fine on a desktop GPU, too many for a phone | expectation, not a measurement; RW2 |
| Memory for 1.5 million bodies as columns | about 45 MB | estimate |

Phones: the Expected stop is the default and costs what the app costs today. Every stop above it is a choice, behind the ceiling mark.

---

## 11. Packages, in build order

Each is sized for one agent. "Now" means nothing blocks it. The star half waits for GZ13 and CC6, as section 3.4 says; the system half does not.

| Package | What | Needs | Owner (proposed) |
|---|---|---|---|
| **RW0** | This note, the status file, the row in START-HERE | | this thread (done with this PR) |
| **RW1** | Read every catalogue's terms, size and count at its source (through the browser on Alex's PC; papers after 1 Dec). Replace every **[unverified]** in this note. A terms table in `docs/citations.md` | now | citations thread |
| **RW2** | Measure, merge nothing: the real size lists and counts per stop; points a phone and AC-ProArt can draw moving; the cost of a pin and of a table row; a 50 pc tracked set. Output: the set points and ceilings | now; Alex's PC for the GPU | quality tiers, with small bodies |
| **RW3** | The catalogue format `sse.catalogue/1` and its reader crate: entries in size order with a range index, frame, anchor, date, completeness, grades, terms. "Never invent." Contract types. Tests: a prefix read never touches the rest of the file | now | kernel |
| **RW4** | Populations: the table read (`population`: a block of ranks as columns) for seeded belts, clouds, rings and moons, and for catalogued ones through RW3, joined with thinning. Own golden area | RW3 | small bodies, with the kernel |
| **RW5** | The Detail control: `ViewControls.detail`, stops as data, the ceiling from the probe, per-device memory, the always-shown rule, the readout in the level's own measure. System level first | RW2, RW4 | chart room, with quality tiers |
| **RW6** | Drawing a crowd: columns to the GPU, orbits in the shader, a press resolves one member, on the system chart, System 3D and True; the phone and text paths. Retires the four-member stand-in (SB-sample) | RW4, RW5 | small bodies, with the renderer |
| **RW7** | The Solar System catalogue: build scripts (run on Alex's PC) for every moon, minor planet and comet into size-ordered files; estimated sizes graded; base-sol gains full orbits at a date, belt masses by group and the Expected stop; the notable flags | RW1, RW3 | data sheet |
| **RW8** | The data service: the storage bucket, Worker routes `/cat/*` with byte ranges, immutable versioned files, the live-query cache, the offline edge | question 8 | kernel hand-offs or the Explorers thread |
| **RW9** | Filters as chips over population columns: group, makeup, orbit, hazard, known, visited, interesting | RW5 | navigation (find by tag) |
| **RW10** | The census: op `census`, the "Against the Solar System" card, the thousand-seed report and the gap list | RW4, RW7 | proven core |
| **RW11** | Claims in the star field: a pin from outside with measured mass, place and velocity; best-fit slot in the count tree; thinning by completeness; the block overdraft rule; the three tests. Core golden | GZ13, CC6, RW3 | kernel (core lane) |
| **RW12** | The real sky pack: the home wish, the galaxy's measured claims, the measured arms, the Sun's slot, the date, the named neighbours as claims; the shipped bright and named stars. The start sheet's Solar System door opens it | GZ12, RW11, the Local Arm fix, RW1 | galaxy charts and histories, with the kernel |
| **RW13** | Star sources: CNS5, doubles, known planets, names, the live route, reusing V3's importer logic through the converter module; planets as formation pins | RW8, RW11 | converter thread |
| **RW14** | The import action: centre, radius, preview, apply; any anchor in any galaxy; the Traveller sector as a catalogue | RW11, RW13 | galaxy charts and histories, Traveller |
| **RW15** | Detail on the other levels: Local stars, Galaxies nearby, the web, a world's moons and rings. The night sky's star budgets move under it | RW5, GZ7 | each view's owner |

First four to open: RW1, RW2, RW3, then RW7. RW4 to RW6 give Alex the slider on generated systems before any real data arrives, and RW7 then makes the Solar System fill it.

**Fudges this track registers:** estimated sizes and masses from brightness; plain-ellipse motion of catalogue orbits away from their date; orbits worked out in the shader for display; the seeded web round a real galaxy; a line-of-sight speed drawn where none was measured; block-level closure for over-full cells.

---

## 12. Questions for Alex

Each has my recommendation. Routine choices went to the coordinator and are not here.

1. **Is the real sky the first thing a new user gets?** The start sheet's Solar System door would open the real Milky Way at Earth, today, with "a new universe" as the other door. This reverses the night sky note's "fictional galaxies are the default". *Recommend yes:* it is what your message says, and a newcomer recognises it at once.
2. **Is a real galaxy in a generated universe acceptable?** Measured out to the Local Group, generated beyond, and labelled so. *Recommend yes.* The alternative is a research project.
3. **What ships, what is a pack, what is live?** In the app: the Solar System to the Expected stop and about 9,000 bright and named stars from open sources. Free optional pack: the complete nearby sample and the full small-body catalogue. Live: anything further. Gaia-derived data stays out of the app, as you ruled. *Recommend yes.*
4. **Detail as a sixth view control on every view**, beside View style, Seen from, Scale, HUD and Band, remembered per device. *Recommend yes.*
5. **The default is the Expected stop, and the machine sets a marked ceiling the user may pass with a warning**, not a hard limit. *Recommend yes:* suggest, never refuse.
6. **"Expected" is a rule, not a list:** everything big enough to be round, plus notables whatever their size (so Phobos, Deimos and Halley's comet show). The same rule gives a generated system its default. *Recommend yes.*
7. **Estimated sizes take part in the slider**, marked as estimates on the card. Otherwise nine in ten known asteroids could never be shown by size. *Recommend yes.*
8. **A Cloudflare storage bucket for catalogue files**, which you would create (a few minutes; I will write the steps), with the build scripts run on your PC because the cloud cannot reach the catalogue sites. *Recommend yes.* Nothing else can serve 45 MB in ranges.
9. **"Known by a date" as a filter**, so a campaign in 1900 shows players only what had been found. *Recommend yes, as part of RW9, not before.*
10. **Hypothetical bodies** (the Planet Nine candidates) as a switch in the pack, off by default and tagged hypothetical. *Recommend yes.*
11. **The Solar System as yardstick, never as target:** the comparison finds missing processes, and no generator number is tuned to match it. The "Against the Solar System" card is shown to the GM on the Explore desk. *Recommend yes.*
12. **Catalogues are placed by an anchor, so the real neighbourhood can be dropped into a fictional galaxy and a fictional star list into the real one.** *Recommend yes;* it costs nothing extra and it is the general form.
13. **A far-off catalogue star stays a table row until someone looks at it**, instead of becoming a full pin at import. A 100 pc import is then megabytes, not most of a gigabyte. *Recommend yes.*

---

## 13. What I checked, and what I did not

Checked by reading main at 7e01502: the handover, decisions, the chain notes, the night sky, galaxy and grand zoom notes, the status files named above, `packs/base-sol` (its bodies, regions and value keys), the store pack `sol-today`, the converter's copy of V3's importer (its file list and the three services it calls), the site Worker's routes, and the roster prototype's report. I tried ten catalogue hosts from the cloud and all were refused.

Not done: I ran no code and measured nothing. I did not read the Local Arm branch. I searched the formation and small-body crates for words (trojan, family, Kirkwood and so on) and did not read how each structure is made, so section 2.4 says what the code mentions, not how well it does it. I did not see GZ12's pull request. Every figure marked unverified is from memory.

## 14. Picking this up without the chat

Read this note, then `handover/status/real-catalogues.md`. Alex's answers to section 12 will be recorded in a section 12A here and in `handover/decisions.md`; where they differ from the text, they win. The readable copy Alex comments on is linked from the status file.
