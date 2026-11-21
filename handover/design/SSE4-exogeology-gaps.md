# SSE4 exogeology: what we cover, what we miss, and what to add

Research and design review for Alex, 21 Nov 2026. Companion to `SSE4-surface-design.md` (authority on how surfaces come from history) and `SSE4-rock3-comparison.md` (accepted 21 Nov). The additions proposed here go into that accepted design as its **section 5.9**, with questions 9 onward and new build-plan rows, so there is one surface plan, not two. Status: **accepted 21 Nov** (Alex: yes to all, on the recommendations; build order as in question 10, with section 10's changes).

Method: I read the surface designs and the built package logs (S0 to S7, W1 to W6), ran eight real Solar System bodies through the built surface code, then went world by world and process by process through the literature. Sources are in section 8 and marked **read** (I read the abstract and the relevant part) or **found** (title, abstract listing or a reliable index only). Consensus was not reachable from this thread, so Alex ran a research prompt through Consensus on his other Claude account. Its report (119 sources, 21 read) is kept at `exogeology/consensus-research-21nov.md` and folded in at section 10. That report was written by another Claude, so I treat it as research data: I reproduced its derived numbers (vapour pressures, the lava-pool edge, the potato radius, freezing extension), and they match. Its read and found marks are kept as its own, separate from mine. Numbers marked *(check)* must be verified against the paper before a builder uses them, as the citations rule requires.

---

## 1. Short answers

- **Would SSE4 do Pluto properly? No, not today.** I ran a Pluto stand-in through the built code. It is classed as an *ice shell* world, the Europa grammar, and gets 58 ridges and 8 bands about 3.2 Gyr old, 742 craters and nothing else. Real Pluto's signature landforms are made by **nitrogen, methane and carbon monoxide ices moving with the seasons**, and SSE4 has no such mechanism: no young crater-free nitrogen plain with convection cells, no glaciers of nitrogen, no bladed methane terrain, no sublimation pits, no tholin colours or haze, no cryovolcanoes, no freezing-ocean chasms. Section 2.
- **Do we have every Solar System feature?** About a third. The built regimes cover the big ones well (plates, stagnant lids with Tharsis and Valles, Io's paterae and mountains, Venus coronae and tesserae, Europa's ridges, bands and chaos, Titan's dunes and lakes, giants' bands, every crater shape). The missing two thirds cluster around **five processes**, not fifty special cases: volatile ices that sublime and condense; stresses in ice shells (tides, freezing, diapirs) and the eruptions they allow; water and ice released from inside a crust; small-body physics; and surface colour from radiation and dust. Section 3.
- **Theoretical features worth having?** Yes, several are well founded and cheap: lava-world shores with rock-vapour air and mineral frost on the night side, hemispheric tectonics on locked worlds, ice trapped on the night side of eyeball worlds, high-pressure ice sea floors under deep oceans, carbon worlds without plates, penitentes predicted on Europa. All are marked hypothetical. Section 5.
- **Bugs found on the way** (small, fix first): Enceladus and Miranda come out as lumpy small bodies, not round worlds; Callisto, the most cratered body in the Solar System, comes out with 587 young-ish ridges and loses 96% of its craters to "renewal"; Triton keeps 497 craters on a surface that is only tens of Myr old. Section 2.2.
- **Can a browser carry it?** Yes. Everything proposed is formulas on the grids we already step, and it fits the accepted detail levels: D1 gets volatile caps, plains and colours from an insolation average (cheap), D2 gets glaciers, convection cells and pit fields, D3 gets penitentes and pits as flourishes. Section 6.

---

## 2. Pluto and its kin through the built code

### 2.1 What the code makes today

Run on `main` (fc31eff) with the base-surface pack. The probe is a throwaway test, kept at `/mnt/project-files/sse4/exogeology/pluto_probe.rs` so a builder can rerun it; nothing was committed. Inputs are published masses, radii and mean surface temperatures; water fractions are rough (check). Heat flow is the built radiogenic model.

| Body | Regime given | Heat flow then → now (mW/m²) | Landforms made | Craters listed (erased) | Reality |
|---|---|---|---|---|---|
| **Pluto** | ice shell | 18 → 3.7 | 58 double ridges, 8 bands, ~3.2 to 3.5 Gyr | 742 (383) | Sputnik Planitia: a 1000 km nitrogen-ice plain with no craters, surface under ~10 Myr, convection cells 10 to 40 km; N2 glaciers; bladed terrain 500 m tall; pits; Wright Mons cryovolcano; extensional chasms; dark tholin Cthulhu; haze. No double ridges or bands |
| **Charon** | ice shell | 8.6 → 1.7 | 2 double ridges | 1122 (60) | a 1000 km belt of chasms from a freezing ocean, a smooth cryovolcanic plain (Vulcan Planitia), a red north pole |
| **Triton** | ice shell | 23 → 4.7 | 135 double ridges, 13 bands | 497 (682) | very young surface (perhaps ~10 to 100 Myr), cantaloupe terrain, active nitrogen geysers, a seasonal polar cap, walled cryovolcanic plains |
| **Enceladus** | **small body** (lumpy) | 203 (given) | none | 1140 | round, south polar tiger stripes with plumes, a crater-free south |
| **Miranda** | **small body** (lumpy) | 2.3 → 0.5 | none | 1201 | round, three huge coronae, a 10 km cliff (Verona Rupes) |
| **Ceres** | ice shell | 8.2 → 1.7 | 1 ridge, 1 band | 1090 (40) | no ridges; salt faculae in Occator, the lone dome Ahuna Mons |
| **Iapetus** | ice shell | 6.5 → 1.3 | 2 ridges | 1152 (22) | a 20 km equatorial ridge, one hemisphere ten times darker than the other |
| **Callisto** | ice shell | 36 → 7.4 | 587 ridges, 69 bands, 7 chaos | 46 (1141) | saturated with craters, no tectonics, crater rims worn into knobs by sublimation |

### 2.2 Why, and the small fixes

1. **Ice shell is the only door for a water-rich cold world.** The rule is `water_fraction ≥ 0.05` and nothing else, so every icy world gets Europa's grammar scaled by heat flow. That suits Europa and Ganymede, but Callisto, Ceres, Iapetus and Pluto are not cracking ice shells. *Fix*: the ice-shell rule also needs an active shell (tidal heat or heat flow above a threshold, or a modelled ocean under a thin shell). The rest go to a **cold ice lid** (the stagnant-lid grammar on ice material: craters kept, no ridges) and get their character from the new mechanisms below.
2. **Roundness ignores material.** `small-body` is `radius_km ≤ 300`. Lineweaver and Norman 2010 put the irregular-to-round transition at about 200 km for ice and 300 km for rock, because ice is weaker. *Fix*: threshold by material (water fraction), about 200 km for icy bodies. Enceladus (252 km) and Miranda (236 km) become round; Vesta (263 km, rock) stays lumpy.| **read**
3. **Renewal depends on today's heat flow alone.** Callisto's early radiogenic heat (36 mW/m²) drives the cube-law renewal that erases its craters. Triton's real heat came from tides after its capture (Agnor and Hamilton 2006), which the code cannot know. *Fix*: renewal only where the regime is actually active (point 1); a moon's **tidal history** as an input (imports and the Milky Way pack give it; formation fills it when F makes captures).
4. **No volatile ices at all.** The cover stage knows water ice only (cold traps at 165 K). N2, CH4, CO, CO2, NH3, SO2 and the hydrocarbons never appear as surface ice. This is the big one: section 4, mechanism M1.

These four are why Pluto fails. Points 1 to 3 are rule changes in `packs/base-surface` and S4, with golden changes on purpose. Point 4 is new physics.

---

## 3. The gap table

Legend: **built** (in `main` with tests), **designed** (in an accepted design, stage named), **gap** (neither). "Takes" says what it would need, by the mechanisms M1 to M9 in section 4. Every row cites at least one source in section 8.

### 3.1 Pluto, Charon, Triton and other cold volatile worlds

| Feature | Where | Physics | SSE4 | Takes |
|---|---|---|---|---|
| Nitrogen-ice plain in a basin, crater-free | Pluto Sputnik Planitia | N2 condenses where it is coldest and lowest; a basin at low latitude becomes a cold trap and grows by runaway albedo and altitude feedback (Bertrand and Forget 2016; Hamilton et al. 2016) | gap | M1 (equilibrium caps at D1) |
| Convection cells 10 to 40 km across | Sputnik Planitia | solid-state convection in a few-km-deep N2 ice layer heated from below; surface renewed in ~0.5 Myr (McKinnon et al. 2016; Trowbridge et al. 2016) | gap | M1 at D2: Rayleigh number from layer depth and N2 rheology; cell size ≈ a few × depth |
| Nitrogen glaciers flowing down valleys into the plain | Pluto east rim of Sputnik | N2 ice is soft enough to flow at 40 K (Yamashita et al. 2010; Umurhan et al. 2017) | gap | M1 at D2, with S3's downhill routing reused for ice |
| Bladed terrain, blades up to ~500 m, a few km apart | Pluto Tartarus Dorsa | methane penitentes: sublimation sharpens ridges in thin air over millions of years (Moores et al. 2017) | gap | M1 sublimation plus a flourish rule (S10) |
| Sublimation pits, pitted plains | Pluto north of Sputnik | sublimation of volatile ice with dark lag (Howard et al. 2017; Moore et al. 2017) | gap | M1 at D2/D3 |
| Seasonal transport, surface pressure changing by orders of magnitude over an orbit | Pluto, Triton | vapour-pressure equilibrium with the ice temperature, driven by obliquity and eccentricity (Bertrand et al. 2018) | gap | M1, reusing W1's Clausius-Clapeyron condensation code |
| Methane dunes | Pluto | wind lofting sublimated grains in very thin air (Telfer et al. 2018) | gap | M1 grains plus the existing dune flag |
| Dark red tholin belt, red pole | Pluto Cthulhu, Charon Mordor Macula | UV and charged-particle photolysis of methane makes tholins; on Charon methane escaped from Pluto is cold-trapped at the winter pole (Grundy et al. 2016) | gap | M7 (colour by chemistry and exposure) fed by M1 |
| Layered haze | Pluto | photochemistry in the upper air makes tholin haze layers (Cheng et al. 2017) | gap (W4 has haze from other sources) | W track: haze source from M1's methane |
| Cryovolcanic mountains with central depressions | Pluto Wright and Piccard Mons | large-scale cryovolcanic resurfacing, recent (Singer et al. 2022) | gap | M3 (cryovolcanism) |
| Extensional chasms, a global belt | Charon Serenity Chasma, Pluto | a freezing subsurface ocean expands, stretching the shell (Beyer et al. 2017) | gap | M2 (freezing stress) |
| Smooth cryolava plain | Charon Vulcan Planitia | flooding by a freezing ocean's last melt (Beyer et al. 2017) | gap | M3 |
| Reorientation of the world by a basin load | Pluto | a loaded basin rolls to the tidal axis; implies an ocean (Nimmo et al. 2016; Keane et al. 2016) | gap | M8 (true polar wander) |
| Cantaloupe terrain | Triton | solid-state diapirs in the ice crust (Schenk and Jackson 1993) | gap | M2 diapirs |
| Nitrogen geysers, dark streaks | Triton (also Mars spiders) | solid-state greenhouse: sunlight heats the base of translucent ice until gas bursts through (Soderblom et al. 1990; Kirk et al. 1990; Kieffer et al. 2006) | gap | M1 at D2, plus W4's plume code |
| Young surface | Triton | tidal heating after capture (Agnor and Hamilton 2006; Schenk and Zahnle 2007) | gap | fix 3 in 2.2 |
| Contact binary of two flattened lobes | Arrokoth | gentle merger in the collapse of a pebble cloud (McKinnon et al. 2020) | gap | M5 (small bodies) |

### 3.2 Inner Solar System

| Feature | Where | Physics | SSE4 | Takes |
|---|---|---|---|---|
| Plates, ridges, arcs, belts, plumes, rifts | Earth | plate kinematics | built (S2), more in S2b | (accepted design) |
| Shields, Tharsis, Valles Marineris, dichotomy | Mars | stagnant lid, plume, flexure | built (S4, S6, S0) | none |
| Outflow channels and chaos terrain | Mars | sudden release of pressurised groundwater from below a frozen cryosphere (Carr 1979) | gap | M4 (water released from a crust), on the accepted water-table block |
| Polar layered deposits | Mars | ice and dust laid down in layers paced by obliquity cycles (Laskar et al. 2002) | gap | M1 over deep time, writing S9 strata |
| Seasonal CO2 caps, "Swiss cheese" pits | Mars south cap | CO2 frost by season; pits retreat by metres a year (Thomas et al. 2000; Byrne and Ingersoll 2003) | gap (W2 has seasonal frost only as weather) | M1 |
| Spiders (araneiforms), dark fans | Mars south polar | CO2 jets under translucent slab ice (Kieffer et al. 2006) | gap | M1 solid-state greenhouse |
| Recurring slope lineae | Mars | seasonal dark streaks on warm slopes; first read as brine (McEwen et al. 2011), now mostly as dry granular flows (Dundas et al. 2017) | gap | M4 rule with the debate stated; low priority |
| Gullies, glaciers under debris, patterned ground | Mars, Earth | CO2 frost or melt on slopes; ice under rock cover; thermal contraction polygons (Levy et al. 2009) | gap (glacial factor in S3 only) | M1, M6 (periglacial rules) |
| Dust devil tracks, yardangs | Mars | wind and dust | designed (S3 wind, 5.3 wind abrasion) | none new |
| Contraction scarps | Mercury, Moon | cooling and shrinking | built (S6) | none |
| Hollows, fresh and bright | Mercury | loss of volatile-bearing (likely sulphide) rock to heat and vacuum (Blewett et al. 2011) | gap | M1 generalised to volatile-bearing rock |
| Polar ice in shadowed craters | Mercury, Moon | cold traps | built (S7 deposits) | none |
| Lava tubes, rilles, skylights | Moon, Mars | | designed (S14a) | none new |
| Lunar swirls | Moon | crustal magnetic fields shield the soil from solar wind darkening (Hood and Schubert 1980, *check*) | gap | M7 colour plus a crustal field |
| Coronae, tesserae, chasmata, global overturns | Venus | | built (S6) | none |
| Radar-bright "metal frost" on highlands | Venus | lead and bismuth sulphides condense where it is cooler at altitude (Schaefer and Fegley 2004) | gap | M7, one condensation rule on the S7 lapse rate |
| Canali (lava channels thousands of km long) | Venus | very fluid lava (perhaps carbonatite) eroding thermally (Kargel et al. 1994, *check*) | gap | S14a's flow routing extended to channels |
| Crater parabolas, wind streaks | Venus | fine ejecta carried west by the super-rotating wind | gap | S4 ejecta plus W1 winds, small |

### 3.3 Outer moons and dwarf planets

| Feature | Where | Physics | SSE4 | Takes |
|---|---|---|---|---|
| Paterae, block mountains, ballistic plumes | Io | heat pipe | built (S6, W4) | none |
| Lava lakes that overturn | Io Loki Patera | a crust that founders periodically (Rathbun and Spencer 2006) | gap | flourish rule on paterae |
| Volcano placement by tidal heating pattern | Io | heating in the mantle or the asthenosphere puts vents at the poles or the equator (Hamilton et al. 2013) | gap (vents placed at random) | M2's tidal stress field |
| Double ridges, bands, chaos | Europa | | built (S6, fixed 45° trend) | none |
| Cycloids | Europa | cracks that grow while the diurnal tidal stress rotates (Hoppa et al. 1999) | gap (S6 known limit) | M2 tidal stress field |
| Lenticulae (pits, domes, spots) | Europa | diapirs in a convecting shell (Pappalardo et al. 1998, *check*) | gap | M2 diapirs |
| Penitentes, metre scale (predicted) | Europa | sublimation by sunlight at low latitudes (Hobley et al. 2018); contested | gap | M1 flourish, **hypothetical** label |
| Grooved terrain | Ganymede | extension of the ice shell | built as bands, roughly | none |
| Knobby, sublimation-worn craters | Callisto | sublimation of ice with dark lag on slopes (Moore et al. 1999) | gap (and fix 1 and 3 in 2.2) | M1 at D2 |
| Tiger stripes and plumes | Enceladus | tidally timed opening of south polar rifts (Hurford et al. 2007; Porco et al. 2006) | gap (and fix 2) | M2 plus M3 |
| Coronae, 10 km cliffs | Miranda | upwelling diapirs and extensional tilt blocks (Pappalardo et al. 1997) | gap (and fix 2) | M2 diapirs |
| Equatorial ridge, up to ~20 km | Iapetus | debated: despinning, a collapsed ring, convection (Dombard et al. 2012, *check*) | gap | M8, **debated** label |
| Albedo dichotomy, leading side dark | Iapetus | dust from an outer moon starts a thermal runaway of ice migration (Spencer and Denk 2010) | gap | M7 fed by M1 |
| Salt faculae in a young crater | Ceres Occator | brine from below, left as carbonate and salt (De Sanctis et al. 2016) | gap | M3 plus S15's impact hydrothermal systems |
| A lone young cryovolcanic dome | Ceres Ahuna Mons | brine-ice extrusion (Ruesch et al. 2016) | gap | M3 |
| Lakes, seas, dunes | Titan | | built (S6) | none; karst in S14b |
| Leading and trailing hemisphere colours | Europa, Galilean and Saturnian moons | sulphur and radiation on the trailing side, dust on the leading side | gap | M7 |

### 3.4 Small bodies and comets

| Feature | Where | Physics | SSE4 | Takes |
|---|---|---|---|---|
| Lumpy shapes | asteroids | | built (S0, lumps too large: S6 known limit) | fix with M5 |
| Top shapes with an equatorial ridge | Bennu, Ryugu, many binaries | spin-up by sunlight (YORP) moves rubble to the equator; spin limits bind it (Walsh et al. 2008) | gap | M5 |
| Ponds of fine dust, boulder fields, Brazil-nut sorting | Eros, Itokawa, Bennu | shaking and electrostatic transport on low gravity | gap | M5 cover rules |
| Binaries and moonlets from spin-up | small asteroids | rotational breakup (Walsh et al. 2008) | gap | M5, an event in the chronicle |
| Active pits, cliffs, outbursts | comets (67P) | sinkhole collapse over sublimating voids (Vincent et al. 2015) | gap | M1 plus M5 |

### 3.5 Exoplanets (theory, all marked hypothetical)

| Feature | Predicted for | Physics | SSE4 | Takes |
|---|---|---|---|---|
| Day-side magma ocean, a shore, night crust | ultra-short-period rocky planets | | built (S0 lava sketch, locked) | none |
| Rock-vapour air, supersonic winds, mineral rain and night frost (SiO, Na) | K2-141b, 55 Cnc e | the magma ocean evaporates; vapour flows to the cold side and condenses (Kite et al. 2016; Nguyen et al. 2020) | gap | M1's condensation code with rock vapours, plus a lava-shore rule |
| One hemisphere of upwelling, the other of downwelling | locked super-Earths (LHS 3844b) | the day-night temperature contrast organises mantle convection (Meier et al. 2021) | gap | M9 regime dial on locked worlds |
| Eyeball: open sea at the substellar point, ice elsewhere | habitable-zone planets of M dwarfs | (Pierrehumbert 2011) | gap (S7 notes "no locked pattern") | W7 locked climate plus S7 cover |
| Night-side ice sheets that flow back to the day side | locked planets with little water | water trapped as ice on the night side; glacier flow sets the balance (Menou 2013; Leconte et al. 2013) | gap | M1's ice flow reused for water ice |
| Ocean over a high-pressure ice floor | deep water worlds | ice VII and other phases form under kilometres of water, cutting the sea from rock (Noack et al. 2016, *check*) | gap (ocean world has a rock floor) | regime note in S0, floor material from pressure |
| Hycean: ocean under hydrogen air | sub-Neptunes (K2-18b debated) | (Madhusudhan et al. 2021) | open question with Alex (C2) | classification first, then S0 |
| Carbon worlds without plates, diamond in the mantle | high C/O systems | graphite and diamond change mantle strength and heat flow (Unterborn et al. 2014) | built as speculative regime; no plate rule | regime note |
| Tidally heated Earths with global volcanism | LP 791-18 d | eccentricity tides from a neighbour (Peterson et al. 2023) | **built**: tidal heat flow feeds the heat-pipe and lid rules | test fixture only |
| Plates or no plates on super-Earths | | research divided (Valencia et al. 2007 against O'Neill and Lenardic 2007) | built as a dial | none |

---

## 4. Recommended mechanisms, in priority order

Rule kept throughout: general mechanisms that make many features, each feature able to say why it is there, real physics with any fudge in `docs/fudges.md`, nothing heavier than formulas on the grids we already step.

### M1. Volatile ices: sublime, condense, flow and convect (new package S17, top priority)

**What it is.** One rule block for every ice that can move as vapour on its world: N2, CH4, CO, CO2, Ar, NH3, SO2 and H2O, plus rock vapours on lava worlds. Each species carries a vapour-pressure curve (pack data from Fray and Schmitt 2009, the same Clausius-Clapeyron form W1 already uses for clouds), a latent heat, an albedo and a flow law where one is published (N2: Yamashita et al. 2010; CO2 and H2O: standard ice laws, *check*).

**What it computes, by detail level** (Alex's "best calc is no calc"):
- **D0**: whether the world has mobile volatiles at all: any species whose vapour pressure at the surface temperature lies between about 1e-9 and the air's pressure. One line per species. Gives Pluto, Triton, Eris, Mars and Io their "volatile world" tag and colour hint.
- **D1**: **equilibrium caps and plains** from the insolation averaged over the orbit and the obliquity cycle (the obliquity history is already in T1): ice sits where the averaged energy balance is coldest, deepened in basins because they are lower and the column is thicker. This gives Sputnik Planitia (Bertrand and Forget 2016), Triton's cap, Mars's residual caps and Iapetus's dark side's ice loss without stepping seasons. Surface pressure follows the coldest ice (the vapour-pressure equilibrium). Cost: a latitude-band calculation per species.
- **D2**: **seasons and flow**. Seasonal transport in a few dozen steps per orbit (Bertrand et al. 2018's approach, simplified to latitude bands and basins); glacier flow down S3's routed valleys with the species' flow law; **convection cells** where a volatile layer is thick enough that its Rayleigh number passes the critical value, with cell size a few times the layer depth (McKinnon et al. 2016) and a renewal age that removes craters (so Sputnik is crater-free for the right reason); sublimation pit fields and lag deposits (Howard et al. 2017); polar layered deposits as dated S9 strata paced by the obliquity cycles (Laskar et al. 2002); solid-state greenhouse jets under translucent slab ice, drawn as plumes through W4's existing plume code (Kieffer et al. 2006; Soderblom et al. 1990).
- **D3**: **penitentes and pits as flourishes** (S10): blade spacing and height from the sublimation rule and the air (Moores et al. 2017), so Pluto gets 500 m blades and a predicted Europa field is labelled hypothetical (Hobley et al. 2018).

**Features from this one block**: Sputnik Planitia and its cells, N2 glaciers, bladed terrain, pitted plains, methane dunes (with the dune flag), Triton's cap and geysers, Mars's CO2 caps, Swiss-cheese terrain, spiders, polar layered deposits, Callisto's knobs, Mercury's hollows (with sulphide as the volatile), comet pits (with M5), Iapetus's ice migration (with M7), night-side ice on locked worlds (water as the species), mineral frost on lava worlds (rock vapours). Around twenty features.

**Why is this here?** "Nitrogen ice about 4 km deep, collected here because this basin, 3 km below its surroundings near the equator, is where nitrogen condenses most over Pluto's 2.8 Myr obliquity cycle (104° to 127°). Heated from below, it convects in cells about 30 km wide, moving a few cm a year and renewing the surface in about half a million years, so no crater older than about 10 Myr survives." (numbers from McKinnon et al. 2016 and Bertrand et al. 2018, read)

### M2. Ice-shell stresses (package S18a)

**What it is.** A stress field on an ice shell from three sources: diurnal tides on an eccentric orbit, non-synchronous rotation, and **freezing of the ocean below** (water expands as it freezes, so a thickening shell stretches its surface). Plus diapirs: buoyant warm ice rising through a convecting shell. Built on the S0 heat flow, the T1 orbit and the S6 landform shapes.

**Features**: cycloids and correct ridge trends (replacing S6's fixed 45°), Enceladus's tiger stripes and their tidal timing, Charon's and Pluto's chasms, Ariel's and Tethys's canyons, Io's vent pattern, lenticulae, Miranda's coronae and tilt blocks, Triton's cantaloupe terrain. Also **viscous relaxation** of craters in warm ice (an S4 known limit), so large old craters on Ganymede and Enceladus flatten as observed.

### M3. Cryovolcanism and brines (package S18b)

**What it is.** Eruption of water, brine or ammonia-water when an ocean is pressurised by freezing or tides and a crack reaches it (Manga and Wang 2007), or when buoyant brine rises (Ceres). Edifice size from the eruption volume and the ice strength, like S4's shields with ice gravity and rheology.

**Features**: Wright and Piccard Mons (Singer et al. 2022), Vulcan Planitia, Ahuna Mons, Occator's faculae (with S15's impact hydrothermal systems), Enceladus's plumes (with M2), Triton's walled plains, Titan's possible cryovolcanoes (labelled speculative).

### M4. Water and ice released from a crust (package S21, after the accepted water-table block)

**What it is.** Groundwater confined under a frozen cryosphere until a crack, impact or heat releases it in a flood. Uses the water table rule Alex accepted (Rock 3 question 5), S3's routing and S9's strata.

**Features**: Martian outflow channels and chaos terrain (Carr 1979), the Martian gullies; recurring slope lineae with the debate stated (McEwen et al. 2011 against Dundas et al. 2017). Earth's megafloods (glacial lake outbursts) come out of the same rule.

### M5. Small-body physics (package S20)

**What it is.** Shape from strength and gravity (the potato radius, fix 2), spin evolution by YORP, spin limits and rotational breakup (Walsh et al. 2008), regolith movement on low gravity (ponds, Brazil-nut sorting), contact binaries from gentle mergers (McKinnon et al. 2020), and cometary activity from M1. Fixes S6's oversized lumps on the way.

**Features**: top shapes, equatorial ridges on rubble piles, binaries and moonlets as chronicle events, dust ponds, boulder fields, Arrokoth-like contact binaries, comet pits and cliffs.

### M6. Periglacial rules (small, inside S9 and S10)

Thermal contraction polygons, patterned ground, frost shattering (already in 5.3), solifluction: where ground ice exists and the temperature crosses a threshold often (W1 seasons). Earth, Mars and Pluto (Levy et al. 2009).

### M7. Surface colour from chemistry and exposure (package S19)

**What it is.** S7's colour currently comes from cover and material only. Add: photolysis of methane into tholins (dark red) by the UV and particle dose the O10 activity track already computes; radiation darkening and reddening over time (space weathering); exogenic dust swept up on the leading side of a moon; sulphur on Io; and condensation colours (Venus's metal frost on the highlands).

**Features**: Cthulhu, Charon's red pole, Iapetus's dichotomy (with M1's thermal runaway), Europa's and the Saturnian moons' hemisphere colours, lunar swirls (with a crustal field), Venus's radar-bright summits. Cheap: a colour term per tile, and every colour can say why.

### M8. Reorientation and spin history (small, with S2b)

True polar wander when a large load (a basin filled by M1, a volcanic rise) moves off the axis (Nimmo et al. 2016); despinning fossil shapes. Rotating the surface frame is a dated event; features keep their addresses. Iapetus's ridge is offered as a **debated** option only.

### M9. Exoplanet regime additions (small, in S0 and S6)

Lava-world shore with rock-vapour air and night frost (uses M1); locked hemispheric tectonics as a dial (Meier et al. 2021); eyeball cover on locked habitable worlds (needs W7's locked climate); high-pressure ice floors under deep oceans; carbon worlds marked without plates (Unterborn et al. 2014). Hycean waits on Alex's classification answer (C2).

---

## 5. Theoretical features worth having (all labelled hypothetical or debated)

| Feature | Status | Why include it |
|---|---|---|
| Penitentes on Europa, ~15 m tall at low latitudes | predicted (Hobley et al. 2018), contested | a landing hazard for any Europa mission story; falls out of M1 for free |
| Mineral frost and rock rain on lava worlds | predicted (Nguyen et al. 2020) | the most alien weather we can justify; uses M1's condensation |
| Hemispheric tectonics on locked worlds | modelled (Meier et al. 2021) | a "why is all the volcanism on one side?" story |
| Night-side ice and glaciers flowing toward the day side | modelled (Menou 2013; Leconte et al. 2013) | where the water is on a dry locked world, and a reason for a terminator settlement |
| Ice VII sea floors | modelled | a world whose ocean never touches rock: changes the life suite's nutrient budget |
| Diamond-rich mantles, no plates | modelled (Unterborn et al. 2014) | gives the carbon world regime real consequences |
| Iapetus-style ridges from a collapsed ring | debated | ring collapse already exists in F5 (a broken moon), so it can be a dated cause |

---

## 6. Browser and detail-level fit

- **D0**: one vapour-pressure check per species, regime and colour hints. Microseconds.
- **D1**: equilibrium volatile caps and plains from an orbit-averaged insolation by latitude band, the fixed regime rules, M7 colour. Well inside the accepted 0.2 s D1 budget (estimate).
- **D2**: seasonal stepping by latitude band and basin, glacier flow on S3's existing routing, convection cells and pit fields as landform lists, M2 stresses evaluated on the D2 grid. Same order as S3 today (estimate, measure).
- **D3**: penitentes, pits, ponds and cryovolcanic vents as S10 flourishes, only at sites.
- Nothing needs a fluid simulation. Seasonal volatile transport in a real GCM (Forget et al. 2017) stays a swappable big block if ever wanted, like weather.

---

## 7. Questions, each with my recommendation (accepted 21 Nov)

9. **A new "volatile" regime, or a layer on top of regimes?** *Recommendation*: a **layer** (M1) that runs on every solid world, not a regime, because Mars (stagnant lid), Triton (ice shell) and Pluto (cold ice lid) all need it. The regime change is only fix 1: an ice-shell rule that needs an active shell, and a "cold ice lid" for the rest.
10. **Order.** *Recommendation*: the four small fixes in 2.2 first (with S16a, since they change the same goldens), then **S17a (M1 at D0 and D1)** right after S14a, then S18a and S18b (ice shells and cryovolcanism), S17b (M1 at D2 and D3), S19 (colour), S20 (small bodies), S21 (water released from a crust), M8 and M9 as they come. Placed in the accepted order in the Rock 3 proposal's question 2.
11. **Mars recurring slope lineae: show them, and as what?** *Recommendation*: include as dry granular flows by default (Dundas et al. 2017) with brine as a GM switch, and say in "why?" that the cause is debated.
12. **Pluto as a test fixture.** *Recommendation*: add Pluto, Charon, Triton, Enceladus, Miranda, Ceres, Iapetus and Callisto to `tests/fixtures/surface/bodies.json` with their expected regimes, and checks per mechanism: Sputnik-like basin plain crater-free, cells at the right size, Callisto saturated, Enceladus round with south polar stripes. Pluto becomes the showcase world for S17, the way Earth is for S2.
13. **Debated mechanisms (Iapetus's ridge, Europa's penitentes, hycean).** *Recommendation*: offered as labelled options with the competing explanations in "why?", default on only where the physics in our own blocks produces them.

---

## 8. Sources

**Read** means I read at least the abstract and the relevant part; **found** means located by title and listing only. Numbers marked *(check)* in the text must be verified before use. Seven key papers were read for exact numbers; those numbers are in section 9.

| Source | Topic | Status |
|---|---|---|
| McKinnon et al. 2016, Nature 534, 82. doi:10.1038/nature18289 | N2 convection, Sputnik Planitia | **read** |
| Trowbridge et al. 2016, Nature 534, 79. doi:10.1038/nature18016 | Sputnik convection | found |
| Bertrand and Forget 2016, Nature 540, 86. doi:10.1038/nature19337 | N2 condensation in Sputnik | found |
| Bertrand et al. 2018, Icarus 309, 277. arXiv:1804.02434 | Pluto N2 cycles over seasons and obliquity | **read** |
| Hamilton et al. 2016, Nature 540, 97. doi:10.1038/nature20586 | early Sputnik formation | found |
| Nimmo et al. 2016, Nature 540, 94. doi:10.1038/nature20148 | reorientation, ocean | found |
| Keane et al. 2016, Nature 540, 90. doi:10.1038/nature20120 | reorientation and tectonics | found |
| Umurhan et al. 2017, Icarus 287, 301 (journal details from the listing, not printed in the preprint). arXiv:1606.05734 | N2 glacial flow | **read** |
| Yamashita et al. 2010, Icarus 207, 972 (ADS 2010Icar..207..972Y) | solid N2 rheology | found |
| Moores et al. 2017, Nature 541, 188. doi:10.1038/nature20779 | Pluto penitentes | **read** |
| Howard et al. 2017, Icarus 287, 287 | Pluto pits and mantles | found |
| Moore et al. 2017, Icarus 287, 320 | sublimation on Pluto | found |
| Telfer et al. 2018, Science 360, 992 | Pluto dunes | found |
| Singer et al. 2022, Nature Communications 13, 1542. arXiv:2207.06557 | Pluto cryovolcanism | found |
| Cheng et al. 2017, Icarus 290, 112. arXiv:1702.07771 | Pluto haze | found |
| Grundy et al. 2016, Nature 539, 65. doi:10.1038/nature19340 | Charon's red pole | found |
| Beyer et al. 2017, Icarus 287, 161 | Charon tectonics | found |
| Fray and Schmitt 2009, Planet. Space Sci. 57, 2053 | vapour pressures of ices | found |
| Schenk and Jackson 1993, Geology 21, 299 | Triton diapirs | found (DOI to check) |
| Soderblom et al. 1990, Science 250, 410. doi:10.1126/science.250.4979.410 | Triton geysers | found |
| Kirk et al. 1990, Science 250, 424 | energy for Triton's plumes | found |
| Agnor and Hamilton 2006, Nature 441, 192. doi:10.1038/nature04792 | Triton capture | found (from memory, check) |
| Schenk and Zahnle 2007, Icarus 192, 135 | Triton surface age | found (check) |
| Porco et al. 2006, Science 311, 1393. doi:10.1126/science.1123013 | Enceladus south pole | found |
| Hurford et al. 2007, Nature 447, 292. doi:10.1038/nature05821 | tidal timing of Enceladus rifts | found |
| Hoppa et al. 1999, Science 285, 1899. doi:10.1126/science.285.5435.1899 | Europa cycloids | found |
| Hobley et al. 2018, Nature Geoscience 11, 901. doi:10.1038/s41561-018-0235-0 | Europa penitentes (predicted) | **read** |
| Pappalardo et al. 1997, JGR 102, 13369. doi:10.1029/97JE00802 | Miranda tilt blocks | found |
| Spencer and Denk 2010, Science 327, 432. doi:10.1126/science.1177132 | Iapetus dichotomy | found |
| Dombard et al. 2012, JGR 117, E03002 | Iapetus ridge from a subsatellite | found (arXiv:1105.1685) |
| Laskar et al. 2002, Nature 419, 375. doi:10.1038/nature01066 | Mars polar layered deposits | found |
| Kieffer et al. 2006, Nature 442, 793. doi:10.1038/nature04945 | Mars CO2 jets, spiders | found |
| Thomas et al. 2000, Nature 404, 161 | Mars Swiss-cheese terrain | found (from memory, check) |
| Byrne and Ingersoll 2003, Science 299, 1051 | Swiss-cheese pit growth | found (from memory, check) |
| McEwen et al. 2011, Science 333, 740. doi:10.1126/science.1204816 | recurring slope lineae | found |
| Dundas et al. 2017, Nature Geoscience 10, 903. doi:10.1038/s41561-017-0012-5 | RSL as granular flows | found |
| Carr 1979, JGR 84, 2995 | Mars outflow channels | found (from memory, check) |
| Levy et al. 2009, JGR 114, E01007 | thermal contraction polygons | found (from memory, check) |
| Blewett et al. 2011, Science 333, 1856. doi:10.1126/science.1211681 | Mercury hollows | found |
| Schaefer and Fegley 2004, Icarus 168, 215 | Venus metal frost | found |
| De Sanctis et al. 2016, Nature 536, 54. doi:10.1038/nature18290 | Ceres faculae | found |
| Ruesch et al. 2016, Science 353, aaf4286. doi:10.1126/science.aaf4286 | Ahuna Mons | found |
| Moore et al. 1999, Icarus 140, 294 | Callisto sublimation degradation | found |
| Rathbun and Spencer 2006, GRL 33, L17201 (arXiv:astro-ph/0605240) | Loki Patera | found |
| Hamilton et al. 2013, EPSL 361, 272 | Io volcano placement | found |
| Lineweaver and Norman 2010, arXiv:1004.1091 | potato radius | found |
| Walsh et al. 2008, Nature 454, 188. doi:10.1038/nature07078 | YORP breakup, top shapes | found |
| Vincent et al. 2015, Nature 523, 63. doi:10.1038/nature14564 | comet sinkholes | found |
| McKinnon et al. 2020, Science 367, eaay6620. doi:10.1126/science.aay6620 | Arrokoth origin | found |
| Manga and Wang 2007, GRL 34, L07202. doi:10.1029/2007GL029297 | pressurised oceans, eruptions | found |
| Kite et al. 2016, ApJ 828, 80. arXiv:1606.06740 | atmosphere-interior exchange on hot rocky planets | found |
| Nguyen et al. 2020, MNRAS 499, 4605. arXiv:2010.14101 | K2-141b rock vapour | found |
| Meier et al. 2021, ApJL 908, L48. arXiv:2103.02374 | hemispheric tectonics | **read** |
| Pierrehumbert 2011, ApJL 726, L8. doi:10.1088/2041-8205/726/1/L8 | eyeball climates | found |
| Menou 2013, ApJ 774, 51. arXiv:1304.6472 | water-trapped worlds | found |
| Leconte et al. 2013, A&A 554, A69. arXiv:1303.7079 | locked planets, ice on the night side | found |
| Madhusudhan et al. 2021, ApJ 918, 1. arXiv:2108.10888 | hycean worlds | found |
| Unterborn et al. 2014, ApJ 793, 124. arXiv:1311.0024 | carbon in planetary geodynamics | found |
| Peterson et al. 2023, Nature 617, 701. doi:10.1038/s41586-023-05934-8 | LP 791-18 d tidal heating | found |
| Valencia et al. 2007 (ApJL 670, L45) and O'Neill and Lenardic 2007 (GRL 34, L19204) | super-Earth plates debate | already in the base-surface pack |
| Hood and Schubert 1980; Kargel et al. 1994; Noack et al. 2016; Pappalardo et al. 1998 | swirls, canali, ice VII floors, lenticulae | named only, check |
| SSE4 `main` fc31eff: `packs/base-surface/pack.json`, `crates/sse-surface`, package logs S0 to S7 | built behaviour | read, and run (2.1) |

---

## 9. Numbers from the papers read (for builders; check each against the published version)

**McKinnon et al. 2016** (N2 convection): cells typically 10 to 40 km across; a 3 to 6 km layer if the aspect ratio is 2 to 3, 10 to 20 km if isoviscous; N2 layers of about 1 km or more convect. Flow law ε̇ ∝ σ^n with n = 2.2 ± 0.2 (Yamashita et al. 2010), A = 3.73e-12 Pa^-2.2 s^-1, E* = 3.5 kJ/mol; for n = 1, E* = 8.6 kJ/mol. Critical Rayleigh number (stagnant lid) 2038 for n = 1 and 310 for n = 2.2. Diffusivity 1.33e-7 m²/s, expansivity 2e-3 /K, density 1000 − 2.14 (T − 36 K) kg/m³, conductivity 0.2 W/m/K, surface 36 K, melting 63 K; 3 mW/m² gives about 15 K/km. Surface speeds a few cm a year, renewal about 500,000 years, crater retention age at most about 10 Myr.

**Bertrand et al. 2018** (Pluto N2 cycles): obliquity is the main driver; it runs 104° to 127° over 2.8 Myr (now 119.6°), precession 3.7 Myr, eccentricity 0.222 to 0.266 over 3.95 Myr. Sputnik Planitia lies about 3 km below its surroundings, so its higher condensation temperature traps N2 and CO. Over an obliquity cycle 25° S to 30° N in the basin is dominated by condensation, 30° to 50° N by sublimation (about 1 km lost at the northern edge in the last 2 Myr). N2 is stable in equatorial depressions, not at the poles. Surface pressure 1 to 1.2 Pa in 2015, tripled since 1988; over Myr it ranges from millipascals to pascals. CO follows N2; CH4 forms its own deposits and seasonal frosts on warmer ground. Thermal inertia reference 800 SI.

**Umurhan et al. 2017** (N2 glaciers): ε̇ = A(T) σ^n, n ≈ 2.1 + 0.0155 (T − 45 K), A45 = 0.005 s^-1 MPa^-n, A(T) = A45 exp(Ta/45 K − Ta/T) with Ta = 422 K; viscosity about 1.6e10 Pa s at 45 K and 10 kPa. Flow is laminar for layers under 400 to 1000 m; convection starts near 1.2 km for 2 to 3 mm grains when the base reaches 63.1 K. Topographic imprints relax in a few decades (so glaciers respond to seasons, not aeons). Some constants differ between the paper's sections; check before use.

**Moores et al. 2017** (Pluto penitentes): observed spacing 3 to 5 km and depth about 500 m; their CH4 model peaks at a 3 km wavelength (laminar sublayer 87 m); N2 makes no penitentes. Penitentes need pressure above about 0.1 Pa (Pluto 2015: 1.1 Pa). They deepen about 1 cm per orbit now, so 500 m takes a few tens of Myr. Albedo 0.5, light penetration 1.6 cm. The read text gives no closed scaling of spacing with pressure; spacing follows the laminar sublayer thickness, which grows as pressure falls.

**Hobley et al. 2018** (Europa penitentes, predicted): spacing up to about 7.5 m and depth up to about 15 m after 50 Myr near the equator; sublimation (about 0.3 m/Myr) beats sputtering (about 0.02 m/Myr) within about ±24° of the equator; shallower and lopsided further out. Noon temperature 134 K, albedo about 0.67.

**Lineweaver and Norman 2010** (roundness): R_pot ≈ 200 km for icy bodies and about 300 km for rocky ones; R_pot ∝ (σ_y)^½ / ρ, with σ_y about 10 MPa for rock and 0.4 to 3 MPa for ice. This replaces the single 300 km rule.

**Meier et al. 2021** (locked super-Earths): three outcomes by lithosphere strength and heating. Weak lid with basal heating: no hemispheric pattern. Strong lid: downwelling on the day side, upwelling on the night side. Weak lid with mixed heating: upwelling on the day side. Melt and outgassing gather on the upwelling side. Modelled for LHS 3844b (day side about 1040 K). A dial with three settings, labelled modelled.

---

## 10. What the Consensus research adds and changes (21 Nov, afternoon)

Source: `exogeology/consensus-research-21nov.md`, written by Claude on Alex's other account using Consensus. Its statuses: READ means that Claude read the abstract plus a relevant section or the full text, FOUND means title or abstract only, (derived) means computed by it from a cited relation, (recalled) means not checked against a paper. I checked its derived values by recomputing them:
- N2 at 36.93 K comes out at 1.15 Pa;
- the CO2 frost point is 148 K at 633 Pa;
- lava-pool edges are 61°, 76° and 84°;
- the potato radius is 233 km for 10 MPa and 2.5 g/cm³;
- about 35 km of frozen water gives 1% extension on Charon.

All of these agree. Its other numbers and DOIs are taken as given and marked "via Consensus research" in the table below.

### 10.1 Numbers that are now ready for builders (S17 and S18)

| Need | What it gives | Status there |
|---|---|---|
| Vapour pressure per ice | Fray and Schmitt 2009 polynomials for N2 (alpha and beta), CO, CH4 and CO2 (coefficients via Lisse et al. 2021, Table 2), plus two-constant fits ln P[Pa] = a − b/T within 1 to 7%, cheap enough for D0 and D1. N2: a 22.505, b 824.8 K (36 to 63 K); CH4: 22.427, 1181.9 K; CO2: 27.919, 3181.7 K | coefficients READ, fits derived (I reproduced them) |
| Pressure sensitivity | d ln p / d ln T = b/T, about 22 for N2 at 37 K: a 4% change in absorbed sunlight moves pressure about 25% | derived |
| Energy balance of an ice surface | εσT⁴ = (1−A) S cos i + L ṁ + F_cond; N2 ice albedo 0.72 to 0.73, emissivity 0.8 and thermal inertia 700 to 900 SI fit Pluto's record (Meza et al. 2019). New Horizons gives emissivity 0.47 to 0.72 (Lewis et al. 2020) | READ / FOUND |
| Poles or equator | obliquity decides. Above about 54° the poles get more yearly sunlight than the equator. The same model with only the orbit changed gives Pluto (low-latitude ice) and Triton (polar caps) (Bertrand et al. 2024) | READ; the 54° is recalled, standard geometry |
| Basin trap | frost point rises about 0.09 K per km of depth on Pluto | derived |
| Convection onset and cells | Ra criteria (n = 1: 1101 to 2038; n = 2.2: 200 to 310) and onset wavelength 2.34 D. Sluggish-lid cells 4 to 6 D, up to 12 D with a free-slip base. Cell centres stand 40 to 150 m above their edges | READ |
| Glacier flow | Umurhan et al. 2017 flow law; shallow-ice slab speeds 0.3 m/yr (50 m, 1°) to 2 km/yr (200 m, 10°). A 200 m channel drains in decades to centuries. N2 and CH4 fail at about 9 to 10 MPa, so km-high relief needs water-ice bedrock | READ, speeds derived |
| Pit growth | Pluto pits widen 0.36 mm/yr; Mars "Swiss cheese" walls retreat 1 to 3 m/yr; Mercury hollows at most about 100 kyr old and up to 19,110 mapped | FOUND |
| Jets | a basal warming under 4 K lifts Triton's plumes to 8 km. Mars slab lift-off at about 6 kPa per metre of slab. Spiders only in cryptic terrain on the polar layered deposits | FOUND, lift-off derived |
| Eruption of water | ΔP ≥ (ρw − ρi) g H; without overpressure water stops about 10% of the shell below the surface. Cracks cross Enceladus's shell when it is under 15 km (Rudolph et al. 2022) | derived / FOUND |
| Edifices | Wright Mons 4 to 5 km tall and 150 km wide; Piccard 7 km and 225 km. Ceres domes relax away in 0.1 to 1 Gyr, so only one sharp dome survives (Sori et al. 2017) | READ / FOUND |
| Freezing extension | ε_area ≈ 2 (ρw/ρi − 1) h/R; Charon's 1% needs about 35 km of frozen water | derived (I reproduced it) |
| Diapir spacing | Triton cantaloupe cells 47 km apart over a crust about 20 km thick (about 2.4 × thickness) | FOUND, ratio derived |
| Crater relaxation in ice | Ganymede craters of 10 km and more need 40 to 50 mW/m² for about 1 Gyr. Large craters on Ganymede and Callisto go shallow in 0.2 to 1 Gyr under radiogenic heat. Pit craters become dome craters within 10 Myr. Relaxation time 4πη/(ρgλ) (textbook) | FOUND |
| Haze colour | haze settles everywhere and shows on old surfaces; renewed ice hides it (Grundy et al. 2018). So **colour = haze flux × surface age**, which M7 can compute from S4's ages | FOUND |
| Iapetus runaway | works only where slow rotation gives warm days for the albedo (Spencer and Denk 2010) | FOUND |
| Lava worlds | pool edge cos θ = (1673 K / T_ss)⁴; above about 2400 K substellar the surface fractionates by evaporation (Na, K, Fe, then Si, Mg, Ca, Al). Vapour pressures of Na, SiO and SiO2; winds about 2 km/s; atmosphere collapses near 90° to 100° from the substellar point (Kite et al. 2016; Nguyen et al. 2020) | READ |
| Carbon worlds | diamond mantle when (Mg + 2Si + Fe + 2C)/O > 1, which makes convection sluggish or stagnant (Unterborn et al. 2014) | FOUND |

### 10.2 What it changes in this proposal

1. **Convection cells do not need interior heat.** Sublimation cooling at the top can drive them with a basal flux of only about 0.3 mW/m² (Morison et al. 2021, read there). S17b therefore tests the Rayleigh number for both drives: cells form wherever a volatile layer is thicker than about 0.5 to 1 km. *Change to M1.*
2. **Glaciers are near steady state.** Imprints relax in decades. S17b therefore sets glacier thickness from the balance of supply and flow, rather than stepping glaciers in time. That makes the volatile block cheaper. *Change to M1, D2.*
3. **Penitentes need air.** On airless Europa the vapour is in free-molecular flow and there is no boundary layer (Hand et al. 2019, read there). The reply (Hobley et al. 2019, read there) keeps only a radiative analogue. So the gate is: penitentes form where the mean free path is much smaller than the spacing. Pluto is on by default; Europa becomes an off-by-default GM option labelled contested. *Change to M1, D3, and to question 13.*
4. **Enceladus erupts by boiling, not by overpressure.** Rudolph et al. 2022 say so against Manga and Wang 2007. M3 therefore keeps two rules apart: when a crack crosses the shell (thickness under about 15 km on Enceladus), and what drives the eruption (decompression boiling by default, overpressure where it suffices). *Change to M3.*
5. **An old surface does not rule out an ocean.** Mimas has a young ocean under a cratered face (Lainey et al. 2024). Fix 1 in 2.2 must therefore keep two questions apart: "is there an ocean?" (heat and tides) and "is the shell active at the surface?" (stress against strength). Mimas is the test case: cold ice lid on top, ocean below. *Change to fix 1.*
6. **Late cryovolcanism with no tidal source is real.** Pluto erupted more than 10⁴ km³ with 5 mW/m² or less (Singer et al. 2022, read there). M3 therefore allows stored-heat eruptions, labelled as an open problem in "why?".
7. **Lava worlds can keep volatile air.** 55 Cancri e shows CO2 or CO (Hu et al. 2024), so the rock-vapour-only atmosphere is one case and not the rule. Near M dwarfs, bare dark rock is a live default (TRAPPIST-1 b, Ih et al. 2023). *Change to M9.* Io keeps a mostly solid mantle despite its heating (Park et al. 2025), so the heat-pipe regime must not imply a shallow magma ocean. **Check S6:** it does not today; it writes paterae over a solid lid.
8. **New bedform class.** Sublimation waves have their own scaling laws (Carpy et al. 2023): a flourish kind for S10 on Mars and Pluto, low priority.
9. **Methane on Eris and Makemake may come from inside** (Grundy et al. 2024; Glein et al. 2024), against a primordial origin (Mousis et al. 2025). M1 can take a supply term from the interior heat history, labelled debated.

### 10.3 Status changes in section 8

These are marked **read (Consensus research)** where that report read them:
- Bertrand et al. 2018: Icarus 309, doi:10.1016/j.icarus.2018.03.012; read by both.
- McKinnon et al. 2016: read by both.
- Umurhan et al. 2017: doi:10.1016/j.icarus.2017.01.017; read by both.
- Singer et al. 2022: doi:10.1038/s41467-022-29056-3.
- Kite et al. 2016: doi:10.3847/0004-637X/828/2/80.
- Nguyen et al. 2020: doi:10.1093/mnras/staa2487.
- Lineweaver and Norman 2010: read by both.

It also gives DOIs I lacked:
- Howard et al. 2017: 10.1016/j.icarus.2017.02.027;
- Moore et al. 2017: 10.1016/j.icarus.2016.08.025;
- Byrne and Ingersoll 2003: 10.1126/science.1080148;
- Beyer et al. 2017: 10.1016/j.icarus.2016.12.018;
- Cheng et al. 2017: 10.1016/j.icarus.2017.02.024;
- Fray and Schmitt 2009: 10.1016/j.pss.2009.09.011;
- Meier et al. 2021: 10.3847/2041-8213/abe400;
- Kirk et al. 1990: 10.1126/science.250.4979.424;
- Hofgartner et al. 2022: 10.1016/j.icarus.2021.114835;
- Fagents 2003: 10.1029/2003JE002128.

These DOIs are as that report gives them, and it says it confirmed each against Crossref, OpenAlex, Semantic Scholar or arXiv. I have not re-checked them; the citations track should do that before the rows go into `docs/citations.md`.

Newly found and worth reading before S17 and S18 are built:
- Meza et al. 2019, doi:10.1051/0004-6361/201834281 (read there);
- Bertrand et al. 2022 (Triton), doi:10.1016/j.icarus.2021.114764 (read there);
- Bertrand et al. 2024, doi:10.1073/pnas.2408226121 (read there);
- Morison et al. 2021, doi:10.1038/s41586-021-04095-w (read there);
- Hand et al. 2019, doi:10.1038/s41561-019-0496-2 (read there);
- Rudolph et al. 2022, doi:10.1029/2021GL094421;
- Lainey et al. 2024, doi:10.1038/s41586-023-06975-9;
- Park et al. 2025, doi:10.1038/s41586-024-08442-5;
- Hu et al. 2024, doi:10.1038/s41586-024-07432-x.

Its full table of 119 sources is in the kept report.

### 10.4 Disagreements it records (kept as labelled options, per question 13)

- **What drives Sputnik's convection**: heat from below, cooling by sublimation at the top, or internal heating.
- **How thick Sputnik's ice is**: 3 to 6 km or about 10 km.
- **Charon's chasms**: a freezing ocean, against expansion and contraction episodes with no ocean needed.
- **Triton's geysers**: driven by sunlight or by internal heat.
- **Ahuna Mons**: a brine and ice cryomagma, or a mud slurry.
- **Occator's brines**: a deep reservoir or impact melt.
- **Mercury's hollows**: lost sulphur or lost carbon.
- **Charon's red pole**: photolysis alone, or radiolysis and ethane clustering as well.
- **Super-Earth plates**: as already dialled.

Each becomes the "why?" text's list of competing causes, with our default named.
