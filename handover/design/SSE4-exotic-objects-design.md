# SSE4 exotic objects on screen: black holes, pulsars and stripped stars (track XO)

Written 24 Nov 2026 by the design thread "Design exotic stars and black holes", after Alex's "yes to all" (24 Nov 16:41) to starting it. **Status: proposed. Nothing here is a ruling until Alex answers the questions in section 13.**

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

Sixteen packages, XO1 to XO16 (section 12). The showpiece (XO1 to XO6) needs no new stellar physics: it runs on a fixture first, like the Mag layer did on the Solar System.

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

### 3.8 Being near one: numbers for the table

Danger is drama, never forbidden. For any observer or ship position the same op gives, each with its "Why?":

| Reading | Rule | What it is for |
|---|---|---|
| **Clock rate** | a clock hovering at distance r runs slow by √(1 − 2GM / rc²); one in a circular orbit by √(1 − 3GM / rc²) | the voyages ruling (campaign clock by home time, crews age by ship time) already carries ship time: this is one more term in it. A day close in costs the crew's families more than a day |
| **Tide** | stretch per metre of height = 2GM / r³ | lethal long before the surface for a stellar hole; gentle at the edge of a supermassive one. Shown as "pull across a two-metre body" and across the ship's length |
| **Dose** | the disc's X-ray and gamma light at this distance, through the hull (existing dose and shield blocks) | the real killer near a fed hole |
| **Orbits** | no stable orbit inside the last stable orbit; none at all inside the photon orbit | the orbital stability card says so, like any other verdict |
| **Colour of the outside** | the sky seen from deep in the well is blueshifted and crowded towards overhead | the Window uses the same `light-path` stage from the inside looking out |

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
| **XO6** | The close-up and the controls: fly to a hole, Seen from, Scale behaviour (3.7), Wireframe bent grid, time stretch chip in ViewControls, card readings (3.8) | XO4 | chart room, with the renderer | e2e: press a hole, change band and view, readings show with Why; chip always present on moving True views |
| **XO7** | Spin: reference integrator in the kernel (tests only), spin pass in the shader on GPU-high and for stills; badge elsewhere | XO4 | Build photoreal renderer, with light | shader against reference at a grid of rays; shadow shift and flattening at spin 0.9 match published figures |
| **XO8** | Real feeds: capture from surroundings, lobe overflow and wind feeding from B4's dated facts, outbursts and tidal disruptions as events; donor star, stream and hot spot | XO2, B4 | proven core, with light | a fixture binary turns on and off on the time ruler; disc edges follow the orbit |
| **XO9** | The galaxy's centre as a body: spin and feed from `feed.rs`; nucleus brightness and jets on the galaxy chart in True; fly to it | XO4, G nucleus package | kernel thread, with the renderer | a galaxy's past quasar phase shows on the time ruler; the close-up opens |
| **XO10** | Neutron stars on screen: surface seen through the strong bender, hot caps, pulse shape; `seen_from`; light cylinder; card lines; pulse under the time stretch chip with the flash cap; glitches as dated events with the spin trace on the card | XO1, B3, MF11 (starts on MF0's three fixture pulsars) | light, then renderer | Crab and Vela rows of section 11; seen-from check; flash cap test |
| **XO11** | Pulsar wind nebula and bow shape as nebula kinds; fading with spin-down | XO10, O3 | Build light and nebulae | Crab ring size; nebula gone by the death line |
| **XO12** | Magnetars: bursts and giant flares on the activity event list with light and reach; twisted field in Mag | XO10, MF11, O10 | light, with the renderer's Mag layer | rate falls with field decay; a giant flare's dose at stated distances |
| **XO13** | Shape from spin and pull, gravity darkening, lit faces: blocks and a shaped-body mesh in the renderer (stars and planets alike) | B1 one body | light for blocks, renderer for mesh | Achernar and Saturn flattening; teardrop donor at lobe filling |
| **XO14** | The wind you can see: optical depth and soft edge; Wolf-Rayet line rows; ring nebula tie; class labels | B2, O4 | light, then renderer | a wind's edge moves out smoothly as mass loss rises; an ordinary star is unchanged |
| **XO15** | Colliding winds and spirals: wall, dust spiral and shells, turning on the time ruler; red giant spiral shells by the same block | XO14, B4 | light, then renderer | WR 140 and WR 104 rows |
| **XO16** | Later: relativistic disc profile; thick and warped discs as volumes; cluster arcs at the web scale; the plunge shot; pack-supplied benders | XO7 | as they come | |

Suggested order: XO1, XO2, XO3, XO4 (the showpiece on screen), then XO5 and XO6, then XO10 (pulsars on MF0's fixtures) while XO7 runs, then the rest as B2 to B4 and MF11 land.

None of XO1 to XO6 changes a golden hash on an existing system: they add an op, blocks and a table. XO8 onward add facts to bodies and will change goldens, one pull request at a time in the golden lane.

---

## 13. Questions for Alex

Each has my recommendation. Answer in the usual way ("yes to 1, 2 and 4").

1. **Exact bending for a non-spinning hole on every device first (the bend table), with spin's effect on the bending as a second pass for capable machines and stills, and phones staying on the no-spin table with a badge?** Recommended: yes. It gets the true picture on screen soonest and keeps phones honest about what they show.
2. **A well-fed black hole's disc is blue-white in Visible, as physics says, and the orange ring is the Radio band's mapped colour with its legend?** Recommended: yes. It is the truth, it teaches something, and the famous look is one press away. The alternative is a warmer "Camera" default that would be a fake.
3. **Pulsar beams are drawn as cones only in the diagram (Chart, 3D, HUD, Mag); True shows the pulse only when the beam crosses the viewer?** Recommended: yes. A visible searchlight in vacuum is the best-known wrong picture in astronomy. HUD on gives the cones over the True view for anyone who wants them.
4. **The `light-path` stage as one general mechanism (strong gravity, weak gravity, motion, pack-supplied), so the fast-ship Window and microlensing use it too?** Recommended: yes.
5. **The feed as one descriptor on any body, with the disc's inner edge from surface, last stable orbit or magnetic field, whichever is largest?** Recommended: yes. It covers black holes, X-ray pulsars, nova-making white dwarfs and newborn stars with no list of kinds.
6. **Shape from spin and pull, and brightness following gravity, for every body (squashed fast stars, teardrop donors, Saturn)?** Recommended: yes, as XO13 after one-body (B1). It is small and fixes spheres everywhere.
7. **The time stretch chip on every moving True view, with a hard cap of three flashes a second?** Recommended: yes.
8. **Start XO1 to XO4 now on fixtures, ahead of B3 and B4, the way the Mag layer started on the Solar System?** Recommended: yes, after the landing run-through's blockers, in the light and renderer threads' queues.
9. **Near-hole readings on the card (clock rate, tide, dose) in 4.0, with ship time picking up the clock rate?** Recommended: yes for the readings (they are one-line blocks); ship time picks the term up when voyages next touches its clock.
10. **Scope for 4.0: XO1 to XO12 (holes, feeds, the galaxy's centre, pulsars, magnetars); XO13 to XO15 (shaped stars, Wolf-Rayet, pinwheels) follow B2 and B4 whenever those land; XO16 later?** Recommended: yes.
11. **This note's name for the track, "XO", and for the op, `read_light_paths`?** Recommended: yes; say if you would rather other words.

---

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
