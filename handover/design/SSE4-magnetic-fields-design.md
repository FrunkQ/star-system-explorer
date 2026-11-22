# SSE4 magnetic fields: made from fundamentals, with consequences, drawn so you can see why

Written 22 Nov 2026 at Alex's request. Status: **accepted 22 Nov 2026** (Alex, 01:06: "agree to all - the shielding is an effect - not a switch and not likely the whole picture as earth field varies over history - so again - stories flow from these facts"). Every recommendation in section 13 is a ruling, and the stages in section 11 are in the build plan as track **MF**. It completes B1d (the one dynamo law, part 1 on main), takes the field half of MP20 (matter pack design 4.3A, "dynamo regions found, not handed over"), and gives the overlay that nebulae-and-light section 5 and star census section 3 promised a home.

Alex's asks, 22 Nov, in his words:

- "magnetic fields too... where are we with them, their generation from fundamentals, and their visualisation"
- "research mag field stuff... it is the edge of astrophysics and blamed for most things including flaring, etc"
- (on the data sheet, minutes earlier) "The ice that created Neptune's mag field should be calculable, and worked out from that table."

Standing rulings this follows: lean into the physics and never fake it; processes, not classes; one general mechanism over specials; every block cited, every fudge registered; detail levels Glance, Orbit, Survey, Site with "best calc is NO calc"; time is first-class; every feature answers "why is this here?"; a tag never feeds physics; dynamic minimalism.

---

## 1. Short answers

1. **Where we are.** The law for how strong a dynamo's field is exists as three editable blocks and is tested against Earth and Jupiter. Nothing yet feeds it from a world's own interior, so **no generated world has a field today**. The magnetopause, the stellar wind and auroral radio are built and work when a field is handed to them. Stars have activity, cycles, flares and ejections, but no field as such. There is no field shape, no history, no belts model, no aurora picture and no overlay. Section 2 has the full table.
2. **Fundamentals.** One mechanism can cover planets, moons, giants, ice giants, stars and remnants: every conducting region keeps a **field ledger** with three terms: *make* (a dynamo, if the region is stirred hard enough), *keep* (the field leaks away on a time set by size and conductivity) and *squeeze* (field is carried with matter when it shrinks or grows). Which term wins is computed, never chosen by type (section 4).
3. **The research changes the scope in four places** (section 3): (a) slow spin is not why Venus has no field, lack of core cooling is, so the dynamo must be tied to each world's heat history; (b) whether a magnetic field protects an atmosphere is genuinely disputed, so escape is computed channel by channel and "shield" is never a switch; (c) field *strength* is predictable to a factor of about two, field *shape* far less, so shape carries an "argued" mark; (d) cycle lengths, flare timing and reversals cannot be predicted by anyone, so they are drawn from measured statistics and say so.
4. **Visualisation.** One "Field" layer in every view style: inked field lines that continue down into the cutaway to the layer that makes them, the bubble shaped by the wind, belts as stipple whose gaps are explained by the moons that sweep them, aurora ovals on the globe, and in True style only what an instrument would really see (aurora glow, a giant's radio belts, a star's corona following its field). Scrub time and the field switches on, flips and dies (section 9).
5. **Two quick fixes found on the way.** NAV gives radiation belts only to bodies over 200 Earth masses, so Saturn and Earth have none and every giant has the same dose. The Sun's cycle comes out at 6 years because the Sun really does sit between the two measured branches; no theory predicts cycle lengths (section 3.2).

---

## 2. Where we are (repo checked at main 204e425, 22 Nov)

| Thing | State | Where |
|---|---|---|
| Dynamo field strength from heat flux and density (Christensen, Holzwarth and Reiners 2009) | **Built** as blocks, flow `sse.base.body-magnetism`, tests for Earth, Jupiter and "no heat, no field" | `crates/sse-atoms/src/magnetic.rs`, `packs/base-physics/flows/body-magnetism.json`, log `handover/packages/B1.md` (B1d part 1) |
| The flow's inputs (dynamo density, radius, convected heat) | **Handed in by tests only.** No kernel code runs the `body.magnetism` hook for a grown or imported world | nothing calls it |
| Stellar wind (Parker), magnetopause stand-off, auroral radio power and top frequency | **Built**, in `read_light`. The moment is an input; only the Solar System fixture supplies real ones | `sse-atoms/src/light.rs`, `sse-light/src/emit.rs` (O2) |
| Stellar activity from the Rossby number: X-rays, spots, cycle, flares, mass ejections that hit or miss planets | **Built** (O10 first slice). No field value is published; the cycle gives the Sun 6.1 years | `sse-activity`, `sse-atoms/src/activity.rs` |
| Star surface cells, spots drawn on stars, X-ray corona | **Built** (O10 convection, R5a). Spot places are drawn from a seed, corona has no field structure | `packages/renderer/src/star.ts` |
| "Dynamo on" on the cutaway | An **authored label** on a fixture layer, not computed | `sse-world`, `tests/fixtures/worlds/earth.json` |
| Radiation belts as a hazard | **Stand-in**: belts if mass ≥ 200 Earth masses, out to 15 radii, 50 mSv per hour for all | `sse-nav/src/hazards.rs`, `packs/base-nav/pack.json` |
| Radiation dose for life | Life **reads** a dose; nothing computes it from a field | `sse-life/src/habitat.rs` |
| V3 imports | V3's field, magnetosphere and aurora fields are kept as authored values; nothing draws them | `sse-import-v3/src/classify.rs` |
| Field shape (tilt, offset, multipoles), field over time, crustal fields, induced fields, aurora ovals, dose maps, compass | **Missing** | |
| Remnant fields (white dwarfs, pulsars, magnetars), fossil fields of hot stars | **Missing** (census marks magnetars "missing") | |
| Field overlay | **Missing.** The renderer has R2c waiting on field data; the UI layout doc says "field overlay on the Globe later" | `handover/packages/R5a.md` |
| Designed, not built | Dynamo layer found from phase conductivity (MP19, MP20, matter pack 4.3A, PR 122); the list of what every body publishes (census section 3); aurora, radio and translated sound (light 4A, sound N6) | design docs |

Fudges already on the register that this design retires: F-S29 (dipole share tuned to two planets), F-S30 (all heat drives the dynamo), the disc inner edge fixed at 8 days for every star (growth.rs:401), spot places from a seed (R5a), isotropic auroral radio (O2), spin from age by Skumanich (O10).

**What V3 did** (read from `star-system-generator/src/lib/physics/magnetism.ts`, `magnetosphere.ts`, `aurora.ts`). Its dynamo was a lookup by interior type with ranges calibrated to the real planets and a rotation factor, so Venus was unshielded "because rotation is far too slow". Its magnetosphere work was good and is worth keeping in spirit: one pressure-balance law, Shue's shape for the outline, a moon inside its host's bubble feeling the host's field and not the wind, the aurora oval as the footprint of the last closed field line, shading the *useful* closed region and only outlining the full extent, and a readable (log) size in 3D. V4 keeps those ideas, replaces the lookup with physics, and adds everything in sections 4 to 9.

---

## 3. What the science says, and how sure it is

Grades used below and on every card: **settled** (measured and understood), **empirical** (measured, cause not agreed), **argued** (leading explanation, live rivals), **open** (nobody knows). The design scope follows the grade: settled and empirical things are computed; argued things are computed behind a swappable block and marked; open things are drawn from measured statistics or left to the GM, and say so.

### 3.1 Planets and moons

| Question | Grade | What is known | Source |
|---|---|---|---|
| What makes a field | settled | A fluid that conducts electricity, stirred by convection, in a rotating body. Growth must beat leakage: the magnetic Reynolds number Rm = u D / η has to pass about 40 to 50 | Stevenson 2003 [P1]; Christensen and Aubert 2006 [P2] |
| How strong | empirical, good | Field energy follows the power available to the flow, B² ∝ ρ^⅓ q^⅔, not spin rate or conductivity. Fits Earth, Jupiter and fast-spinning stars over many orders of magnitude | Christensen, Holzwarth and Reiners 2009 [P3]; Christensen 2010 [P4] |
| Tidy dipole or a mess | argued | Simulations turn from dipolar to many-poled when the local Rossby number passes about 0.12. Real planets need more: stable layers and shell shape matter | Olson and Christensen 2006 [P5]; Christensen and Aubert 2006 [P2] |
| Why Venus has none | argued | Not slow spin: even at 243 days its core would be "fast" in dynamo terms. The core is not losing heat fast enough to convect (no plate tectonics, perhaps no inner core) | Stevenson 2003 [P1]; Driscoll and Bercovici 2014 [P6] |
| Why Mars lost its field | empirical | The crust records a field from about 4.5 to 3.7 billion years ago; the giant basins that formed later are unmagnetised. The core stopped convecting as the planet cooled | Acuña et al. 1999 [P7]; Lillis et al. 2013 [P8]; Mittelholz et al. 2020 [P9] |
| How Earth kept its field for 3.5 billion years | **open** | Iron conducts heat so well that a young, fully liquid core struggles to convect (the "new core paradox"). Proposed fixes: magnesium or silica coming out of solution, a molten rock layer acting as its own dynamo. The core's conductivity itself is disputed by a factor of several | Pozzo et al. 2012 [P10]; Labrosse 2015 [P11]; O'Rourke and Stevenson 2016 [P12]; Stixrude et al. 2020 [P13]; Tarduno et al. 2010 [P14] |
| Mercury's weak field, Saturn's perfectly aligned one | argued | A stable, conducting layer above the dynamo hides most of the field and smooths away everything not symmetric about the spin axis | Christensen 2006 [P15]; Anderson et al. 2011 [P16]; Dougherty et al. 2018 [P17] |
| Jupiter | empirical | Juno found a field unlike any model: mostly dipolar in the north, with a strong patch near the equator | Connerney et al. 2018 [P18]; Moore et al. 2018 [P19] |
| Uranus and Neptune | **argued, thinly measured** | Tilted 59° and 47°, off-centre, many-poled. One flyby each. Explanations: a thin convecting shell of ionic water over a stable interior; superionic ice below; layers of ices that will not mix. Uranus has almost no heat flow yet a field as strong as Neptune's, which no model explains comfortably | Ness et al. 1986, 1989 [P20]; Stanley and Bloxham 2004 [P21]; Redmer et al. 2011 [P22]; Millot et al. 2019 [P23]; Soderlund and Stanley 2020 [P24]; Militzer 2024 [P25] |
| Ganymede (a moon with its own dynamo) | argued | Iron "snow" in an iron and sulphur core drives it by composition, not heat | Kivelson et al. 1996 [P26]; Hauck et al. 2006 [P27] |
| Europa, Callisto (induced fields) | settled | A salty ocean in Jupiter's changing field carries currents that make a small field of their own. This is how the oceans were found | Kivelson et al. 2000 [P28]; Zimmer et al. 2000 [P29] |
| The Moon and small bodies | argued | The Moon had a field for a billion years or more, at times oddly strong. Some meteorite parent bodies a few hundred km across ran dynamos for a few million years | Tikoo et al. 2017 [P30]; Weiss et al. 2010 [P31]; Bryson et al. 2015 [P32] |
| Reversals | **open** for timing | Earth flips a few times per million years on average, with quiet spans of tens of millions of years. Simulations reverse; none predicts when | Glatzmaier and Roberts 1995 [P33] |
| Exoplanet fields | **open** | No secure measurement. Candidates: radio bursts from YZ Ceti in step with its planet; hot Jupiters argued to have fields of tens to over a hundred gauss from their extra heat | Pineda and Villadsen 2023 [P34]; Cauley et al. 2019 [P35]; Yadav and Thorngren 2017 [P36] |

### 3.2 Stars

| Question | Grade | What is known | Source |
|---|---|---|---|
| Why cool stars are magnetic | settled in outline | Spin plus a convecting outer layer. Activity rises as the Rossby number falls, then saturates below about 0.13 | Noyes et al. 1984 [S1]; Wright et al. 2011 [S2] |
| How strong the surface field is | empirical, good | Measured in 292 red dwarfs: the mean field rises as Rossby number falls and at saturation matches the convective energy available, the same limit as the planetary law | Reiners et al. 2022 [S3]; Kochukhov 2021 [S4] |
| Where in the star the dynamo sits | **open** | Fully convective stars with no boundary layer follow the same activity law as the Sun, which upset the leading model | Wright and Drake 2016 [S5]; Charbonneau 2020 [S6]; Brun and Browning 2017 [S7] |
| Cycle length (the Sun's 11 years) | **open** | No theory predicts a cycle length. Measured stars fall on two loose branches and the Sun sits between them. Our 6-year answer is the lower branch, not a code fault | Böhm-Vitense 2007 [S8]; Charbonneau 2020 [S6] |
| X-rays and the corona | empirical | X-ray power follows total magnetic flux nearly linearly from quiet Sun patches to the most active stars. How the corona is heated is not agreed | Pevtsov et al. 2003 [S9]; Klimchuk 2006 [S10] |
| Flares | settled cause, **open** timing | Stored magnetic energy released by reconnection. Sizes follow a power law, as in a sand pile. The largest possible flare is set by the biggest spot group and its field | Shibata and Magara 2011 [S11]; Benz and Güdel 2010 [S12]; Aschwanden et al. 2016 [S13]; Shibata et al. 2013 [S14] |
| Mass ejections on very active stars | **open** | On the Sun big flares nearly always eject. On red dwarfs a strong overlying field may hold ejections in; very few have been seen | Alvarado-Gómez et al. 2018 [S15] |
| Winds and spin-down | empirical | The magnetised wind brakes the star (spin falls roughly as the square root of age), but braking appears to stall near the Sun's present Rossby number. Winds of other stars are measured for only a few dozen | Weber and Davis 1967 [S16]; Matt et al. 2015 [S17]; van Saders et al. 2016 [S18]; Wood et al. 2021 [S19]; Vidotto et al. 2014 [S20] |
| Hot stars (no convecting outer layer) | empirical, origin **open** | About 7% of A, B and O stars have strong, steady "fossil" fields of 0.3 to 30 kG that do not depend on spin. Mergers are one proposed origin | Donati and Landstreet 2009 [S21]; Wade et al. 2016 [S22]; Braithwaite and Spruit 2004 [S23]; Schneider et al. 2019 [S24] |

### 3.3 Discs, jets and how systems form

| Question | Grade | What is known | Source |
|---|---|---|---|
| A young star's field and its disc | empirical | Kilogauss fields hold the disc off at a few stellar radii and channel gas onto the star along field lines; this sets where the inner disc ends and helps set the star's birth spin | Koenigl 1991 [D1]; Hartmann, Herczeg and Calvet 2016 [D2] |
| What makes disc gas fall inward | **argued, shifting** | Twenty years of "magnetic turbulence" has given way to magnetised winds carrying spin away, with large dead zones. Still not agreed | Balbus and Hawley 1991 [D3]; Bai and Stone 2013 [D4]; Lesur et al. 2023 [D5] |
| The solar nebula's field | empirical | Meteorites record about 50 µT in the inner disc, gone by about 4 million years | Fu et al. 2014 [D6]; Weiss et al. 2021 [D7] |
| Jets | argued | Launched by field threading a disc or a spinning black hole. The Event Horizon Telescope has imaged ordered field at M87's ring | Blandford and Payne 1982 [D8]; Blandford and Znajek 1977 [D9]; EHT 2021 [D10] |

### 3.4 Magnetospheres, air, radiation, aurorae

| Question | Grade | What is known | Source |
|---|---|---|---|
| Where the bubble ends, and its shape | settled | Pressure balance between the field and the wind; shape from fits to thousands of crossings | Shue et al. 1997 [M1]; Kivelson and Russell 1995 [M2] |
| **Does a field protect an atmosphere?** | **disputed** | Earth, Venus and Mars lose ions at similar rates. A field stops some loss channels, opens others at the poles, and makes the planet a bigger target for the wind's energy. Mars lost its air mainly because it is small. Under a young star's violent wind the answer may differ | Gunell et al. 2018 [M3]; Ramstad and Barabash 2021 [M4]; Jakosky et al. 2018 [M5]; Egan et al. 2019 [M6] |
| Shielding from cosmic rays | settled | A dipole turns away charged particles below a cut-off energy that depends on latitude. For a world with thick air, the air is the bigger shield; the field matters most for thin air, for orbit and for the upper atmosphere's chemistry | Smart and Shea 2005 [M7]; Atri et al. 2013 [M8]; Grießmeier et al. 2005 [M9] |
| Radiation belts | settled in outline | Every strongly magnetised planet has them. Intensity is capped by a self-limiting instability that Earth, Jupiter and Uranus reach; Saturn's are thinned by rings, moons and gas | Kennel and Petschek 1966 [M10]; Mauk and Fox 2010 [M11]; Li and Hudson 2019 [M12] |
| Aurorae | settled | Power from the wind's coupling to the field; the oval is the edge of the open polar cap; colours are the air's own emission lines. Mars has patchy aurorae over old crustal fields | Newell et al. 2007 [M13]; Feldstein and Starkov 1967 [M14]; Bertaux et al. 2005 [M15] |
| Auroral radio | empirical | The same small share of the wind's power comes out as radio at every magnetised planet | Zarka 2007 [M16] (in the code) |
| Moons feeding a magnetosphere | settled | Io's volcanoes load a tonne a second into Jupiter's field and light a spot in its aurora | Goldreich and Lynden-Bell 1969 [M17]; Bagenal and Delamere 2011 [M18] |
| Crustal fields and lunar swirls | argued | Old magnetised crust holds off the wind over small patches, keeping soil bright | Hood and Schubert 1980 [M19] |

### 3.5 Remnants

| Question | Grade | What is known | Source |
|---|---|---|---|
| White dwarf fields | empirical, origin **open** | About a fifth of nearby white dwarfs are magnetic, 10³ to 10⁹ G, and the share rises with age. Origins proposed: fossil, a past merger, or a dynamo when the core crystallises (argued over) | Ferrario et al. 2015 [R1]; Bagnulo and Landstreet 2021 [R2]; Isern et al. 2017 [R3] |
| Pulsars | settled for spin-down, **open** for the radio beam | Field from spin and spin-down rate; a spinning dipole loses energy at a known rate. How the radio beam is made is unsolved after 55 years | Goldreich and Julian 1969 [R4]; Spitkovsky 2006 [R5]; Philippov and Kramer 2022 [R6] |
| Magnetars | argued | 10¹⁴ to 10¹⁵ G, probably from a dynamo in the first seconds of a fast-spinning newborn. The field decays in 10⁴ to 10⁶ years and powers bursts and giant flares; one was caught making a fast radio burst | Duncan and Thompson 1992 [R7]; Kaspi and Beloborodov 2017 [R8]; Viganò et al. 2013 [R9]; Raynaud et al. 2020 [R10]; CHIME/FRB 2020 [R11]; Hurley et al. 2005 [R12] |
| Black holes | settled | None of their own. Any field is held there by the disc | [D9] |

### 3.6 Galaxies and beyond

| Question | Grade | What is known | Source |
|---|---|---|---|
| Galactic fields | empirical | A few microgauss, following the spiral arms, about as much energy as the gas turbulence. Maintained by a large dynamo | Beck 2016 [G1]; Brandenburg and Subramanian 2005 [G2]; Jansson and Farrar 2012 [G3] |
| What they do | settled | Hold cosmic rays in, make the galaxy glow in radio, shape nebulae, help decide whether clouds collapse | Crutcher 2012 [G4] |
| Where the first field came from | **open** | Seeds from the early universe or from the first stars | Subramanian 2016 [G5]; Neronov and Vovk 2010 [G6] |
| The star's bubble in the galaxy | settled in outline | The wind's own field keeps out most low-energy cosmic rays; the dose at a world depends on the star's wind and on where it is in the galaxy | Potgieter 2013 [G7] |

### 3.7 What this means for scope

- **Compute, as fact:** field strength of planets, moons and cool stars; whether a layer can run a dynamo; when it starts and stops over a world's history; bubble size and shape; cosmic-ray cut-off and dose; belt limits; aurora power, place and colour; radio; spin-down; induced fields; pulsar spin-down.
- **Compute behind a swappable block, marked "argued":** field shape (dipole or many-poled, tilt, offset); ice giants; Mercury and Saturn (stable layers); white dwarf crystallisation fields; magnetar birth; what a field does to air loss; ejections from very active stars; hot Jupiter fields.
- **Draw from measured statistics, marked "drawn":** cycle length within the two branches; which hot stars have fossil fields; reversal dates; flare and storm dates (already done this way in O10); which way a tilt points.
- **Describe only:** how discs accrete, how jets launch in detail, the pulsar radio mechanism, where the first fields came from. These get a card that says what is and is not known, and no pretend number.

---

## 4. The mechanism: one field ledger for every conducting region

**The idea.** Anything that conducts electricity can hold a magnetic field. What happens to that field is one equation (induction) with three terms. Every body's interior is already a list of layers (the phase profile of MP19; a star's zones from its life line). Each conducting layer keeps a ledger:

| Term | Plain meaning | Number that decides it | Who it dominates |
|---|---|---|---|
| **Make** | stirring stretches and folds field faster than it leaks | magnetic Reynolds number Rm = u D / η above about 50, and somewhere to get energy from (convected heat or sinking and rising material) | Earth, Jupiter, Neptune, the Sun, red dwarfs, crystallising white dwarfs, newborn neutron stars |
| **Keep** | with no stirring, field leaks away on a set time | decay time τ = D² / (π² η) | hot stars' fossil fields (τ far longer than the star's life), old neutron stars, a dead planetary core (τ of thousands of years, so the field is gone at once on a geological clock) |
| **Squeeze** | field is carried with the matter when it shrinks or grows | flux Φ = B R² stays fixed | a star's core becoming a white dwarf or neutron star; a wind carrying field out past the planets |

A fourth, separate thing is **rock that remembers**: minerals that cool below their Curie temperature while a field is present keep a record of it (section 6.7). That is material memory, not a ledger term, and is read from the data sheet's mineral rows.

η is the magnetic diffusivity, 1 / (μ₀ σ), from the layer's phase row on the data sheet (electrical conductivity σ, matter pack 4.3A). u is the convective speed, D the layer's thickness. Nothing in this table is a class: iron, metallic hydrogen, ionic water, salty ocean, stellar plasma and a GM's unobtainium are rows with different σ.

**Why this is "one general mechanism" and not six specials.** A Sun-like star and Earth differ only in which layer passes the Make test and how fast it spins compared with its stirring. A magnetar is Make for ten seconds, then Squeeze, then Keep. Mars is Make for 800 million years, then nothing, plus rock that remembers. Europa's layer fails Make but, sitting in Jupiter's changing field, carries induced currents (section 6.6).

### 4.1 Make, step by step (blocks, each editable and cited)

1. **Is there energy to stir with?** `sse.dynamo.convected_flux`: the heat a layer passes minus what it can carry by conduction alone along its adiabat, q_conv = q − k α g T / c_p. If negative, the layer is stably layered and does not convect by heat. k, α and c_p come from the phase row. This retires F-S30. [P4], [P11]
2. **Any other stirring?** `sse.dynamo.compositional_power`: a freezing inner core pushing light elements up, iron snow, helium rain, magnesium coming out of solution. Each is a power per area that adds to step 1; each reads the phase boundaries it crosses. [P11], [P12], [P27]
3. **How fast does it stir?** `sse.dynamo.flow_speed`: from the available power, rotation and layer depth (Christensen and Aubert's scaling for rotating shells; mixing-length speed for a star's envelope, which the convection blocks of O10 already give). [P2]
4. **Does field grow?** `sse.dynamo.reynolds`: Rm = u D / η. `sse.dynamo.onset`: strength 0 to 1 rising through Rm about 40 to 60. This is a real threshold in nature (a bifurcation), so a narrow edge here is honest; its width is the spread seen in simulations. [P2]
5. **How strong?** The existing `sse.magnetic.dynamo_field`, fed q_conv plus compositional power instead of all heat, times onset. [P3], [P4]
6. **Spin regime.** `sse.dynamo.rossby`: spin period over stirring time. Planets all have tiny values (even Venus), so step 5 stands alone. Stars run from tiny to large: above saturation the field falls as about Ro^−1.3. One curve, one block, so planets and stars are the same calculation. [S3]
7. **Shape.** Section 5.
8. **Out to the surface.** The existing `sse.magnetic.surface_dipole`, with its tuned share (F-S29) replaced by the shape block's dipole share and by damping through any stable conducting layer above.

**Giants need no typed dynamo radius.** In Jupiter conductivity rises smoothly with depth. Walking down the hydrogen row, the dynamo's top is where local Rm passes onset. The "0.85 of the radius" in today's test becomes a result. Saturn's comes out deeper because it is colder, and helium rain (MP21) puts a stable layer over it.

**Ice giants.** Ionic water conducts about a thousand times worse than iron, so Rm is only marginal and the dynamo region is a shell. What lies under it (convecting, stably layered, or superionic and nearly solid) decides the shape. The base pack follows Stanley and Bloxham with Redmer's phase layers; Militzer's unmixed layers is a second interior block a GM can swap in. The card says both are argued and that Uranus's low heat flow is not explained by either.

### 4.2 Heat history: fields that switch on and die

`sse.core.history`: a small energy-and-entropy budget for a rocky world's core through time [P6], [P11]. Inputs already exist or are being built: heat leaving through the mantle (P1 heat budget and the plate regime of S2/S2b, because a world with plate tectonics cools its core faster than one with a stagnant lid), radioactive heat, tidal heat, and the iron row's melting curve (MP21) for when an inner core starts to freeze. Output: for every epoch, convected flux, compositional power, inner core size, and so the ledger's answer.

- It is one ordinary differential equation per world over a few hundred time steps, well under a millisecond. It runs once when a world's interior is first resolved, is cached, and every detail level reads the same history. Budget and timing are part of the package's evidence.
- Because Earth's early dynamo is an open problem (3.1), the block exposes the disputed number (core thermal conductivity, on the data sheet with its range) and ships with the magnesium route on, so that Earth keeps a field for 3.5 billion years as the rock record requires. Listed on the fudge register as a choice among published models, not as settled.
- Giants and brown dwarfs: the cooling law in `self_light` (B1a) already gives heat flow through time. Stars: the life line gives the envelope. So "time is first-class" costs nothing new there.
- Events for the record and the time ruler: dynamo start, inner core birth, dynamo death, each with a why ("the core's heat loss fell below what conduction alone carries, 3.9 billion years ago").

### 4.2A The field's history is a set of dated facts (Alex, 22 Nov: "stories flow from these facts")

Earth's own field has varied several-fold over its history, dipped to about a tenth during reversals and excursions, and flipped hundreds of times. So a field is never one number, and what it does for a world (6.2, 6.3) is an **effect that varies through time**, never a switch that is on or off.

- Every magnetised body carries a **field history**: strength through time from the heat history, plus dated events on the same timeline as everything else: dynamo start, inner core birth, strengthening and weakening spans, reversals, excursions (a dip that recovers without flipping), long quiet spans with no reversals, dynamo death, and for a star each cycle's flip and its grand minima when drawn. Each event has a date, a size, a why, and its grade (computed or drawn).
- Effects are computed **at a date**, from the field at that date: the cut-off, the dose, the oval's latitude, the bubble's size, each air-loss channel. So "during the dip 41,000 years ago the dose at the surface doubled and aurorae reached the tropics for three centuries" is a fact the kernel can state, not a flourish.
- These are ordinary record events (J1), so the story finder's sifters, Minerva, programs and logic blocks can use them like any other: a navigation guild whose compasses failed, strata whose magnetic stripes date a ruin, a colony founded in a quiet span.
- "Why is this here?" reads them: stripes of opposite magnetisation either side of a spreading ridge are the reversal list written into the sea floor (S2's plate ages times this history); a rock's recorded field direction gives the latitude it formed at.
- Nothing here asserts a link the science does not support. Whether dips harmed life on Earth is disputed; the kernel states the dose and the ozone change, and leaves the consequence to the life blocks and the GM.

### 4.3 Keep and Squeeze

- **Fossil fields of hot stars.** No convecting envelope, so Make fails; but a field present at birth is kept for the star's whole life. Which stars are born with one is not understood, so 7% are **drawn** (keyed on the star's address), strength from the measured spread, and a merger in the star's history (B1c) raises the chance. Marked drawn. [S21] to [S24]
- **White dwarfs.** Squeeze from the progenitor's core gives a floor. When the carbon and oxygen interior starts to crystallise (a phase boundary on the data sheet), the Make test runs on the fluid above it like any other layer. Marked argued. [R1] to [R3]
- **Neutron stars.** Squeeze gives up to about 10¹² G. The newborn star's first seconds are a Make episode whose outcome depends on birth spin: fast spinners become magnetars. Birth spin is drawn from the measured spread. After that, Keep: decay by Hall drift and resistance over 10⁴ to 10⁶ years [R9], and spin-down by the settled dipole law [R5]. Old pulsars spun up by a companion (B1c mass transfer) have buried fields near 10⁸ G.
- **Black holes** carry none; the disc's field is the jet design's (O11).
- **Winds.** The star's open field is carried out and wound into the spiral that O2 already computes. New here: the surface where the wind outruns magnetic waves (the Alfvén surface). A planet inside it, as close-in planets of red dwarfs can be, is magnetically connected to its star like Io to Jupiter: no bow shock, power flows both ways. [S16], [P34]

---

## 5. Field shape, honestly

What is published for every magnetised body (the list star census section 3 asked for, now with where each part comes from):

| Published | From | Grade |
|---|---|---|
| Dipole moment, surface field | section 4 | computed |
| Dipole share (tidy or many-poled) | local Rossby number, smooth through about 0.12 [P5]; shell thickness; a stable layer above | argued |
| Low-order coefficients (to degree 4 for planets, 8 on request) | the spectrum's *shape* is physics (flat at the dynamo's top, steeper the deeper the dynamo; a stable conducting layer removes what is not symmetric about the spin axis); the *signs and phases* are drawn from the body's address | shape argued, phases drawn |
| Tilt and offset | read off the coefficients (they are the degree 1 and 2 terms), so a thin-shell dynamo comes out tilted and off-centre by itself, and a body under a stable layer comes out aligned | argued |
| Which way the tilt points | nothing can predict it | drawn, and said to be |
| Reversal dates | a keyed Poisson process, rate rising as the dynamo nears the dipolar edge; the flip itself takes a few thousand years with the field down to about a tenth | drawn |
| Slow drift of the poles | a slow keyed rotation of the non-dipole part | drawn |

The same coefficient form serves stars (their large-scale field, flipping each cycle, with the dipole share from measured trends [S20]), so the corona, the wind's sectors and the overlay all read one description.

The checks do not pretend: the base pack must put Earth, Jupiter, Saturn, Uranus, Neptune, Mercury and Ganymede in the right *family* (aligned dipole, tilted many-poled, weak and aligned), with strengths inside stated factors. It will not reproduce Jupiter's "blue spot".

---

## 6. Consequences

Each is a block or a small flow reading the published field. None is a switch on "has a magnetosphere".

### 6.1 The obstacle to the wind
Three things can stop a wind, and all three are computed for every body: its own field (the existing stand-off block), an ionosphere pushed against by the wind (Venus, Mars, Titan, comets: an "induced magnetosphere"), or the bare surface (the Moon). The outermost one is the obstacle. The shape is Shue's form [M1]. For a moon inside its host's bubble the outside pressure is the host's field and plasma, not the wind (V3 had this right). On an eccentric or gusty day a moon can cross in and out (Titan does).

### 6.2 Air loss, channel by channel
`sse.escape.*`: thermal escape, escape driven by the star's extreme ultraviolet, photochemical escape, pick-up by the wind, sputtering, and outflow along open polar field lines. Each computes its own rate. A field changes them one at a time: it cuts pick-up and sputtering, opens the polar route, and enlarges the area gathering wind energy [M3], [M4], [M6]. The total may go up or down. The card for Mars will say, truthfully, that it lost its air mostly because it is small, and that the loss of its field changed how, more than how much. Marked argued. This corrects the life design's "loss of magnetic shielding" extinction cause into two measurable things: dose (6.3) and loss of ozone during storms.

### 6.3 Cosmic rays and dose
`sse.shield.cutoff`: the lowest particle energy that can reach each magnetic latitude (Størmer's result; 15 GV at Earth's equator, zero at the poles) [M7]. `sse.shield.dose`: galactic cosmic rays (set by the galaxy place and the star's own bubble [G7]) and the star's particle storms (from O10's events), through the cut-off, then through the air column, to a dose at the surface, at altitude and in orbit [M8]. This is the number life's `radiation_dose_msv_yr` reads, NAV reads for parking orbits, and the classification modifier "magnetically unshielded" is read off.

### 6.4 Radiation belts
A belt can exist wherever field lines close and particles can bounce without hitting air, rings or the bubble's edge. `sse.belts.zone` finds that region. `sse.belts.intensity` caps the flux at the self-limiting value for the local field [M10], [M11], filled by sources that are each computed: cosmic rays splashing neutrons off the air or rings (always present), wind injection, and moons that feed gas (an eruptive moon inside the bubble makes a plasma torus [M18]). Moons and rings sweep gaps. Output: zones with particle kind, flux, dose rate behind three standard shield thicknesses, and a why for each edge and gap. NAV's mass gate goes. Checks: Earth has two belts and a slot; Saturn's stop at the rings; the dose at Europa's orbit is hundreds of times that at Callisto's.

### 6.5 Aurorae and radio
`sse.aurora.oval`: the footprint of the boundary between closed and open field lines, from the open flux the wind drives [M13]; it follows the *magnetic* axis, so a tilted, offset field puts ovals far from the spin poles. `sse.aurora.power`: a share of the wind power through the bubble's cross-section. Colours are not chosen: the light suite's line lists for the air's own gases give them (oxygen green and red, nitrogen blue and violet, hydrogen ultraviolet and infrared on giants, carbon dioxide ultraviolet on Mars-like air). Also: a spot and trail where each inner moon's field line lands; patchy aurorae over crustal fields [M15]; a diffuse glow on worlds with air and no field. When one of O10's ejections hits, the bubble shrinks, the oval widens toward the equator and brightens for a day or two, as a dated event. The auroral radio in O2 gains beaming and rotation (retiring its "isotropic" fudge), and for a planet inside its star's Alfvén surface the radio comes from the star.

### 6.6 Induced fields: finding oceans
`sse.induction.response`: a conducting shell (a salty ocean, a magma ocean, an iron core) in a field that changes with time answers with a field of its own, whose size depends on the shell's conductivity and thickness and how fast the outside field changes [P29]. For play this is a real survey method: a magnetometer pass that reveals an ocean under ice, through the sensing rules (O9) and the knowledge ladder.

### 6.7 Rock that remembers; the compass
At Survey level, crust that formed while the dynamo ran is magnetised in proportion to the field then and its iron minerals (data sheet: Curie temperature and susceptibility of magnetite, hematite, pyrrhotite and iron metal); later large impacts and volcanism wipe it. The surface's own age map (already generated) decides which is which, so a magnetised highland beside a blank basin has a why with dates [P8]. These patches make small bubbles, patchy aurorae and swirl markings (S19 colour).

At Site level the field is a vector: strength, dip and declination. "A compass works here" is a label read off it: it needs enough horizontal field and a field that is mostly dipole. It fails near magnetic poles, misleads on a many-poled world, points at the nearest crustal patch on a Mars-like world, and points the other way before a reversal. The record can carry "the compass swung" when a storm hits.

### 6.8 Hazards to ships and settlements
Belt dose along a course (6.4, replacing the stand-in); particle storms outside any bubble (O10 events, with the warning time light gives); passage through a tail or a plasma torus; a storm's induced currents in long conductors on the ground, with size from the rate of change of the field (the 1859 Carrington storm is the fixture); a magnetar's giant flare as a galaxy-scale event [R12]. A ship's or station's own shield field is an authored dipole on a construct and gets its bubble and cut-off from the same blocks.

### 6.9 Stars: what the field adds to O10
- `star.mean_field` and total flux published from section 4. The measured law "X-rays follow magnetic flux" [S9] becomes a cross-check on O10's activity chain: the two routes must agree within a factor of three across the fixture stars, or the test fails.
- The largest possible flare comes from the largest spot group's stored energy [S14], replacing O10's fixed 10²⁹ J.
- Spin over a life: the wind torque [S17] with the measured stall [S18] replaces Skumanich in the life line's spin (F7), which fixes old Sun-like stars being too slow and red dwarfs staying active too briefly.
- Cycle: stays empirical. For a generated star the branch is drawn from the measured scatter rather than picked by a cut-off; an authored cycle wins (the Sun pack authors 11 years). The field's polarity flips each cycle, so a full magnetic cycle is two.
- Ejections from saturated stars are thinned by the confining field, marked argued [S15].
- The corona and spots get structure: spots emerge where the large-scale field says, and the corona's streamers follow it (dipole-like at minimum, tangled at maximum).

### 6.10 Formation and the galaxy
- The young star's field (kilogauss, from section 4 applied to a fully convective, fast-spinning young star) sets the disc's inner edge at the radius where its field pressure meets the inflow, and locks the birth spin [D1]. This replaces the fixed 8-day inner edge.
- Disc accretion itself stays a labelled rate (argued, 3.3), with its card explaining the debate.
- The galaxy carries a smooth field model at galaxy-chart level: strength from the gas's turbulent energy, direction along the arms [G1]; the Milky Way pack uses a fitted model [G3]. It feeds the radio band of the night sky (the galaxy's glow in radio is this field), the cosmic-ray flux by place, and nebula texture as decoration.

---

## 7. Detail levels

| Level | What is computed | Cost rule |
|---|---|---|
| **Glance** | Nothing new. If the interior and its heat history are already cached: moment, surface field, stand-off, one line of words. If not: the envelope's likely range, marked as unresolved | no walk of the interior for a click |
| **Orbit** | The ledger for each layer, the heat history, dipole share, tilt and offset, obstacle and bubble shape, belt zones, aurora ovals and power, radio | scalars and a short ODE; cached with the world |
| **Survey** | Coefficients to degree 4 (8 on request), dose by latitude, crustal patches from the surface age map, belt detail with moon gaps, aurora statistics over a cycle | reuses the surveyed surface; no new terrain |
| **Site** | The local field vector, compass quality, tonight's aurora chance and where in the sky, local crustal field | reads Survey, adds nothing global |

Levels are the same world: deeper levels add coefficients on top of the same dipole; they never change it. Heavy physics is not run in the browser at all: flow speed scalings and the spectrum shapes are closed forms or small tables fitted to published simulations, behind blocks that a pack can swap.

---

## 8. Kernel surface

- Facts on every body (census section 3): `body.magnetic_moment` and `body.surface_magnetic_field` (exist), plus `field.source` (which layer, which ledger term), `field.dipole_share`, `field.tilt`, `field.offset`, `field.evidence` (computed, argued, drawn, authored), `field.since` and `field.until`.
- Op `read_field { system, body, at_s, detail }` → `field_read`: the ledger per layer with why lines, coefficients, obstacle (kind, stand-off, shape numbers), belts, aurora, cut-off by latitude, doses. Read only, cached, cancellable like the other survey ops.
- Op `read_field_history { system, body }` → the dated on, off and reversal events, for the time ruler.
- `read_light` and `read_activity` take the moment and field from these facts instead of being handed them.
- A GM pins strength, tilt or "no field" like any other value; it is marked authored, and authored-impossible with the physics' own answer beside it when they disagree. The reason on the pin (tags design) is what players read.
- Field-line tracing for drawing is done in the views from the coefficients by one shared, tested tracer; lines are a picture of a fact, not a new fact.

---

## 9. Visualisation

One layer, called **Field**, offered in every view style (Chart, 3D, True, Wireframe) by the usual layer control. It is not a tab or a screen. V3's overlay showed a shaded bubble, a belt and an oval. This one shows cause.

**On the system chart.** Each magnetised world carries a small inked teardrop pointing down-wind, at true size once zoomed in, a minimum glyph when not. Moons sit visibly inside or outside their host's bubble. Near the star: its Alfvén surface as a thin ring and the wind's spiral sectors as faint curved rules. At the system's edge: the star's own bubble in the galaxy, drawn with the Oort sphere.

**On the Body screen, Chart style** (the main view, a cut through the magnetic axis):
- Field lines in ink: closed lines solid, open polar lines thin and fading. Line spacing means strength. The magnetic axis and the spin axis are both ruled, with the angle written between them.
- **The lines continue into the cutaway**, down to the layer that makes them, which is marked (MP19b). Tap that layer: "The field is made here: ionic water, convecting, 4,000 km thick. Why?"
- The bubble's edge as a single line shaped by the wind, the bow shock as a hatched line, the closed region lightly washed (V3's good idea: show where shielding is useful, outline the full extent).
- Belts as stipple, denser where flux is higher. Gaps are labelled by what sweeps them ("swept by Ys", "ends at the rings").
- Aurora ovals inked on the globe at their true latitude, with moon footprints as dots.
- Old crustal patches as small hatch marks at Survey level.
- Every element answers a tap with why it is there, at the three depths (Curious, Science, Engine), with the evidence mark (computed, argued, drawn, authored) on GM views only.

**3D style.** Line bundles as fine tubes, belts as soft tori, the bubble as a faint shell with a tail. Sizes use the existing readable scale law with its "not to scale" badge, because Jupiter's true tail would cross the system. Particles drift and bounce along a few lines at a stated speed-up, as the only decoration.

**True style** shows only what could be seen, by band (the rule of the light design): aurora glow on the night side in visible light; ultraviolet and infrared ovals on giants; a giant's radio belts as the bright lobes either side of the disc that radio telescopes really image; a star's corona in X-rays following its field; a pulsar's sweeping beams. No drawn field lines unless the user turns annotation on, and then the badge says "diagram over photograph".

**Stars and remnants.** Same layer: loops over spot groups, streamers, the tilted current sheet in the wind; a pulsar's light cylinder and beam cones; a magnetar's twisted field.

**Time.** The layer follows the time ruler. Drag back four billion years on a Mars-like world and the bubble is there; drag forward and it shrinks to patches. Reversals show as a fading and re-growing. A stellar cycle flips the star's field. An ejection arriving squeezes the bubble and lights the oval, as it happens.

**Motion and feel** (interface style): on opening, lines ink outward from the dynamo layer in 240 ms or less; dragging the wind strength or the time ruler reshapes the bubble under the finger; reduced-motion setting draws the end state. **Phone:** the axis cut only, with the card below. **Text tier:** "Field 31 µT at the equator, tilted 11°. Holds the wind off at 10 radii. Two belts. Aurorae at 67° north and south. Why?"

**Players** see the field only through what their instruments have sensed (a magnetometer reading, a radio detection, an aurora seen from orbit), in their theme. Sound: the radio chorus and whistlers as translated signals (N6) read the same facts.

**Reference screen.** The Constants page already promises "change G and see". The same for μ₀ and for any phase's conductivity: change ionic water's conductivity and watch Neptune's field fade, with the worlds affected listed.

---

## 10. What the data sheet must hold (for the "Design master data sheet" thread)

Per phase row: electrical conductivity with its pressure and temperature range, thermal conductivity, thermal expansion, heat capacity, viscosity, density curve (all already in 4.3A's list; this design is their first reader). New asks: (1) conductivity of mixtures by a stated rule (hydrogen with helium; water with ammonia and methane; iron with sulphur or silicon), since no dynamo region is pure; (2) iron alloy melting curves and the light-element partition for inner core freezing; (3) Curie temperature and magnetic susceptibility on mineral rows; (4) evidence grades carried through, because core and ionic water conductivities are the most disputed numbers in the whole field model; (5) constants μ₀, e, mₑ on the Constants page with the units T, µT, G, A m². Every MF block reads substance properties through the visible lookup block `sse.reference.lookup` (matter pack section 3A, MP17a), so the flow shows which row each number came from. No magnetic constant or material property is typed into a crate (the existing `MU0` in `magnetic.rs` moves to the sheet under MP1c).

---

## 11. Packages (track MF)

Order: MF0 and MF1 first (they unblock everything); MF2 to MF4 give a real field on every world; MF6 and MF14a give something to look at; the rest in any order. Each lists its evidence; goldens change only on purpose.

| Stage | Contents | Needs | Thread | Evidence (tests) |
|---|---|---|---|---|
| MF0 | Fixture of measured fields with sources: eight planets, Ganymede, Europa and Callisto (induced), Moon and Mars (crustal), the Sun, six stars across Rossby number, two hot stars, three white dwarfs, three pulsars, one magnetar. Every reference in section 14 checked. NAV's belt mass gate replaced by "has a moment" as a stop-gap | none | Verify physics citations, with Build galaxy charts and travel screens for the NAV line | fixture loads; every DOI resolves; Earth and Saturn now have belts in `plan_route` |
| MF1 | Convected flux and compositional power blocks; core heat history (`sse.core.history`); events for dynamo start, inner core birth, dynamo death. Completes B1d part 2 | P1 heat, S2 plate regime; iron rows of MP21 (a stand-in melting curve until then, registered) | Build SSE4 proven core | Earth: field continuous for 3.5 Gyr, inner core age inside the published range; Mars: off between 4.1 and 3.6 Gyr ago; Venus: off today with the reason "core not cooling"; under 1 ms per world; F-S30 retired |
| MF2 | The ledger: flow speed, Rm, onset, Rossby factor, per layer of the phase profile; dynamo top found by walking down (no typed radius); wired into resolve so every world publishes its field facts. This is MP20's field half | MF1, MP19 (handed layers until it lands) | Build SSE4 proven core | Earth and Jupiter still pass with inputs now found; Saturn within a factor of 2; Uranus and Neptune within a factor of 2 of Voyager; Ganymede has a field, Io and Callisto do not; no list of field-making materials in code |
| MF3 | Shape: dipole share, coefficients, tilt, offset, stable-layer filtering; the field history of 4.2A (strength spans, reversals, excursions, quiet spans and drift as dated, keyed record events; effects asked for at a date) | MF2 | Build SSE4 proven core | families right for the seven magnetised bodies; Saturn's tilt under 1°; Uranus and Neptune tilted and off-centre; same coefficients on two devices; F-S29 retired |
| MF4 | Stars on the same ledger: mean field and flux, saturation, large-scale coefficients that flip each cycle, cross-check against the X-ray chain, largest flare from spots, cycle branch drawn from scatter | MF2, O10 | Build light and nebulae | Sun: mean field and open flux within a factor of 2; red dwarf fixture stars within the measured scatter; X-ray routes agree within a factor of 3; the Sun pack's authored 11 years wins |
| MF5 | Wind and spin: torque with the stall, replacing Skumanich in the life line's spin; Alfvén surface; star's bubble in the galaxy; star and close planet connection | MF4, F7 | Build light and nebulae, with Build kernel resolve and rules for the life line | Sun's spin at 4.57 Gyr within 10%; old Kepler-like stars spin faster than Skumanich says; Alfvén surface near 10 to 20 solar radii |
| MF6 | Obstacle and bubble: intrinsic, induced or surface; Shue shape; moons inside a host's bubble; induced response of conducting shells; ops `read_field` and `read_field_history` | MF2 | Build light and nebulae (it owns the stand-off block) | Earth 10 radii, Jupiter 60 to 90 with its plasma, Mercury 1.5; Venus and Mars get induced obstacles; Europa's induced field within 30% of Galileo's; Titan crosses Saturn's boundary |
| MF7 | Cut-off, dose at surface, altitude and orbit; belts with sources, limits, sweepers; NAV and life read them | MF6, O10 | Build light and nebulae for the blocks; Build galaxy charts and travel screens for NAV; Build kernel resolve and rules for life | Earth's cut-off 15 GV at the equator; sea-level dose within a factor of 2; Mars surface dose within a factor of 2 of Curiosity's; Jupiter's moons in the right order; life fixture's typed 230 mSv replaced by a computed value |
| MF8 | Air loss by channel, with and without a field | MF6, W7a | Build shared climate and weather | Earth, Venus and Mars ion loss within an order of magnitude of measured and within an order of each other; a small world loses air with or without a field; marked argued |
| MF9 | Aurorae: ovals, power, lines from the air, moon footprints, crustal patches, storm response; beamed radio | MF6, O2, O10 | Build light and nebulae | Earth's oval near 67°, tens of GW; Jupiter's Io spot present; Uranus's ovals far from the spin poles; an ejection hit moves the oval equatorward on its date |
| MF10 | Rock that remembers and the compass: crustal patches from surface ages; sea-floor stripes from plate ages and the reversal list; site field vector; compass label | MF1, MF3, S16 | Build SSE4 proven core | a Mars-like world: old highlands magnetised, big late basins blank, each with dated why; compass works on Earth, not on Mars or Neptune |
| MF11 | Remnants: white dwarf fields, pulsar spin-down and death, magnetar birth, decay and giant flares as events, recycled pulsars | MF2, B3 | Build SSE4 proven core (B3 remnants) | Crab-like pulsar: field and age from spin within a factor of 2; a fifth of old white dwarfs magnetic; magnetar field gone in about a million years |
| MF12 | Formation hooks: young star's field sets the disc inner edge and birth spin | MF4 | Build SSE4 proven core (formation) | inner planets pile up near a spread of periods around 3 to 10 days, not all at 8; growth.rs fudge retired; formation goldens re-blessed on purpose |
| MF13 | Galactic field model; radio glow and cosmic-ray flux by place | G3, Y3 | Build galaxy charts and travel screens | Milky Way pack: a few µG at the Sun; cosmic-ray flux higher toward the centre |
| MF14a | Field layer, data to picture: shared line tracer, layer recipes for Chart and 3D, readable scaling, band emissions for True (radio belts, aurora glow, corona structure) | MF3, MF6 (starts on the Solar System fixture) | Build photoreal renderer (its R2c and R5) | Earth, Jupiter, Neptune, Sun and a pulsar screenshots; tracer matches a dipole analytically; phone tier draws the cut; frame budget held |
| MF14b | Field layer on the Body screen and system chart: layer control, lines into the cutaway, tap for why, time ruler following, words tier | MF14a, MP19b | Rebuild the chart room's top level | browser checks on desk, phone and Night chart; every element has a why; evidence marks on GM view only |
| MF15 | Game hooks: magnetometer as a sensor kind (finds oceans and fields), record events (storm, reversal, giant flare, compass swing), sound signals | MF6, MF9, O9, J1 | Build kernel resolve and rules, with Build Traveller pack for the sensor table | a pass by a Europa-like moon with a magnetometer reveals "conducting ocean"; without one it does not |
| MF16 | **Folded into the matter pack** (agreed with its design thread, 22 Nov; matter pack 4.3A "What the magnetic fields design asks of the sheet"): mixture conductivity rule and evidence grades in MP17 and MP17a; μ₀, e and mₑ in MP1c; Curie temperature and susceptibility in MP13's minerals wave; iron alloy melting and partition in MP21's iron wave | those packages | their owners | changing ionic water's conductivity changes Neptune's field in a test |

---

## 12. Fudge register entries this design will add (stated now)

Drawn reversal and cycle statistics; drawn incidence of fossil fields and of magnetar birth spins; coefficient phases; the magnesium route for Earth's early dynamo; mixture conductivity rule; the outer bubble's field lines beyond about two thirds of the stand-off (a simple compressed-dipole form: illustrative); belt source strengths; the confinement of ejections on active stars; the galactic field as a smooth model. Each with what would replace it.

---

## 13. Rulings (the questions as put to Alex; all thirteen recommendations accepted 22 Nov 2026, "agree to all")

1. **One field ledger (make, keep, squeeze) on every conducting layer of every body, stars and remnants included, with no list of field-making materials?** Recommended: yes. It is the general mechanism, and it is what the built dynamo blocks already assume.
2. **Tie a rocky world's field to its heat history, so fields switch on and die by themselves (Mars), and drop V3's "slow spin means no field" (Venus)?** Recommended: yes. The research is clear that Venus's spin is not the reason.
3. **Air loss computed channel by channel, with the field helping some and hurting others, so "a magnetic field protects the atmosphere" is never assumed?** Recommended: yes. It is the honest state of the science and makes a better card than the myth.
4. **Field shape (tilt, offset, many poles) computed but marked "argued", with the direction of a tilt and reversal dates marked "drawn"?** Recommended: yes.
5. **For things nobody can predict (cycle lengths, which hot stars are magnetic, reversal timing), draw from measured statistics and say so, rather than invent a formula?** Recommended: yes. The Sun keeps its authored 11 years.
6. **Ice giants: ship the thin-shell model as the default and the unmixed-layers model as a swappable alternative, both marked argued?** Recommended: yes.
7. **Earth's early dynamo: ship the magnesium route so Earth keeps its field for 3.5 billion years, registered as a choice among published answers?** Recommended: yes.
8. **Field as one layer in every view style, not a screen or tab, with field lines running down into the cutaway to the layer that makes them?** Recommended: yes.
9. **In True style show only what an instrument would see (aurora, radio belts, corona), with drawn field lines only as an annotation the user turns on?** Recommended: yes.
10. **Replace NAV's "over 200 Earth masses" belt rule now with the stop-gap in MF0, ahead of the real belts in MF7?** Recommended: yes. It is a small fix and Saturn and Earth are wrong today.
11. **Scope for 4.0: MF0 to MF10 and MF14 (planets, stars, consequences, the overlay); remnants, formation hooks, the galaxy and game hooks (MF11 to MF13, MF15) follow as their owner threads reach them?** Recommended: yes.
12. **A magnetometer as a real survey instrument that can reveal a buried ocean (MF15)?** Recommended: yes. It is how Europa's ocean was found, and it is a good reason to fly past a moon.
13. **Shall I draw the Field layer as a mockup (Earth, Neptune with its cutaway, a Mars-like world four billion years apart, a red dwarf) before MF14 is built?** Recommended: yes, once 8 and 9 are answered.

---

## 14. References

Checked 22 Nov by web search: **(a)** the DOI itself was seen on the publisher's or an index's page; **(b)** the paper was found by title, DOI as remembered. Unmarked entries are from memory and are for MF0 to verify, as B1 and O10 did with theirs.

**Planets and moons**
- [P1] Stevenson 2003, Planetary magnetic fields, EPSL 208, 1. doi:10.1016/S0012-821X(02)01126-3
- [P2] Christensen and Aubert 2006, GJI 166, 97. doi:10.1111/j.1365-246X.2006.03009.x (the code had 02972.x; corrected in MF0)
- [P3] Christensen, Holzwarth and Reiners 2009, Nature 457, 167. doi:10.1038/nature07626 (in the code)
- [P4] Christensen 2010, Space Sci. Rev. 152, 565. doi:10.1007/s11214-009-9553-2 (in the code)
- [P5] Olson and Christensen 2006, Dipole moment scaling for convection-driven planetary dynamos, EPSL 250, 561. doi:10.1016/j.epsl.2006.08.008 (b)
- [P6] Driscoll and Bercovici 2014, On the thermal and magnetic histories of Earth and Venus, PEPI 236, 36. doi:10.1016/j.pepi.2014.08.004 (b)
- [P7] Acuña et al. 1999, Science 284, 790. doi:10.1126/science.284.5415.790
- [P8] Lillis et al. 2013, Time history of the Martian dynamo from crater magnetic field analysis, JGR Planets 118, 1488. doi:10.1002/jgre.20105
- [P9] Mittelholz et al. 2020, Timing of the martian dynamo, Science Advances 6, eaba0513. doi:10.1126/sciadv.aba0513 (b)
- [P10] Pozzo et al. 2012, Thermal and electrical conductivity of iron at Earth's core conditions, Nature 485, 355. doi:10.1038/nature11031
- [P11] Labrosse 2015, Thermal evolution of the core with a high thermal conductivity, PEPI 247, 36. doi:10.1016/j.pepi.2015.02.002 (b)
- [P12] O'Rourke and Stevenson 2016, Powering Earth's dynamo with magnesium precipitation from the core, Nature 529, 387. doi:10.1038/nature16495
- [P13] Stixrude et al. 2020, A silicate dynamo in the early Earth, Nature Communications 11, 935. doi:10.1038/s41467-020-14773-4
- [P14] Tarduno et al. 2010, Geodynamo, solar wind, and magnetopause 3.4 to 3.45 billion years ago, Science 327, 1238. doi:10.1126/science.1183445
- [P15] Christensen 2006, A deep dynamo generating Mercury's magnetic field, Nature 444, 1056. doi:10.1038/nature05342
- [P16] Anderson et al. 2011, The global magnetic field of Mercury from MESSENGER orbital observations, Science 333, 1859. doi:10.1126/science.1211001
- [P17] Dougherty et al. 2018, Saturn's magnetic field revealed by the Cassini Grand Finale, Science 362, eaat5434. doi:10.1126/science.aat5434
- [P18] Connerney et al. 2018, GRL 45, 2590. doi:10.1002/2018GL077312 (in the code)
- [P19] Moore et al. 2018, A complex dynamo inferred from the hemispheric dichotomy of Jupiter's magnetic field, Nature 561, 76. doi:10.1038/s41586-018-0468-5
- [P20] Ness et al. 1986, Science 233, 85. doi:10.1126/science.233.4759.85; Ness et al. 1989, Science 246, 1473. doi:10.1126/science.246.4936.1473
- [P21] Stanley and Bloxham 2004, Nature 428, 151. doi:10.1038/nature02376 (matter pack design)
- [P22] Redmer et al. 2011, Icarus 211, 798. doi:10.1016/j.icarus.2010.08.008 (matter pack design)
- [P23] Millot et al. 2019, Nature 569, 251. doi:10.1038/s41586-019-1114-6 (matter pack design)
- [P24] Soderlund and Stanley 2020, The underexplored frontier of ice giant dynamos, Phil. Trans. R. Soc. A 378, 20190479. doi:10.1098/rsta.2019.0479 (b)
- [P25] Militzer 2024, Phase separation of planetary ices explains nondipolar magnetic fields of Uranus and Neptune, PNAS 121, e2403981121. doi:10.1073/pnas.2403981121 (a)
- [P26] Kivelson et al. 1996, Discovery of Ganymede's magnetic field, Nature 384, 537. doi:10.1038/384537a0
- [P27] Hauck, Aurnou and Dombard 2006, Sulfur's impact on core evolution and magnetic field generation on Ganymede, JGR 111, E09008. doi:10.1029/2005JE002557
- [P28] Kivelson et al. 2000, Galileo magnetometer measurements: a stronger case for a subsurface ocean at Europa, Science 289, 1340. doi:10.1126/science.289.5483.1340
- [P29] Zimmer, Khurana and Kivelson 2000, Subsurface oceans on Europa and Callisto, Icarus 147, 329. doi:10.1006/icar.2000.6456
- [P30] Tikoo et al. 2017, A two-billion-year history for the lunar dynamo, Science Advances 3, e1700207. doi:10.1126/sciadv.1700207
- [P31] Weiss et al. 2010, Paleomagnetic records of meteorites and early planetesimal differentiation, Space Sci. Rev. 152, 341. doi:10.1007/s11214-009-9580-z
- [P32] Bryson et al. 2015, Long-lived magnetism from solidification-driven convection on the pallasite parent body, Nature 517, 472. doi:10.1038/nature14114
- [P33] Glatzmaier and Roberts 1995, Nature 377, 203. doi:10.1038/377203a0
- [P34] Pineda and Villadsen 2023, Coherent radio bursts from known M-dwarf planet-host YZ Ceti, Nature Astronomy 7, 569. doi:10.1038/s41550-023-01914-0 (b)
- [P35] Cauley et al. 2019, Magnetic field strengths of hot Jupiters from signals of star-planet interactions, Nature Astronomy 3, 1128. doi:10.1038/s41550-019-0840-x
- [P36] Yadav and Thorngren 2017, Estimating the magnetic field strength in hot Jupiters, ApJL 849, L12. doi:10.3847/2041-8213/aa93fd

**Stars**
- [S1] Noyes et al. 1984, ApJ 279, 763. doi:10.1086/161945 (in the code)
- [S2] Wright et al. 2011, ApJ 743, 48. doi:10.1088/0004-637X/743/1/48 (in the code)
- [S3] Reiners et al. 2022, Magnetism, rotation, and nonthermal emission in cool stars, A&A 662, A41. doi:10.1051/0004-6361/202243251 (a)
- [S4] Kochukhov 2021, Magnetic fields of M dwarfs, A&A Rev. 29, 1. doi:10.1007/s00159-020-00130-3
- [S5] Wright and Drake 2016, Solar-type dynamo behaviour in fully convective stars without a tachocline, Nature 535, 526. doi:10.1038/nature18638
- [S6] Charbonneau 2020, Dynamo models of the solar cycle, Living Rev. Solar Phys. 17, 4. doi:10.1007/s41116-020-00025-6
- [S7] Brun and Browning 2017, Magnetism, dynamo action and the solar-stellar connection, Living Rev. Solar Phys. 14, 4. doi:10.1007/s41116-017-0007-8
- [S8] Böhm-Vitense 2007, Chromospheric activity in G and K main-sequence stars, and what it tells us about stellar dynamos, ApJ 657, 486. doi:10.1086/510482
- [S9] Pevtsov et al. 2003, The relationship between X-ray radiance and magnetic flux, ApJ 598, 1387. doi:10.1086/378944
- [S10] Klimchuk 2006, On solving the coronal heating problem, Solar Physics 234, 41. doi:10.1007/s11207-006-0055-z
- [S11] Shibata and Magara 2011, Solar flares: magnetohydrodynamic processes, Living Rev. Solar Phys. 8, 6. doi:10.12942/lrsp-2011-6
- [S12] Benz and Güdel 2010, Physical processes in magnetically driven flares on the Sun, stars, and young stellar objects, ARA&A 48, 241. doi:10.1146/annurev-astro-082708-101757
- [S13] Aschwanden et al. 2016, 25 years of self-organized criticality: solar and astrophysics, Space Sci. Rev. 198, 47. doi:10.1007/s11214-014-0054-6
- [S14] Shibata et al. 2013, Can superflares occur on our Sun?, PASJ 65, 49. doi:10.1093/pasj/65.3.49
- [S15] Alvarado-Gómez et al. 2018, Suppression of coronal mass ejections in active stars by an overlying large-scale stellar magnetic field, ApJ 862, 93. doi:10.3847/1538-4357/aacb7f
- [S16] Weber and Davis 1967, The angular momentum of the solar wind, ApJ 148, 217. doi:10.1086/149138
- [S17] Matt et al. 2015, The mass-dependence of angular momentum evolution in Sun-like stars, ApJL 799, L23. doi:10.1088/2041-8205/799/2/L23
- [S18] van Saders et al. 2016, Weakened magnetic braking as the origin of anomalously rapid rotation in old field stars, Nature 529, 181. doi:10.1038/nature16168 (a)
- [S19] Wood et al. 2021, New observational constraints on the winds of M dwarf stars, ApJ 915, 37. doi:10.3847/1538-4357/abfda5
- [S20] Vidotto et al. 2014, Stellar magnetism: empirical trends with age and rotation, MNRAS 441, 2361. doi:10.1093/mnras/stu728 (b)
- [S21] Donati and Landstreet 2009, Magnetic fields of nondegenerate stars, ARA&A 47, 333. doi:10.1146/annurev-astro-082708-101833
- [S22] Wade et al. 2016, The MiMeS survey of magnetism in massive stars, MNRAS 456, 2. doi:10.1093/mnras/stv2568
- [S23] Braithwaite and Spruit 2004, A fossil origin for the magnetic field in A stars and white dwarfs, Nature 431, 819. doi:10.1038/nature02934
- [S24] Schneider et al. 2019, Stellar mergers as the origin of magnetic massive stars, Nature 574, 211. doi:10.1038/s41586-019-1621-5

**Discs, jets, formation**
- [D1] Koenigl 1991, Disk accretion onto magnetic T Tauri stars, ApJ 370, L39. doi:10.1086/185972
- [D2] Hartmann, Herczeg and Calvet 2016, Accretion onto pre-main-sequence stars, ARA&A 54, 135. doi:10.1146/annurev-astro-081915-023347
- [D3] Balbus and Hawley 1991, ApJ 376, 214. doi:10.1086/170270
- [D4] Bai and Stone 2013, Wind-driven accretion in protoplanetary disks I, ApJ 769, 76. doi:10.1088/0004-637X/769/1/76
- [D5] Lesur et al. 2023, Hydro-, magnetohydro-, and dust-gas dynamics of protoplanetary disks, Protostars and Planets VII. https://arxiv.org/abs/2203.09821 (a)
- [D6] Fu et al. 2014, Solar nebula magnetic fields recorded in the Semarkona meteorite, Science 346, 1089. doi:10.1126/science.1258022
- [D7] Weiss, Bai and Fu 2021, History of the solar nebula from meteorite paleomagnetism, Science Advances 7, eaba5967. doi:10.1126/sciadv.aba5967
- [D8] Blandford and Payne 1982, MNRAS 199, 883. doi:10.1093/mnras/199.4.883
- [D9] Blandford and Znajek 1977, MNRAS 179, 433. doi:10.1093/mnras/179.3.433
- [D10] Event Horizon Telescope Collaboration 2021, First M87 results VII: polarization of the ring, ApJL 910, L12. doi:10.3847/2041-8213/abe71d

**Magnetospheres, air, radiation, aurorae**
- [M1] Shue et al. 1997, A new functional form to study the solar wind control of the magnetopause size and shape, JGR 102, 9497. doi:10.1029/97JA00196 (a)
- [M2] Kivelson and Russell (eds) 1995, Introduction to Space Physics, Cambridge University Press (in the code)
- [M3] Gunell et al. 2018, Why an intrinsic magnetic field does not protect a planet against atmospheric escape, A&A 614, L3. doi:10.1051/0004-6361/201832934 (a)
- [M4] Ramstad and Barabash 2021, Do intrinsic magnetic fields protect planetary atmospheres from stellar winds?, Space Sci. Rev. 217, 36. doi:10.1007/s11214-021-00791-1 (a)
- [M5] Jakosky et al. 2018, Loss of the Martian atmosphere to space, Icarus 315, 146. doi:10.1016/j.icarus.2018.05.030
- [M6] Egan et al. 2019, Planetary magnetic field control of ion escape from weakly magnetized planets, MNRAS 488, 2108. doi:10.1093/mnras/stz1819
- [M7] Smart and Shea 2005, A review of geomagnetic cutoff rigidities for earth-orbiting spacecraft, Adv. Space Res. 36, 2012. doi:10.1016/j.asr.2004.09.015
- [M8] Atri, Hariharan and Grießmeier 2013, Galactic cosmic ray-induced radiation dose on terrestrial exoplanets, Astrobiology 13, 910. doi:10.1089/ast.2013.1052
- [M9] Grießmeier et al. 2005, Cosmic ray impact on extrasolar Earth-like planets in close-in habitable zones, Astrobiology 5, 587. doi:10.1089/ast.2005.5.587
- [M10] Kennel and Petschek 1966, Limit on stably trapped particle fluxes, JGR 71, 1. doi:10.1029/JZ071i001p00001
- [M11] Mauk and Fox 2010, Electron radiation belts of the solar system, JGR 115, A12220. doi:10.1029/2010JA015660 (a)
- [M12] Li and Hudson 2019, Earth's Van Allen radiation belts: from discovery to the Van Allen Probes era, JGR Space Physics 124, 8319. doi:10.1029/2018JA025940
- [M13] Newell et al. 2007, A nearly universal solar wind-magnetosphere coupling function, JGR 112, A01206. doi:10.1029/2006JA012015
- [M14] Feldstein and Starkov 1967, Dynamics of auroral belt and polar geomagnetic disturbances, Planet. Space Sci. 15, 209. doi:10.1016/0032-0633(67)90190-0
- [M15] Bertaux et al. 2005, Discovery of an aurora on Mars, Nature 435, 790. doi:10.1038/nature03603
- [M16] Zarka 2007, Planet. Space Sci. 55, 598. doi:10.1016/j.pss.2006.05.045 (in the code)
- [M17] Goldreich and Lynden-Bell 1969, Io, a Jovian unipolar inductor, ApJ 156, 59. doi:10.1086/149947
- [M18] Bagenal and Delamere 2011, Flow of mass and energy in the magnetospheres of Jupiter and Saturn, JGR 116, A05209. doi:10.1029/2010JA016294
- [M19] Hood and Schubert 1980, Lunar magnetic anomalies and surface optical properties, Science 208, 49. doi:10.1126/science.208.4439.49

**Remnants**
- [R1] Ferrario, de Martino and Gänsicke 2015, Magnetic white dwarfs, Space Sci. Rev. 191, 111. doi:10.1007/s11214-015-0152-0
- [R2] Bagnulo and Landstreet 2021, New insight into the magnetism of degenerate stars from the analysis of a volume-limited sample of white dwarfs, MNRAS 507, 5902. doi:10.1093/mnras/stab2046
- [R3] Isern et al. 2017, A common origin of magnetism from planets to white dwarfs, ApJL 836, L28. doi:10.3847/2041-8213/aa5eae (b)
- [R4] Goldreich and Julian 1969, Pulsar electrodynamics, ApJ 157, 869. doi:10.1086/150119
- [R5] Spitkovsky 2006, Time-dependent force-free pulsar magnetospheres, ApJL 648, L51. doi:10.1086/507518
- [R6] Philippov and Kramer 2022, Pulsar magnetospheres and their radiation, ARA&A 60, 495. doi:10.1146/annurev-astro-052920-112338
- [R7] Duncan and Thompson 1992, Formation of very strongly magnetized neutron stars, ApJ 392, L9. doi:10.1086/186413
- [R8] Kaspi and Beloborodov 2017, Magnetars, ARA&A 55, 261. doi:10.1146/annurev-astro-081915-023329
- [R9] Viganò et al. 2013, Unifying the observational diversity of isolated neutron stars via magneto-thermal evolution models, MNRAS 434, 123. doi:10.1093/mnras/stt1008 (b)
- [R10] Raynaud et al. 2020, Magnetar formation through a convective dynamo in protoneutron stars, Science Advances 6, eaay2732. doi:10.1126/sciadv.aay2732
- [R11] CHIME/FRB Collaboration 2020, A bright millisecond-duration radio burst from a Galactic magnetar, Nature 587, 54. doi:10.1038/s41586-020-2863-y
- [R12] Hurley et al. 2005, An exceptionally bright flare from SGR 1806-20, Nature 434, 1098. doi:10.1038/nature03519

**Galaxies**
- [G1] Beck 2016, Magnetic fields in spiral galaxies, A&A Rev. 24, 4. doi:10.1007/s00159-015-0084-4
- [G2] Brandenburg and Subramanian 2005, Astrophysical magnetic fields and nonlinear dynamo theory, Physics Reports 417, 1. doi:10.1016/j.physrep.2005.06.005
- [G3] Jansson and Farrar 2012, A new model of the Galactic magnetic field, ApJ 757, 14. doi:10.1088/0004-637X/757/1/14
- [G4] Crutcher 2012, Magnetic fields in molecular clouds, ARA&A 50, 29. doi:10.1146/annurev-astro-081811-125514
- [G5] Subramanian 2016, The origin, evolution and signatures of primordial magnetic fields, Rep. Prog. Phys. 79, 076901. doi:10.1088/0034-4885/79/7/076901
- [G6] Neronov and Vovk 2010, Evidence for strong extragalactic magnetic fields from Fermi observations of TeV blazars, Science 328, 73. doi:10.1126/science.1184192
- [G7] Potgieter 2013, Solar modulation of cosmic rays, Living Rev. Solar Phys. 10, 3. doi:10.12942/lrsp-2013-3
