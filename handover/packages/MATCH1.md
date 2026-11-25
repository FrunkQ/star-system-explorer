# MATCH1: test shots against famous pictures

Owner: renderer thread. Asked by Alex on 25 Nov: "make the postcards and compare them against famous pictures and see
if they match... and if false coloured, should we have similar view options to match". This package depends on PC2,
the still of any True view from a pose (postcards design, accepted 25 Nov). Until PC2 lands, the poses below are the
plan, and nothing is drawn yet.

How each test works:
1. A postcard is made at the pose and date of the real picture, in the band that picture used.
2. It is set beside the real picture, which is opened from its link. The real image is never copied into the repo.
3. The postcard's back says what differs, using its "Why it looks like this" lines.

A match is judged by eye, in three parts:
- **Where things are:** the terminator, the limb, the rings' tilt.
- **Colour family:** for example, the blue of the oceans and the brown of the land.
- **What the picture shows that the postcard can't.**

A miss becomes a fudge row, or a package for the owning thread.

Credits travel with each link. Every image below is a NASA, JPL or ESA public release. Before anyone reuses an image,
check each agency's media-use page; for now these are references only.

## The list

| # | Picture | Where to find it | Our pose | Band and options | Today |
|---|---|---|---|---|---|
| 1 | Earthrise, Apollo 8, 24 Dec 1968 (NASA, Bill Anders) | NASA image AS08-14-2383 | Lunar orbit about 110 km over the Moon's far-side limb, Earth on the horizon; System True, Earth frame, Moon close-up (CU1) | Visible, Eye | Pose possible with CU1; the Moon's ground is soft until the detail ladder's deep tiles (VP3 part 4); Earth's clouds come from the climate thread |
| 2 | The Blue Marble, Apollo 17, 7 Dec 1972 (NASA) | NASA image AS17-148-22727 | About 29,000 km from Earth, Sun behind the camera (full phase), Africa and Antarctica in view | Visible, Eye | Pose possible; the land's colours come from the data sheet. Needs the date to set Earth's turn: the system view does not spin worlds yet (fudge row "The close-up's floor and tilt") |
| 3 | The Pale Blue Dot, Voyager 1, 14 Feb 1990 (NASA/JPL) | JPL Photojournal PIA00452 | About 6 billion km out, 32° above the ecliptic, looking back past the Sun | Visible, Long exposure | Earth is under a pixel, so it tests the finish's glare and Real scale's point-of-light floor. The real picture's sunbeam streak is scattered light in the camera's optics: a lens effect, off by grade (design section 3) |
| 4 | "The Day the Earth Smiled", Cassini, 19 Jul 2013 (NASA/JPL-Caltech/SSI) | JPL Photojournal PIA17172 | In Saturn's shadow, the Sun eclipsed, with the rings backlit | Visible plus a long exposure; their mosaic is a composite of colour filters | Needs forward scatter in the rings (bright when backlit: the E ring and the dusty rings) and the planet's backlit limb haze. Our rings use one albedo and optical depth, with no phase function. **Missing: a ring phase function.** |
| 5 | Jupiter from Voyager 1, 1979, approach mosaic (NASA/JPL) | JPL Photojournal, Voyager 1 Jupiter approach | About 30 million km out, the Great Red Spot near the centre | Visible; Voyager's colour was made from orange, green and blue filters, a natural-ish composite | Bands come from the giant bands thread (PR 477); the Red Spot's place and colour come from the kernel's storms. Fair test of colour family |
| 6 | Pillars of Creation, Hubble, 2014 reprocess (NASA, ESA, Hubble Heritage Team) | ESA/Hubble heic1501a | Eagle Nebula from Earth's distance | False colour: the Hubble palette (S II as red, H-alpha as green, O III as blue) | **Missing:** narrowband line filters as bands. Composite (XO17) mixes our bands, but there are no emission-line bands yet. Owned by the light and nebulae thread |
| 7 | Saturn and its rings, Cassini natural colour (NASA/JPL-Caltech/SSI) | JPL Photojournal, Cassini natural-colour Saturn | Over the lit side of the rings, about 1.2 million km | Visible, Eye | Ring shadows on the planet and the planet's shadow on the rings. Check that both draw |
| 8 | Crab Nebula, multiwavelength (NASA, ESA, CXC, JPL-Caltech) | NASA/Chandra composite releases of M1 | From Earth's distance | Composite: radio, infrared, visible, ultraviolet, X-ray | Composite on every view (XO17, PR 443) is the matching option; needs the pulsar wind nebula's own emission per band (exotic objects thread) |

## The reference pictures (25 Nov)

Alex widened the brief: every world captured close up with decent kit, Webb's infrared planets, Venus, Pluto as the tholin
test, and distant objects. There are 54 references in project files under `sse4/match/<id>/`. Each folder has
`ref.jpg`, a `credit.txt` (credit, source and licence) and a `pose.json` (the place, date, camera, view, the band it
tests, and what SSE4 still needs). `sse4/match/README.md` indexes them with their licences.

- **Visible, the Solar System:** Earthrise, Apollo 11, Blue Marble, Pale Blue Dot, Galileo's Earth and Moon, Mercury,
  Venus (Mariner 10), Mars (Viking, the 2003 approach), Phobos, the four Galilean moons, Io, Europa, Ganymede (Juno),
  Callisto, Jupiter (Juno, Red Spot), Saturn (portrait, northern summer, in shadow, The Day the Earth Smiled),
  Enceladus, Rhea at full phase, Uranus, Neptune, Triton, Ceres, Vesta, comet 67P, and Pluto (the tholin test,
  its blue haze, and Charon's red pole).
- **Infrared (Webb, Cassini):** Jupiter's Red Spot and aurorae, Uranus, Neptune, Europa's CO2 ice, Enceladus's plume,
  Titan through its haze (Cassini VIMS and Webb).
- **False colour and Composite:** Titan, Pluto enhanced, Venus by radar (Magellan), SOHO's ultraviolet prominence,
  Neptune's aurorae.
- **Distant:** Pillars of Creation, the Crab (Hubble and Webb), Helix, Butterfly, the Ring Nebula (Webb), Wolf-Rayet 124
  shedding dusty shells, and M87's black-hole jet.
- **Link only:** the Venera 13 surface panorama. Its rights are uncertain, and Don Mitchell's reprocessings carry his
  own terms, so there is no picture in the folder.

The trailer thread turns matched postcards into side-by-side stills, photograph left and SSE4 right. It never retouches
SSE4's side and checks each licence before publishing.

## View options the false-colour pictures ask for

1. **Emission-line bands** (H-alpha, O III, S II, and the Hubble palette as a preset in Composite). The light and nebulae
   thread, as an addition to the Band list. Without them, nebula pictures can't be matched.
2. **A colour stretch** (each channel scaled to its own range, as mission teams publish "enhanced colour"). Belongs in the
   finish as a labelled option beside Long exposure, with the honesty mark of a composite.
3. **Long exposure** exists for stars (VP2). It should also lift faint diffuse light (nebulae, zodiacal light, aurorae) by
   the same stated ratio: the "lift ratio" item in renderer.md's Next list.
4. **Lens effects** (diffraction spikes, the Pale Blue Dot's streak), as a separate pass, off by default (design section 3).
   This is only needed to match space-telescope looks.
5. **A ring phase function** (forward scatter), to match backlit ring pictures.

## When

- PC2 comes first. After it, MATCH1's first three shots (Blue Marble, Earthrise, Jupiter) need nothing else, and land
  as postcards in project files `sse4/handover-screens/MATCH1-screens/` with a side-by-side note per shot.
- The false-colour four (Pale Blue Dot's streak, Cassini backlit, Pillars, Crab) wait on the missing options above,
  each with its owner.
- If PC2 doesn't fit before the 25 Nov 7 pm London close, this note and the status row carry it to the next account.
