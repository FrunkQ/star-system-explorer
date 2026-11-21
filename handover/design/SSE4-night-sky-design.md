# SSE4 night sky design: the real view from anywhere, and cataloguing what you see

Written 21 Nov 2026 at Alex's request. Status: **accepted 21 Nov 2026** (Alex: yes to the design and its recommendations). Every recommendation in section 12 is a ruling, and the stages in section 11 are in the build plan as track Y. It widens design handoff section 10 ("Real sky backgrounds"), galaxy design sections 8.3 and 10, and nebulae and light design sections 5A and 5B, and it replaces the one-line build plan rows V7 (Sky view) and G7 (Milky Way pack) with track **Y** below. Where this document and those sections differ, this one is the authority.

Alex's asks, 21 Nov, in his words where possible:

- "On 'real views' we will see the other planets, charted stars and charted galaxies in the night sky? Also any features that may be seen."
- "Be able to use various telescopes to 'see them' to reveal details." On an Earth analogue, "different sensors to see 'the sky' differently".
- "We will need procgen'd unnamed stuff too, to fill in based on densities. But 'known stuff' can resolve it. Perhaps cataloguing a procgen star does resolve it like a normal one. Practical or dumb?" Answered as practical; he then asked "will this also do the same with galaxies?" (yes) and said yes to all three.
- On real data: "we have the Gaia importer. Can we get better than that for star imports? Also we are about fictional universes, so we don't need full Milky Way resolution."
- "The default solar system has an asteroid belt and an Oort belt. Really should be a sphere."

What is built that this stands on: `sse-cosmos::stars` (G5: stars drawn per 10 pc co-rotating cell and 1 Myr frame, pins, `promote`), G2 haloes per cell and `halo_cell`, G3 and G4 galaxy and patch envelopes, `sse-light` spectra and band integrals (O1, O2), `read_nebulae` (O3), shifts (O7), `detect` (O9), `read_spectrum` (O12), `survey_planets` (O13), the campaign record (J1), programs (J6 part 1), and the Window (`window_view`, V8 part 2a), which today draws the system's own bodies and **no background stars, moons or galaxies** (V8.md, "Honest limits").

---

## 1. Short answers

1. **Yes, the sky from any world shows everything that is really there.** The system's planets and moons (already built), the stars, the Milky Way's glowing band, nebulae, other galaxies, and the system's own features (zodiacal light from its dust, aurorae, a giant planet's rings edge-on, a supernova as a new star). Each is drawn from the same object that you could travel to, so the dot and the place are one thing.
2. **Instruments see what they could really see.** Eyes, binoculars, a telescope, an infrared or radio dish: each has bands, a faintness limit and a sharpness. A better one shows fainter stars, splits doubles, shows a galaxy's shape, takes a spectrum. Switching band shows the same sky as it looks in that light (dust lanes go clear in infrared, the radio sky is the galaxy's own glow and its pulsars). Nothing is shown that the instrument could not tell.
3. **The unnamed background is drawn from real densities and stays the same for everyone.** It is the probability cloud, resolved only as far as the eye or instrument reaches.
4. **Cataloguing resolves.** Cataloguing a background star or galaxy makes it a permanent object with a designation and a record entry (discovered on this date, by this observer, with this instrument). It does not form its planets yet: that waits until someone goes there or asks (Alex, 21 Nov: "best calc is NO calc"). It nests: cataloguing a galaxy fixes the galaxy, not its stars.
5. **Real data: yes, better than a Gaia import, and much smaller.** A curated set (section 8) is more complete where it matters (near stars, bright stars, names, doubles, planets) and a fraction of the size. Fictional universes need none of it: they get a grown galaxy and the fill. Checking the suggestions turned up one thing Alex must know: **Gaia data is now licensed non-commercial** (CC BY-NC 3.0 IGO), so anything derived from Gaia lives in an optional pack, never in the app (question Q3).
6. **The Oort cloud is a shell.** The formation code already knows it is isotropic; the chart draws every cloud as a flat ring. Package Y0 fixes that first.

---

## 2. Principles

1. **One object, two looks.** A sky point and the resolved star are the same description read twice. The sky's brightness and colour for a star are computed by the same `sse-light` function from the same star state that its system chart uses. If the two ever differ, that is a bug, and a test says so (Y7).
2. **Draw only what can be seen.** The sky never draws a galaxy's worth of stars. It draws, per brightness class, only the volume in which that class can reach the instrument's limit, and everything fainter is integrated light (section 4). Reuse before compute: the sky is cached per vantage and per frame, and a glance never triggers formation.
3. **Facts in the kernel, pictures on the GPU.** The star list, the glow map at coarse resolution, galaxy list and their band fluxes are kernel facts (pinned maths, identical on every device), because sensing (O9) and survey (O13) read them as the background. The smooth sky picture, twinkling and fine glow texture are decoration.
4. **Light takes time** (ruling 19 Nov, on by default). Each object is seen as it was when its light left. For a star a few hundred light-years away that changes nothing visible; for a galaxy it is millions of years, and the view says so.
5. **Consistency over completeness.** The fill may be redrawn when time jumps a long way; a catalogued object never changes. The view always says which is which.

---

## 3. What is in the sky, and where each part comes from

| Layer | What you see | Source | Fact or decoration |
|---|---|---|---|
| The system's own bodies | planets, moons, the star, phases, eclipses | `window_view` (built), extended to moons and inclined orbits | fact |
| The system's own glows | zodiacal light and gegenschein from inner dust; aurorae (from a surface); a ring edge-on; comet tails later | F5 clouds' dust mass and extent; O2 aurora; rings | fact (brightness per band), decoration (shape detail) |
| Near and bright stars | individual points with colour and brightness, doubles, variable and flaring stars | count tree and G5 draws (section 4), permanent pins, catalogues in a real-sky pack | fact |
| Unresolved starlight | the band of the Milky Way, the bulge, dark dust lanes | sight-line integral through the galaxy model, minus what is drawn as points (section 5) | fact at coarse resolution, decoration finer |
| Nebulae | glowing clouds, planetary nebulae, remnants | `read_nebulae` and patch glows (O3), drawn by O5 | fact (descriptor), decoration (structure) |
| Star clusters | open clusters and globulars as groups | needs sibling groups, which G5 lacks (G5.md limits); a later package, see 11 | fact when built |
| Satellite galaxies and the near galaxies | the Magellanic-cloud-like smudges, the nearest big spiral | host halo's satellites and the G2 halo catalogue (section 6) | fact |
| Far galaxies | faint smudges, only in telescopes | halo count tree by mass (section 6) | fact for drawn ones |
| Transient events | a supernova as a new star, a nova, a burst | G6 events and O10 flares, each seen after its light time | fact, dated |
| The air (from a surface) | blue day sky hiding the stars, twilight, horizon dimming, twinkling | `read_air_optics` (built) for extinction and sky glow; twinkling decoration | fact (extinction), decoration |

---

## 4. Stars: the count tree, so a bright distant star is cheap and consistent

### 4.1 The problem with today's draw

G5 draws a Poisson count per 10 pc cell, then each star. To find the naked-eye stars you would need every cell out to the distance the most luminous stars are seen from. Before dust, to magnitude 6.5 **[arithmetic from the distance modulus; absolute magnitudes from memory, check]**:

| Kind | Absolute magnitude (visual) | Seen to magnitude 6.5 out to | 10 pc cells inside that radius |
|---|---|---|---|
| Red dwarf (M4) | about +12 | 0.8 pc | 1 |
| Sun-like (G2) | +4.8 | 22 pc | 45 |
| A0 | +0.6 | 150 pc | 14,000 |
| B0 | −4 | 1,260 pc | 8.4 million |
| Red supergiant | −7 | 5,000 pc (dust limits it well before) | far too many |

Drawing millions of cells for a handful of rare stars is wrong, and drawing them a different way (coarse cells for bright stars) would break consistency: the B star seen in the sky must be the same B star a 10 pc cell draw gives when you fly there.

### 4.2 The fix: split counts down a tree, by brightness class

Group stars into a few **classes by luminosity** (about eight bins over the IMF and age mix: for example below 0.1, 0.1 to 1, 1 to 10, 10 to 100, 100 to 1,000, above 1,000 solar luminosities, plus white dwarfs and giants by phase). Then, per class:

- A **root block** (for example 640 pc, which is 2⁶ cells of 10 pc on a side, on the same co-rotating grid) gets a Poisson count from the expected number of that class in the block, keyed by (class, block, frame).
- Each block splits its count among its eight children by a keyed multinomial draw, with each child's share in proportion to its expected number (from the envelope, so density gradients are honoured). This repeats down to the 10 pc cell.
- At the leaf, each star is drawn exactly as G5 draws it, but its luminosity class is already known, so its mass and age are drawn conditioned on the class.

This is exact statistics, not an approximation: splitting a Poisson count multinomially in proportion to the children's expectations gives independent Poisson counts in the children **[standard result, the splitting property of the Poisson distribution]**. So every 10 pc cell has the same distribution of stars as now, but:

- a sky draw walks down only branches that hold stars of a class visible at that distance, so the cost scales with the number of stars seen, times the depth (6 levels), not with the volume;
- one star, one place: the B star found from 1 kpc away is the same B star a cell draw gives, because both walk the same tree with the same keys.

What it costs: it changes which fill stars are drawn, so the G5 golden hash changes on purpose. Anything already promoted must keep its description first (Y1 depends on Y7a; question Q1).

### 4.3 Which stars a sky draw fetches

For an observer at a place and time with an instrument of limiting magnitude *m* in a band: for each class, the visibility radius is the distance at which the class's brightest member reaches *m*, shortened by dust along the way (extinction from the patch dust model, the same one O9 uses). The draw walks the tree inside that radius, in a cone if the instrument's field is narrow. A wide-field view to magnitude 6.5 gives a few thousand stars from the fill in a Milky-Way-like disc; a 1 degree telescope field to magnitude 15 gives about ten thousand **[estimates; Y2 measures]**.

Each drawn star gets its band fluxes from `sse-light` (thermal spectrum from its state, companions summed as in B1), reddening by the dust column, and its position at the time its light left. Points carry their addresses, so clicking one opens a card and, if wanted, the system (handoff 10.5).

### 4.4 Frames and time

G5 redraws the fill per 1 Myr frame. Within a campaign that never matters. For the sky, the fill is drawn **for the observer's frame**, even though a star 1 kpc away is seen 3,300 years in the past: far smaller than a frame, so the only error is at a frame boundary, and it is listed in `docs/fudges.md`. Permanent objects are exact: they are read at the time their light left (a catalogued star that went supernova 200 years ago, 500 light-years away, still shines in the sky for 300 more years, and the GM sees that).

Scrubbing deep time moves permanent stars on their orbits (constellations distort over a million years, which is real) and redraws the fill each frame, labelled "background redrawn for this era".

---

## 5. The glow: unresolved starlight, dust and the Milky Way band

Everything fainter than what the sky draws as points still shines. The **glow map** is the light of the stars not drawn, integrated along sight lines.

- **Pixels:** an equal-area sphere grid (HEALPix, Górski et al. 2005, ApJ 622, 759 **[from memory, check]**; written fresh from the paper, since the reference library is GPL). About 12,000 pixels (Nside 32) as kernel facts; finer is decoration.
- **Per pixel:** step outward through the galaxy model (G4 patch envelopes give star density by class, the gas and dust), adding each class's light only where it is fainter than the draw limit at that distance, and dimming by the dust in front. The result is a brightness per band per pixel. Because the classes drawn as points are subtracted where they are drawn, there is **no double counting**, which is G7's acceptance check.
- **Bands:** visible and near infrared from starlight; far infrared from dust warmed by that starlight (dust column times a heating rule, a documented fudge until a real dust temperature block exists); radio from the galaxy's synchrotron glow, for which there is no physics in SSE4 yet: proposed as emissivity proportional to the local star formation rate (a proxy for cosmic rays) times the field strength, as a documented fudge with its reason; X-ray from hot gas in remnant bubbles (O3 counts) only, with the rest stated as missing.
- **Reuse:** the map changes slowly with position. It is cached per 50 pc vantage cell and per frame, and inside one system it is computed once. Other galaxies' light is added in section 6.
- **Who reads it:** the sky view; O9 `detect` as its background in that direction and band (replacing any flat background); the descent and photograph views as a sky dome.

---

## 6. Galaxies in the sky, the same way

Galaxies follow the stars exactly, one level up (Alex, 21 Nov: "will this also do the same with galaxies?", yes).

- **Satellites of our own galaxy** (the Magellanic-cloud-like ones): from the host halo's satellites. G2 has merger histories but not a satellite list at a time; Y4 adds one (subhaloes drawn from the host's mass and history, those already inked by G6 kept).
- **Other galaxies:** a halo count tree by mass class over the cosmic web cells, as in 4.2, so a bright nearby galaxy is found without drawing every faint one. The floor drops with distance: near galaxies down to small dwarfs, far ones only above what the instrument can see. Today's `halo_cell` floor (10¹⁰ solar masses) stays the default for the cluster map.
- **What each looks like:** stellar mass, size, shape (disc or spheroid, tilt to the line of sight) and colour from G3 reservoirs, through O6 when it lands; until then a smudge with the right brightness, size and colour, labelled.
- **Light time:** each galaxy is read at the epoch its light left (G2 has the growth history), redshifted by O7 in the campaign's cosmology. The card says "seen as it was 2.5 million years ago" and the GM sees what it is now.
- **Nesting:** cataloguing a galaxy makes the halo and galaxy permanent; its stars stay a cloud until a patch inside it is looked at.
- **Fictional galaxies** are the default; a universe pack may pin named ones (galaxy design 10).

---

## 7. Instruments: seeing more, and seeing differently

An instrument is data, as now (`nav.window.sensors`: eyes, 10×50 binoculars, a 0.5 m telescope; X3d sensors; O9 sensor parameters). Each gives bands, a limiting magnitude per band for a stated stare, a resolution and a field of view. From those, with no special rules:

| Instrument property | What it changes in the sky |
|---|---|
| Limiting magnitude | the draw limit, so more stars and fainter galaxies, and less left in the glow |
| Resolution | close doubles split, planets show discs and phases (built), galaxies and nebulae show shape, a cluster resolves into stars |
| Field of view | a narrow cone, so deep draws stay cheap |
| Bands | the same sky in another light (O1 band recipes): dust lanes clear in infrared, cool brown dwarfs appear, hot stars fade; radio shows the glow, pulsars and the galaxy's centre |
| Spectrograph | any point opens its spectrum chart (O12), which is the "Catalogued" to "Spectrum taken" rung of O14 |
| Stare time | fainter limit with a longer exposure, as O9 already computes |
| Ground or space | from a surface, the air (built `read_air_optics`) dims, reddens and, by day, hides the sky |

A seat sees only through its own instruments (O8). The GM can always see the truth layer, marked GM only, as the Window does today.

---

## 8. Real star data: better than a Gaia import

### 8.1 What exists now

V3's real-sky importer (vendored in the converter, `src/lib/vendor/import/realsky/`) queries SIMBAD for stars, the NASA Exoplanet Archive for planets, and Gaia DR3 through its TAP service, with a bundled snapshot to 12.7 pc for offline use. It is a good importer for a region; it is not a sky.

### 8.2 Why not Gaia as the base

Gaia is magnificent but the wrong base for this: about 1.8 billion sources is far more than a fictional-first app needs; its brightest stars are the least well measured **[from memory: saturation near G = 3, check]**; it carries no proper names and no curated multiples; and its data licence is now **CC BY-NC 3.0 IGO** (ESA's Gaia licence page, read 21 Nov 2026: "Gaia data are distributed under the CC BY-NC 3.0 IGO license"), which is non-commercial.

### 8.3 The proposed curated set (checked 21 Nov unless marked)

| Layer | Catalogue | What it gives | Size | Licence and notes |
|---|---|---|---|---|
| Within 25 pc | **CNS5**, Fifth Catalogue of Nearby Stars (Golovin et al. 2023, A&A 670, A19) | 5,931 objects: 5,230 stars (264 white dwarfs, 20 giants) and 701 brown dwarfs; statistically complete to absolute G 19.7, about spectral type L8 | small | the paper is CC BY 4.0; built from Gaia EDR3, Hipparcos and ground parallaxes, so the data terms need checking at CDS before bundling |
| Bright and named stars, whole sky | **AT-HYG v4** (astronexus, on Codeberg) | over 2.5 million stars (all of Tycho-2 plus HYG), nearly 99% with Gaia DR3 distances, 92% with full 3D velocities, IAU official names | about 1 GB raw; the naked-eye subset is tiny | CC BY-SA 4.0, share-alike; it contains Gaia DR3 values, so the Gaia terms may also apply (Q3) |
| Optional middle depth | **Gaia Catalogue of Nearby Stars** (Gaia Collaboration, Smart et al. 2021, A&A 649, A6) | 331,312 objects within 100 pc, at least 92% complete to M9 | moderate | Gaia data, so non-commercial |
| Doubles and multiples | Washington Double Star Catalog (USNO) | measured pairs and orbits | moderate | **not checked this session**: US Naval Observatory data, likely public domain; check |
| Known planets | NASA Exoplanet Archive | measured orbits, masses, radii | small | already used by V3's importer; terms ask for acknowledgement **[check]** |
| Names lookup | SIMBAD (CDS) | any name to a position | live query | already used by V3's importer; acknowledgement required **[check]** |
| Near galaxies | **Updated Nearby Galaxy Catalog** (Karachentsev, Makarov and Kaisina 2013, AJ 145, 101) | 869 galaxies within 11 Mpc | small | terms not checked |
| Named deep-sky objects | **OpenNGC** (Mattia Verga) | NGC and IC objects: galaxies, clusters, nebulae, with names | small | CC BY-SA 4.0 |
| Deep stars, optional | Gaia DR3 via its TAP service | anything else, on request | huge | non-commercial; fetch on demand only |

### 8.4 How the real pack fits the fill

- Catalogue stars are **pins**: they claim from the count tree first.
- Each catalogue declares its **completeness** as a function of absolute brightness and distance (CNS5: complete to L8 within 25 pc; AT-HYG: complete to its Tycho-2 apparent limit **[about V 11, check]**). The fill is **thinned** by that completeness: a fill star is kept only if a keyed draw exceeds the catalogue's completeness for its class and distance. So inside 25 pc almost nothing is fill; out where the catalogue thins, the fill takes over smoothly, and nothing is counted twice.
- Catalogue versions are pinned in the run manifest (handoff 10.1). Data comes as sky tiles through SSE4's own Worker and R2 cache (surfaces and rendering 3.3). The naked-eye set (a few hundred kilobytes **[estimate]**) ships inside the pack so the real sky works offline.
- Unmeasured facts about a real star (its planets if none are known, its age if unmeasured) are resolved around the measured ones and tagged fill, as already ruled.

### 8.5 Fictional universes

The default. No catalogue: a grown galaxy (G0 to G6), the count tree, and whatever the GM and players have catalogued. Nothing in track Y needs a network connection unless a real-sky pack is loaded.

---

## 9. Catalogue to resolve

### 9.1 What cataloguing does

**Catalogue** is an intent on any sky point (star, galaxy, nebula, cluster): from the GM (always allowed), from a player seat through a survey (O14's timed intent), or from a program (J6: "a survey probe catalogues everything brighter than magnitude 12 in this field"). It:

1. **Promotes the object to permanent** (G5 `promote`, extended to galaxies), **storing its full drawn description** (address, class, mass, age, metals, orbit, companions). From then on it is a pin and is read from the store, not redrawn, so no later change to the fill rules can move or change it.
2. **Writes a record entry** (J1): a note or kept value "Catalogued" with the designation, date, observer, instrument and what was measured (with error bars, per audience as O14 rules: a player's entry holds what their instrument measured; the GM's holds the truth).
3. **Gives it a designation** from the campaign's naming scheme (pack data: for example a survey name plus a number, "Second Survey 4127", or in a Traveller campaign the pack's own style). The GM or the discovering party may name it; names are per audience, like feature names (S7).
4. **Does not form its system.** Its planets stay a cloud until someone goes there, plans a route to it, surveys it for planets (O13), or asks. Forming runs then, from G5's `birth`, as it does today.

### 9.2 The ladder it starts

Catalogued (a point with position, brightness, colour) → spectrum taken → surveyed from afar → characterised → visited. These are O14's rungs; track Y supplies the bottom one. Each rung is a per-audience disclosure, so two parties can know different things about the same star.

### 9.3 Why it is a game

- A survey campaign is a real activity: point the array, wait, get a dated list of finds, some candidates that turn out to be nothing.
- Discovery is recorded: who found it, when, with what. Sifters (U8) can find stories in it ("the rival survey catalogued it first").
- Light time makes surprises: a catalogued quiet star may be found, on arrival, to have flared or died after its light left.
- Old catalogues (O14) are produced by running these rules with an older instrument.

### 9.4 Undo and edits

Cataloguing is an authored event, so undo and what-if branches work as for any edit. A GM may also **release** a catalogued object back to the cloud if no one has visited it (it becomes fill again; the record entry is kept, marked released). Anything visited cannot be released.

---

## 10. The Oort cloud and other shells (Alex, 21 Nov)

F5 already draws the Oort cloud as isotropic (`Tilt::Isotropic` in `sse-formation/src/leftovers.rs`, from Duncan, Quinn and Tremaine 1987) and the scattered disc as a thick disc. The chart room draws every cloud as a flat band, and the grown-system preview leaves the Oort cloud off altogether (`ui-contract/src/make.ts` line 313). The fix:

- `resolved_system` clouds gain `shape` (`ring`, `thick-disc` or `shell`) and the inclination spread, from F5's tilt.
- The system chart draws a shell as a soft hatched disc out to its outer radius with a "sphere" mark, distinct from a belt's ring; System 3D draws it as a sparse round haze of points; it never appears in the sky (it is far too faint, and the card says so).
- A V3 import's belt named as an Oort cloud, or authored with a large spread, becomes a shell, and the import report says so. Alex, 21 Nov: the default Solar System's data is all V3's own, authored over time and possibly wrong; its Oort belt is one such entry, so Y0's import rule (and a check of that data) fixes it. The chart room thread was already asked to draw the Oort cloud as a sphere; Y0 is that work.

---

## 11. Stages (track Y, for the sky)

Order: Y0 is a small fix and can go now (the chart room thread already has it). Then Y7a (keep promoted descriptions) before Y1 (count tree), because Y1 changes the fill. Then Y2, Y3, Y5 for a first real sky, then Y4, Y6, Y7, then Y8 for the real-sky pack, Y9 last.

| Stage | Contents | Needs | Thread | Evidence (tests) |
|---|---|---|---|---|
| Y0 | Clouds have shapes: `shape` and spread in `resolved_system` clouds; shells drawn as shells on the system chart and in System 3D; V3 Oort belts imported as shells | F5, U13 | kernel (field), chart room shell (drawing) | a grown Sun-like system's Oort cloud has shape `shell`; the chart draws it differently from the asteroid belt (screenshot); a V3 save with an "Oort" belt imports as a shell with a report line |
| Y7a | Promotion keeps the drawn description: `promote` stores the star's full state as a pin; existing saves migrate by storing what today's draw gives | G5, J1 | kernel resolve and rules | a promoted star reads bit-identical after the fill rules change (simulated by a changed key version); an old save with promoted stars loads unchanged |
| Y1 | Count tree: luminosity classes, root blocks, keyed multinomial splitting to 10 pc cells; leaf draws conditioned on class; G5's golden re-blessed on purpose with the reason in the log | Y7a, G5 | kernel resolve and rules | per-cell counts match Poisson with the envelope mean (chi-squared over 10,000 cells); the G5 neighbourhood checks still pass (RECONS band, M-dwarf share, multiplicity); order independence; a B star found by a 1 kpc sky draw is identical to the cell draw at its position; cost of a magnitude-6.5 sky draw grows with stars found, not volume (measured) |
| Y2 | Sky list: kernel op `sky_from_here {observer, time, sensor or limit, bands, cone?}` giving stars (fill and pins) with address, direction, band fluxes, reddening, colour, light time, double and variable flags; moons and inclined orbits added to the system layer | Y1, O1, O2, O9 | kernel (draw), light (band fluxes, dust extinction) | from a Solar System fixture, the brightest drawn stars fall in the expected count range per magnitude bin (a fill sky is statistical; check the cumulative count against a published star count curve within a stated band); same list on every harness browser; a point's magnitude equals the magnitude computed from its resolved system |
| Y3 | Glow map: HEALPix written fresh, sight-line integral of unresolved light by band through G4 envelopes with dust; subtraction of drawn classes; cache per 50 pc vantage and frame; O9 `detect` reads it as background | Y2, G4, O9 | light and nebulae | no double counting: total sky light with the draw limit at magnitude 4 and at 8 agrees within a stated tolerance; the band lies along the galactic plane and is brightest toward the centre; dust lanes dark in visible and clear in near infrared; sky brightness sane along three sight lines (G7's old check); fudges for dust heating and radio listed |
| Y5 | Sky drawn: the Window (and a "Sky from here" mode from a surface) draws background stars, glow, nebulae from `read_nebulae`, transient events; points clickable to cards; baked to a cube map per vantage and band recipe on the GPU tier, canvas points on low tiers, words on the text tier | Y2, Y3, R1, O1, V8 | renderer (cube map, passes), galaxy charts and travel screens (Window) | screenshots desktop and phone, light and night; a click on a point opens that star's card; band switch changes the sky and never the state hash; phone frame budget held by the degrade order; honest-limits line updated |
| Y4 | Galaxies: host satellites at a time; halo count tree by mass class; galaxy list in `sky_from_here` with size, tilt, colour, brightness by band, look-back epoch and redshift | Y1, G2, G3, O7 | kernel resolve and rules (G), light (appearance until O6) | the nearest big galaxy in a default universe appears at a brightness consistent with its stellar mass and distance; a galaxy's card shows look-back time; the same halo read through `galaxy_chart` and the sky has the same stellar mass; order independence |
| Y6 | Instruments in the sky: limit, resolution, field and stare from sensor data; doubles split, galaxy shapes, clusters; spectrograph to the spectrum chart; seat sees only its own instruments | Y5, O8, O12 | light and nebulae (rules), galaxy charts and travel screens (screen) | eyes, binoculars and the 0.5 m telescope give star counts in the ratio their limits predict; a close double splits only above the resolution; a seat without infrared cannot pick that band; clicking a point with a spectrograph opens its chart |
| Y7 | Catalogue to resolve: `catalogue` intent for stars, galaxies, nebulae, clusters; record entry with observer, instrument, date and per-audience measurements; designations from a pack naming scheme; release; survey programs that catalogue in bulk; player intents through GM approval or auto-approve | Y2, Y4, Y7a, J1, J6, O14, V5 | kernel resolve and rules (intent, record), chart room shell (Catalogue button, card) | catalogue a fill star: it is permanent and identical at 20 times; its record entry holds the instrument's measurements for the player and truth for the GM; its system is not formed until visited; cataloguing a galaxy leaves its stars unresolved; a released, unvisited star returns to the cloud; undo works |
| Y8 | Real-sky pack (replaces G7's catalogue part): curated catalogues of 8.3 as pins with declared completeness and thinning; tiles via the site Worker and R2; naked-eye set bundled; versions in the manifest; SIMBAD name lookup and exoplanets reusing V3's importer through the converter module; licence notes per catalogue | Y2, Y4, G7 galaxy pins, X-convert-wasm | kernel (pins, thinning), converter thread (fetch and import logic), citations thread (licence checks) | stars at catalogue positions; no double counting in magnitude counts (G7's check); inside 25 pc no fill star brighter than CNS5's limit; offline, the bundled set loads and the edge is shown; a manifest upgrade shows what changed |
| Y9 | Deep-time sky: scrubbing time moves permanent stars on their orbits and redraws the fill per frame with a label; "the sky from here, a million years ago"; star clusters and sibling groups (a G5 gap) | Y5, G5 | galaxy charts and travel screens, kernel | a catalogued star's sky position over 1 Myr follows its orbit; the fill label appears when a frame boundary is crossed; a fixture cluster's members share age and metals |

Constellations and asterisms (named shapes in a world's sky, drawn by the GM or the players, kept in the record per audience) are a small extra on Y5 and Y7, not a stage of their own.

---

## 12. Rulings **[accepted by Alex, 21 Nov 2026; each was put as a question with a recommendation, and the recommendation is the ruling]**

**Q1. May Y1 change which fill stars are drawn (G5 golden re-blessed)?**
*Ruling: yes.* The statistics are unchanged (the splitting property makes it exact), and Y7a first stores every promoted star's description so nothing anyone has looked at moves. The alternative, keeping today's draw and adding a separate bright-star draw, cannot guarantee that the sky's B star and the cell's B star are the same.

**Q2. Should the sky's fill be drawn for the observer's frame rather than each star's light-departure time?**
*Ruling: yes,* as a documented fudge. The error is at most a few thousand years against 1 Myr frames, and only at a frame boundary. Permanent objects stay exact.

**Q3. Gaia's non-commercial licence: will SSE4 ever be sold or carry paid features?**
*Ruling (Alex did not say whether SSE4 will ever take money, so the cautious recommendation stands):* keep every Gaia-derived dataset (GCNS, Gaia DR3 fetches and, until its terms are clear, AT-HYG and CNS5) in the optional real-sky pack, served from R2 with its licence shown, never inside the app. The app stays Apache-2.0 and free of data licences. If SSE4 stays free, nothing else changes; if it might be sold, the pack would need ESA's commercial terms or a non-Gaia base (Hipparcos and Tycho-2 alone). The citations thread should confirm each catalogue's data terms at source before Y8.

**Q4. Does cataloguing form the system?**
*Ruling: no.* Cataloguing fixes the star; forming its planets waits for a visit, a route, a planet survey or an explicit ask. That follows "best calc is no calc", and it keeps a survey of a thousand stars cheap.

**Q5. Can players catalogue?**
*Ruling: yes, through a survey intent from a science seat,* with the GM's approval by default and an auto-approve scope available (V10). The shared record needs GM approval under the existing ruling; a player's own journal note does not.

**Q6. Default designations?**
*Ruling:* a pack naming scheme, defaulting to the campaign's survey name plus a running number. Real-sky campaigns use real names and designations. Party names are per audience.

**Q7. How deep should a device draw before the rest becomes glow?**
*Ruling:* by capability tier, with the light accounted for either way: about 20,000 points on phones and 200,000 on desktops **[estimates; Y5 measures]**. The glow absorbs the rest, so a phone's sky is just as bright, only less granular, and the quality badge says so.

---

## 13. Sources

Checked 21 Nov 2026 against the source pages:

- Golovin, A. et al. 2023, "The Fifth Catalogue of Nearby Stars (CNS5)", A&A 670, A19, doi:10.1051/0004-6361/202244250 (arXiv 2211.01449).
- Gaia Collaboration, Smart, R. L. et al. 2021, "Gaia Early Data Release 3: The Gaia Catalogue of Nearby Stars", A&A 649, A6, doi:10.1051/0004-6361/202039498.
- ESA Gaia data licence page, https://cosmos.esa.int/web/gaia-users/license (CC BY-NC 3.0 IGO).
- AT-HYG v4, https://codeberg.org/astronexus/athyg (CC BY-SA 4.0; the GitHub copy is archived).
- Karachentsev, I. D., Makarov, D. I. and Kaisina, E. I. 2013, "Updated Nearby Galaxy Catalog", AJ 145, 101 (arXiv 1303.5328).
- OpenNGC, https://github.com/mattiaverga/OpenNGC (CC BY-SA 4.0).

From memory, to check before use: Górski, K. M. et al. 2005, ApJ 622, 759 (HEALPix); Leinert, Ch. et al. 1998, A&AS 127, 1 (night sky brightness, for the glow's sanity check); the Washington Double Star Catalog's terms; the NASA Exoplanet Archive and SIMBAD acknowledgement terms; Gaia's bright-star saturation limit; absolute magnitudes in the table of 4.1.

Already in the repo: Duncan, Quinn and Tremaine 1987, AJ 94, 1330 (Oort cloud, `leftovers.rs`); Henry et al. 2018 (RECONS, G5 checks); Kroupa 2001 (IMF, G5).
