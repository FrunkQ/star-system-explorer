# What SSE3 does that SSE4 does not (yet)

Date 23 Nov 2026. Status: **obvious items going ahead** (Alex, 23 Nov: "the obvious ones you can push on and develop"); Alex answered the six open questions on 23 Nov 02:51 (recorded on each item) and added items 35 and 36. **Nothing is waiting on him.**

- Going ahead, each through the thread that owns its area and in its stage: 1 to 9, 11 to 14, 16, 18 to 23, 25 to 29, 31.
- Settled 23 Nov: 10 (replaced by the gallery, item 35), 15 (megastructures as constructs carrying physics blocks), 17 (yes), 24 (purpose-built report screens), 30 (yes), 32 to 34 (dropped for now). New: 35 (class gallery), 36 (gas giant colours from the physics).

Read on Alex's machine, read only: V3.1's `README.md` and `GettingStarted.md` (the user's account of every screen) and its component list (`star-system-generator/src/lib/components`). Checked against SSE4 `main` at a26c1a5 (`handover/START-HERE.md`, the screens and rail entries in `packages/shell/src/documents`, the views and crates). `docs/v3-parity.md` (K0) already gave each V3 mechanism a verdict; this file is the other half: **what a V3 user could do with a click that an SSE4 user cannot today**, and where it lands in the six stages of `SSE4-coming-together.md`. Tags and the V3 save importer are covered elsewhere and are not repeated.

Guiding rule for every call below (Alex, 23 Nov): intuitive and approachable, the most common things a click away, rarely touched things in menus that carry the context they need. Also: don't copy V3, do it better; one general mechanism over specials.

Already in SSE4 and so not listed: size comparison, measure, undo and redo, copy and paste, "why?" on every number (the Newton panel's job), what-if time and look-ahead, the players' page with seats and themes, voyages with fuel and relativistic flight, the Solar System as a start door, the converter for Universe Sandbox and SpaceEngine files, naming schemes (track NM), calendars and unit schemes (MP15).

## A. On the system chart, a click away

**1. Where can something orbit (Hill spheres, Lagrange points, the hole round a binary).** V3 draws Hill spheres for the picked body's neighbourhood, L4 and L5 as true tadpoles, L1 to L3 as station-keeping envelopes, the stable ring round a pair, and lets you right-click inside any of them to put a world or a ship there, with a mass ceiling as advice. SSE4 has the atoms in the kernel and nothing on the chart. *Do better:* one chart layer, "Where things can orbit", drawn from the same stability blocks that judge a placed world, so the picture and the verdict can never disagree; right-click inside it to place. **Stage 2.**

**2. Add a planet, moon or ship here.** V3's commonest edit: right-click a body or empty space, get the types that suit that orbit, gates as switches you can turn off. SSE4 can grow, import and paste, but not add one body by hand. *Do better:* "Add here" on the Build desk offers what this orbit would *make* (a sample from the resolve at that place), plus "anything, marked impossible". No type list to keep in step, because classes are labels read off results. **Stage 1** (it is part of "edit a world" in the walk).

**3. Grow a system with a star you choose and the four dials.** V3's wizard: pick stars on an HR diagram (binaries and triples nest as you add them), an age slider bound to the star's life, then metallicity, disc mass, dynamical history and rarity, each with a green, amber or red realism band, plus character presets. SSE4's grow has a seed, an optional star mass and a formation rule. *Keep, done the SSE4 way:* the dials are named inputs to the formation flow and the bands are its validity labels (as parity said); the HR diagram becomes the star picker; presets become small packs. **Stage 2.**

**4. Pin a number the physics would not give, with a reason.** V3's Overrides tab: albedo (even negative), surface temperature, density, pressure, field, heat; a green, amber, red track with the engine's own answer marked; a reason from the Anomaly list ("Precursor engineering"); "Reset to calculated". SSE4 edits authored inputs and marks impossible ones, but cannot pin a derived figure from the card. *Do better:* no separate tab. Every number's "why?" gets **Pin this value**, which feeds the figure into the flow so everything downstream follows, asks for an optional reason, and puts the honesty mark on the GM view. **Stage 2.**

**5. Change a unit by clicking it.** V3: click any unit and every reading of that kind on that body type follows; players inherit it. SSE4 has unit schemes on the Reference screen design but no click on a number. *Do better:* clicking a unit offers the units of that quantity plus "use the setting's scheme"; the choice is a user preference per quantity kind, saved with the campaign, and a player seat inherits the GM's unless its lens says otherwise. **Stage 2** (with Reference finished).

**6. Next eclipse.** V3 shows when each moon next crosses its sun and how much is covered. SSE4 has nothing. *Keep:* an Equation block over `orbits_at`, shown on the card at Science depth and as dated marks on the time ruler, so "the next eclipse" is something a GM can jump to. **Stage 2.**

**7. Which worlds are shielded, at a glance.** V3 shades each magnetosphere on the system map (shielding part dark, full extent pale). SSE4 shows a field on one body's screen. *Keep:* this is Mag in the band switch at system scale (MF14c), not a new layer. **Stage 2.**

## B. A world

**8. Stand on it.** V3's Surface view: the world's ground, sky and light, and how far you can see and a lamp carries, in metres. SSE4 has the globe, the cutaway and "Sky from here", but no standing view. *Do better:* tap any point on the globe, **Stand here**: the sky from that spot (Window), the light at the ground, sight distance and lamp reach from the air optics, and a place mark a GM can hand to players. **Stage 2** for the reading, the picture improves with True views in stage 6.

**9. Colours under its own daylight.** V3's "Colours" view and true-colour worlds: land, sea, sky and plant colour from the light that reaches the ground. SSE4's screens still read stubbed light. *Keep:* it falls out of light through gas (ND2). **Stage 3.**

**10. The type picture (artist's impression per class).** V3 shows a painted picture per planet type. *Drop:* SSE4 draws each world from its own physics, and a picture per class fights "processes, not classes". A GM's own uploaded picture survives as item 22.

*Alex, 23 Nov:* replaced by a gallery of every class on the classification pages (item 35).

**11. Edit an atmosphere gas by gas.** V3: log sliders across eight decades, ppm below 1%, a cross to remove a gas, always summing to 100%. *Do better:* one **mix editor** for any mixture (air, interior makeup, ocean salts, ices, a ship's fuel), reading substances from the data sheet, with V3's log sliders and ppm. **Stage 2.**

**12. Reclassify by hand when the call is close.** V3 flags a borderline class, shows the competing fingerprints and lets you pin a type that survives reload. SSE4 has class pins in the kernel, nothing on screen. *Keep:* the class line's "why?" shows the runner-up and a **Pin this class** button, same mechanism as item 4. **Stage 2** (beside the class atlas, coming-together question 4).

## C. Ships and stations

**13. Place and edit ships and stations.** V3: Add construct here, a template library (hard SF, The Expanse, Aliens, Mothership), mass, crew, power, fuel, engines, cargo, sensors, and a flight profile that says whether it can land here and why not. SSE4 flies ships from pack data in Voyages, but you cannot add or edit one. *Keep, the SSE4 way:* a ship is a construct record (X3d) placed with item 2's "Add here"; its card uses the depth levels; templates are universe packs on Explorers (free fan packs for named settings). **Stage 5.**

**14. 3D hulls and drive plumes.** V3: upload GLB, STL or OBJ, six NASA craft, finishes, plume scaled to thrust. *Keep, later:* assets in a pack with credit fields; drawn by the renderer. **Stage 6.**

**15. Megastructures and docking.** V3: seven types (elevator, ring, ringworld, Dyson sphere and swarm, collector, battle station) that obey the physics (a swarm dims the star and every world cools), docking at elevator levels and ring rims with the cost priced. SSE4 has none. *Do better:* a megastructure is a construct that declares its physical effects through the same blocks as everything else (a swarm is an absorber in the light path, a ring is a body with an orbit), so no special code per type; docking is a NAV relation. **Stage 5**, late in it, after item 13.

*Alex, 23 Nov:* "constructs are just constructs. Megaconstructs are big enough they need to be tied to physics and impact the system at stellar levels... A construct can have blocks attached (an autopilot)... thrust physics blocks for stellar engines." **Design direction:** any construct may carry a small flow of physics blocks, exactly as a body does. An ordinary ship carries few (its drive, its autopilot program). A megastructure carries the blocks that make it matter at stellar scale: an absorber in the star's light path (swarm, sphere), a mass and an orbit (ring, ringworld), a thrust block (a stellar engine that moves its star over Myr). No per-type code. Needs a short design note before building (kernel thread with the Traveller thread for constructs).

**16. Ship's log, drift and the stress graph.** V3: a log of every journey, cargo and refuel; a stopped ship coasting under gravity; a graph of g-force and dose along a trip. SSE4 has autopilot roles designed as pack programs (J6) and drift queued in V8. *Keep:* the log is a view of the campaign record, so it cannot drift from what happened; g-force and dose go on the voyage card. **Stage 5.**

## D. The map of your systems

**17. A campaign map of your own systems.** V3's home screen: every system in the campaign on one pan-and-zoom map with depth, routes drawn between them, Add system here, Add near here (by bearing, elevation and distance), Set depth, Centre map here with rings from that star, and hover a star to see what is in it. SSE4 has the galaxy, cluster and web charts, but systems are not yet placed in galaxies, so there is no map of *your* systems to go back to. *Do better:* one **Your systems** scale on the place line between galaxy and system (it is the patch chart, U6 part 2b), routes as authored relations, the hover peek that bodies already have, and the Traveller hex map as a view style of this same chart rather than its own screen. I would bring this **forward into stage 2**: for a GM it is the most used screen V3 has.

*Alex, 23 Nov:* agreed (the coordinator is explaining it further). Stage 2.

**18. Grids, snap and your own picture behind the stars.** V3: square, hex, subsector and Traveller grids, snap to grid, a sector map or empire borders pinned to map coordinates. *Keep:* grids and a pinned picture are layers in the view controls (rule 12) on item 17's map; nothing per screen. **Stage 5.**

**19. Stars that show what they do.** V3 draws flares, jets and shed shells on the star glyphs, and spreads glyph size by class. SSE4 computes activity and nebulae in the kernel. *Keep:* the chart symbology reads those facts. **Stage 4.**

**20. Import the real sky, and the Local Neighbourhood starter.** V3: regions by preset or centre and radius, star search by name (SIMBAD live), optional fill-out tagged as generated; two shipped maps (Local Neighbourhood and its science-fiction twin). The converter already does the catalogue import. *Keep:* the real-sky pack (Y8, X5) brings it in; Local Neighbourhood becomes a start door next to the Solar System; the science-fiction twin becomes a free fan pack, never paid. Star search becomes part of Find. **Stage 4.**

## E. At the table

**21. Hide this from players, and GM notes, on every card.** V3: an eye on every object (hide a secret base), a second eye for just the description, and private notes on every object. SSE4 does this better underneath (audiences, fog of war, data views, `gm.notes` never reaching a player scene), but there is no eye on a card. *Do better:* the eye is a shortcut that writes a rule into the audience's data view, so it is one mechanism, and shows who can see this card; the notes field sits on every card at every depth. **Stage 2**: it is too common to wait for stage 5.

**22. Your own pictures, with credit.** V3: a picture for any body, star or ship, with credit, licence and source, and an `ATTRIBUTIONS.md` in every save naming anything without provenance. *Keep:* assets in the campaign's own pack with credit fields, and an attributions page generated from the pack stack. **Stage 5.**

**23. Describe this world (AI) and a description on every object.** V3: Expand with AI on a body, with style, notes, tags and length, Accept and close. SSE4 has Minerva reading one audience's scene, but no button on a card. *Keep:* **Describe** on the card hands Minerva this world and its audience; the result is an authored description with its own visibility. **Stage 5.**

**24. Print a report, and save a player-safe copy.** V3: GM or Player reports in three themes, print or PDF; a spoiler-free export of the campaign. *Do better:* **Print** on any screen document, in the seat's theme, which is player-safe by construction because it prints what that audience's data view holds; **Save as seen by...** an audience gives the spoiler-free file. One mechanism, no report generator. **Stage 5.**

*Alex, 23 Nov:* not every screen. **Settled:** report screens built for the purpose (a GM briefing, a player survey sheet, a world dossier), shipped as screen documents and printable; the lens is the filter, so a player report shows only that audience's knowledge so far. "Save as seen by..." stays for the spoiler-free file. Stage 5.

**25. Branding, join message and quick overrides on player views.** V3: a faction name and logo across every view, an "Incoming transmission" join screen with the size coming, live Hide labels and Hide orbit lines that are never saved. *Keep:* branding as theme data, the join text as pack vocabulary, quick overrides as an unsaved toggle on the seat panel. **Stage 5.**

**26. Show it inside a virtual tabletop.** V3's Mappadux bridge and `?embed=1`, with an exact origin allowlist. *Keep:* re-specified on the V4 view contract, same allowlist rule. **Stage 5.**

## F. Looking after people's work

**27. Your data, and when a map will not load.** V3: storage used and "ask the browser to keep it", a recovery screen when a load never finished (try again, start without it, download the raw copy), Stop load naming the system it was on, a diagnostic zip that saves to your own machine, a memory warning, and Clear all data. SSE4 saves in the browser, with nothing for when that goes wrong. *Keep, small:* a **Your data** page under Workshop › Campaign (space used, keep request, download a backup, clear all), a recovery screen on a failed open, and a diagnostic file that Tars can read. **Stage 1**: testers will lose work without it, and it is cheap.

**28. Open the right way whatever file you give it.** V3 names a wrong file ("this is a campaign, open it as one?") by looking inside it. *Keep:* one Open door that sniffs V3 saves, V4 saves, clips, `.ubox` and `.sc` (through the converter) and says what each is. **Stage 1.**

**29. Credit the maker.** V3 lets you put your name, contact and version in a file, and keeps the chain of cartographers on anything pasted from Explorers. *Keep:* author fields in the campaign manifest; the credit chain rides in the clip. **Stage 5**, with Explorers V4 packs.

**30. A colour-blind-safe look.** V3's "Clarity" skin (Okabe-Ito palette, contrast up). SSE4 has the chart room and the night chart only, and the parity call to drop GM skins stands. *Do better:* no third skin; instead a colour-blind and contrast check on both GM themes and every player theme, in the theme tests, so every look is safe. **Stage 1** for the check. *Alex, 23 Nov:* yes.

**31. Help, guides and what's new.** V3: a Help hub with Getting started, the Tags and Autopilot guides, the physics reference and the changelog. *Do better:* the guides become tours (stage 6) and the physics reference is the flows themselves at Curious depth; what is left is one **Help** door in More with "What's new" from the stage-end notes. **Stage 1** for the door, **stage 6** for tours.

## G. Drop

**32. Toytown spacing, the orbit-line dial and the system overlay grids.** The log-scale chart (U18) already fits a whole system honestly, and view controls own the rest. *Drop.*

**33. Six GM skins and the skin editor; floating controls that fold to a puck and remember where you left them.** Replaced by the two GM themes, player themes, desks and the rail. *Drop* (item 30 keeps the part that matters).

**34. A low-power switch.** Capability tiers already decide what runs. *Drop*, unless testers on weak machines ask; then it is one "Lighter" toggle that lowers the tier.

*Alex, 23 Nov, on 32 to 34:* "the way we are going seems fine", dropped for now.

## H. Added by Alex, 23 Nov

**35. A gallery of every planet class.** V3 had a gallery screen of each planet type. Alex: put it on the classification pages, which "will have them all", with a selectable icon, photo or animation per class; unsure whether to use V3's old pictures or "our own imagery now, as it should be accurate". *Recommendation:* our own imagery. Each class's picture is a real world rendered from the physics: the class atlas picks a representative sample world for each class (a seed whose resolved world lands in the middle of the class's bands), and the gallery draws it three ways from one recipe, a flat icon (chart symbology), a still (globe render) and a slow turn (globe with time lapse). Nothing painted, so a picture can never disagree with the world. V3's pictures are not carried over (their licences are CC BY-SA and they show V3's classes). It also gives item 10 back what people liked about it. **Stage 2**, on the class atlas ("Names and classes" in Reference, coming-together question 4); the turning version waits on the renderer's globe in True for the best look, stage 6.

**36. Gas giants in every colour.** V3's gas giant recipes gave "rainbows of colours", and that variety must come along. *Recommendation:* not as copied recipes. In SSE4 a giant's colours come from its cloud decks and their chemistry: which condensates form at its temperature (ammonia, ammonium hydrosulphide, water, silicates and iron for hot giants, methane haze), chromophores and hazes as substances on the data sheet with absorption rows, and bands from the zonal winds. That is the same light-through-gas block as item 9 (ND2), so a giant's colour is computed, varied by temperature, metallicity and age, and explained by "why?". Check: a set of grown giants spans at least the palette of V3's recipes (V3's recipe list, read on Alex's machine, is the target to beat, not data to copy). **Stage 3**, light thread with the proven core (cloud decks) and the data sheet thread (chromophore rows).

**36a. The target, read from V3 (23 Nov, read only, `star-system-generator/src/lib`).** V3's "recipes" are already physics in shape, so they translate directly. A recipe (`catalogue/giantRecipe.ts`) is only an authored **gas mix and anchor pressure**, plus the **temperature it needs**, which V3 derives and never lets you set ("a giant is that colour because it is that cold"). The colour then comes from three steps:

1. **Trace gases by mass** (`physics/giantTraces.ts`): H₂ and He with CH₄, NH₃ and H₂S at solar baselines (0.00075, 0.000065, 0.00002) times an enrichment of 126·M⁻⁰·⁶ (M in Earth masses, clamped 1 to 120), which gives Jupiter about 4x, Saturn about 8x and Uranus about 25x. Each trace is jittered by 0.65 to 1.35 from the seed. He is 13% to 19%.
2. **Cloud decks by saturation** (`physics/cloudDecks.ts`): each condensable gas is tested level by level against its saturation pressure, the cloud base is the lifting condensation level, the column (kg/m²) becomes an optical depth (droplets of 10 µm, opaque at τ = 5), coverage is bucketed (wisps, scattered, broken, overcast, veil), and decks stack by base pressure, warmest-condensing deepest.
3. **Colour from the condensate** (`data/liquids.json`, `rendering/apparentColor.ts`, `planetAppearance.ts`): each condensate carries a hand-picked colour, a cloud opacity and albedo, and a "tint distance" from white (default 60 of 255). The visible giant is the top decks' tints over each other, plus banding.

The palette to match or beat (V3's condensate colours, all **hand-picked hex, not derived**: in SSE4 they must come from absorption and scattering rows on the data sheet, or be listed as fudges):

| Condensate | V3 colour | Where it forms in V3's lab |
|---|---|---|
| Ammonia (NH₃) ice | #c9b08a, cream | top deck of a Jupiter, 110 to 165 K |
| Ammonium hydrosulphide (NH₄SH) | #b8845a, brown, opacity 0.65 | deeper deck, the brown belts |
| Water | #2b6cb0 as a liquid, white as cloud | steam giant at 320 K, deepest deck of a cold one |
| Methane (CH₄) | #6a8caf | ice giants below about 90 K; the blue is CH₄ absorption above the decks |
| Hydrogen sulphide (H₂S) | #d8d090 | ammonia-poor, sulphur-rich giants |
| Sulphur dioxide (SO₂) | #ffffe0, pale yellow | "sulphurous" giant at 240 K |
| Sulphuric acid | #efe6c0 | "acid giant" at 290 K |
| Nitrogen | #cfe0ff | very cold N₂ and CH₄ giant, 100 K |
| Sodium, potassium | #d8d3c8, #cfc8bd, opacity 0.7 | hot Jupiters at 700 to 1100 K |
| Silicate (SiO) "molten glass" | #d98a3c, glowing | 1100 to 1400 K |
| Iron | #e8631f, glowing, opacity 0.75 | 1400 to 2400 K |
| Clear (nothing condenses) | the gas's own scattering | the hottest, or past every condensation point |

V3's giant lab rows are the check to rerun in SSE4, all with the same rule and no per-row authoring: **one Jovian chemistry cooled** (CH₄ 0.3%, NH₃ 0.026%, H₂S 0.008% at 220, 190, 165, 134, 110, 80 K: NH₄SH appears first and deepest, then NH₃, then CH₄); **where methane switches on** (2.3% CH₄ at 140 to 60 K: Saturn has no methane deck, Uranus does); **ice giants by methane** (0.5% to 8% CH₄ at 76 K: Uranus to Neptune blue); **other chemistries** (sulphurous, steam, ammonia-rich, acid, N₂ and CH₄); **hot Jupiters** (Na and K at 700 K up to SiO and Fe at 2400 K); **one air anchored at 0.1 to 20 bar**; and **one Jupiter growing old** (987 K at 10 Myr down to 108 K at 1 Gyr: young giants glow, which is how V3 answers "make a really hot giant").

What SSE4 should do better than V3: the colours themselves become physics (a condensate's colour from its refractive index and absorption rows through ND2; Jupiter's reds from a chromophore row, which V3 does not have, it tints NH₄SH brown by hand; ice giant blue from CH₄ absorption, not a hex); banding count from the zonal wind cells, not authored per gallery world (V3's gallery sets `banding: 3` to `9` by hand); and the age row follows from the core and interior heat history already in the kernel.

## Stage summary if all are accepted

- Stage 1 (test drive): 2, 27, 28, 30, 31 (door only).
- Stage 2 (one system in full): 1, 3, 4, 5, 6, 7, 8, 11, 12, 17 (brought forward), 21, 35.
- Stage 3: 9, 36. Stage 4: 19, 20.
- Stage 5 (run a game): 13, 15, 16, 18, 22 to 26, 29.
- Stage 6: 14, 31 (tours).
