# SSE4 exotic objects on screen: black holes, pulsars and stripped stars (track XO)

Written 24 Nov 2026 by the design thread "Design exotic stars and black holes", after Alex's "yes to all" (24 Nov 16:41) to starting it. **Status: ACCEPTED by Alex, 24 Nov 2026 17:03 ("Yes to all" to the eleven questions), with two additions he asked for the same day: Composite in the band switch (section 6A) and a diagram in place of a list of readings, which at 17:19 he widened into the Hazard map for any hazardous body (section 3.8). Section 13 records the rulings.** **Addendum of 24 Nov 18:25 (section 15): pulsating variable stars and clock readouts, packages XO19 and XO20, questions 16 to 21 awaiting Alex.** "XO" stands for exotic objects. It is only the short code on this track's packages and never appears in the product.

Read with `SSE4-star-census.md` (which objects exist and who makes them), `SSE4-nebulae-and-light-design.md` (light descriptions, band recipes, shifts, jets, activity), `SSE4-magnetic-fields-design.md` (the field ledger, remnants in MF11, the Mag band) and `interface-style.md` rule 12 (View style, Seen from, Scale, HUD, Band).

Mockup: https://claude.ai/artifact/RHxpiUGRM3QVemo4EveD1n (file and screenshots in `/mnt/project-files/sse4/handover-screens/XO-screens/`). It is a working picture of section 3, traced from the real bending equation in the browser, with the band switch and a "straight light" comparison.

---

## 0. The short version

1. **A black hole is drawn from the real bending of light, on every device, phones included.** For a hole that does not spin, every light ray near it follows one known curve that depends on a single number. The kernel works that curve out once as a small table (the **bend table**), and the renderer looks pixels up in it. That is exact, cheap, and the same on every device. Spin is added second, as a heavier pass for capable machines.
2. **The disc's look is computed, not painted.** Each ring of the disc glows at its own temperature (already in the kernel). The side turning towards you is brighter and bluer, the side turning away dimmer and redder, by the exact factors the kernel already has. The far side of the disc appears arched over the top of the hole and tucked under the bottom, because its light is bent round. None of that is drawn by hand: it falls out of the table.
3. **It will not look like the film by default, and the note says why.** A feeding black hole's disc is millions of degrees near the middle, so to the eye it is blue-white and blinding, not orange. The orange ring people know is a radio picture in mapped colour. The band switch gives both, each with its honest label.
4. **Three general mechanisms carry almost everything else** (section 2): light paths that bend, a feed onto any body through a disc, and a body's shape and outflow set by spin, a companion's pull and its own wind. Black holes, pulsars, nova-making white dwarfs, newborn stars, Wolf-Rayet stars and squashed fast spinners are these three in different amounts, with no object-type list anywhere.
5. **Pulsars are honest too.** A beam is not a visible shaft of light. You see a pulse only when the beam sweeps over you, which is geometry the kernel decides per observer. The cones are a diagram under HUD and Mag. What True shows is the tiny hot star, its glowing wind nebula, and the flicker, slowed down and labelled.
6. **Stripped giants have no surface to see.** A Wolf-Rayet star is seen through its own wind, so its edge is soft and it sits in a nebula it blew itself. With a companion, the two winds collide and make a dust spiral that turns with the orbit. That spiral is plain arithmetic from facts the kernel has.

Eighteen packages, XO1 to XO18 (section 12). The showpiece (XO1 to XO6) needs no new stellar physics: it runs on a fixture first, like the Mag layer did on the Solar System.

---

## 1. What exists today (checked on `main` at 67a5957)

| Thing | State |
|---|---|
| White dwarf, neutron star, black hole as life-line phases (`sse-formation/src/lifeline.rs`) | mass only. Neutron star mass is fixed at 1.4 Suns, black hole mass is a quarter of the starting mass (sketches in the fudge register). No spin, field, kick or radius table |
| A thin disc as a light source (`sse_light`, `Part::Disc`, Shakura and Sunyaev 1973, rings by temperature) | built and tested. Its own limit note says "seen from all sides alike: no tilt, no relativistic beaming or redshift" |
| Shifts (`sse_light::shift`): Doppler and beaming, gravitational and cosmological redshift as exact spectrum transforms | built (O7). Not yet applied per point of a disc |
| Feeding of a galaxy's central black hole over time (`sse-cosmos/src/feed.rs`, dated `agn` episodes) | built as history. Nothing draws the nucleus |
| Binaries (`sse-time/src/binary.rs`): Roche-lobe overflow, common envelope, mergers | reported as outcomes, not fed back into either star (B4), so no star is ever "being fed" |
| Stellar activity, cells and flares (`sse-activity`, `read_activity`), the living star in True (V10c), X-ray and infrared bands in True (V10d) | built. Star glow, churn and visible flares are queued with the renderer thread separately; this note only reads them |
| Jets from accretion at every scale | designed (light design 4A, stage O11). Not built |
| Remnant fields, pulsar spin-down, magnetars | designed (MF11). Not built |
| Light bending near a compact object | named once, as "a later render pass, flagged as such" (light design section 5). **Not designed anywhere until this note** |
| Renderer package R5b (discs, jets, gamma views) | named in the renderer's order, not started |

So the coordinator's reading was right. The showpiece can start on a fixture straight away, and it is the missing design, not missing physics, that has held it up.

---

## 2. Three general mechanisms (thinking wide)

Alex's standing rule is one general mechanism over specials. Behind "draw a black hole" are three capabilities, each with many other uses.

### 2.1 Light paths that bend

Today every view assumes light travels in straight lines. Add one stage to the True pipeline, **`light-path`**, that runs before a pixel looks anything up: it takes the direction a pixel looks in and returns the direction the light really came from, what it passed through on the way, and how its colour and brightness changed. A **bender** is a small data-described rule for doing that. Three benders cover everything:

| Bender | Rule | Uses |
|---|---|---|
| **Strong gravity** | the bend table (3.2), exact near a non-spinning mass; the spin pass (3.6) on capable machines | black holes with and without discs; a neutron star's surface (you see about three quarters of it at once, because light from the back curves round to you, which changes how its hot spots pulse) |
| **Weak gravity** | deflection angle 4GM / (c² b) for a ray passing at distance b | a star's light bent by a white dwarf crossing in front of it (seen in real binaries); microlensing surveys (light design stage O15, which then needs no rule of its own); a galaxy cluster drawing background galaxies into arcs at the cosmic web scale; the classic check, starlight bent 1.75 arcseconds at the Sun's edge |
| **Motion** | aberration: the sky bunches forward at a good fraction of light speed | the Window on a fast ship (light design section 5), which is the same "remap the direction, then look up" step |

One stage, three rules, and the shift pass (O7) rides on its output. A pack can add a fourth bender (a wormhole mouth, a fictional drive bubble) as data, marked fictional.

### 2.2 The feed: matter falling onto anything through a disc

One descriptor, the **feed**, on any body that is gaining matter. It never asks what kind of body it is.

- **Where the matter comes from** (each a block that gives a rate in kg/s and a direction): a companion overflowing its Roche lobe or blowing a wind (B4); gas swept up from the surroundings (Bondi and Hoyle); a star torn apart on a close pass; a newborn star's own disc (formation stage 3); a galaxy's feeding episodes (`feed.rs`).
- **Where the disc ends on the inside**: whichever is largest of the body's surface, its last stable orbit (only compact things have one outside the surface), and the distance at which its magnetic field takes over from the disc (the magnetospheric radius, from the field ledger). That one rule gives: a black hole's dark gap, a white dwarf's bright boundary layer, a strongly magnetic white dwarf with no disc at all and two glowing accretion columns, an X-ray pulsar with hot polar caps, and a T Tauri star's funnel flows.
- **What state the flow is in** follows the ratio of the feeding rate to the Eddington rate (the rate at which the light pushing out balances gravity pulling in). Processes, not classes: three flows each have a strength from that ratio, and they blend where they overlap.

| Feeding, as a share of the Eddington rate | Flow | Look |
|---|---|---|
| far below a hundredth | hot, puffed-up, thin gas that hardly radiates | very faint; a fuzzy glow in radio and X-rays, a steady jet. This is the M87 and Milky Way centre picture |
| about a hundredth to one | thin bright disc | the classic disc: hot inside, cooler outside. Quasars, bright X-ray binaries, dwarf novae in outburst |
| above one | thick, swollen disc that blows much of the matter back out | a funnel with wide outflows; the brightest X-ray binaries and SS 433 |
| none | nothing | **a bare black hole is invisible.** It shows only by bending the stars behind it |

- **Jets** come from the feed and the spin, as the light design already says (O11). This note adds only that the jet's two sides differ in brightness by the beaming factor through the same shift pass, and that the base of the jet is bent by the `light-path` stage like everything else.
- **The other star** is part of the picture (2.3): the donor is stretched towards its companion, a stream leaves its point, and a bright spot marks where the stream strikes the disc's rim.

### 2.3 Shape and outflow: spin, pull and wind

Stars are drawn as spheres with sharp edges. Three effects break that, and one small block family covers them for every body, planets included.

- **Shape from spin and pull.** A body's surface follows the level surface of its own gravity, its spin and any close companion's pull (the Roche model). Three numbers come out: flattening at the poles, stretch towards the companion, and how full its Roche lobe is. Uses: fast-spinning stars half as wide again at the equator as at the poles (Achernar, Regulus, Vega); the teardrop donor in every feeding binary; contact binaries as peanuts; a hot Jupiter stretched towards its star; Saturn and Haumea.
- **Brightness follows gravity.** Where the surface is pulled outward it is cooler and dimmer (von Zeipel's law), so a fast spinner has bright poles and a dark belt, and a teardrop has a dark nose. A face lit hard by a close companion is hotter (the day side of a star beside a pulsar).
- **The wind as something you can see.** Every star has a wind (the activity track has its strength). When the wind is thick enough to hide the surface, what you see is the wind. One number, the wind's optical depth, moves the visible edge out and softens it. When there is a companion, the outflow is wound into a spiral with a spacing of wind speed times orbital period, or pinched into two lobes (already how the light design shapes planetary nebulae). Uses: Wolf-Rayet stars and their dust pinwheels; the spiral shells round dying red giants; a Be star's ring of thrown-off gas; an eruptive giant's twin lobes; a hot Jupiter's comet tail.

### 2.4 A fourth, small one: the time stretch chip

Exotic objects move too fast or too slowly to watch. A pulsar turns thirty times a second; a disc's inner edge orbits a stellar black hole a few hundred times a second, and a supermassive one in hours to days. The living star already runs "on its own clock, six hours a second". Make that one stated control: any True view of something with its own motion carries a quiet **time stretch chip** ("slowed 600 times", "one day a second"), chosen by the renderer so the motion reads, shown always, and pressable to pick real rate where the screen can show it. Positions still come from the kernel's clock (interface rule 10); the chip only says how the view's own clock maps onto it.

Safety, as a hard rule in the recipe schema: **nothing on screen flashes more than three times a second**, whatever the real rate, and `prefers-reduced-motion` turns pulses into a steady glow with the rate in words.

---

## 3. The black hole (the showpiece)

### 3.1 What you would really see

In order from the outside in, for a hole with a thin disc seen from a little above the disc's plane:

1. **The stars behind are pushed outward and smeared** round the hole. A star exactly behind becomes a ring (an Einstein ring). Each star also has a faint second image on the opposite side of the hole, close in.
2. **The disc's near half** crosses in front, as you would expect.
3. **The disc's far half appears above the hole**, as an arch, because its light leaves upward and is bent over the top towards you. You are seeing the top of the far side of the disc.
4. **A thinner arc under the hole** is the underside of the far half, its light bent round below.
5. **A hair-thin bright ring** hugging the dark centre: light that went all the way round the hole once or more before leaving. Its radius is the square root of 27 (about 5.2) gravitational radii, where one gravitational radius is GM / c².
6. **The shadow**: the dark middle, about 5.2 gravitational radii in radius, which is two and a half times the radius of the hole's own surface. Its size depends on mass alone, to within about 4% whatever the spin.
7. **One side of the disc is much brighter.** Gas at the inner edge moves at about half the speed of light. The side coming towards you is brighter and bluer, the side going away fainter and redder. Seen edge-on, the difference in brightness can be tens to one. The film *Interstellar* left this out on purpose; SSE4 keeps it.
8. **A dark gap** between the bright ring and the disc's inner edge, because nothing can orbit stably inside six gravitational radii (for no spin), so the gas there falls in quickly and hardly glows.

Seen from directly above, the disc is a plain ring with a slightly enlarged dark middle. Seen edge-on it is the full "hat brim over a ball" picture. That falls out of "Seen from" with no extra work.

### 3.2 The bend table: exact bending that a phone can do

For a mass that does not spin, a light ray stays in one flat plane, and its path in that plane obeys

  d²u/dφ² + u = 3 (GM / c²) u²,  where u = 1/r and φ is the angle swept round the hole.

The whole family of paths depends on one number: the ray's **impact parameter** b (how far from the hole it would pass if space were flat), measured in gravitational radii. Rays with b below √27 are swallowed. So:

- **`bend_table`** is a two-dimensional table: impact parameter by angle swept, giving the distance from the hole at that angle. A third small column per row gives the total turn for rays that escape, and a flag for capture. About 256 by 256 values. It is the same table for every non-spinning mass in the universe, because everything scales with the gravitational radius. The kernel computes it once with its own pinned maths, hashes it, and ships it as data. It is a "heavy physics ships as tables behind swappable blocks" item with no heaviness at all.
- **To colour a pixel** the renderer works out the pixel's impact parameter and the plane its ray lies in, then asks the table two things. First, at the angles where that plane crosses the disc's plane (there is one such angle per half turn), is the ray's distance within the disc's inner and outer edge? The first crossing that hits is the direct image, the second is the arch or the under-arc, the third is the thin ring. Second, if the ray escapes, by what angle has it turned? That gives the direction to look up in the sky behind.
- **Cost**: a handful of table reads per pixel, and only for pixels near the hole. Everything further than about fifty gravitational radii on screen uses the weak bender's one-line formula. This is the method of the first computed picture of a black hole (Luminet 1979), which was done on punched cards.

Because the table is a kernel fact and the lookups are plain arithmetic, **the picture's geometry is identical on every device**. Sizes on screen (shadow, ring, where the arch sits) are checkable numbers, not looks.

A camera close to the hole (inside about fifty gravitational radii) reads the same table from part-way along each curve; the builder may instead march the equation in the shader on GPU tiers, checked against the table. Closer than the photon orbit (three gravitational radii) is left to the spin pass and the plunge shot (3.9).

### 3.3 The disc's light, point by point

For the place on the disc a pixel's ray hits, at distance r:

- **Temperature** from the existing thin-disc block (`sse_light::disc_temperature`).
- **Shift factor** g (observed frequency over emitted), from the gas's orbital speed, the direction the ray left in, and the climb out of the gravity well. For no spin and a far observer it has the closed form  1/g = (1 − 3M/r)^(−½) · (1 + (M/r³)^½ · b · sin i · sin α), with i the tilt of the view and α the angle round the picture (Luminet 1979). The kernel publishes it as a block and the shader evaluates the same expression.
- **What reaches you**: a glowing body at temperature T, shifted by g, looks exactly like a glowing body at temperature g·T, with total brightness times g⁴. So the pixel's light description is "thermal, at g·T", and the existing band recipes turn that into colour for whatever band is chosen. No new colour code.
- **Texture** (streaks, clumps, flicker) is decoration below the fact, from the seed, turning at the orbital rate of each radius (shown through the time stretch chip). Two devices may differ in a streak, never in the brightness profile.

Later, as a swappable block: the relativistic disc of Novikov and Thorne (1973) with Page and Thorne's (1974) brightness profile, which moves the peak and depends on spin. Until then the Newtonian thin disc is a logged fudge.

### 3.4 What each band shows (and why it is not orange)

How hot the disc's inner edge is depends on the mass: hotter for small holes. The same hole therefore looks completely different by band, and different holes differ from each other. This table is the acceptance picture for XO4.

| Band | Stellar black hole, 10 Suns, well fed | Supermassive, 100 million Suns, well fed (a quasar) | Supermassive, starved (M87, the Milky Way's centre) |
|---|---|---|---|
| **Visible** (Eye, Camera) | inner disc about ten million K: every part hot enough to be blue-white; the outer disc, heated by X-rays from the middle, yellow to red. Brilliant | inner disc about a hundred thousand K: blue-white throughout, outshining its whole galaxy | almost nothing: a dark patch with bent stars |
| **Ultraviolet** | bright across the mid disc | **the peak**: this is where a quasar's disc shines | faint |
| **X-ray** | **the peak**: only the innermost few tens of radii show, a small fierce ring with the strongest one-sided brightening | a compact corona above the middle, flickering | a faint fuzzy glow |
| **Infrared** | the cool outer disc and the donor star | a wide, thick ring of warm dust much further out, glowing and hiding the middle from the side | cool gas and dust nearby |
| **Radio** | the jet, when there is one | jets and lobes reaching far beyond the galaxy | **the ring people know**: glowing thin gas round the shadow, as the Event Horizon Telescope saw it, drawn in a mapped orange ramp with its legend chip saying so |
| **Mag** | field threaded through the disc and wound up the jet's axis; none of the hole's own (the MF design already says a hole carries none) | same | same |

- The mapped orange of the radio band is the default mapped ramp for that view, so the famous picture is one press away and labelled "mapped: radio at 1.3 mm".
- In Eye, a well-fed disc is far too bright to look at. The recipe's exposure note says how far it has been dimmed ("dimmed 40,000 times to show the detail"), like the existing "this would take a 40 minute exposure" note in reverse.
- Under the one-body rule the disc's light also falls on everything nearby: a planet's sky, a ship's hull, the donor star's lit face. Climate and dose read it like any other luminous body.

### 3.5 What the kernel hands over

One op, proposed name **`read_light_paths`**, for a body and an observer at a date. It returns facts only; nothing in it is a picture.

```
LightPaths {
  bender: "strong" | "weak" | none,       // which rule applies at this distance and size on screen
  gravitational_radius_m,
  spin: { a: 0..0.998, axis },             // 0 until XO7; axis from the body's spin
  horizon_m, photon_orbit_m, last_stable_orbit_m,
  shadow_angle_rad, einstein_angle_rad,    // as seen by this observer; checkable numbers
  table: content hash of the bend table (the table itself is fetched once),
  feed: null | {
     rate_kg_s, eddington_share, source,   // which block supplied the matter
     flows: [{ kind: thin | hot | thick, strength 0..1 }],
     disc: { inner_m, outer_m, plane, warp?, parts: light description (rings) },
     hot_spot?, stream?, corona?, jet?     // jet per O11
  },
  observer: { tilt_to_disc, distance_m, clock_rate, tide_per_metre, dose }   // see 3.8
  marks: [ fudges and "argued" items that touched this answer ]
}
```

`read_light` keeps giving the body's total light by band for charts, sensing and the spectrum view. It gains the tilt and the shifts it lacks today (XO3), so the number in the spectrum chart agrees with the picture.

### 3.6 Spin (second, after the showpiece)

A spinning hole drags space round with it. Rays no longer stay in a plane, so the one-number table does not apply. Spin matters in two ways, and the design splits them:

- **Spin's effect on the disc** (in from the start, cheap): the last stable orbit moves in from six gravitational radii to as little as about one for the fastest forward spin, or out to nine for a disc orbiting backwards (Bardeen, Press and Teukolsky 1972). The disc is then hotter, brighter and more efficient (6% of the fuel's rest energy for no spin, up to about 32% at the practical limit of spin, Thorne 1974). All of that is a formula block feeding the same disc.
- **Spin's effect on the bending** (XO7, capable machines): the shadow shifts sideways and flattens on one side, and the thin ring splits. The renderer integrates the ray equations of a spinning hole in the shader on the GPU-high tier. The kernel carries a slow reference integrator used only by tests, so the shader is checked against numbers.

**Until XO7, and always on phones, a spinning hole is drawn with the non-spinning bend table and its true disc edge.** That is a real shortcut and goes in the fudge register: shadow size wrong by up to about 4%, the one-sided flattening missing. The quality badge on the view says "bending drawn for no spin" when it applies.

Where a hole's spin comes from: for stellar holes, drawn from the measured spread at birth and raised by feeding (B3, B4); for a galaxy's centre, from its merger and feeding history (the galaxy histories design already dates both).

### 3.7 Scale: the honest sizes

A black hole of ten Suns is 60 km across. In a system view at **Real** scale it is a point, as it should be, and its bending shows only as a tiny displacement of stars directly behind. Its disc, though, can be a million km wide, so the disc is what you see from afar. The Milky Way's central hole, at four million Suns, has a shadow that would fit inside Mercury's orbit.

- **Real**: everything at true size. The `light-path` stage switches itself to the weak bender, then off, as the hole shrinks on screen. HUD on puts the usual hover outline and name on it.
- **Readable**: the body's mark is enlarged like any other body's, and the bending is drawn round the enlarged mark at the same ratio, so the picture reads. This is the same exaggeration Readable already applies to planets, and the Scale switch already says so.
- **The close-up** (press the body, "click to fly" as on any other) is where the showpiece lives: the camera sits a few hundred gravitational radii out, where Real and Readable agree.

### 3.8 The hazard map: how close can you get, round anything dangerous **[Alex, 24 Nov 17:03 and 17:19]**

Alex asked first for a diagram, not a list of orbit options: how close you can get, the radiation at each distance, what it costs to climb back out, and for a ship the closest it can go before it is "too late", as a figure the GM has and can choose to show the players. Then (17:19) he made it general: "we need them for most objects that are hazardous, a kind of hazard map that can be brought up", naming Jupiter's radiation belts as something to treat carefully. So this is **not a black hole feature**. It is one instrument, called **Hazard map** on screen, for any body with something dangerous round it: a black hole, a neutron star, the Sun, a flare star, Jupiter with its belts, Saturn with its rings, Io's ring of charged gas.

**Where it is.** An entry on the instrument rail at world and star level (declared in the screen document, as every screen is; never a tab), and a "Hazard map" line on any body's card that opens it from the body. At system scale the same zones can be laid over the chart as a HUD overlay ("Hazards"), so the GM sees every body's danger zones at a glance on one chart. Danger is drama, never forbidden: the map informs, and nothing stops a ship crossing a line.

**Why a map and not only a strip.** Several hazards are not a simple matter of distance. A radiation belt is a doughnut round the magnetic equator with an inner edge, an outer edge and gaps, and nothing over the poles. A jet or a pulsar's beam is a cone. Rings and discs are flat. So the instrument has two readings of the same facts:

- **Map**: a cut through the body and its surroundings, in the plane holding its spin and magnetic axes (the same cut the Mag layer already draws in Chart style), with each hazard's zone shaded where it really is. It is a diagram and obeys the one set of view controls: Chart is the cut, 3D turns it, Seen from and Scale work as everywhere.
- **Profile**: the strip chart. Distance along the bottom on a stretched (logarithmic) scale, one thin row per hazard in three tones: clear, harsh, deadly. A profile is always **along a stated line**: "in the equator's plane", "over the poles", or "along this ship's planned course" (from voyages). The line is drawn on the Map. Changing the line changes the profile, and that is the point: over Jupiter's poles the radiation row is nearly clear where in the equator's plane it is deadly.

**The hazards (rows on the Profile, zones on the Map).** Each is an existing block or a one-line formula block. A row or zone appears only where it leaves "clear".

| Hazard | What it reads | Shape on the Map | From | Where it bites |
|---|---|---|---|---|
| **Heat** | the temperature a hull settles at in the body's light, against what this hull can take | round, or shadowed behind a disc or planet | `read_light`, the hull's skin from the data sheet | the Sun and every star (this is what stopped Parker Solar Probe at about ten Sun radii); any fed black hole, from astonishingly far out |
| **Radiation: trapped belts** | dose per hour behind this ship's shield | doughnuts along the field's own lines, with edges and gaps (see below) | `read_field`'s `belts` (built, MF7) | Jupiter above all; Earth, Saturn, Uranus, Neptune; any world or star with a field and something to fill it |
| **Radiation: light** | X-ray and gamma dose from a disc, a corona or a hot surface | round, or a cone for a beamed source | the body's light by band, the shield blocks | fed black holes, neutron stars, the wall between colliding winds |
| **Radiation: storms** | particle dose from flares and mass ejections, quiet against stormy | a wide fan from the star; blocked inside a planet's magnetic bubble | `read_activity` events, `read_field`'s shielding (built) | flare stars, young stars, anywhere outside a bubble during a storm |
| **Beams and jets** | dose and heat inside a jet or a pulsar's cone | a cone along the axis, turning with the spin | O11 jets, XO10 pulsar geometry | accreting compact objects, pulsars, galaxy centres |
| **Debris** | chance of a strike per pass through rings, a belt of rocks, a disc or a comet's dust | a flat sheet with its real gaps, or a cloud | ring and belt regions with optical depth (already data), relative speed | Saturn's rings, debris discs, a young system's leftovers |
| **Tide** | the stretch across the ship's length and across a person | round | 2GM / r³ per metre | small dense things: stellar black holes and neutron stars, thousands of km out. Never, outside the surface, for a supermassive hole or the Sun |
| **Field** | magnetic field strength against what electronics and bodies stand | the field's own shape | the Mag layer's field | neutron stars; magnetars from a thousand km and more |
| **Clocks** | how slowly the ship's clock runs against home | round | √(1 − 2GM / rc²) hovering, √(1 − 3GM / rc²) orbiting | compact bodies only. Shown as "an hour here is 1 hour 25 minutes at home". Feeds ship time in voyages |
| **Air** | heating and drag where a pass would dip into the atmosphere | a thin shell | the aerobraking and landing blocks (built, per ship per world) | any world with air |
| **Climbing out** | the speed change needed to leave from a circular orbit here, against the fuel on board | round | orbital speed there (relativistic where it matters), the ship record's remaining delta-v | everything. This row makes the Park mark |

Rows that never leave "clear" for this body fold into one line ("Tide, field and clocks: nothing to worry about here"), so an ordinary star shows two or three rows and a neutron star most of them. A pack can add a hazard (a fictional field, a psychic zone) as data, marked fictional.

**Radiation belts, done properly.** Belts come from the body's magnetic field, so everything about them is read from the magnetic fields work, not from a rule of thumb about distance:

- **No field, no belts.** `read_field` says whether the body has a field now (measured, computed, none or unresolved). Venus and Mars have no belt zone. A world whose dynamo died has none after that date, so the map follows the time ruler like every effect in the MF design.
- **Shape.** A belt is the set of field lines between its inner and outer shells, as far along each line as particles bounce. For a plain dipole that is the familiar crescent pair either side of the planet in the cut; for a tilted or offset field (Uranus, Neptune) the doughnut is tilted and off-centre, and it wobbles as the planet turns. The Map draws it by tracing the same field lines the Mag layer traces (`renderer/src/field.ts`), so the two pictures always agree.
- **Edges.** The inner edge is where the lines dip into air, rings or the surface; the outer edge is the last closed drift shell inside the magnetic bubble. The kernel already gives both and what sets each (`inner_m`, `outer_m`, `inner_set_by`, `outer_set_by`).
- **Gaps.** Moons and rings sweep particles out along the shells they cross. The kernel already lists them (`gaps`, each with the moon's name). On the Map a gap is a clear lane inside the doughnut with the moon's name on it; on the Profile it is a break in the row. A gap is somewhere a ship can loiter, and a GM should see it.
- **Strength varies across the belt.** Each shell carries its own dose behind six shield thicknesses (`shells[].dose_sv_h`), and the dose at each moon's orbit (`orbits`). The map picks the ship's own shield from that list, so the same belt is deadly to a thin hull and merely harsh to a heavy one.
- **Latitude.** Dose falls away from the magnetic equator along each shell. The first version uses the standard fall-off for particles bouncing on a dipole line as one formula block (`sse.belts.latitude_falloff`), marked as a simplification in the fudge register until the belt model carries pitch angles.
- **Quiet and stormy.** Belts swell after the star's storms. Where the kernel gives a range, harsh and deadly edges are drawn for now and, faintly, for the worst expected, as the aurora design already does for quiet and gusty.
- **What this buys the GM.** The real ways through are visible: come in over the poles and under the belt, as the Juno probe does at Jupiter; park in a moon's swept lane; stay outside the outer edge; or hide in the body's shadow from a storm. And the real traps are visible: Io and Europa sit inside Jupiter's deadly zone, Ganymede in the harsh one, Callisto outside.
- **Honest limits today** (already in the fudge register, F40 and the belts note): electrons only, no proton belt and no slot yet, so Earth's inner belt and thick-shield doses read low. The map carries a mark saying so until MF7 part 2b-ii lands, and needs no change when it does.
- **The Mag band shows the same thing in light.** In True, Mag draws the field and Radio shows the belts' own glow. The Hazard map is the diagram of what that means for a ship.

**Fixed marks**, from the body alone: its surface and the top of its air; for a black hole the surface of no return, the last orbit light can hold and the last stable orbit; the distance inside which a loosely held ship or moon is pulled apart; moons' orbits; ring and disc edges; the edge of the magnetic bubble.

**Two marks for a ship**, in the accent colour, each saying which hazard set it, **for the line chosen**:

- **Park: the closest orbit this ship can settle into and still climb out of.** Set by the fuel on board against "Climbing out", or by whichever hazard turns deadly first if that comes sooner. A deep well makes this surprisingly far out: a ship with 30 km/s in its tanks cannot park closer to the Sun than about a sixth of Earth's distance and still leave. Where a belt has a clear lane or a clear space beneath it, the map shows each safe band, not only the outermost.
- **Dive: the closest pass on a fall-through that still comes back out.** Alex's point about stretched orbits is right, and it is the heart of the map: a ship that falls in from far away and swings past leaves again **for free**, because it arrives with the speed it needs to escape. So a dive goes far deeper than a park, limited only by heat, the radiation soaked up over the hours of the pass, tide, air, and, for a black hole, the distance inside which even a fall-through is swallowed (about four gravitational radii for no spin). It is easier and more vicious, exactly as he said.

Inside the Dive mark is "too late": the ship does not come back, or comes back dead. The card says which ("limited by heat: hull reaches 1,600 K"). For a course from voyages the map also gives **the total for the pass**: dose soaked up, peak hull temperature, peak tide, so "can we cut through the belt quickly?" has a number.

**A quiet hole lets you get very close; a fed one does not.** This falls out of the rows. Round a starved or bare black hole Heat and Radiation are clear almost to the last stable orbit, so the Dive mark sits a few radii from the shadow and only the tide (for a small hole) decides. Round a well-fed stellar hole the heat stops a ship several times Earth's distance from the Sun away, and the hard X-rays stop a thinly shielded one far further out still.

**Who sees it.** The GM always. It shows for the party's ship by default (the ship record gives fuel, hull skin, shield and length); with no ship chosen it uses a stated stock ship. The players see it when the GM opens the eye on it, through the one "Known to" control like anything else, and a seat sees only the hazards its sensors could measure: a ship with no particle detector does not know where the belt is.

**Kernel side.** One op, proposed name `hazard_map {body, ship?, at, line?}`. It returns: the zones, each a shape in the body's own frame (round shell, field-line doughnut by shell numbers with gaps, cone, flat sheet) with its tone edges; the Profile rows along the line; the fixed marks; the ship marks with the hazard that set each; and, for a course, the totals for the pass. It gathers `read_field`, `read_light`, `read_activity`, the ring and belt regions and the ship record; it adds almost no physics of its own. Cost class light.

**Under the map, not on it**: full orbit planning stays in voyages. The Hazard map is the GM's quick answer to "how close dare they go, and which way in".

### 3.9 From further away, and over time

- **In a world's sky** (the Window): a fed hole in the system is a bright star-like point with the colour its band gives; the bent star field shows only through a telescope. A world orbiting a fed hole has a sun that is a disc seen at whatever tilt the orbit gives.
- **At a galaxy's centre**: the galaxy chart in True gets a nucleus whose brightness and jets follow `feed.rs`'s dated episodes, and the centre becomes a place you can fly to with the same close-up. Quasar, quiet nucleus and radio galaxy are one object at different feeding rates and viewing angles. The kernel thread already owns publishing the nucleus as a body (census section 3).
- **On the time ruler**: the disc brightens and fades with the feed; an outburst is a dated event; a star torn apart gives a flare that fades over months; two holes spiralling together is B4's dated merger. Nothing in this note adds a clock.
- **Events for later packages, named so they are not lost**: the plunge shot (camera falling through the horizon, needs the spin pass); the ripples of a merger; evaporation in deep time (the lifetime is already in the kernel).

### 3.10 Players, secrets and the honesty marks

- A seat sees the bands its sensors carry, as everywhere. A party with only visible-light cameras flying past a starved hole sees bent stars and nothing else, which is a fine scene.
- Pulsar beams, jets pointed away and bare black holes are things a survey can miss. Detection is the sensing design's arithmetic on the light this note publishes; nothing here adds a stealth rule.
- A bare hole can be found by its pull (the gravity channel in RK11), by a companion's wobble (O13), or by the stars it bends (O15 through the weak bender).
- Hypothetical objects from the census (boson stars, gravastars and the like) can supply their own bender as pack data. They carry the hypothetical mark on GM views.

---

## 4. Neutron stars, pulsars and magnetars

### 4.1 What is really there

A ball about 24 km across with more mass than the Sun, spinning from a thousand times a second to once in ten seconds, with a magnetic field tilted to its spin. The field and spin-down are MF11's; the mass, radius, birth spin and kick are B3's. This note says what the screen does with them.

| Part | Physics | Seen in |
|---|---|---|
| **The surface** | cools from about a million K over a million years (a cooling table). So compact that light from the back curves round: you see about three quarters of the surface at once (the strong bender, or Beloborodov's one-line form for it) | soft X-rays. In visible light, almost nothing: a faint blue point |
| **Hot polar caps** | where the field lines that reach the light cylinder touch the surface, heated by returning particles, or by infalling gas if the star is fed | X-rays, rising and falling with the spin. The bending of light flattens the pulse, by a computable amount |
| **The radio beam** | a narrow cone along each magnetic pole. Its half-width is a few degrees and widens for faster spin (about 6 degrees divided by the square root of the period in seconds). How the beam is made is unsolved; the MF design marks it "describe only", so the cone's width is an empirical relation and the card says so | **only where the beam points.** See 4.2 |
| **The light cylinder** | the distance at which keeping up with the spin would need light speed: c × period / 2π. About 1,600 km for the Crab, 50,000 km for a one-second pulsar | a HUD and Mag ring |
| **The wind nebula** | the spin's lost energy leaves as a wind of particles that piles up where its push matches the surroundings', and glows. A bright ring round the equator and a jet along each pole, with ripples moving outward over weeks | **the best real sight a young pulsar offers**: visible, X-ray and radio. Size from the spin-down power; it fades as the pulsar ages. For a fast-moving pulsar it is swept back into a bow shape |
| **The remnant** | the supernova shell round a young one (light design section 3) | as designed |

### 4.2 The beam is not a searchlight

A beam of radio waves crossing empty space cannot be seen from the side. There is nothing for it to light. What a real observer gets is this: if the beam's cone sweeps across them once a turn, they see pulses; if it never does, they see no pulsar at all. About one pulsar in five to ten points its beam at any given place.

So, per observer, the kernel decides by geometry whether the pulsar is **seen pulsing**: the angle between the spin axis and the magnetic axis, the angle between the spin axis and the line of sight, and the cone's half-width. It is the same kind of fact as "this planet transits from here". Fly round the star and the pulses appear and vanish. That is true, it is good for stories (a pulsar nobody has charted because its beam misses every inhabited system), and it makes pulsars a navigation aid for those who can see them.

| View | What is drawn |
|---|---|
| **True, any band** | the star as a point or tiny disc; the wind nebula where there is one; the pulse as a brightening of the point when the beam crosses the camera, at the time stretch chip's rate. No cones |
| **True with HUD on, 3D, Chart, Mag** | the diagram: spin axis, magnetic axis, the two cones turning, the light cylinder, the field lines from the Mag layer. Clearly a diagram, as HUD already is |
| **The card** | "Seen pulsing from here: yes, 33 times a second" or "no: its beam passes 14 degrees wide of here", with Why. A small folded pulse shape, drawn from the cone geometry and the bending |
| **Sound** (N track) | a pulsar is a click track or, for the fastest, a musical note. The sound design already translates signals; this hands it the period and pulse shape |

Question 3 in section 13 asks whether True should also offer the cones as faint light. The recommendation is no.

### 4.3 Life story on the time ruler

All of this is MF11 and B3's physics; the screen needs it as dated facts.

- **Born** spinning fast with a kick of hundreds of km/s, inside its supernova's shell. Often it has left the system it was born in, which is why old pulsars are found alone.
- **Spins down** steadily: field ≈ 3.2 × 10¹⁹ √(P Ṗ) gauss, age ≈ P / 2Ṗ, power lost = 4π² I Ṗ / P³. The wind nebula dims with the power.
- **Glitches** (Alex, 24 Nov 16:53, "cheap cool physics wins"): now and then a young pulsar's spin suddenly jumps faster, by about a part in a million, then settles back over days to months. The cause is the one confirmed natural superfluid: the neutrons inside flow without friction, so they do not slow down with the crust. They fall out of step, and every so often they catch the crust up with a jerk. It is far too small to see as a change in turning. So it is shown as what it is: a dated event on the pulsar's own timeline, a small step on the spin trace on its card, a tiny rise in pitch in the sound, and, for anyone using pulsars as clocks to navigate by, a clock that has just jumped. One small block, `sse.pulsar.glitch`: how often and how large from the pulsar's age and spin-down rate, drawn from measured statistics and marked so, using the activity track's event list. The card's Why says what is inside. The same superfluid explains why the young neutron star in Cassiopeia A is cooling faster than it otherwise would; the cooling table carries that as a cited row.
- **Dies** as a radio pulsar after ten million years or so, when it crosses the "death line" in spin and field. It is then a cold dark ball, findable only by its pull or by the gas it sweeps up.
- **Reborn** if a companion feeds it: gas spins it up to hundreds of turns a second (a millisecond pulsar), and while it is fed it is an X-ray source with a disc cut off at the magnetospheric radius (the feed, 2.2), sometimes with bursts when the piled-up gas ignites.
- **Companions cooked**: a spun-up pulsar's wind can boil its small companion away. The companion has a hot day side and a comet-like tail (shape and outflow, 2.3). Planets round pulsars are real, and O13's timing method already finds them.

### 4.4 Magnetars

A neutron star born spinning fast enough to wind its field up a thousand times stronger (MF design 4.x: Make for ten seconds, then Squeeze, then Keep). They turn slowly, once in 2 to 12 seconds, so their rotation can be shown at real rate.

- **Steady look**: X-ray bright from field decay, not from spin; hot spots; in Mag, a twisted field unlike a pulsar's tidy dipole.
- **Bursts and giant flares** are keyed events on the activity track's existing event machinery (the same list that carries stellar flares): a rate that falls as the field decays over ten thousand to a million years, a power-law spread of sizes, each with a light description. A giant flare is a tenth of a second of gamma rays brighter than the whole galaxy, then minutes of glow pulsing at the spin rate.
- **Consequences reach far**: the MF design already lists a giant flare as a galaxy-scale event. The 2004 flare disturbed Earth's upper air from fifty thousand light-years. Within a few light-years it is a catastrophe for unshielded life. Dose and warning time come from the existing blocks.
- Honest limit: the starquake picture of why flares happen is argued. Timing is drawn from measured statistics, as for stellar flares, and marked.

### 4.5 White dwarfs, briefly

Their look (colour by temperature and atmosphere, crystallising, coloured cooling) belongs to the remnants package and to the new-physics demo, which Orac builds on camera. **Nothing in this note builds coloured white dwarfs.** What this note gives them is the feed (discs, bright spots, nova shells, and the disc-less magnetic kind with two glowing columns) and the weak bender (a white dwarf crossing its companion brightens it very slightly instead of dimming it, a real and measured effect).

---

## 5. Stripped giants and other stars that are not spheres

### 5.1 Wolf-Rayet stars: a star seen through its own wind

A massive star that has lost its hydrogen, by its own wind or to a companion (B2 and B4 decide which and when). It blows off a Sun's mass every hundred thousand years at one to three thousand km/s. The wind is so thick that the surface is hidden inside it.

| What is published | From | What the screen does with it |
|---|---|---|
| Mass-loss rate, wind speed, the hidden surface's temperature (30 to 200 thousand K) | B2's stripped-star phase and a wind recipe (Nugis and Lamers 2000; Sander and Vink 2020) | sets everything below |
| **Wind optical depth and the radius where the wind turns clear** | one block (2.3) | the star is drawn as a soft-edged glow with no sharp limb, larger than its real surface. As the number falls towards zero, the same pass gives an ordinary sharp-edged star, so there is no "Wolf-Rayet mode" |
| Surface make-up: helium and nitrogen (the WN kind), then carbon (WC), then oxygen (WO) | which layer has been exposed, from the life line | the spectrum is dominated by bright emission lines from the wind, not dark lines. The line table gains the helium, carbon, nitrogen and oxygen lines that mark them. In Camera the star is blue-white with a faint coloured cast; in the astro palettes and the Spectrum view it is unmistakable. The kind (WN, WC, WO) is a label the class atlas reads off the make-up |
| **The ring nebula** | the fast wind catching up with the slow wind the star blew as a red giant (already a nebula kind, "wind bubbles and shells") | a clumpy shell a few light-years across, lit by the star |
| What comes next | the life line | a supernova with no hydrogen in it, or a quiet collapse to a black hole. The card says which is expected |

### 5.2 The pinwheel: two winds colliding

When a carbon-rich Wolf-Rayet star has a hot massive companion, their winds meet in a curved wall. Dust forms in the wall and streams outward, and because the pair orbit, the stream winds into a spiral, like water from a turning sprinkler.

- **Spacing between turns = wind speed × orbital period.** For a 241-day pair that is a tight spiral; for an eight-year pair on a stretched orbit, dust forms only at closest approach, so each orbit leaves one shell, and there is a nested set of them (seventeen were counted round WR 140).
- **Opening angle of the wall** from the ratio of the two winds' push.
- **Seen in infrared**, glowing at a few hundred to a thousand K, cooling outward. Dark in visible light. So this is an Infrared band sight, and a real reason to own an infrared sensor.
- It **turns with the orbit** on the time ruler. Scrub a year and the spiral winds on.
- The same block draws the thin spiral shells round a dying red giant with a companion (the light design's "shed layers" get their spiral for free), and feeds the lobes of planetary nebulae.
- Hazard note for the table: the wall itself is a strong X-ray source, and the star is a supernova waiting.

### 5.3 Other shapes the same blocks give

| Star | Mechanism (2.3) | Sight |
|---|---|---|
| Fast spinners (Achernar, Regulus, Vega, Altair) | shape from spin; brightness follows gravity | visibly squashed, with bright poles and a darker belt. Seen pole-on it looks round and too bright, which is how Vega fooled people |
| Be stars | a fast spinner at the edge of flying apart sheds a ring of gas at its equator | a thin glowing ring in hydrogen light, coming and going over years |
| The donor in a feeding pair | shape from pull, lobe full | a teardrop with a dark nose, a stream from the point, a bright spot on the disc's rim |
| Contact pairs | two lobes overfilled | a peanut, one colour all over because they share an envelope |
| Eruptive giants (Eta Carinae) | an outburst as a dated event, outflow pinched by spin or a companion | two lobes and a skirt, expanding on the time ruler (a nebula kind) |
| Red supergiants | a few enormous cells, uneven dusty mass loss | lumpy and changing. **This is the renderer's queued star-glow and churn work, reading `read_activity`'s cells. Not redesigned here** |
| A neutron star swallowed by a giant (hypothetical) | the census's Thorne-Zytkow object | looks like a red supergiant with the wrong spectrum. Label only |

---

## 6. View controls: what each switch does here

| Switch | Black hole | Pulsar or magnetar | Wolf-Rayet and shaped stars |
|---|---|---|---|
| **Chart** | the body's symbol; a ring for the disc, a tick for the jet axis; honesty marks as usual | symbol with a small beam tick when seen pulsing from here | symbol; a spiral mark when there is a pinwheel |
| **3D** | the diagram turned: disc plane, jet axis, the last stable orbit and photon orbit as rings | spin and magnetic axes, cones, light cylinder | orbit, the wall between the winds, the spiral's centre line |
| **True** | section 3: bent light, by band | section 4.2: the point, the nebula, the flicker | soft-edged star, nebula, spiral in infrared |
| **Wireframe** | the bent grid: a flat grid of lines behind the hole, drawn through the same `light-path` stage, so the wireframe shows the bending itself | cones and field lines as wire | shells and spiral as wire |
| **Seen from** | Above: ring. Tilted: the hat. Edge-on: the full arch and the strongest one-sided brightness. Free as usual | changes whether you are in the beam | changes whether the spiral reads as a spiral or as stacked arcs |
| **Scale** | 3.7 | the star is a point at any scale short of the close-up; the nebula is what Real shows | Real shows how large the wind's glow is beside the hidden star |
| **HUD** | names and rings for: shadow, photon ring, last stable orbit, the approaching side, the far side's arch, jet axis, with distances | cones, axes, light cylinder | the star's true surface inside the glow, the wall, the companion |
| **Band** | 3.4 | X-ray for the surface and caps, Radio for the pulse and nebula, Mag for the field | Infrared for the dust, Ultraviolet and X-ray for the wall |

No new switch. One addition to every True view of these objects: the time stretch chip (2.4).

### 6A. Composite: several bands in one picture, on every view **[Alex, 24 Nov 17:03]**

Alex asked for a **Composite** entry at the end of the band switch, as the early demo had: press it and a drop-down lets you combine bands into one picture. The light design already names composites as a kind of band recipe ("several bands layered, as in observatory press images"). This makes it a control, and it is one mechanism for every view that has a band switch, not something for exotics.

- **Where.** Last entry of Band, after Mag: Visible, Infrared, Ultraviolet, X-ray, Radio, Mag, **Composite**. Pressing it shows the last composite used at once (so it is one press for the common case) and opens a small drop-down from the entry itself.
- **The drop-down** is a tick list of the bands this view and this seat can show, each with its colour beside it. Tick two or more. Visible keeps its natural colours; every other band is laid over it in its own stated colour. The starting colours follow observatory habit: X-ray violet, ultraviolet blue, infrared red, radio orange. A colour can be changed from the same row. Mag may be ticked too: field lines drawn over the light picture.
- **Presets** at the top of the list, as plain data in the presentation pack: "Observatory" (X-ray, visible, infrared, radio), "Hot and cold" (X-ray with infrared), "Radio over visible". A pack or Orac can add more. The viewer's own mix is remembered per device, like the rest of the view choice.
- **Honesty.** The legend chip lists every layer and its colour ("Composite: visible actual · X-ray violet · radio orange"), and reads "mapped" as soon as any band but Visible is in. It never degrades.
- **Each band keeps its own brightness scale** before the layers are added, so a faint radio jet is not drowned by a bright disc. The chip says so ("each band scaled to its own brightest").
- **Seats** can tick only the bands their sensors carry, as with single bands. A composite is never a way round that.
- **Cost.** The picture is worked out once per ticked band and added. Phones allow two bands at a time and say why on the third.
- **What it is for here.** A fed black hole in Observatory shows the X-ray inner ring, the blue-white disc and the radio jet in one frame. A pulsar shows its X-ray surface inside its radio and visible wind nebula. A Wolf-Rayet pair shows the infrared spiral round the visible stars with the X-ray wall between them.
- **In the code.** Band becomes "one band or a list with colours" in the saved view choice (`ui-contract/view-controls.ts`), read from the sheet's Bands table like every band id. A view that already declares its bands gets Composite with no work of its own.

---

## 7. Rendering, tiers and what a phone gets

| Tier | Black hole | Pulsar | Stripped and shaped stars |
|---|---|---|---|
| **GPU-high** (desktop WebGPU) | bend table or marched rays; the spin pass after XO7; disc with moving texture; jets as volumes; full sky bent; several ring orders | wind nebula as a volume; surface with bent-light hot spots | wind glow as a volume; spiral as a volume with its own light; shaped star meshes |
| **GPU-basic** (most laptops) | bend table; two ring orders; disc texture simpler; sky bent | nebula as layered sheets | glow as a shell; spiral as sheets |
| **Phone (WebGL2)** | **the same bend table, the same geometry.** One float texture, a few reads per pixel, only inside a small box round the hole; direct image and the arch (two orders), no thin ring; the sky behind bent from the baked cube map; disc untextured, coloured by temperature and shift. Never the spin pass | point, flicker (capped), nebula as one sprite | glow as a sprite; spiral as a drawn curve with thickness |
| **Canvas / Chart** | symbol and rings | symbol | symbol |
| **Words** | Minerva describes it from the same descriptor, including what cannot be seen from here | same | same |
| **Still** ("Take photograph") | the spin pass at full quality, any device that can wait | | |

- **Degrade order** for the hole: drop disc texture, then the thin ring, then the under-arc, then sky bending outside twice the shadow. The shadow's size, the arch and the one-sided brightness never degrade, because they are the physics.
- **The badge** says what was dropped, as it does now, and "bending drawn for no spin" where that applies.
- **Deterministic facts**: the bend table, every radius and angle, the disc's rings and shift factors, whether a pulsar is seen pulsing, shell and spiral sizes. **Decoration**: streaks, clumps, wisps, flicker, granules.
- **Cost class** for `read_light_paths`: light (closed-form numbers and a cached table). The reference integrator for spin is test-only and never runs for a click.
- **Phone checks are emulated**, as the renderer track's are; say so in the evidence.

---

## 8. Blocks (small, cited, editable)

Names are plain words. Each is a formula block or a table in a pack, with its source on the card. "Have" means it is in the kernel today.

| Block | What it gives | Source | Standing |
|---|---|---|---|
| `sse.gravity.gravitational_radius` | GM / c² | textbook | settled |
| `sse.gravity.horizon` | surface of no return, with spin | Kerr 1963 | settled |
| `sse.gravity.photon_orbit` | where light can circle; the shadow's edge, √27 for no spin | Bardeen 1973; Luminet 1979 | settled |
| `sse.gravity.last_stable_orbit` | inner edge for orbiting matter, by spin and direction | Bardeen, Press and Teukolsky 1972 | settled |
| `sse.gravity.light_path` (the bend table) | distance against angle swept, by impact parameter | Schwarzschild 1916; Darwin 1959; Luminet 1979 | settled |
| `sse.gravity.weak_bend` | 4GM / (c² b); Einstein ring angle | Einstein 1915, 1936 | settled |
| `sse.gravity.clock_rate` | how slowly a clock runs, hovering or orbiting | textbook (Misner, Thorne and Wheeler 1973) | settled |
| `sse.gravity.tide` | stretch per metre | textbook | settled |
| `sse.belts.latitude_falloff` | how a belt's dose falls away from the magnetic equator along a field line | textbook trapped-particle motion (Roederer 1970; Walt 1994) | settled for a dipole; a stand-in for real pitch angles |
| `sse.hazard.debris_strike` | chance of a strike on a pass through a ring or belt, from optical depth, particle size and speed | textbook cross-section argument | rough |
| `sse.approach.hull_heat` | the temperature a hull settles at in a body's light | Stefan and Boltzmann; textbook radiative balance | settled |
| `sse.approach.climb_out` | speed change to leave from a circular orbit, and the deepest free fall-through | textbook; Misner, Thorne and Wheeler 1973 for the swallowed distance | settled |
| `sse.gravity.spin_light_path` (reference, XO7) | ray paths round a spinning hole | Carter 1968; Bardeen 1973; James et al. 2015 | settled |
| `sse.feed.eddington_rate` | feeding rate at which light balances gravity | Eddington 1926; textbook | settled |
| `sse.feed.capture_from_surroundings` | gas swept up by a moving mass | Bondi and Hoyle 1944; Bondi 1952 | settled, rough in practice |
| `sse.feed.roche_lobe` | size of each star's lobe | Eggleton 1983 | settled (check `binary.rs` for an existing copy first) |
| `sse.feed.disc_outer_edge` | where the companion's pull cuts the disc off | Paczyński 1977 | settled |
| `sse.feed.inner_edge` | largest of surface, last stable orbit, magnetospheric radius | Ghosh and Lamb 1979 for the last | settled, factor of 2 |
| `sse.feed.efficiency` | share of rest energy radiated, by spin | Novikov and Thorne 1973; Thorne 1974 | settled |
| `sse.feed.flow_strengths` | thin, hot and thick flow strengths from the Eddington share | Narayan and Yi 1994; Abramowicz et al. 1988; Yuan and Narayan 2014 | **argued** at the joins |
| `sse.light.disc_temperature` | ring temperatures | Shakura and Sunyaev 1973 | **Have** |
| `sse.light.disc_relativistic` (later) | the relativistic brightness profile | Novikov and Thorne 1973; Page and Thorne 1974 | settled |
| `sse.light.disc_shift` | the shift factor g at a point of the disc | Luminet 1979; Cunningham 1975 | settled |
| `sse.light.shift` | g applied to a spectrum | Rybicki and Lightman | **Have** (O7) |
| `sse.remnant.surface_seen` | share of a compact star's surface visible, and hot-spot pulse shape | Pechenick, Ftaclas and Cohen 1983; Beloborodov 2002 | settled |
| `sse.pulsar.beam_width` | cone half-angle from period | Rankin 1993; Lorimer and Kramer 2005 | **empirical** |
| `sse.pulsar.seen_from` | whether the cone crosses an observer | geometry (Radhakrishnan and Cooke 1969) | settled |
| `sse.pulsar.light_cylinder` | c P / 2π | Goldreich and Julian 1969 | settled |
| `sse.pulsar.wind_nebula` | size of the glowing ring from spin-down power and surrounding pressure | Rees and Gunn 1974; Kennel and Coroniti 1984; Gaensler and Slane 2006 | settled, rough |
| `sse.pulsar.glitch` | how often the spin jumps and by how much, and how it settles | Anderson and Itoh 1975; Link, Epstein and Lattimer 1999; Espinoza et al. 2011 | **drawn from statistics** |
| `sse.magnetar.burst_rate` | rate and sizes of bursts by field and age | Kaspi and Beloborodov 2017 | **drawn from statistics** |
| `sse.shape.spin_and_pull` | flattening, stretch and lobe filling | Roche model (Kopal 1959) | settled |
| `sse.shape.gravity_darkening` | surface brightness against local gravity | von Zeipel 1924; Lucy 1967; Espinosa Lara and Rieutord 2011 | settled, exponent argued for cool stars |
| `sse.wind.optical_depth` | how far out the visible edge sits | de Loore et al. 1982; Hamann, Gräfener and Liermann 2006 | settled |
| `sse.wind.hot_star_mass_loss` | mass-loss rate for stripped stars | Nugis and Lamers 2000; Sander and Vink 2020 | **argued** (factor of 3) |
| `sse.wind.collision_wall` | where two winds meet and its opening angle | Stevens, Blondin and Pollock 1992; Cantó, Raga and Wilkin 1996 | settled |
| `sse.wind.spiral` | spacing and shape of an outflow wound by an orbit | Tuthill, Monnier and Danchi 1999; Lau et al. 2022 | settled |

Constants (c, G, the Thomson cross-section, proton mass) come through the one "Look up" door to the master sheet. No typed copies.

---

## 9. Shortcuts that will go in the fudge register

Each is logged in `docs/fudges.md` by the pull request that adds it.

| Shortcut | Why | What replaces it |
|---|---|---|
| A spinning hole's light drawn with the no-spin table (always on phones; everywhere before XO7) | the exact paths need per-pixel integration | the spin pass (XO7) on capable tiers; stays on phones, with the badge |
| The thin disc's brightness from the Newtonian law | it is built and close outside about ten gravitational radii | `sse.light.disc_relativistic` |
| The disc is flat and infinitely thin, and does not shadow or light itself | a thick or warped disc needs a volume | thick flow as a volume on GPU-high; warps from a tilted spin (Bardeen and Petterson 1975) later |
| The hot thin flow and the corona drawn as a glow of stated size and slope, not a computed gas | the real thing is a magnetised fluid simulation | a tabulated set of such simulations as a swappable data block |
| Disc streaks, flicker and jet knots are seeded decoration | turbulence cannot be computed in a browser | none planned; the brightness profile stays the fact |
| Only two or three image orders drawn | higher ones are thinner than a pixel | none needed |
| Pulsar beam width from an empirical relation; the beam's own brightness pattern a plain cone | the mechanism is unsolved | whatever the field one day agrees on |
| Hazard map: a belt's dose away from the magnetic equator from a dipole fall-off rule; belts are electrons only until the proton belt lands (F40) | the belt model does not carry pitch angles or protons yet | MF7 part 2b-ii and a pitch-angle belt model, behind the same op |
| Readable scale draws the bending round an enlarged mark | so it can be seen at system scale | Real scale, already offered |
| Motion shown slowed or sped | the eye cannot follow thirty turns a second | none; the chip states the rate |

---

## 10. How this joins the other tracks

| Track | What this note asks of it | What it gets back |
|---|---|---|
| **B3 remnants, B4 binaries fed back** (proven core) | spin, radius, kick and birth facts for remnants; feeding rates and lobe filling as dated facts on each star | nothing new to build for the screen; XO reads those facts |
| **MF11** (remnant fields) | field strength, tilt and decay for neutron stars; the magnetospheric radius | pulsars and magnetars on screen through the Mag layer already built |
| **O track** (light) | `read_light`'s disc gains tilt and shifts (XO3); wind nebula, collision wall and spiral as nebula kinds; line table rows for Wolf-Rayet lines; O11 jets; O15 uses the weak bender | the picture and the spectrum chart agree |
| **G track** (kernel) | the nucleus as a body with spin and a feed from `feed.rs` | the galaxy's centre drawn and visitable |
| **Renderer** | the `light-path` stage, the hole, the pulsar and wind passes (its R5b becomes XO4 to XO6) | one stage that also serves the fast-ship Window and cluster arcs |
| **Chart room** | the time stretch chip in the one ViewControls; card lines ("Seen pulsing from here") | |
| **NAV and voyages** | clock rate near a mass in ship time; "no stable orbit inside" in the stability card | |
| **Sound** | period and pulse shape for pulsars | |
| **Class atlas** | stellar scheme labels read off these facts: black hole, neutron star, pulsar, magnetar, WN, WC, WO, Be, X-ray binary, nova-maker, microquasar, quasar | no class ever gates a block |

---

## 11. Checks a builder can run

Real objects as fixtures, each a number against a measurement.

| Fixture | Check |
|---|---|
| Sun's limb | weak bend 1.75 arcseconds |
| Bend table | captured below b = √27; total turn matches 4/b for large b to four figures; table hash identical natively and in the browser harness |
| M87's centre (6.5 thousand million Suns, 16.8 megaparsecs) | shadow diameter about 40 millionths of an arcsecond (measured ring 42 ± 3) |
| Milky Way's centre (4.15 million Suns, 8.2 kiloparsecs) | shadow about 50 millionths of an arcsecond |
| Luminet's picture (no spin, view 10 degrees above the disc) | positions of the direct image, the arch and the ring at a list of radii; brightest point on the approaching side at the stated radius |
| Cygnus X-1 and a quasar fixture | disc peak in X-rays for one and ultraviolet for the other |
| Last stable orbit | 6, about 1.24 (at spin 0.998) and 9 gravitational radii |
| Clock rate | hovering at 3 Schwarzschild radii: 0.816 |
| Crab pulsar (P = 0.0334 s, Ṗ = 4.2 × 10⁻¹³) | field about 4 × 10¹² gauss, age about 1,260 years (true age 970), power about 4.5 × 10³¹ W, light cylinder about 1,600 km, wind ring about 0.14 parsec |
| Vela pulsar | a glitch of about two parts in a million roughly every three years; none expected on a pulsar a hundred million years old |
| A pulsar fixture, observer moved round it | seen pulsing only inside the cone; share of sky covered matches the cone |
| WR 140 | spacing between dust shells = wind speed × 7.93 years |
| WR 104 | one turn of the spiral per 241 days |
| Achernar | flattening near the measured 1.35 to 1 or more; poles brighter |
| Hazard map, the Sun, a probe with Parker Solar Probe's shield | heat limit close to its real closest pass, 9.9 Sun radii from the centre |
| Hazard map, Jupiter (from `read_field`'s Solar System fixture) | in the equator's plane the radiation row is deadly at Io and Europa and harsh at Ganymede for a thin shield, clear at Callisto; there is a break at each moon's swept lane; over the poles the same row is clear down to under the belt, where Juno passes at about 1.06 Jupiter radii; Venus and Mars show no belt zone |
| Hazard map, Saturn | the Debris zone is the rings with their real gaps; the belt's inner edge is set by the rings |
| Hazard map, a world whose field has died | the belt zone is gone after that date on the time ruler |
| Hazard map, a 10 Sun hole, bare | Dive mark set by tide at thousands of km; fed at a tenth of the Eddington rate, set by heat or radiation at several au or more, by the shield |
| Composite | legend lists every ticked band; a seat cannot tick a band it lacks; a phone stops at two |
| Any view | no element flashes above 3 per second; reduced motion gives a steady glow |

---

## 12. Packages (track XO)

One package, one builder, one pull request, evidence in it. Owners are proposals; the coordinator assigns. XO1 to XO6 are the showpiece and need nothing from B3 or B4.

| Id | Package | Needs | Proposed owner | Evidence |
|---|---|---|---|---|
| **XO1** | Gravity blocks and the bend table: gravitational radius, horizon, photon orbit, last stable orbit by spin, weak bend, clock rate, tide; `bend_table` computed, hashed and shipped; fixtures | none | Build light and nebulae | section 11 rows 1 to 4, 7, 8; table hash the same on every device |
| **XO2** | The feed descriptor on a fixture: Eddington rate, flow strengths, inner and outer edge, efficiency; three fixture bodies (a 10 Sun hole with a companion, a quasar, a starved nucleus); op `read_light_paths` | XO1 | Build light and nebulae | fixtures reproduce the three columns of 3.4 as band totals |
| **XO3** | The disc's light point by point: `disc_shift`, tilt and shifts into `read_light` so chart and picture agree; exposure note for over-bright sources | XO2, O7 | Build light and nebulae | Luminet fixture brightness profile; approaching side brighter by the stated factor |
| **XO4** | The `light-path` stage in the renderer with the strong and weak benders; the black hole in True in every band; sky bent from the cube map; HUD labels; degrade order and badge | XO1 to XO3 | Build photoreal renderer (its R5b) | screenshots for 3.4's nine cells at desktop and phone width; shadow and arch positions measured off the frame against the table; frame budget held |
| **XO5** | Phone path: table lookups in WebGL2 inside a screen box; two orders | XO4 | Build photoreal renderer | emulated phone holds its frame rate; geometry identical to desktop within a pixel |
| **XO6** | The close-up and the controls: fly to a hole, Seen from, Scale behaviour (3.7), Wireframe bent grid, time stretch chip in ViewControls | XO4 | chart room, with the renderer | e2e: press a hole, change band and view, readings show with Why; chip always present on moving True views |
| **XO7** | Spin: reference integrator in the kernel (tests only), spin pass in the shader on GPU-high and for stills; badge elsewhere | XO4 | Build photoreal renderer, with light | shader against reference at a grid of rays; shadow shift and flattening at spin 0.9 match published figures |
| **XO8** | Real feeds: capture from surroundings, lobe overflow and wind feeding from B4's dated facts, outbursts and tidal disruptions as events; donor star, stream and hot spot | XO2, B4 | proven core, with light | a fixture binary turns on and off on the time ruler; disc edges follow the orbit |
| **XO9** | The galaxy's centre as a body: spin and feed from `feed.rs`; nucleus brightness and jets on the galaxy chart in True; fly to it | XO4, G nucleus package | kernel thread, with the renderer | a galaxy's past quasar phase shows on the time ruler; the close-up opens |
| **XO10** | Neutron stars on screen: surface seen through the strong bender, hot caps, pulse shape; `seen_from`; light cylinder; card lines; pulse under the time stretch chip with the flash cap; glitches as dated events with the spin trace on the card | XO1, B3, MF11 (starts on MF0's three fixture pulsars) | light, then renderer | Crab and Vela rows of section 11; seen-from check; flash cap test |
| **XO11** | Pulsar wind nebula and bow shape as nebula kinds; fading with spin-down | XO10, O3 | Build light and nebulae | Crab ring size; nebula gone by the death line |
| **XO12** | Magnetars: bursts and giant flares on the activity event list with light and reach; twisted field in Mag | XO10, MF11, O10 | light, with the renderer's Mag layer | rate falls with field decay; a giant flare's dose at stated distances |
| **XO13** | Shape from spin and pull, gravity darkening, lit faces: blocks and a shaped-body mesh in the renderer (stars and planets alike) | B1 one body | light for blocks, renderer for mesh | Achernar and Saturn flattening; teardrop donor at lobe filling |
| **XO14** | The wind you can see: optical depth and soft edge; Wolf-Rayet line rows; ring nebula tie; class labels | B2, O4 | light, then renderer | a wind's edge moves out smoothly as mass loss rises; an ordinary star is unchanged |
| **XO15** | Colliding winds and spirals: wall, dust spiral and shells, turning on the time ruler; red giant spiral shells by the same block | XO14, B4 | light, then renderer | WR 140 and WR 104 rows |
| **XO17** | Composite in the band switch, every view (6A): the entry, drop-down, presets as pack data, per-band scaling, legend, seat and phone limits; the saved view choice carries a band list | V10b band switch (built) | chart room for the control, renderer for the layering | section 11 Composite row; works on the system chart, the galaxy chart and the globe as well as XO4's hole |
| **XO18** | The Hazard map (3.8), for any hazardous body: op `hazard_map`, the Map (a cut drawn with the Mag layer's tracer) and the Profile along a chosen line, belts from `read_field` with edges, gaps, shield and latitude, storms, debris, Park and Dive marks from the party's ship, totals for a pass, rail entry and card line, the system-scale Hazards overlay, shown to players through the eye | MF7 belts and shielding, `read_light`, `read_activity`, MF14a tracer (all built), the ship record; XO1 only for compact bodies. **Jupiter, Saturn, Earth and the Sun need nothing from XO: start there** | light for the op, renderer for the Map, chart room for the rail entry and card | section 11's Hazard map rows |
| **XO16** | Later: relativistic disc profile; thick and warped discs as volumes; cluster arcs at the web scale; the plunge shot; pack-supplied benders | XO7 | as they come | |

XO19 (pulsating stars) and XO20 (clocks) are in section 15.12.

Suggested order: XO1, XO2, XO3, XO4 (the showpiece on screen), then XO5 and XO6, with XO17 and XO18 beside them (both are general and useful at once on ordinary stars and planets; XO18 starts on Jupiter and the Sun), then XO10 (pulsars on MF0's fixtures) while XO7 runs, then the rest as B2 to B4 and MF11 land.

None of XO1 to XO6 changes a golden hash on an existing system: they add an op, blocks and a table. XO8 onward add facts to bodies and will change goldens, one pull request at a time in the golden lane.

---

## 13. Rulings (Alex, 24 Nov 2026 17:03: "Yes to all")

1. **Decided.** Exact bending for a non-spinning hole on every device first (the bend table); spin's effect on the bending as a second pass for capable machines and stills; phones stay on the no-spin table with a badge.
2. **Decided.** A well-fed black hole's disc is blue-white in Visible; the orange ring is the Radio band's mapped colour with its legend.
3. **Decided.** Pulsar beams are cones only in the diagram (Chart, 3D, HUD, Mag); True shows the pulse only when the beam crosses the viewer.
4. **Decided.** `light-path` is one general stage (strong gravity, weak gravity, motion, pack-supplied).
5. **Decided.** The feed is one descriptor on any body.
6. **Decided.** Shape from spin and pull, and brightness following gravity, for every body (XO13, after B1).
7. **Decided.** The time stretch chip on every moving True view; never more than three flashes a second.
8. **Decided.** XO1 to XO4 start on fixtures, ahead of B3 and B4, behind the landing run-through's blockers.
9. **Decided, changed by Alex:** not a list of readings but a diagram, widened by him at 17:19 into **the Hazard map** (3.8) for any hazardous body, Jupiter's belts named; with the closest a ship can go before it is too late as a figure the GM has and may show the players. Alex said he was unsure of the details; 3.8 is the proposal and the mockup shows it. Small open points are below.
10. **Decided.** 4.0 scope is XO1 to XO12, plus XO17 and XO18; XO13 to XO15 follow B2 and B4; XO16 later.
11. **Decided**, with Alex's query answered: "XO" (exotic objects) is only the package code. The op is `read_light_paths`.

**Added by Alex the same day:** Composite at the end of the band switch with a drop-down to combine bands (6A, package XO17). Pulsar glitches (4.3, 16:53).

### Still open (small; builders go ahead on the recommendation)

12. **Hazard map: two ship marks, Park and Dive, as in 3.8?** Recommended: yes. They are the two honest answers to "how close": one for staying, one for a pass.
13. **With no ship chosen, the Hazard map uses a stated stock ship, not a blank?** Recommended: yes, so every star's card has it from the first press.
14. **Composite on a phone stops at two bands?** Recommended: yes, with the reason on the third tick.
15. **Hazard map as its own rail entry at world and star level, plus a Hazards overlay under HUD at system scale?** Recommended: yes. The rail entry is where a GM would look for it; the overlay gives the whole system at a glance.

## 14. Sources

**All written from memory in this session (doi.org and the journals are blocked from the design container). Every one must be checked against its source before a block relies on it, as the standing rule says. The citations thread can take the list.**

Bending and the picture
- Schwarzschild 1916, Sitzungsber. Preuss. Akad. Wiss., 189 (the metric).
- Darwin 1959, The gravity field of a particle, Proc. R. Soc. A 249, 180 (light orbits).
- Bardeen 1973, Timelike and null geodesics in the Kerr metric, in *Black Holes* (Les Houches), 215 (the shadow).
- Cunningham and Bardeen 1973, ApJ 183, 237; Cunningham 1975, ApJ 202, 788 (images and shifts of orbiting matter).
- Luminet 1979, Image of a spherical black hole with thin accretion disk, A&A 75, 228 (the method of section 3.2 and the shift formula).
- Carter 1968, Phys. Rev. 174, 1559 (ray equations for a spinning hole).
- James, von Tunzelmann, Franklin and Thorne 2015, Gravitational lensing by spinning black holes in astrophysics, and in the movie Interstellar, Class. Quantum Grav. 32, 065001.
- Event Horizon Telescope Collaboration 2019, First M87 results I, ApJL 875, L1 (ring 42 ± 3 microarcseconds; shadow size varies about 4% with spin); 2022, First Sagittarius A* results I, ApJL 930, L12.
- Gralla, Holz and Wald 2019, Black hole shadows, photon rings, and lensing rings, Phys. Rev. D 100, 024018.
- Einstein 1936, Science 84, 506 (lens-like action of a star); Dyson, Eddington and Davidson 1920, Phil. Trans. A 220, 291 (1.75 arcseconds).
- Misner, Thorne and Wheeler 1973, *Gravitation* (clock rates, tides).

Discs, feeds and flows
- Shakura and Sunyaev 1973, A&A 24, 337 (in the kernel).
- Novikov and Thorne 1973, in *Black Holes* (Les Houches), 343; Page and Thorne 1974, ApJ 191, 499.
- Bardeen, Press and Teukolsky 1972, ApJ 178, 347 (last stable orbit by spin).
- Thorne 1974, ApJ 191, 507 (spin limit 0.998).
- Bondi and Hoyle 1944, MNRAS 104, 273; Bondi 1952, MNRAS 112, 195.
- Eggleton 1983, ApJ 268, 368 (lobe size); Paczyński 1977, ApJ 216, 822 (disc outer edge).
- Ghosh and Lamb 1979, ApJ 234, 296 (magnetospheric radius).
- Narayan and Yi 1994, ApJ 428, L13; Abramowicz et al. 1988, ApJ 332, 646; Yuan and Narayan 2014, ARA&A 52, 529 (flow states).
- Frank, King and Raine, *Accretion Power in Astrophysics* (3rd ed., 2002).
- Blandford and Znajek 1977; Blandford and Payne 1982 (jets; already cited in the light and MF designs).
- Bardeen and Petterson 1975, ApJ 195, L65 (warped discs).

Neutron stars
- Goldreich and Julian 1969, ApJ 157, 869; Radhakrishnan and Cooke 1969, Astrophys. Lett. 3, 225 (the turning-beam geometry).
- Rankin 1993, ApJ 405, 285 (cone widths); Lorimer and Kramer 2005, *Handbook of Pulsar Astronomy*.
- Pechenick, Ftaclas and Cohen 1983, ApJ 274, 846; Beloborodov 2002, ApJ 566, L85 (light bending at the surface).
- Rees and Gunn 1974, MNRAS 167, 1; Kennel and Coroniti 1984, ApJ 283, 694; Gaensler and Slane 2006, ARA&A 44, 17 (wind nebulae).
- Chen and Ruderman 1993, ApJ 402, 264 (death line).
- Anderson and Itoh 1975, Nature 256, 25 (glitches from the superfluid's vortices letting go); Link, Epstein and Lattimer 1999, PRL 83, 3362; Espinoza et al. 2011, MNRAS 414, 1679 (glitch statistics).
- Heinke and Ho 2010, ApJL 719, L167; Page et al. 2011, PRL 106, 081101; Shternin et al. 2011, MNRAS 412, L108 (Cassiopeia A's fast cooling and the superfluid).
- Kaspi and Beloborodov 2017; Duncan and Thompson 1992; Hurley et al. 2005 (magnetars; in the MF design's list).
- Wolszczan and Frail 1992, Nature 355, 145 (planets round a pulsar).

Hazards
- Roederer 1970, *Dynamics of Geomagnetically Trapped Radiation*; Walt 1994, *Introduction to Geomagnetically Trapped Radiation* (how trapped particles bounce and drift; fall-off with latitude).
- Mauk and Fox 2010, JGR 115, A12220 (belt limits; already the kernel's source for MF7).
- Bolton et al. 2017, Space Sci. Rev. 213, 5 (the Juno mission and its polar orbit under Jupiter's belts).
- Fox et al. 2016, Space Sci. Rev. 204, 7 (Parker Solar Probe; closest pass 9.86 Sun radii, December 2024).

Shapes and winds
- von Zeipel 1924, MNRAS 84, 665; Lucy 1967, Z. Astrophys. 65, 89; Espinosa Lara and Rieutord 2011, A&A 533, A43.
- Domiciano de Souza et al. 2003, A&A 407, L47 (Achernar's shape).
- Crowther 2007, Physical properties of Wolf-Rayet stars, ARA&A 45, 177.
- Nugis and Lamers 2000, A&A 360, 227; Sander and Vink 2020, MNRAS 499, 873 (mass loss).
- Hamann, Gräfener and Liermann 2006, A&A 457, 1015 (winds hiding the surface).
- Stevens, Blondin and Pollock 1992, ApJ 386, 265; Cantó, Raga and Wilkin 1996, ApJ 469, 729 (colliding winds).
- Tuthill, Monnier and Danchi 1999, Nature 398, 487 (WR 104's spiral); Lau et al. 2022, Nature Astronomy 6, 1308 (WR 140's shells).
- Mauron and Huggins 2006, A&A 452, 257; Maercker et al. 2012, Nature 490, 232 (spiral shells round red giants).
- Kruckow et al. 2014 and Kawahara et al. 2018 (a white dwarf brightening its companion by bending its light; KOI-3278, Kruse and Agol 2014, Science 344, 275).

---

## 15. Addendum (Alex, 24 Nov 2026 18:19 and 18:24): stars that breathe, and clocks that disagree

Alex read a list of what another space simulator does not do (no intrinsic variable stars, no time dilation, stars that do not drift, galaxies that do not turn) and asked whether we do these. He said yes to two additions here: **pulsating variable stars** as a small block set, and **time dilation readouts**. Both are cheap, exact enough to check with a calculator, and of the "cool physics win" kind. Neither needs a new screen. Packages XO19 and XO20; questions 16 to 21 at the end, each with a recommendation builders follow until Alex says otherwise.

Why most simulators skip pulsating stars, as far as this thread can tell: they keep each star as one catalogue row with one brightness. Ours already keep a star's state through time, so a star that changes is only one more formula of time.

### 15.1 What already exists, so nothing here redoes it

| Thing | State |
|---|---|
| Stars drifting, constellations changing | designed: every star has a described galactic orbit (galaxy design, G5) and the night sky reads each star where it was when its light left; "constellations distort over a million years" (night sky design 4.4) |
| Galaxies turning | designed and visible: rotation curve and an arm pattern speed (galaxy design) |
| A star's brightness changing with time | partly built: the activity cycle multiplies X-rays and spots (`sse-activity`, `cycle_factor`, `moment`); flares are dated events. Nothing yet changes the star's own steady light |
| What reads a star's brightness | `star.luminosity_at_age` (mass and age only) feeds climate (`sse-place`), light (`sse-light`), nav hazards, sensing and the sky |
| Ship time on a fast voyage | built in the finder: time aboard and time at home on every relativistic leg (voyages design; `sse-nav` `voyage.rs`). Ruling: the campaign clock moves by home time, each crew member ages by time aboard |
| Clocks near a mass | designed: the Clocks row of the Hazard map (3.8) and the block `sse.gravity.clock_rate` in XO1 |
| The census | lists classical and Type II Cepheids, RR Lyrae, delta Scuti, beta Cephei and SPB stars as Missing, with the recipe "instability strip, period-luminosity and period-density relations" |

So the gaps are: no star pulsates; no screen shows a ship's clock against home; and a stellar engine's push does not yet reach the star's path (15.9).

### 15.2 Pulsating stars: one process, read off the star's state

Processes, not classes. A star pulsates when a layer inside it works as a heat valve: squeezed, the layer turns more opaque, holds heat back and pushes the star out; expanded, it clears, lets the heat go and the star falls back. Whether that valve drives the whole star depends only on where the layer sits, which is set by the star's temperature, brightness and metal content. So no star is drawn as "a Cepheid". The kernel asks one question of every star at every age, "is a valve working here?", and the class names are labels read off the answer.

| Valve | Layer | Where it works | Labels read off (never inputs) |
|---|---|---|---|
| **Helium valve** | where helium loses its second electron, near 40,000 K | a sloping band in the temperature and brightness plane, roughly 6,000 to 8,500 K: "the instability strip". Too hot and the layer is too shallow to push; too cool and churning gas carries the heat round it | delta Scuti (on and near the main sequence, 1.5 to 2.5 Suns), RR Lyrae (old stars burning helium), classical Cepheids (4 to 20 Suns crossing as supergiants), Type II Cepheids (old low-mass stars in the same band) |
| **Iron valve** | where iron-group atoms crowd the opacity, near 200,000 K | hot massive main-sequence stars, and only where there is enough metal | beta Cephei (8 to 18 Suns, hours), slowly pulsating B stars (3 to 8 Suns, days) |
| later: **hydrogen valve with churning** | the outer envelope of the coolest giants | Miras and semiregulars (census row "AGB pulsation") | same blocks, a later package |

A small correct touch a clever user can test: because the iron valve needs iron, a metal-poor galaxy has almost no beta Cephei stars. Nothing special makes that happen; the strip block reads the star's metal content.

Not every star inside a strip pulsates enough to see. The share that does is a stated pack number per valve (about half for delta Scuti stars, nearly all for Cepheids and RR Lyrae), drawn by key from the star's seed, so the same star always answers the same way.

### 15.3 The period comes from the star, and Leavitt's law falls out

The period is not looked up from brightness. A star rings like a bell at a period set by its mean density:

> period × √(mean density, in Suns) = Q, with Q about 0.04 days for the basic in-and-out mode

The kernel already has every star's mass and radius, so it has the period. Checks a builder can run: the Sun's own basic ring comes out at about an hour (it is measured at about 63 minutes); Delta Cephei, at 4.5 Suns and 44 Sun radii, comes out at 5.4 days (it is 5.37).

Then the famous period and brightness law (Leavitt 1912), the one that measured the universe, is **a result and a test, not an input**: plot period against brightness for every Cepheid the kernel makes in a galaxy and the points must lie along the observed line within its real scatter. Type II Cepheids land about one and a half magnitudes fainter at the same period, because they are lighter stars, and nothing special is needed for that either. The observed laws stay in the pack as the check, with their papers.

This gives the players something real to do: a science seat that watches a pulsating star in another galaxy for a few periods gets its true brightness from the period, and so its distance. That is how it was really done, and it works because the numbers underneath are consistent.

Overtones (the star ringing in its second or third mode) are a table of period ratios (about 0.71 of the basic period for Cepheids, 0.745 for RR Lyrae). Delta Scuti stars ring in several modes at once; the block allows up to three, summed. The slow B stars ring in a different kind of mode that the density rule does not give, so their periods come from an observed table, said as such.

### 15.4 The light curve is a formula of time, like an orbit

One block, `sse.star.pulse_curve`, gives the star at any instant without stepping:

- **Phase** = the fraction part of (time ÷ period + the star's own offset from its seed). Same answer on every device, at any date asked, in any order.
- **Shape** = a short sum of sine waves whose sizes are a table against period (the method of Simon and Lee 1981). That one table gives the sawtooth of an RR Lyrae, the smooth wave of an overtone star, and the bump that walks along a Cepheid's curve near ten days.
- **Size of the swing** = a table by valve and place in the strip. This is the honest shortcut: the period is physics, but how far the star swings is set by non-linear gas motion that only a full simulation gives, so the table is from observation and goes in the fudge register (15.10).
- **Outputs, all from the one phase**: the star's radius, surface temperature and total light as multipliers on its steady values, and the speed of its surface. From those `sse-light` makes the light in every band, so the colour change comes free: a Cepheid swings by a factor of two or three in Visible, less in total light, and more in Ultraviolet, because it is hotter when bright.
- **A detail worth getting right**: the star is brightest when it is expanding fastest through its middle size, not when it is smallest. The block carries that quarter-cycle lag as data, and the card's little chart shows size and brightness out of step. Someone who knows will look for it.

Over deep time the period drifts as the star evolves across the strip, which is real and measured. Phase is kept continuous only inside one evolution step; across steps it restarts from the key. Nobody can test phase across a million years, and it is logged as a shortcut.

### 15.5 One general mechanism: the star's light now

Thinking wide: pulsation is one of several things that change a star's light with time. So the addition is not a pulsation feature bolted on; it is one place every reader asks.

**`star.light_now`** = the steady light for the star's mass and age (what `star.luminosity_at_age` gives today) × a list of **modulators**, each a formula of time with a name, a size and a period:

| Modulator | State |
|---|---|
| activity cycle | built (`cycle_factor`); joins the list |
| spots turning with the star | designed (light design 4A) |
| pulsation | new, XO19 |
| a companion or planet crossing the face, seen from a stated place | designed (light design, transits and eclipses); depends on the viewer, so it is applied on the "seen from" side only |
| flares and outbursts | built as dated events; they stay events and are added on top |

A new op, proposed name **`read_light_curve {star, from, to, band?, seen_from?}`**, returns the samples and the list of modulators with each one's share, so every chart of a star's brightness in the product is the same curve with the same Why.

Who feels the star breathe, each by reading `star.light_now` instead of the steady value:

- **The views.** In True the star swells, brightens and whitens together, under the time stretch chip ("one day a second" for a Cepheid, "one hour a second" for a delta Scuti). The three-flashes-a-second cap and reduced motion apply as written in 2.4. From a planet's surface the daylight itself rises and falls. In a globular cluster the sky's RR Lyrae stars slowly wink, each on its own half-day beat.
- **Climate.** The yearly balance uses the mean, which is unchanged, so no existing world's climate moves. The swing goes to climate and weather as a size and a period, and each surface answers by its own heat store: bare rock and thin air follow the star almost fully (temperature moves by about a quarter of the swing in light, so a star swinging by half moves an airless 300 K world by tens of degrees every few days); an ocean world barely notices a five-day beat. The habitable band on the system chart gains a faint second edge for brightest and dimmest.
- **Hazard map.** Heat and Radiation: light rows are drawn for the mean with the worst case faint beside it, the same device the belts use for quiet and stormy.
- **Sensing and surveys.** A watched star yields its period, and from it brightness and distance (15.3). A planet hunt round a pulsating star is harder, as it really is, because the star's own beat hides the dip.
- **Sound.** The period and shape go to the sound design as a slow swell.
- **The card.** One line, "Pulsating: helium valve, 5.4 days", with a small light curve; the Why opens the strip, the density rule and the tables. The honesty marks are unchanged: this is computed, not authored.
- **Phones.** Everything. It is a formula, with no cost to speak of.

### 15.6 What it needs from stellar evolution

The strip is crossed by stars in particular stages, so what turns up depends on the proven core's stages:

- **Works with today's stars**: delta Scuti and beta Cephei stars and the slow B stars are main-sequence stars the kernel already makes.
- **Needs B2 (evolved stars)**: RR Lyrae need the helium-burning stage split out of "giant"; classical Cepheids need supergiants that loop back across the strip. Until then they are checked on fixture stars with stated mass, radius and temperature, the way XO1 to XO4 run on fixtures.

No class gates a block. When B2 lands, Cepheids appear in galaxies without a change to XO19.

### 15.7 Clocks: one reading, shown where it matters

The rule that there is one universe clock stands. The campaign clock is home time. A **ship clock** is a reading derived from it, never a second clock to scrub.

One block does all of it: **`sse.gravity.clock_rate`** (already in XO1 for hovering and circular orbits) is widened to "clock rate from pull and speed": how many seconds pass aboard for each second far away, given where the ship is and how fast it moves. Far from any mass it is the fast-ship rule the voyage finder already uses; in a slow orbit near a deep well it is the Hazard map's rule; in between it is both at once. For a spinning hole the rate on a circular orbit has a closed formula too, so it needs none of XO7's ray work.

The **offset** (how far the ship's clock is behind home) is the sum over the ship's flight log. The log is already cut into dated pieces (voyages ruling), and every kind of piece has an exact formula: parked in an orbit (rate constant), coasting (rate constant), a steady burn (the finder's own formula). So the offset is exact at any date without stepping, and identical on every device.

Where it shows:

| Place | What |
|---|---|
| **Hazard map** (3.8) | the Clocks row as designed, plus the rate written at the Park and Dive marks ("parked here, an hour is 1 hour 25 minutes at home") and, in the totals for a pass, "time lost against home on this pass". Hovering closer than any orbit is possible and slows the clock without limit, so the row also gives the thrust needed to hover there, which is the price |
| **The ship's card and the players' ship page** | a **clock chip**: "Ship clock 14 Mar 2301 · home 2 Jun 2309 · running at 0.42". Pressing it opens the Why: which legs cost how much. Each crew member's age reads from time aboard, as already ruled |
| **Voyage plan** | already gives both times for a leg; gains the rate along the leg as a small strip, and the offset the crew will arrive with |
| **Universe clock in the place line** | when Seen from is a ship or a place whose clock differs, the ship's reading appears beside the campaign clock; otherwise nothing changes |
| **Time stretch chip** (2.4) | from a slowed seat the outside really does run fast and blue. The chip says so ("the outside runs 1.4 times fast from here"), and the shifts already in the light-path stage colour it |
| **Messages** | already delayed by light time. Added: a stream from home plays slow on the way out and fast on the way back, by the same shift factor the renderer uses for colour. A small touch that makes the twin story come out right from both ends |

**How big it gets, honestly.** Round a hole that does not spin, the deepest stable orbit slows a clock only to 0.71 (the "1 hour 25 minutes" above). The film-sized numbers, an hour for years, need a hole spinning at nearly the limit and an orbit skimming it, or hovering on a great deal of thrust. The readout will show that, and a GM who wants the film can have it by building that hole. A neutron star's surface runs at about 0.8. For any planet, star or ordinary orbit the effect is millionths or less.

**The cheap win that can be checked.** The same block, with no special case, must reproduce the satellite navigation correction: a clock in a navigation satellite's orbit gains about 38 millionths of a second a day on one at Earth's surface (about 45 gained from weaker pull, about 7 lost to speed). A space station's clock loses about 25. Both are fixtures. The clever person who sets up a navigation satellite and reads the clock chip at Engine depth finds the real number.

**Players and secrets.** The ship clock is the crew's own and they always see it. The home reading is a fact they hold only as well as they could know it (the "Known to" eye, as for everything else); the GM sees both.

**Kernel side.** No new op for the Hazard map (the row is already in `hazard_map`). Ship state gains `clock {rate, offset_s, set_by}` where `set_by` names the speed share and each mass's share, and `plan_voyage` legs gain the rate. Cost: nothing to speak of.

### 15.8 Blocks added

| Block | What it gives | Source | Standing |
|---|---|---|---|
| `sse.star.pulse_strip` | is a valve working, and which, from temperature, brightness and metal content; strip edges as lines in pack data | Cox 1980; for the iron valve Moskalik and Dziembowski 1992, Pamyatnykh 1999 | settled; edges are observed lines |
| `sse.star.pulse_period` | period from mean density; overtone ratios | Ritter 1879; Eddington 1926; Cox 1980 | settled |
| `sse.star.pulse_curve` | radius, temperature and light against phase; the size and shape tables | Simon and Lee 1981; Bailey's diagram for RR Lyrae | shape and size are observed tables |
| `sse.star.pulse_share` | share of stars in a strip that visibly pulsate, per valve | Kepler-mission counts for delta Scuti (Murphy et al. 2019) | observed number |
| `sse.star.period_brightness_check` (data, used by tests) | the observed laws the results must fall on | Leavitt and Pickering 1912; Benedict et al. 2007; Riess et al. 2022 for the modern line | observed |
| `sse.star.light_now` | steady light × the modulator list | this note | a rule of the engine |
| `sse.surface.swing_response` | how far a surface's temperature follows a star's beat, from its heat store and the period | textbook heat flow into a slab | settled |
| `sse.gravity.clock_rate` (widened) | rate from pull and speed together; closed form on a circular orbit round a spinning hole | Misner, Thorne and Wheeler 1973; Bardeen, Press and Teukolsky 1972; Ashby 2003 for the satellite check | settled |
| `sse.ship.clock_offset` | the sum over the flight log's pieces | Rindler 1960 (motion under a steady push) | settled |
| `sse.signal.stream_rate` | how fast a received stream plays | textbook Doppler factor | settled |

All sources from memory, to be checked like section 14's.

### 15.9 The stellar engine and the sky (Alex's point)

Alex: we know the nearby stars' speeds and directions, so constellations change over time, and a stellar engine changes a star's speed, so it should change the sky too.

The first half already follows from the designs (15.1). The second half **does not yet follow and needs one line**. A star's path through the galaxy is described by a formula (a guiding circle with wobbles), not stepped, and the stellar engine is a thrust block on the star (future tech design) whose output nobody reads. The line that joins them:

> A star's described orbit accepts **dated pushes**. A thrust block on a star writes one: from this date, this acceleration in this direction. The star's place at a later date is its described orbit plus the drift the pushes have built up; when a push ends, the orbit is re-described from the star's place and speed at that date.

Then everything downstream is free, because the night sky, voyages and encounters already ask the kernel where each star is at a date: the engine's own sky shifts, and so does every sky that star is in. It is one general mechanism (a passing star's tug or a fictional drive can write a push as well), and it belongs to the galaxy track (kernel thread) with the megastructure work, not to XO. It is noted here so it is not lost, and passed to the coordinator.

The honest size, which the GM should see on the engine's card: a mirror engine on the Sun gives about 10 metres a second after a million years and moves the Sun about a thousand times Earth's distance in that time. Constellations shift far more in a million years from the stars' ordinary motions. Over a billion years it is about 10 kilometres a second and thousands of light-years. So it matters only in deep time, or with a stronger engine from a pack marked fictional.

### 15.10 Shortcuts for the fudge register

| Shortcut | Why | What replaces it |
|---|---|---|
| The size and shape of a pulsating star's swing come from observed tables | only a non-linear gas simulation gives them | a pulsation model as a swappable block |
| One mode per star (up to three for delta Scuti) | enough to look and measure right | a mode list per star |
| No slow beat on top of RR Lyrae curves (the Blazhko effect) | its cause is not settled | a modulator, when wanted |
| Non-radial ringing changes brightness only; no pattern is drawn moving over the star's face | small in light, costly to draw | a surface pattern in the star shader |
| Phase restarts across evolution steps | untestable over such spans | carry phase through |
| Pulsation does not drive extra mass loss | matters for Miras, which are a later package | with the Mira package |
| The home clock is "far-away time" of the home system: the home world's own slowing (about a billionth) and the galaxy's are left out of the campaign clock | invisible in play; the satellite check still works because it compares two clocks in the same well | a per-place clock if a setting ever needs it |
| No extra delay for a message passing close to a mass | tiny | with the weak bender (O15) |

### 15.11 Checks a builder can run

1. The Sun's basic ringing period from `pulse_period`: about an hour.
2. Delta Cephei fixture (4.5 Suns, 44 Sun radii, 5,900 K): inside the helium strip; period 5.4 days within a few percent; brightest a quarter cycle after smallest.
3. An RR Lyrae fixture (0.65 Suns, 5.5 Sun radii): period about 0.6 days; sawtooth shape.
4. A 1.8 Sun main-sequence star from today's kernel: in the strip; period one to three hours. The Sun: not in the strip.
5. A 12 Sun main-sequence star at the Sun's metal content: iron valve working, period of hours. The same star at a fiftieth of the metal: not working.
6. Period against brightness over the Cepheid fixtures lies on the observed line within its scatter; the Type II fixtures lie about 1.5 magnitudes below.
7. `read_light_curve` at one date equals `star.light_now` at that date; asking dates in any order gives the same answers.
8. Climate: the yearly mean for a planet of a pulsating star equals the steady-star answer; an airless fixture's swing is a quarter of the light's swing to within the block's stated error.
9. Clock rate: 0.707 on the deepest stable orbit of a non-spinning hole; about 0.8 on a 1.4 Sun, 12 km neutron star's surface; a navigation satellite against Earth's surface gains 38 millionths of a second a day within one; a station at 400 km loses about 25.
10. Clock offset: a there-and-back fixture voyage gives the same crew age summed by pieces as the finder's leg totals; splitting a piece in two changes nothing.
11. No flashing: a 30 minute delta Scuti at the chip's fastest setting stays under three beats a second.

### 15.12 Packages

| Id | Package | Needs | Proposed owner | Evidence |
|---|---|---|---|---|
| **XO19** | Pulsating stars: `pulse_strip`, `pulse_period`, `pulse_curve`, `pulse_share` with their tables; `star.light_now` with the modulator list (the built activity cycle joins it); op `read_light_curve`; the card line and light curve; True views and the sky read it; climate and weather get the swing through `swing_response`; Hazard map worst case. In two parts if wanted: (a) kernel blocks and op on fixtures and today's main-sequence stars, (b) the readers | none for (a); B2 for Cepheids and RR Lyrae in real galaxies | light for (a); renderer, climate and chart room for (b) | 15.11 rows 1 to 8 and 11 |
| **XO20** | Clocks: `clock_rate` widened (with XO1), `clock_offset` over the flight log, ship state `clock`, the rate on voyage legs; the clock chip on the ship card and players' page; the place line reading; Hazard map marks and pass total; the time stretch chip's "outside runs fast" line; stream rate on messages | XO1 for the block; voyages V8 (built) | travel thread for the kernel side, chart room for the chip, with XO18's builder for the map | 15.11 rows 9 and 10; e2e: a fixture ship parks deep, the chip shows the offset growing, the Why names the leg |

XO19 part (a) adds published facts to stars inside a strip, so it changes goldens for those stars only and goes through the golden lane; the mean light is unchanged, so no climate golden moves. XO20 adds fields to ship state and changes no body.

Order: XO19 (a) can start now beside XO1; it needs nothing from the rest of the track. XO20's kernel side follows XO1; its Hazard map lines ride with XO18.

### 15.13 Questions for Alex (builders follow the recommendation meanwhile)

16. **Period from the star's own density, with the famous period and brightness law as a test it must pass, not a table it reads?** Recommended: yes. It is the "never fake it" way and it makes the distance trick work for players.
17. **The size and shape of the swing from observed tables, logged as a shortcut?** Recommended: yes. The only alternative is a gas simulation per star.
18. **One "star's light now" with a list of modulators that every reader uses (views, climate, hazards, sensing), not a pulsation special case?** Recommended: yes. Spots, eclipses and anything a pack invents then come the same way.
19. **Pulsating stars in 4.0?** Recommended: XO19 in 4.0, since delta Scuti and beta Cephei stars work with today's stars; Cepheids and RR Lyrae appear in galaxies when B2 lands, with no further work here. Miras later.
20. **When does the clock chip show?** Recommended: always on a ship's card at Engine depth (so the satellite check can be found); at Curious and Science depth only once the ship is more than a minute off home or running more than a thousandth slow, so ordinary ships carry no clutter.
21. **Stellar engine pushes changing a star's path and so the sky (15.9): add the "dated push" line to the galaxy track?** Recommended: yes, as a small kernel package with the megastructure work, late; it is general and cheap, and it matters only over deep time.
