# SSE4 nebulae and light design: glowing gas, galaxies that look right, and seeing beyond the visible

Written 19 Nov by the design thread at Alex's request. Authority for nebulae as objects, for how galaxies look, and for how every view treats light: true colour, astro-imagery colour, bands outside the visible, and relativistic and distance colouring. Read with the galaxy design (cohorts, stellar deaths, events, mergers), the formation design (stars' life lines), the research report on rendering (recipes, passes, tiers), and handoff 8.4A (symbology), 8.6 (lenses), 10 (real sky).

Alex's brief (19 Nov): nebulae were a big omission from SSE 3. Galaxies should look like people expect, in colour and beauty, and ragged if battered. Nebulae must lean into physics and be beautiful: shed layers, patterns and the colours of excited gas. Where false colour is used there must be a switch between "actual" and "astro imagery". Beyond that: spectra outside the visible with view options, and colour to show relativistic effects or distance shift.

---

## 1. The idea that makes all of it work: light is a spectrum until the last step

Today's tools paint objects in screen colours. SSE4 does not. Every glowing or lit thing publishes a compact **light description** as a kernel fact, and a **band recipe** chosen per view turns it into screen colour at the very end. That one separation gives true colour, Hubble-style palettes, radio and X-ray views, redshift and Doppler colouring, all from the same facts and all labelled.

```
LightDescription {
  continuum: [{ kind: thermal(T) | starlight(spectral class) | synchrotron(slope) | free-free | dust-thermal(T),
                strength }],
  lines:     [{ line: id from the line table, strength }],          e.g. H-alpha 656 nm, [O III] 501 nm
  dust:      { extinction, reddening, scattering }                  what it blocks and reflects
  velocity, distance, redshift: carried so shifts can be applied }
```

- Internally the spectrum is held in a small fixed set of bands (radio, millimetre, far, mid and near infrared, several visible slices, ultraviolet, X-ray, gamma) plus a table of named lines. That is enough to shift, mix and map, and cheap enough for a phone.
- **Line table** (pack data): hydrogen alpha and beta, doubly ionised oxygen [O III], ionised nitrogen [N II], ionised sulphur [S II], ionised helium, [O II], Lyman alpha, neutral hydrogen at 21 cm, carbon monoxide, and more as packs add them.
- This applies to **everything**, not just nebulae: a planet has a thermal infrared glow and an ultraviolet aurora; a magnetosphere shines in radio; a star's flare is bright in X-rays; a city shows at night in visible light.

## 2. Band recipes: the switch Alex asked for

A band recipe is a small declarative document (presentation pack data, Orac-editable, H8.9) mapping the light description to screen colour. Every view has a **Light** control, and the active recipe always shows a legend chip saying what the colours mean.

| Recipe | What it shows | Honesty label |
|---|---|---|
| **Eye** | what a dark-adapted human would actually see from here. Most nebulae are faint grey or grey-green smudges, because the eye loses colour in dim light and is most sensitive near the green oxygen line. Only the brightest show a hint of colour | actual |
| **Camera** | a long exposure with an ordinary colour camera: real colours at real wavelengths, just brighter. Hydrogen is pink-red, oxygen teal, reflected starlight blue, dust brown | actual, enhanced |
| **Astro palettes** | narrowband mappings used by observatories and astrophotographers: the Hubble palette (sulphur to red, hydrogen to green, oxygen to blue), bicolour hydrogen-oxygen, and others | mapped |
| **Single band** | radio, infrared, ultraviolet or X-ray alone, shown in a stated colour ramp | mapped |
| **Composite** | several bands layered, as in observatory press images (X-ray in violet, visible in natural colour, infrared in red, radio in orange) | mapped |
| **Diagnostic** | colour used as a measuring tool: speed towards or away, temperature, density, age | mapped |

- Exposure and stretch are part of the recipe, with an honest "this would take a 40 minute exposure" note where relevant.
- **Players see with what they have.** A game-system pack can tie the bands a seat may use to the ship's sensors (H7A part 6), so a radio map or an infrared view is something the crew earns.
- Band choice is presentation only. It never changes the state hash.

## 3. Nebulae: kinds from causes

A nebula is an addressable object with a birth, a life and an end on the time ruler. Each kind is produced by something the model already tracks. Unvisited regions hold them as counts in the patch envelope (rate of the cause times how long the nebula lasts); looking resolves particular ones, and any tied to a resolved star shares its identity: the white dwarf at the centre of a ring nebula is a star you can visit.

| Kind | Cause already in the model | Size and life | Shape comes from | Light |
|---|---|---|---|---|
| **Star-forming glow** (emission nebula) | a young cohort with hot massive stars inside a gas-rich patch | grows to the radius where the stars' ionising output is used up (the Strömgren radius, from star output and gas density); lasts a few million years | the parent cloud: it bursts out of the thin side as a blister; dense clumps shadow the gas behind them into pillars pointing at the stars; winds blow bubbles | hydrogen red throughout, oxygen teal near the hottest stars, dark dust lanes, blue reflection at the edges |
| **Dark cloud** | cold dense gas and dust not yet lit | parsecs to tens of parsecs; millions of years | turbulence and gravity; small round globules | a silhouette against whatever lies behind; glows in far infrared and in carbon monoxide |
| **Reflection nebula** | dust near stars too cool to ionise it | around the star or cluster | the dust's own structure | blue, because dust scatters blue light best |
| **Planetary nebula** | the death of a Sun-like star (a dated event on its life line) | expands at tens of km/s; visible for ten to thirty thousand years, then fades | a fast late wind ploughing into slow earlier wind (the interacting-winds picture). A single slow-spinning star gives a round or elliptical shell. **A companion star or fast spin gives two lobes or a butterfly along the orbit's axis**, with jets and mirrored knots. **Each earlier pulse of mass loss is an outer halo ring: the shed layers** | layered by how hot the gas is: ionised helium and oxygen (blue-green) inside, hydrogen and nitrogen (red) outside |
| **Supernova remnant** | a supernova event (from a resolved star or a cohort's rate) | free expansion, then decelerating blast wave (radius grows as time to the two-fifths power, set by explosion energy and surrounding density), then a cooling shell that merges with its surroundings after a hundred thousand years or more | the surrounding gas: it bulges into thin regions and flattens against dense ones; the decelerating shell breaks into fingers. A core-collapse leaves a neutron star that may fill the centre with a wind nebula; a white-dwarf explosion is rounder | filaments bright in oxygen and sulphur (strong sulphur marks shocked gas), hot interior in X-rays, shell in radio |
| **Wind bubbles and shells** | very massive stars, luminous variables, novae | parsecs; thousands to millions of years | wind strength against surroundings; eruptions give lobes | hydrogen, nitrogen, oxygen |
| **Superbubble** | many supernovae from one cluster | hundreds of parsecs; tens of millions of years | breaks out of the galaxy's disc as a chimney | faint filaments; hot X-ray interior |
| **Small and brief** | newborn stars' jets, discs silhouetted against a glow, light echoes after an outburst | short | the source | each as its physics dictates |

**Colour is computed, not painted.** Line strengths for a zone come from a tabulated grid, indexed by the lighting star's temperature, how strongly the gas is lit for its density, its density, and its element abundances from the patch. The grid is a heavy block shipped as data (H11): results of standard photoionisation and shock models. So a metal-poor nebula in an ancient region is paler in oxygen and nitrogen than one in a rich spiral arm, a nebula round a cooler star lacks the teal core, and explain mode can say why.

**Structure: fact above, decoration below.** The kernel publishes a small **nebula descriptor**: shells and their radii and speeds, lobes and their axis, the density gradient of the surroundings, zones with their light descriptions, the lighting stars, and a seed. That is the fact, identical for everyone. The wisps, filaments and knots are generated on the GPU from the descriptor: turbulence with a physical spectrum, stretched along the flow, with rule-shaped detail where physics calls for it (fingers on a decelerating shell, pillars behind clumps, radial knots in a ring). Two devices may differ in a wisp; they never differ in size, shape, age or colour balance.

**On the time ruler** a nebula grows, changes shape and colour balance, and fades: scrub forward and a ring nebula swells and dims to nothing around its cooling white dwarf; scrub back and a remnant shrinks to the moment of the explosion, which is a chronicle event with a date, visible in the sky of nearby worlds as a new bright star.

**From inside.** A system inside or near a nebula has it in its sky (H10), sees its star reddened by the dust, and takes the consequences that already exist in the model: a supernova within about ten parsecs harms biospheres; a dense cloud dims the nights. Flying through one is slow fog only at interstellar speeds; the density is given honestly (a good laboratory vacuum).

## 4. Galaxies that look as people expect, and ragged when battered

A galaxy's picture is computed from the galaxy design's own quantities, through the same light descriptions.

| Feature people expect | Comes from |
|---|---|
| Blue spiral arms beaded with pink knots | star formation by ring and arm phase: young blue cohorts, and a star-forming glow (section 3) wherever hot stars sit in gas |
| Dark dust lanes on the arms' inner edges and across an edge-on disc | gas and metal content by ring; arm shock position |
| Yellow-white bulge and bar | old cohorts; bar strength from the model |
| Smooth red ellipticals; blue clumpy irregulars | star formation history: quenched long ago, or bursting now |
| A faint halo with globular clusters and streams | accreted dwarfs and old cohorts |
| A bright nucleus, sometimes with jets and radio lobes | black hole feeding episodes (events) |

**Battered** galaxies wear their history, read from merger and encounter events in the chronicle:
- a recent close pass draws out **tidal tails and bridges**; a head-on hit through the disc makes a **ring galaxy**;
- a merger in progress is a **train wreck** with a starburst; an old one leaves **shells and streams** round a smooth body;
- a **warp** or lopsided disc follows a smaller disturbance;
- a galaxy falling through a cluster's hot gas trails **stripped gas with new stars** behind it;
- a starburst blows a **wind** out of the disc, glowing in hydrogen above and below.

Each feature fades over the time the physics says, so "ragged" is a clock: an expert, or Tars, can read roughly how long ago the collision was.

Rendering follows the tiers: a painted disc with arms on low tiers; particles and dust layers with nebula knots on basic GPU; a volume with dust absorption on high GPU. The Milky Way seen from inside is the same model integrated along sight lines (H10).

## 4A. Stellar activity: spots, prominences, flares, ejections and jets **[Alex, 19 Nov]**

The same approach, applied to the stars themselves: causes the model already has, rules with published relations behind them, keyed events on the timeline, light by band, and consequences handed to everything nearby. Rule-based and deterministic, like the weather.

**Activity level, as a fact per star and age.** Magnetic activity is driven by spin and the depth of the star's churning outer layer, summarised by the ratio of rotation period to convective turnover time (the Rossby number). From mass, age and rotation (the star's life line already carries spin-down with age):

| Star | Activity |
|---|---|
| Young, fast-spinning stars of any low mass | saturated: heavily spotted, constant flaring, strong X-rays, powerful wind |
| Sun-like, middle-aged | moderate, with an **activity cycle** of years to decades (period from rotation; some stars drawn as flat or in a long quiet spell) |
| Red dwarfs | stay active for billions of years; fully convective ones flare hardest relative to their brightness |
| Old slow rotators | quiet |
| Hot massive stars | no spots of this kind; strong line-driven winds instead (they feed wind bubbles, section 3) |
| Close binaries | spun up by tides, so active at any age |
| Giants | large slow convective cells, mottled surfaces, dusty mass loss |

**What appears, by rule:**

- **Starspots and bright patches**: coverage from activity level; latitude drifting through the cycle; they rotate with the star and so modulate its brightness by a stated fraction. Drawn as facts at the level of groups, with fine texture as decoration.
- **Prominences and filaments**: cool gas held in magnetic loops, seen bright at the limb and as dark threads on the disc in hydrogen light; number and size from activity; they last days to weeks and some erupt.
- **Corona and wind**: X-ray brightness from the activity relation; wind strength and speed; shape (streamers at cycle minimum, all round at maximum). Seen in eclipses, which the kernel can predict.
- **Flares**: events drawn from a power-law spread of energies whose rate follows the activity level, so small flares are constant and great ones rare. Each has a rise and decay time, and a light description: a white-light flash, a large ultraviolet and X-ray spike, a radio burst. Superflares are possible on the stars where they are observed.
- **Mass ejections**: tied to the larger flares; speed, direction and mass drawn; each is an object that travels outward on the time ruler and either hits or misses each planet by geometry.
- **Activity over deep time** follows the life line: a system's first few hundred million years are violent for its planets.

**Consequences, all read by existing parts of the design:**

| Reader | Effect |
|---|---|
| Planet atmospheres (formation design stage 8 and 9) | X-ray and ultraviolet history drives air loss; a red dwarf's planets may be stripped or dried, which changes their class (classification design) |
| Life suite | surface ultraviolet and particle dose; flares as catastrophes or, for some chemistry, as a help |
| Aurorae | on worlds with air and a magnetic field, brightness and latitude from the wind and ejections; visible in the Body and Surface views, and in ultraviolet and radio bands |
| Ships and crews | radiation hazard as a number with warning time (light arrives before particles); a reason to shelter, delay or reroute on the astrogation console |
| Sensing (5A) | a flare raises the background and blinds sensors in its bands; radio bursts drown communications; a quiet star is easier to hide near in X-rays, harder in visible |
| Sound | radio bursts and aurorae as translated signals (sound design N6) |
| Weather | sudden upper-air heating; on thin-aired worlds a change in what reaches the ground |
| Story | Minerva and the sifter get dated flares, storms and blackouts |

**Jets.** One rule set for every scale, because the physics is the same: matter falling onto something through a disc, with part thrown out along the spin axis.

| Source | Cause in the model | Size | Speed | Light |
|---|---|---|---|---|
| Newborn stars | disc accretion during formation stage 3 | up to a light-year, with knots and bow shocks where it meets surrounding gas | hundreds of km/s | hydrogen and sulphur lines in the knots |
| Dying stars with companions | shapes planetary nebulae (section 3) | within the nebula | hundreds of km/s | as the nebula |
| White dwarfs, neutron stars and black holes fed by a companion | mass transfer events (formation design, binaries) | light-years | up to near light speed | radio and X-ray; episodic with the feeding |
| Pulsars | spin and magnetic field of the remnant | wind nebula inside the remnant | near light speed | radio to X-ray, pulsing at the spin rate |
| Galaxy nuclei | black hole feeding episodes (galaxy design events) | thousands to millions of light-years, ending in radio lobes | near light speed | radio, optical and X-ray |

- **Power and length** follow the accretion rate and how long the episode has run; the jet turns on and off with the feeding, and old lobes fade on a clock like battered-galaxy features.
- **Direction** is the spin axis; a wobbling axis gives a corkscrew.
- **Beaming** uses the shift pass already specified (section 5): a jet pointed near the viewer is far brighter and bluer, and its twin fainter, so the same object looks different from different systems, which is true and useful for navigation and story.
- A jet is a hazard along its axis and a landmark visible across a galaxy.

**Rendering**: star surfaces get passes for spots, limb darkening, prominences and corona, under any band recipe (a star in hydrogen light or X-rays looks nothing like its white-light disc); flares and ejections are timed effects driven by the event list; jets reuse the nebula passes. Tiers and degrade order as elsewhere.

**What a later simulation block could replace**: a surface magnetic flux transport model for spots and cycles, a wind model for the space between planets, and for jets a small fluid model of the knots. Each would sit behind the same outputs (activity level, event list, light descriptions), so nothing downstream changes, as with weather.

Sources **[from memory, verify]**: Noyes et al. (1984) and Wright et al. (2011) for activity against rotation and Rossby number; Skumanich (1972) for spin-down with age; Kepler-mission flare statistics (Maehara et al. 2012 on superflares; Davenport 2016); Blandford and Znajek (1977) and Blandford and Payne (1982) for jet launching; Lammer et al. and Owen and Wu (2013, 2017) for atmospheric loss under stellar X-ray and ultraviolet light.

## 5. Relativistic and distance colouring

Because light stays a spectrum until the band recipe, shifting it is simple and exact.

- **Motion (Doppler shift and beaming)**: light from something approaching is shifted bluer and brightened, receding redder and dimmed, by the standard factor from speed and angle. At ordinary speeds nothing visible changes, which is itself true. It matters for fast ships (the Window view at a good fraction of light speed: stars ahead blue and bunched forward, stars behind red and sparse), for jets from black holes and pulsars (the approaching jet outshines the receding one), and for close binary stars.
- **Distance (cosmological redshift)**: far galaxies are reddened by the expansion in the cosmology the campaign uses; at great distances ultraviolet light arrives as visible and visible as infrared, so an infrared band recipe sees further back.
- **Gravity**: light climbing away from a compact object is reddened. Bending of light near black holes is a later render pass, flagged as such.
- Two ways to show any of these: **physical** (apply the shift, then the chosen band recipe, so you see what you would see) and **diagnostic** (a plain blue-to-red map of speed or distance laid over the view, labelled as a measurement).
- Fictional drives may not have a physical view of travel. A physics pack states what, if anything, is seen.

## 5A. Sensing: emitters, sensors and what a station can detect **[Alex, 19 Nov: "we can sense different things"]**

The same light descriptions that colour the views drive detection as gameplay. There is no separate "stealth rating": what can be found, by whom and from how far falls out of two things.

- **Every object has an emission by band**, from its physics: heat (anything warmer than space glows in infrared in proportion to its area and temperature, so a running reactor and its radiators are bright and a powered-down hull is dim but never zero), reflected starlight (brighter near a star, by size and surface), radio (transmitters, radar, magnetospheres, jamming), drive output (in whatever bands the physics pack gives a drive), and for natural things everything in sections 3 and 4.
- **Every sensor has bands, a sensitivity and a resolution**, as construct data (H7A part 7): aperture, how long it may stare, whether it is passive or sends its own pulse.
- **Detection** is then arithmetic: the signal arriving (emission, thinned by distance and by any dust or gas between) against the sensor's threshold and the **background** in that direction and band. The kernel publishes, per observer and target and band: detectable or not, the range at which it becomes so, and how long a stare it needs. Deterministic, with a "Why?".

What that gives the table, with no special rules:

| Situation | What happens, and why |
|---|---|
| A ship running hot | seen in infrared across a system; cannot be hidden except behind something |
| A ship gone cold and silent | invisible on radio and nearly so in infrared; found only by reflected light close in, or by blocking the stars behind it |
| A ship using radar or talking | lights itself up in radio for everyone, further than its own radar can see |
| A jammer | drowns radio in a region; marks its own bearing |
| A hidden base | no lights, no radio; betrayed by waste heat on a cold moon, by a smog dome (weather design), or by fresh tracks at survey resolution |
| Hiding against a bright background | in front of the star, a gas giant's radio belts, or a nebula's glow, the background swamps the signal in that band |
| Hiding behind dust | a dark cloud blocks visible light but passes infrared and radio |
| A dead world's ruins | cold, so only surveyed imaging finds them |
| A living world | oxygen and methane together in its spectrum, vegetation's colour edge, night lights if inhabited: readable from afar with the right instrument |

- **Players and data views.** Detection feeds the disclosure ladder (H6 OBSERVE, H7A part 6): a contact moves from "something warm at this bearing" to identified as more bands and closer range are brought to bear. A seat's station shows only its own sensors' bands (O8). The GM sees the truth and, through lenses, exactly what each seat can sense.
- **Active sensing has a cost**: a pulse reveals the sender. The console shows that trade before the player commits.
- Game-system packs may simplify the result into range bands or dice modifiers; the physical answer stays underneath for those who want it. Fictional sensors and cloaks are physics-pack items and carry the fictional label.

## 5B. Remote survey: the spectrum chart, and finding planets from afar **[Alex, 19 Nov]**

Alex's point: since every object's light is stored across the whole spectrum, showing it is almost free, and it should be the first thing a survey gives you. And planets round distant stars should be discoverable the way they are today: by the dimming when one crosses its star, or by the star's wobble. All of this is simple, exact arithmetic on facts the kernel already holds, so it is cheap, deterministic and runs anywhere.

### The spectrum chart: the first survey result for anything

A view in the core stack (**Spectrum**), available for any body under the one-body ruling (`SSE4-star-census.md`): star, planet, nebula, ship, galaxy.

- Brightness against wavelength, from radio to gamma rays on one stretched axis, drawn from the object's light description as it arrives at the observer (after distance, dust in between and any shift).
- The smooth curve and what it tells you: its peak gives temperature; its level with distance gives size and power.
- Lines marked and named: each is an element or molecule, so composition is read off the chart. A line's displacement gives speed towards or away; its width gives spin or turbulence; doubled lines betray a second star; a changing displacement over days is a wobble (below).
- Shaded strips show which bands **this observer's instruments** cover, so a party with only an optical telescope sees a narrow window of the chart and the rest greyed.
- Every reading on the chart is clickable for "Why?", and the chart is the natural teaching screen for the education purpose: this is how astronomers know what stars are made of.
- What a first look yields without going there: class and temperature, power, rough size, metal content, an age estimate from activity and spin (4A), whether it is double, how active and dangerous it is, and a dust glow in infrared that means belts or a disc.

### Finding planets: the real methods

Whether a planet is found depends on **geometry** (where the observer is: a different vantage sees different transits), **the instrument** (aperture, bands, precision, as construct data), **distance**, **how long it watches**, and **the star's own noise** (spots and flares from 4A hide small signals). The kernel computes the signal and the noise; the result is found, candidate (marginal, firmed up or dismissed by more watching) or missed.

| Method | The arithmetic | Finds best | Gives | Verdict |
|---|---|---|---|---|
| **Transit** | the orbit must cross the star's face from where you are (a fact here, not a chance); dimming equals the planet's area over the star's; needs two or three passes to fix the period | large planets close in, round small quiet stars | size, period, orbit distance; with a spectrum taken during the crossing, the **air's composition** | include first |
| **Wobble** (radial velocity) | the star's line shift swings with a size set by the planet's mass, period and the tilt of the orbit; needs about one full orbit | heavy planets close in | period, orbit shape, a **minimum** mass (true mass only if the tilt is known) | include first |
| **Transit and wobble together** | size plus mass gives density | | what it is made of, so a class from the class atlas with honest error bars ("probably rocky") | falls out |
| **Position wobble** (astrometry) | the star's tiny circle on the sky, larger for heavy planets on wide orbits and for nearby stars | giants far out, near the observer | true mass and tilt | include; it fills the gap the first two leave |
| **Direct imaging** | the planet must be far enough from its star on the sky for the telescope's sharpness, and bright enough against the star's glare; young hot giants in infrared are easiest | wide, young, large; anything at all with a big enough instrument | its own light, so its own spectrum chart, seasons, even a map over time | include; in settings with great telescopes it becomes the main method, and it uses the band recipes directly |
| **Timing** | regular clocks arriving early and late: pulsars, eclipsing pairs, and transits shifting because another planet tugs | planets round pulsars; unseen neighbours | mass and period of the unseen body | include as a small rule; same maths as the wobble |
| **Microlensing** | a foreground mass briefly brightening a background star | cold planets, rogue planets, dark remnants | a mass, once, with no repeat | later, as rare drawn events during long surveys; the only way to stumble on rogues |

Free extras from the same light: an **infrared excess** means dust and belts; a white dwarf with metals in its spectrum has been eating its planets; the day-side temperature from the dip when a planet passes behind its star; **signs of life** (oxygen with methane, a vegetation edge) and **signs of industry** (waste heat, radio leakage, night-side lights, manufactured gases), which is the long-range version of sensing (5A) and a gift for stories.

The built-in biases are the real ones, and they should be: big planets close to small stars are found first; an Earth twin needs years of watching and a fine instrument; face-on systems show no transits and little wobble.

### How knowledge grows: plugging into the ladder, data views and seats

The disclosure ladder (H6 OBSERVE, H7A part 6) gains rungs below "long-range", and it is kept **per audience** (H8.7), so two parties can know different things about the same system:

| Rung | Got by | What the party's view shows |
|---|---|---|
| Catalogued | a sky position and brightness | a point of light |
| Spectrum taken | one look with a spectrograph | the star's nature, activity, companions, dust |
| Surveyed from afar | a survey campaign: an instrument, a duration | some planets, each with what the method measured and its error bars; the rest of the system blank |
| Characterised | longer, better, or several methods | densities and likely classes, air composition, signs of life or industry |
| Long-range, scanned, surveyed | being there (as already designed) | as already designed |

- **Players see measurements, not truth.** A wobble planet shows its minimum mass with the caveat; error bars shrink with more watching; a candidate can turn out to be nothing. The GM always sees the truth alongside what each audience knows, through lenses (H8.6).
- **A survey is an action with a cost in time**, taken as an intent from a science seat ("watch these forty stars for six months with the main array") and resolved by the kernel into a dated list of finds. It makes a fine downtime activity and a reason to own a better telescope.
- **Old catalogues are pack data**: "the Second Survey lists three planets here" can be wrong or incomplete in exactly the ways its instruments would make it so, generated by running the survey rules with that era's equipment.
- **Light takes time [decided, Alex 19 Nov: on by default, with a campaign switch].** A system a thousand light-years away is seen as it was a thousand years ago. The kernel honours this exactly, since it has every system's timeline: remote observation reads the target at the date its light left. With faster-than-light travel in the setting this is a real source of surprises (the catalogue says a quiet star; you arrive to a nebula), and it works in reverse: jump to another galaxy, look back, and see Earth as it was millions of years ago. The view states the look-back time.

The magnetic field overlay wanted today (improved over V3) belongs with the band views: field lines and belts drawn from the field model, lit by the radio and auroral emission they cause. It is specified with the body overlays, and is noted here because its radio glow appears on the spectrum chart.

Sources **[from memory, verify]**: Seager and Mallén-Ornelas (2003) for transit geometry and what it measures; Winn (2010), "Transits and Occultations", in *Exoplanets* (ed. S. Seager; arXiv 1001.2010); Lovis and Fischer (2010) for the wobble method; Perryman, *The Exoplanet Handbook*, for all methods and their limits; Gaudi (2012) for microlensing; Wolszczan and Frail (1992) for pulsar planets.

## 6. Rendering, tiers and determinism

- New passes in the render pass library (research report section 5): `emission-volume` (raymarched glow with dust absorption), `shell-billboards` (layered sheets for mid tiers), `nebula-sprite` (a baked image per vantage and band recipe for low tiers), `band-map` (the recipe as a final colour stage on any view), `shift` (Doppler, beaming, redshift, applied before `band-map`). Text tier: Minerva describes it from the same descriptor.
- Baked sky cube maps (H10) are keyed by vantage, epoch **and band recipe**.
- Degrade order: fewer march steps, then sheets, then sprite. The legend chip and the honesty label never degrade.
- Deterministic facts: descriptor, light descriptions, shifts. Decoration: fine structure.
- Real objects: in a real-universe campaign, catalogued nebulae and galaxies are pins (H10), with measured size, distance and line strengths where known, and procedural structure fitted to them and tagged as fill. Published images are not embedded unless their licence allows.

## 7. Sources to build from **[all from memory; verify titles before use, as for physics blocks]**

- Strömgren (1939) on ionised regions round hot stars; Osterbrock and Ferland, *Astrophysics of Gaseous Nebulae and Active Galactic Nuclei* (2nd ed., University Science Books, 2006), the standard text for line emission.
- Ferland et al., the Cloudy photoionisation code and its release papers (latest checked: "The 2017 Release of Cloudy", RMxAA 53, 385), as the source of the line-strength grid; Allen et al. (2008) MAPPINGS shock model library for remnants.
- Kwok, Purton and Fitzgerald (1978) for the interacting-winds picture of planetary nebulae; Balick (1987) for the shape sequence; Balick and Frank (2002) review of shapes and shaping, including companions.
- Sedov and Taylor for the blast wave; Cioffi, McKee and Bertschinger (1988) for the later cooling phases; Weaver et al. (1977) for wind-blown bubbles.
- Toomre and Toomre (1972) for tidal tails and bridges; Lynds and Toomre (1976) for ring galaxies; Gunn and Gott (1972) for stripping in clusters.
- Rybicki and Lightman, *Radiative Processes in Astrophysics*, for Doppler shift, beaming and synchrotron light.
- For colour practice: the Hubble palette as used for the Eagle Nebula image (released 1995; Hester et al. 1996, AJ 111, 2349, doi:10.1086/117968), and observatory guidance on representative colour.

## 8. Stages (track O, for optics)

| Stage | Contents | Needs | Evidence |
|---|---|---|---|
| O1 | Light descriptions, band set and line table; band recipes with Eye, Camera, two astro palettes and single bands; legend chip and honesty label; `band-map` pass | V1, V12, R1 | a thermal source at a known temperature gives the published colour; switching recipe never changes the state hash; every recipe shows its legend |
| O2 | Stars, planets, moons and built things in all bands (thermal glow, aurora, radio, night lights) | O1, P1 | Solar System fixture: bodies' infrared brightness temperatures within tolerance; Jupiter-like radio emission present only with a magnetic field |
| O3 | Nebula kinds from causes; descriptors; counts in patch envelopes; life on the time ruler; ties to stars and events | O1, G5, F0 | counts per patch follow cause rates times lifetimes; a ring nebula's shell radius matches age times speed; a remnant follows the blast-wave law in a uniform fixture; same descriptor on every device |
| O4 | Line-strength grid as a results-as-data block; colour from composition and lighting star | O3, K10 | a standard reference nebula's line ratios within stated tolerance; metal-poor fixture is weaker in oxygen and nitrogen |
| O5 | Nebula rendering on all tiers (`emission-volume`, `shell-billboards`, `nebula-sprite`, text) and in sky backgrounds | O3, R2, G7 | visual fixtures per kind and tier; frame budget held by the degrade order; sky cube maps keyed by recipe |
| O6 | Galaxy appearance from the model, including battered features with fading clocks | O1, G4, V9 | a quiet spiral, an elliptical and an irregular fixture look as specified; a scripted flyby produces tails that fade on the stated timescale |
| O7 | Shift pass: Doppler, beaming, cosmological and gravitational redshift; physical and diagnostic modes | O1, T3 | colour and brightness against the formulae at set speeds; no visible change below a stated speed; far-galaxy colours match the campaign cosmology |
| O8 | Bands by sensors for player seats; composite recipes; Orac recipe editing | O1, V10, X3d | a seat without an infrared sensor cannot select that band; a composite's legend lists every layer |
| O9 | Sensing (5A): emission by band for ships, bases and natural objects; sensor data on constructs; detection range and stare time against background and intervening dust; contacts feeding the disclosure ladder; active-pulse exposure | O2, O8, T3, V5 | a hot ship is detected in infrared at the range the arithmetic gives and a cold one is not; a target in front of the star is lost in visible; dust blocks visible but not infrared; a radar pulse makes the sender detectable further than it can see; every result answers why |
| O10 | Stellar activity (4A): activity level from mass, age and rotation; cycles; spots, prominences, corona and wind; flares and mass ejections as keyed events with light by band; consequences for atmospheres, life, aurorae, hazards, sensing and sound | O2, F0, F7 | Sun fixture: cycle length, X-ray brightness and flare rates within stated ranges; a young red dwarf fixture is saturated; flare energies follow the stated power law; an ejection hits or misses a planet by geometry; same events on every device |
| O11 | Jets at every scale from accretion episodes; power, length and on-off history; beaming through the shift pass; star-surface render passes under any band recipe | O3, O7, O10 | jet length and brightness follow the feeding history in fixtures; approaching and receding jets differ by the beaming formula; a star's hydrogen-light and X-ray views differ as specified |
| O12 | Spectrum view (5B): chart of any body's light as it reaches the observer, named lines, readings with Why, instrument coverage strips | O1, O2, V1 | Sun fixture: temperature from the peak and listed lines match; a double star shows doubled lines; an observer without a band sees it greyed |
| O13 | Planet finding from afar: transit, wobble, position wobble, direct imaging and timing, each from geometry, instrument, distance, watch time and stellar noise; candidates; infrared excess, life and industry signs | O12, O9, O10, F0 | Solar System seen from fixed vantages: Jupiter by wobble and position, Earth by transit only from near the orbit plane and only with the stated precision and years; results change correctly when the observer moves; same result on every device |
| O14 | Knowledge ladder rungs below long-range, per audience; survey as a timed intent; measured values with error bars in player views; old catalogues generated by rule; light-time option | O13, K9, V5, V10 | two audiences hold different knowledge of one system; a wobble planet shows minimum mass to players and true mass to the GM; with light-time on, a remote view reads the earlier date |
| O15 | Microlensing and other rare survey events (later) | O13, G6 | event rates per survey match the stated odds; a rogue planet can be found this way |
