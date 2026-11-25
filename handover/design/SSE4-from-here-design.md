# SSE4 "From here": standing inside a world, seeing and hearing it

Written 25 Nov 2026 by the design thread "Design views and sounds from inside worlds" (Fable, design only) at Alex's request. **ACCEPTED by Alex 25 Nov 2026 12:15 UTC** ("otherwise yup to your questions": yes to all 14, with the three steers in section 17A, which wins where the rest differs). Track code **FH** (a code only, never shown in the product). Mockups: project files `sse4/handover-screens/FH-screens/` (13 screens and `from-here-mockup.html`). Status row: `handover/status/from-here.md`.

Read with: `SSE4-surfaces-and-rendering.md` 4.1 (land anywhere, walk about), `SSE4-sound-design.md` section 4 (the atmosphere filter), `SSE4-visual-polish-design.md` 16.7 (the Exposure control), `SSE4-postcards-design.md` (a view that travels), `handover/interface-style.md` rule 12 (one set of view controls).

Alex's brief, 25 Nov 2026, in his words:

- "what it looks like down the levels as deep as practical within gas giants"
- "a version when your eyes get used to it, and a version which shows actual midday brightness rather than a normalized brightness"
- "you can see how far you can see, because there are marker buoys or marker posts out into the distance, and floaters on gas giants"
- "make this look glorious and have a hand holding a color card up, obscuring some of the image, so you can see what colors look like and how they look differently on these planets"
- "to be able to play back standard pieces of audio through a filter to simulate what it sounds like. Or indeed, if it's a vacuum, no sound at all"
- "this WAS done in SSE3 - but I think with the new rendering engine we can do it SO much better", and "make it BETTER!"
- "on planets - when standing there it should be an option on the viewer - to add these objects to the 'view' to get a better feel"
- "we have stuff from venus and remember pluto - that is a good tholin test case"

---

## 1. The short answer

1. **No new screen.** The standing view that already exists (Land, built as SL1 by the renderer thread) gains two things: a place to stand that can be **anywhere in a fluid**, not only on the ground, and a row of four **Aids** the viewer switches on: **Range markers, Colour card, Light meter, Listen**.
2. **One mechanism for every world and every depth.** The kernel answers one question, "what are light and sound like at this point?", for a point on Mars, a point 20 bar down in Jupiter, a point in Titan's haze or (later) a point under a sea. Nothing in it names a kind of world. A gas giant is simply a world where the ground never arrives.
3. **Two honest exposures, added to the control that exists.** The Exposure control from visual polish (Long exposure, Eye) gains a third stop, **As lit**: the screen's white is Earth's noon, nothing is brightened and no colour cast is taken out. **Eye** becomes a real model of a settled eye, including the loss of colour in deep dusk.
4. **Seeing how far you can see** is done with identical objects at known distances, each one a real object with a lamp: posts on ground, buoys on liquid, floaters in gas. They fade for the true reason at the true distance, and the label says which reason: "lost in the air", "below the horizon", "lamp only".
5. **The colour card** is held up in a suit glove in front of the scene. Each patch is a real substance from the data sheet under this world's light; a corner of each patch shows the same patch at home. The card tells you in words which colours can no longer be told apart.
6. **Listen** plays a standard clip from one of the markers, so sight and sound share the same distances. The air's own numbers do everything: speed, loudness, which notes survive the distance. In a vacuum the result is silence, and the panel says why.
7. **As deep as practical** is not a round number. The descent has named levels that physics sets: each cloud deck, the shirt-sleeve level, last light, first glow, and "checked physics ends". Below the last one the control stops and says why.

The mockups show the whole of it. The contact sheet `sheet-going-down.png` is the quickest read: six stations, the same midday, both exposures.

## 2. What SSE3 did, and where this goes further

SSE3 (`star-system-generator`, files `physics/depthView.ts`, `physics/imageUnderLight.ts`, `charts/surfaceScene.ts`, `charts/cloudscape.ts`, `charts/UnderThisLight.svelte`) had the right ideas, and its code comments say the reasons well: a painted landscape with marker posts veiled by airlight, balloons as the markers in a giant, a depth slider to 100 bar, a colour chart re-lit through a wipe, and the two check boxes "once your eyes adjust" and "midday brightness".

What was limited there, and what FH changes:

| SSE3 | FH |
|---|---|
| A flat painted scene (sky, mid and floor colours) | The real standing view: the kernel's terrain or the kernel's cloud decks drawn in 3D, with the markers standing in it |
| Light dimmed by grey cloud decks plus Rayleigh only; 16 broad colours | The column solved by wavelength through every absorber the data sheet has (methane, pressed hydrogen, hazes), so the colour changes with depth for real: gold between Jupiter's decks, blue-green under the water cloud |
| Stopped at 100 bar because nothing below was modelled; glow from a colour ramp | Goes down to where the kernel's own checks end. The glow of hot gas is not a separate model: the same absorption rows that dim the daylight also emit (Kirchhoff's law) |
| Adaptation was a white balance and a brightness scale | An eye model as editable blocks: partial colour adaptation, brightness adaptation, and night vision that loses colour and shifts toward blue |
| A colour chart image beside the scene | A card held in the scene, lit by the same light as everything else, whose patches are sheet substances |
| A "gas giant" special case (`makeupFractions(body).gas > 0.5`) | No special case. Standing in a fluid is the general case and ground is one thing a station can rest on |
| No sound | Listen, tied to the same markers |
| Its own screen | Aids on the standing view, as Alex asked on 25 Nov |

## 3. Words

| Word | Meaning |
|---|---|
| **Station** | Where the eye is: a world, a place on it, a height or depth, a time. On the ground, afloat in gas, later afloat on or under a liquid |
| **Aids** | Things added to the standing view to give a feel for the place. Four to start: Range markers, Colour card, Light meter, Listen |
| **Range markers** | Identical objects at set distances: **posts** on ground, **buoys** on liquid, **floaters** in gas |
| **Sounding line** | The ladder at the right of the view: the column of air above and below with its named levels and a handle for the station |
| **As lit** | The exposure where the screen's white is Earth's noon |
| **Eye** | The exposure of a settled human eye |
| **Shirt-sleeve level, Last light, First glow, Checked physics ends** | Named levels read off the column (section 6) |

In the product the feature has no name of its own. The rail entry stays **Land** on a world with ground. On a world with no ground the same entry reads **Go down**, and its leave button reads "Climb out" instead of "Take off". That is a label chosen from a kernel result (no ground in the column), not from a class.

## 4. One mechanism: light and sound at a station

### 4.1 The kernel answer

One new op, working name `read_from_here` (kernel thread to name it), takes a station and a time and returns, with provenance:

| Group | Contents | From |
|---|---|---|
| The medium here | pressure, temperature, density, gas mix, what particles are present (cloud, haze, dust, rain) and their sizes | climate and weather: the same column the `weather` op's `column` already walks, continued down past the reference level |
| Light arriving | downward light by wavelength (direct beam and scattered separately), upward light by wavelength, both as spectra on the light pack's slices | light: star spectrum through everything above |
| The medium's optics here | scattering and absorption per metre by wavelength, split into gas, pair absorption and each particle kind, with a phase table per particle kind | light: `read_air_optics` and `light_through_gas`, the particle blocks from TA2 |
| Its own glow | emission per metre by wavelength, from the same absorption rows and the local temperature | light |
| Sound | speed, density, how much each frequency loses per metre, how speed changes with frequency | new small blocks (section 9) |
| Named levels | the list for the sounding line, each with the result that put it there | read off the column |
| Limits | where the column's checks end and why | each block's stated range |

It is asked for the visible slices by default and for any band on the Band control, so Infrared at a station is the same call. The answer is plain data and deterministic. Nothing reads the picture back.

### 4.2 The chain of blocks (all small, cited, editable; no Big Block)

1. **Column**: the climate's temperature and pressure profile, continued downward (dry or wet as the climate says), with each deck's top, base and particle load. Already exists above the reference level; the ask is to continue it and to give decks a real thickness (SSE3's "0.55 of base pressure" guess is not carried over).
2. **Optical depth per layer per wavelength**: Rayleigh from `sse.air.*`, lines, bands and pair absorption from `sse.gas.*`, particles from `sse.light.particle_albedo` and the phase table.
3. **Light down the column**: a two-stream solution per wavelength (Toon et al. 1989 form), direct beam kept separate. This is the one new piece of light physics. It is a flow of the same small block applied layer by layer.
4. **Glow**: emission = absorption × Planck at the layer's temperature, sent through the same two-stream. One row in the fudge register is avoided by this: there is no "glow starts at 800 K" switch. Clear hydrogen that absorbs nothing at a wavelength emits nothing at it.
5. **How far a thing shows** (section 7): contrast falling with distance, per wavelength.
6. **The eye** (section 5): blocks under `sse.eye.*`.
7. **Sound in this gas** (section 9): blocks under `sse.sound.*`.

Every block gets its Workshop view. The natural one for blocks 2 to 4 is the sounding line itself drawn as a chart: light against depth, one line per colour.

### 4.3 Why this is general

- A rocky world with air: the column ends at ground. Markers are posts.
- A giant: the column ends where the checks end. Markers are floaters.
- A world with no air: the column is empty; light arrives unchanged, sound has nothing to travel in.
- A sea (later, section 13): the column continues into the liquid with the sheet's absorption for that liquid. Markers are buoys on top and, below, floats on a line. The helium sea test world from the seas ruling works by the same call.
- A hot Jupiter, a brown dwarf, a star's outer layers: the same call; the glow term dominates. Whether anyone could be there is a hazard, shown, never a refusal.

## 5. Exposure: Eye, As lit, Long exposure

One control, three stops, in the shared view controls (visual polish 16.7 put Exposure there; Long exposure stays the default in every True view by Alex's ruling of 24 Nov 19:12).

| Stop | What the screen shows | Use |
|---|---|---|
| **Long exposure** (default) | Faint light lifted, as a camera left open would | the glorious picture, the postcard |
| **Eye** | What a person would see after a few minutes there | "what is it like to stand here" |
| **As lit** | The light as it is. The screen's white is Earth's noon; nothing brightened, no cast removed | "how dim, how orange, really" |

Alex's two SSE3 check boxes become the two new-meaning stops: "once your eyes adjust" is Eye, "midday brightness" is As lit.

**As lit** is a fixed exposure and no white balance. Titan at midday is almost black (about a thousandth of Earth's noon), Venus is a deep dull orange, Luna is a little brighter than home. On a screen that can show high dynamic range, worlds brighter than Earth's noon (Mercury, Venus's cloud tops) go above white instead of clipping. The light meter says the anchor in words.

**Eye** is three blocks, each with a stated source and a setting:

1. **Brightness adaptation**: the eye settles to the light about it, but not completely. A dim place still looks dim. A curve from light level to how bright mid-grey looks.
2. **Colour adaptation**: the eye removes most of an overall cast, less of a strong one (a CAT16-style transform with a degree of adaptation that falls as the light gets dimmer or the cast stronger). This is why Venus still reads orange to a settled eye but Mars's ground does not read as red as a raw camera shows it.
3. **Night vision**: below about dusk level the cones hand over to the rods. Colour drains, the picture shifts toward blue-green, fine detail goes, and deep red goes dark first (Purkinje). At Jupiter's 200 bar level at midday the card is grey until the lamp goes on.

Settling takes time. When the station or the light changes sharply (stepping out, a lamp going off, dropping through a deck) the Eye view eases to its new state over a few seconds, marked "eyes settling" in the HUD. That is presentation (interface rule 10) and it skips under reduced motion. The real times (tens of seconds for cones, half an hour for rods) are given in the "Why?".

The existing fudge row "The eye's adaptation on the ground (SL-night)" is replaced by these blocks. New fudge rows are in section 15.

**A human eye is one row of data.** The three blocks take an observer: sensitivity by wavelength for day and night vision. The base observer is the standard human one. Another species' eye, or a camera, is another row. Nothing more is built for that now; the seam is left open for 4.1's richer aliens.

## 6. Going down: the sounding line

A vertical ladder at the right of the standing view, opened and closed with the view (mockups, right side).

- It draws the column to scale in pressure (log) for deep air, or height for a thin one, whichever the column makes readable.
- Hatched bands are cloud and haze decks, at their true tops and bases.
- A handle marks the station. Drag it, or use ▲ ▼, and the view, the aids and the sound all follow live.
- **You see yourself go down; the view never jumps** (Alex, 25 Nov: "we will see ourselves get lower rather than jump"). Dragging the handle moves the eye through every level between, at the speed of the drag. Pressing a named level, or ▲ ▼, flies the eye there as a sinking camera: the deck comes up, the view whites out inside it, the base opens below, the light and colour change all the way, the markers and the sound follow. The speed is a readable one, not the true one (a real descent takes hours), it is marked as sped up, it can be skipped, and under reduced motion it is a plain fade. When a ship of the campaign is the thing descending, the same camera follows the kernel's planned descent. On a world with ground the handle rests at the bottom by default ("You · ground") and can be lifted into the air: the same control gives the view from a balloon over Titan or from above Venus's clouds.
- **Named levels** are read off the column and carry a "Why?":

| Level | Read from | Jupiter (sample), estimate |
|---|---|---|
| Top of the air | where the column begins | |
| Each deck, by what it is made of | condensation in the column | ammonia near 0.7 bar, ammonium hydrosulphide near 2 bar, water near 6 bar |
| **Shirt-sleeve level** | temperature passes about 20 °C | near 5 to 7 bar |
| **Last light** | daylight at midday falls below what a settled eye can tell from dark | near 200 bar |
| **First glow** | the gas's own light rises above that same threshold | near 400 bar |
| **Crush depth of this hull** | only when a ship or probe of the campaign is the thing descending: its rated pressure | Galileo's probe went silent near 22 bar |
| **Checked physics ends** | the shallowest level where any block in the chain leaves its stated range | to be found by the kernel and climate threads; the mockup guesses about 1,600 bar |

- Below "Checked physics ends" the handle does not go. The line there is dashed and says what would be needed (a real equation of state for dense hydrogen, measured pair absorption at that density).
- **Hazards are shown beside the ladder, never as a stop** (danger is drama): heat, pressure, wind shear, lightning, from the weather column's hazards. "464 °C" at Venus's ground is a fact on the ladder, not a refusal.
- The time ruler still drives time. Dropping to night shows the lamps and the stars above the decks, and nothing below them.

What "as deep as practical" comes to: for a cold giant the interesting descent is from the top of the haze, through three decks, past last light, into the dark, and on to first glow. Those are five very different pictures (mockups 06 to 10), and the last two were not reachable in SSE3.

## 7. Range markers: seeing how far you can see

### 7.1 The objects

One standard marker, three mounts, all in the render catalogue as constructs (so the construct style mechanism covers them and a pack can restyle them):

- **Post** (ground): a 4 m mast banded in one-metre lengths of white and black, a one-metre board on top quartered white, red, black and yellow, a lamp above it.
- **Buoy** (liquid): the same board and lamp on a float, riding the kernel's waves.
- **Floater** (gas): a balloon carrying the same board and lamp. Its envelope is sized by a block from the gas it floats in: in hydrogen nothing is lighter than the air, so on a giant it is a hot-gas balloon and the block says how big and how hot for a standard 100 kg load. That is a small "cool physics win": the floater on Jupiter is visibly a different machine from the one on Titan, and the Workshop shows why.

Board colours and the lamp are sheet substances and a stated lamp (white, with a stated output), so they are lit and dimmed by the same light as the card.

### 7.2 Where they stand

A straight surveyed line from beside the eye toward the view's centre, at 10 m, 30 m, 100 m, 300 m, 1 km, 3 km, 10 km, and for floaters on to 30, 100 and 300 km. On ground the far ones stand on the kernel's terrain. On a world with air, floaters can be added above the posts so that range beyond the horizon can be judged too.

The **Scale** control already on the view does the right thing: **Real** draws them true size (a post at 3 km is a speck with a lamp), **Readable** enlarges the far ones and says so.

### 7.3 Why each one fades

For each marker the kernel block gives the contrast left at that distance by wavelength. A marker is counted lost when its board cannot be told from the air behind it (the 2% contrast rule of Koschmieder, a block input). The HUD label says the reason:

- **lost in the air**: scattering and absorption between you and it. On Venus this happens near 4 km in perfectly clear gas, because 92 bar of carbon dioxide scatters that much.
- **sinking / below the horizon**: the ground curves away. On Luna every post is razor sharp and the 3 km one is half hidden.
- **lamp only**: the shape is lost but the lamp still shows. Lamps carry further than shapes in the dark, and less far than shapes in bright haze.
- **lost in the glow**: deep down, the lamp cannot be told from the glowing gas.

Colour matters and is shown for free: in Rayleigh air the board's blue and white go first, in Titan's haze red lasts longest.

At Curious the HUD line is a sentence: "The air alone hides a post at about 4 km." At Science it is the figures per colour. At Engine it opens the block.

## 8. The colour card

- **What it is**: a card of 24 patches held in a suit glove at the lower left of the view, covering part of the scene. It is an object in the scene: lit by the same sun, sky and lamp, shaded when you turn it. Drag it to move it; press the glove to put it away.
- **The patches are substances on the data sheet**, each with a measured reflectance by wavelength, read only through Look up (no typed copies): a white, four greys and a black; six strong pigments; then familiar things: rust, leaf, sky, two skin tones, blood, gold, copper, water ice. A second face of the card, "Wires", carries the cable colours (red, brown, orange, yellow, green, blue, violet, grey, white, black, pink) for "cut the red wire". The exact list is the data sheet thread's to settle (ask DS-FH1).
- **At home in the corner**: with HUD on, the lower corner of each patch shows that patch under Earth's noon. One glance gives the shift. HUD off, the card is just a card.
- **It says what it found**: for every pair of patches the kernel block gives how far apart they look to a settled eye here. Pairs that have fallen together are named in a line above the card ("Blue and violet read as black. Green and yellow are hard to tell apart.") and joined by a small tie on the card. This is the GM's answer to "can the players tell the wires apart", and it is a fact a clever player can test.
- **The lamp**: the Lamp button turns on your own light (a stated white lamp). Within its reach the card reads true again, and the fog in front of you lights up. At last light this is the only way to see colour at all.
- **It respects the exposure**: in As lit the card is as dark and as tinted as everything else; in Eye it is what you would judge it to be.
- **Real landers carry one.** Viking, the Venera landers, Curiosity and Perseverance all took colour targets for exactly this reason. That gives the test (section 14) and a line for the "Why?".
- **Why a glove**: almost nowhere this is used can a hand be bare. The glove is a plain pale suit glove in the base look, and its look comes from the construct style in use (a pack's suits change it).

Name on the card in the mockup: "SSE reference card". It must not copy a commercial chart's layout or name.

## 9. Listen: what it sounds like here

The sound design note (18 Nov) section 4 already sets the physics and the stage N4. FH adds the way it is used and four refinements.

**How it is used**

- **Listen** is the fourth aid. It opens a panel beside the pointer with: the clips; **Coming from** (beside you, or any range marker); **Heard** (suit microphone, bare ear, inside the hull); Play at home and Play here; a chart of what survives.
- **Sound comes from the markers.** Pick the 100 m post and the clip arrives late by 100 m over the speed of sound, quieter, and with the high notes gone as that gas takes them. Tapping a marker in the view picks it. Sight and sound now answer the same question with the same objects: on Mars you see the 1 km post clearly and cannot hear a shout from the 100 m one.
- **Never sound alone** (sound design principle 5): the chart shows the clip's spectrum at home and here, the words say the speed, the delay and how far a shout carries, and captions are available. All of that works with sound switched off, which is the default.
- It follows the sounding line live. Going down through Jupiter the same spoken line gets faster, brighter and louder.

**The clips** (shipped, a few seconds each): a spoken line, a sung phrase, a flute, a guitar, a hand clap, a dropped spanner. Or your own file or microphone, processed on the device and never sent anywhere.

**Where the shipped clips come from (Alex, 25 Nov 2026 12:10 UTC: agreed).** No film, television or game clips, however short and however credited: a clip carries the script, the recording and the performance, and "a few seconds with a credit" is not a rule that exists. Three sources only:

1. **Recorded fresh for the project**: the plain spoken line, the sung phrase, the instruments and the knocks. These are the clean test clips, recorded close and dry so the filter has the whole range to work on.
2. **Public-domain science fiction text, read fresh.** Alex's pick is the opening of H. G. Wells's *The War of the Worlds* ("No one would have believed in the last years of the nineteenth century..."). The novel's text is out of copyright in the UK. The 1938 radio play's script and recording, and the later musical version, are not ours to use, so the line is a new reading of Wells's own words, not a clip of either. Check each text's status before recording.
3. **Real spaceflight audio**, credited to NASA with no suggestion of endorsement. Alex's caution stands: most of the famous lines came down a narrow radio link and are already thin and distorted, so the filter has little left to take away. So prefer recordings made in the cabin (onboard tapes, station and shuttle cabin audio) where they exist, and label every such clip with how it was recorded ("over the radio, 1969"). The chart shows the clip's own range, so the user sees why a radio clip changes less than a fresh one. The real Mars microphone recordings are offered as themselves ("recorded there"), played without the filter, as the check on it.

Anyone who wants a film line at their own table can load it through "Your own…"; it stays on their device.

**"Your own…" lists the campaign's sounds** (Alex, 25 Nov: "Any audio uploaded to the presentation/routing layer should be available under Your own"). Every sound already loaded for the Playbook and Switchboard (a pack's sounds, the GM's own files; Playbook note 4.7) appears there by name, beside "Choose a file" and "Microphone". One store of sounds, not two. A sound played "as heard here" to the table goes out through the Switchboard under its usual rules (a pack asset by name; a GM's own short file once).

**Refinements to the 18 Nov physics**

1. **No "pressure floor" switch for vacuum.** As gas thins, the distance a molecule travels between collisions grows until it is as long as the sound wave; the loss per metre climbs steeply and takes the high notes first, then everything. The blocks do this by themselves, so thin air fades out for the right reason and Luna is silent because the sums say so. The panel then says "Silence. There is no air to carry sound" and offers "Through the ground" (contact sound through boots and hull, stage N5).
2. **How far a shout carries** is computed the same way as how far a post shows: a stated source level, spreading, the gas's loss, a stated hearing threshold. One number in words per world, at Curious.
3. **A bare voice is its own choice.** With "suit microphone" the source is a normal voice inside a suit and only the path changes. With "bare ear" the speaker is taken to be breathing the gas: the mouth's resonances move by the ratio of sound speeds and the voice goes squeaky in hydrogen, deep in carbon dioxide. That is a hypothetical almost everywhere and the panel says so.
4. **Bending** by temperature and wind layers (sound trapped near the ground or thrown upward) is listed in the sound note; here it only moves "how far a shout carries" and can follow later.

**Label**: "real sound", per the sound note's three labels.

**Blocks** (`sse.sound.*`, small, cited): speed from the gas mix and temperature; density and stiffness against Earth's; loss per metre by frequency (classical, rotational, and each gas's vibrational relaxation, with the relaxation rows on the data sheet, ask DS-FH2); spread of speed with frequency; reach of a stated source.

## 10. On screen

Everything below is on the existing standing view and uses the shared controls (mockups 01 to 11, phone-1).

- **View controls row**: View style, Scale, **Exposure** (Long exposure, Eye, As lit), then the folded chip. **Aids fold under the HUD chip** (Alex, 25 Nov: "the aids line at the top works fine - fold under hud"): the chip opens HUD, Band and the four aid toggles, and its words say what is on ("HUD on · Visible · Card, Markers"). **Every aid is off until switched on**: the uninterrupted view is the default ("uninterrupted view is better"). The choice is a HUD option of the standing view at planet level and is **remembered** on the viewer's device, so a GM who uses the card sometimes finds it as they left it. A view that cannot do an aid leaves it in place, quiet, with the reason (interface rule 12). The mockups show the aids as their own row beside Exposure; the fold is the ruling.
- **Light meter** (top left): one line in words ("About 1/20 of Earth's noon"), a scale with six things everyone knows (starlight, full Moon, dusk, office, overcast, Earth noon) and a mark for here, one sentence on what the light is like, and one on what the chosen exposure is doing. Science adds lux. Engine opens the spectrum.
- **Sounding line** (right): section 6.
- **Card and markers**: in the scene. Their words are HUD lines, so HUD off gives a clean photograph with the objects still in it.
- **Bar at the bottom** (already there on Land): turn, up and down, Time of day, then **Lamp**, Take photograph, **Make a postcard**, Take off or Climb out.
- **Depth levels**: Curious is sentences. Science adds figures with units through the one figures door. Engine opens the flow at the block behind whatever was pressed.
- **Honesty marks** stay GM only. A station below "Checked physics ends" cannot be reached, so no mark is needed for it; an authored atmosphere carries its usual mark on the meter.
- **Guide, don't leave them alone**: the first time the view opens on a world, a single line suggests the most telling aid for that world ("Try the colour card here: this star gives almost no blue").
- **Motion**: aids arrive from their button (the card comes up from the bottom edge, markers fade in along the line), 120 to 240 ms, none under reduced motion.

### Players and the Run desk

- A player's standing view has the same Aids. What they show comes through the player's data view, so a player hears and sees nothing they are not allowed to know. Bands other than Visible still come through their sensors.
- The GM can put a station on any screen through the Switchboard like any other view, and a Playbook preset can carry "Listen here" with a clip: the table hears the line as it would sound on that world. It is a recipe id and settings, a few hundred bytes.
- Players never pilot. Moving the station up and down in gas is the GM's, or follows the ship's planned descent.

### Postcards

The postcard pose (PC1) gains three optional fields: the station's height or depth, the exposure, and which aids were on. A postcard of "Jupiter, between the decks, card in hand" is then drawn live like any other, and its back carries the reasons the view already has: how bright, how far, which colours fell together, the speed of sound. Seam asked of the postcards thread (section 12).

## 11. Making it glorious: the picture itself

The aids are only as good as the scene they stand in. What the standing view needs, by owner:

- **Inside clouds and between decks**: the kernel's decks as volumes seen from inside, with tops that billow, bases that hang, gaps that let shafts of light through, and towers that rise past the eye. The decks' places, thickness and particle loads are the kernel's; the billows inside a deck are detail from the detail ladder (grade 2: drawn from the kernel's numbers, nothing in play reads it).
- **The medium everywhere**: one fog term per pixel from the station's optics, by colour, with in-scattered light from the two-stream result. The same term veils the terrain, the markers, the card at arm's length (it matters at 1,000 bar) and the lamp's beam.
- **The sky from inside thick air**: when the air is thick the single-scattering sky the walk view uses is not enough. The two-stream result gives the brightness by height; Hillaire's look-ups (already planned as VP-A3) give the look.
- **Glow**: emission in the fog term. In the red dark the light comes from everywhere and is brightest below.
- **The sun as it is**: the right width for the distance (a fifth of ours from Jupiter), dimmed and reddened by the air above, gone under a deck.
- **Tiers**: phones get the fog term, flat decks and the aids (the phone mockup). Desktop tiers add volumes, shafts and the lamp beam. The still tier (Take photograph, postcards) spends longest. Each is placed by the measured probe, with a reason line when something is left out.
- **Rule 5 of visual polish** applies: nothing here is "fixed for Jupiter". Each effect is switched on by the column's numbers and is proven on a Sol body, a non-Sol world and a world where it stays off.

The mockup pictures are painted by a small stand-in shader with estimated figures. They show intent and layout; they are not SSE4 output and the numbers on them are the design thread's estimates.

## 12. Packages and owners

Proposed. Names are today's threads on the old account; after the handover the same rows go to whichever thread owns that status file.

| Package | What | Owner (status file) | Needs |
|---|---|---|---|
| FH1 | The column continued down: profile below the reference level, real deck tops and bases, particle loads, the range each block is checked over, hazards at any level | Build shared climate and weather (`weather.md`) | on main now |
| FH2 | Light down the column: two-stream per wavelength with a separate direct beam; glow by Kirchhoff; the optics here split by cause. Blocks, citations, golden | Build light and nebulae (`light.md`) | FH1; TA2's particle blocks; the methane row fix |
| FH3 | The op `read_from_here` (station in, section 4.1 out), named levels, limits. Golden-changing: takes its turn in the golden lane | Build kernel resolve and rules (`kernel.md`) | FH1, FH2 |
| FH4 | How far a thing shows: contrast by distance and wavelength, the lost-because reasons, lamp reach | Build light and nebulae (`light.md`) | FH2 |
| FH5 | The eye: `sse.eye.*` blocks (brightness, colour, night vision), the observer row, patch-pair distances | Build light and nebulae (`light.md`) | none |
| FH6 | Sheet rows: the card's substances with reflectance by wavelength (DS-FH1), gas relaxation rows for sound (DS-FH2), the standard lamp (DS-FH3) | Build the master matter data sheet (`data-sheet.md`) | none |
| FH7 | Sound in a gas: `sse.sound.*` blocks, reach of a shout; folded into `read_from_here` | Build light and nebulae, or the kernel thread (question 9) | FH6 |
| FH8 | The standing view takes a station at any height: camera afloat and the flown descent between levels, the fog term by colour, sun as it is, Exposure gains As lit, Eye from FH5 with settling | Build photoreal renderer (`renderer.md`) | FH3, FH5; `finish.ts` (VP1) |
| FH9 | Decks from inside: volumes, shafts, towers, the thick-air sky, glow | Build giant bands, clouds and hazes (`true-air.md`), with the detail ladder thread | FH8; TA5, VP-A3 |
| FH10 | Range markers: the post, buoy and floater as catalogue constructs, the floater sizing block, placing them, fading them from FH4, the labels | Build photoreal renderer (`renderer.md`); the sizing block with the kernel thread | FH4, FH8 |
| FH11 | The colour card and glove: the object, lighting, the home corners, ties and the line of words, the Lamp | Build photoreal renderer (`renderer.md`) | FH5, FH6, FH8 |
| FH12 | The controls: Aids as toggles under the HUD chip in the shared view controls contract, off by default and remembered, the light meter, the sounding line, Go down on the rail for worlds with no ground, depth levels, first-visit hint | Make the SSE4 test drive work end to end (`chart-room.md`), with Build landing run-through | FH3; can start on fixtures |
| FH13 | Listen: the panel, clips, "Your own…" listing the Playbook's sounds, Coming from a marker, the three listeners, the chart, captions; this is sound stage N4 delivered through FH | Make the SSE4 test drive work end to end (`chart-room.md`, owner of N1 and `packages/sound`) | FH7 |
| FH14 | Players and the table: aids on the players' page through the data view, a Playbook preset that plays a clip "as heard here" | Build Run desk playbook and players' side (`run-desk.md`) | FH12, FH13 |
| FH15 | Postcards: pose fields for station, exposure and aids; the back's reason lines | Build postcards and hold backdrops (`postcards.md`) | PC1, FH8 |
| FH16 | Orac knows the aids and the blocks; "show me what red looks like on this world" works | Build Orac, Minerva and Tars (`agents.md`) | FH12 |
| FH17 | Checks against real pictures and recordings (section 14) and the trailer clips | each builder for its part; the pictures with the renderer thread's `match/` set | as they land |

A sensible first slice that shows the idea end to end on a fixture: FH5, FH6 (card rows only), FH8 with the existing single-scattering sky, FH11 and FH12. That gives the card, the meter and both exposures on Mars, Luna and Titan before any deep column exists. The second slice is FH1 to FH4 and FH10 (markers, and going down). The third is FH9 (glorious) and FH7 with FH13 (sound).

## 13. Not now

- **Under a sea**: the same op and the same aids with buoys and floats. Waits for the seas work (W-sea, MP34's water scattering rows). Designed in, not built.
- **Other eyes** (alien species, cameras, a ship's sensors as a "card"): the observer row leaves the seam. 4.1.
- **Inside caves and built sites**: interior sites are parked to about 4.2.
- **Sound bending by layers, echoes off terrain, thunder and wind as heard here**: sound stage N5.
- **A fluorescent patch** that lights up under a star rich in ultraviolet: needs fluorescence rows on the sheet. A good later flourish.

## 14. Checks: how a builder proves it

| Check | Against |
|---|---|
| Venus's ground: colour of sky and rock, how far the ground shows, light level | the Venera 13 and 14 colour panoramas (their own colour target is in frame), the landers' light readings |
| Titan's ground: colour, light level, range | Huygens' surface picture and its light measurements on the way down |
| A tholin world lit from far away | Pluto (New Horizons), as Alex asked: the card's "tholin" patch and the ground should agree with the real colour picture |
| Mars: butterscotch sky, blue glow round the Sun, the colour target | Perseverance and Curiosity colour-target frames |
| Jupiter's column to 22 bar: temperature, pressure, light falling with depth | the Galileo probe's readings |
| Earth: clean-air range, card unchanged, sound unchanged | standard visibility and ISO 9613-1 |
| Mars sound: speed near 240 m/s, high notes lost within tens of metres | Perseverance's microphone results |
| Luna: silence, sharp posts to the horizon | arithmetic |
| A world where each effect stays off | rule 5 |

Reference pictures with credits live with the renderer thread's set in project files `sse4/match/`.

## 15. Fudges this adds (each goes in `docs/fudges.md` in the PR that builds it)

| Fudge | Why it exists | What would replace it |
|---|---|---|
| A screen cannot show real brightness; As lit pins the screen's white to Earth's noon | no display reaches daylight | nothing; it is stated on the meter |
| Two streams instead of the full angular light field | fast enough to follow a dragged handle | many-stream or Monte Carlo for stills |
| The column is treated as flat layers under the station | good while the sun is well up | spherical shells near sunset |
| Eye settling is shown in seconds | nobody waits half an hour | nothing; said in the "Why?" |
| One standard human observer | data for others does not exist yet | observer rows |
| Billows inside a deck are drawn detail | the kernel carries decks, not every cloud | finer weather cells |
| A bare voice is modelled by moving the mouth's resonances only | a full voice model is out of scope | a source model with gas loading |
| Markers in Readable scale are enlarged | so far ones can be seen | Real scale is one press away |

## 16. Sources to verify before use

The Consensus connector was not available in this design session, so **every entry below is from memory and unverified**. The citations thread's check applies before any of them goes into a pack.

- Koschmieder (1924), the contrast law for visual range; Middleton, *Vision through the Atmosphere* (1952).
- Toon, McKay, Ackerman and Santhanam (1989), two-stream solutions for scattering atmospheres, *J. Geophys. Res.* 94.
- Hillaire (2020), already in the renderer's references.
- Karkoschka (1994, 1998), methane absorption in the visible; Borysow and colleagues, hydrogen pair absorption.
- Seiff et al. (1998), Galileo probe atmosphere structure; Sromovsky et al. (1998), Galileo net flux radiometer.
- Tomasko et al. (2005), Huygens imager results at Titan. Pieters et al. (1986), colour of Venus's surface from Venera 13 and 14.
- Li et al. (2017), CAT16 and CAM16; Stockman and Sharpe cone sensitivities; the CIE day and night sensitivity curves.
- Ohta (1997) or later measured sets for reference chart reflectances, for comparison only; the card's own rows come from the sheet.
- Bass, Sutherland, Zuckerwar et al. (1995), atmospheric absorption of sound; ISO 9613-1; Petculescu and Lueptow (2007); Bass and Chambers (2001); Maurice et al. (2022); Leighton and Petculescu (2009). The last four are already listed in the sound note.
- Draper (1847), the temperature at which hot bodies begin to show light.

## 17. Questions, all answered yes (25 Nov 2026 12:15 UTC; see 17A)

1. **Aids on the standing view, no new screen**: Range markers, Colour card, Light meter, Listen as four switches beside Exposure. Recommend yes (it is what you asked for on 25 Nov).
2. **Exposure gains a third stop, "As lit"**, beside Long exposure (still the default) and Eye. Recommend yes.
3. **The word "As lit"**. Alternatives: "True level", "Camera at Earth settings". Recommend "As lit".
4. **On a world with no ground the rail entry reads "Go down"** and leaving reads "Climb out". Recommend yes.
5. **How deep**: the handle stops at "Checked physics ends", wherever the blocks' own ranges put it, not at a fixed pressure. Recommend yes, and that the climate and light threads push that level as deep as cited data allows (the mockup guesses about 1,600 bar on Jupiter).
6. **The named levels** Shirt-sleeve level, Last light, First glow. Recommend yes to all three names.
7. **A suit glove always holds the card**, styled by the construct style in use; never a bare hand. Recommend yes.
8. **The card's patches are real substances from the data sheet**, with a second "Wires" face. Recommend yes, with the data sheet thread settling the list.
9. **Sound blocks live with the light thread** (it already owns gas rows and absorption by wavelength; sound is the same shape by frequency). Recommend yes; the alternative is the kernel thread.
10. **Sound comes from the range markers**, so sight and sound share distances. Recommend yes.
11. **The lifted handle**: on a world with ground the station can also be raised into the air (the view from a balloon over Titan or above Venus's clouds). Recommend yes; it is the same mechanism and costs nothing extra.
12. **Floaters above the posts** on worlds with air, to judge range past the horizon. Recommend yes, off by default, one more press on Range markers.
13. **Release**: FH1 to FH13 and FH15 for 4.0 in the three slices of section 12; players and Playbook (FH14) and Orac (FH16) as their threads reach them; under a sea and other eyes later. Recommend yes.
14. **A builder thread**: open "Build views and sounds from inside worlds" (Opus) on the new account to own FH8, FH10, FH11 and the join-up, with the other packages going to the threads named. Recommend yes, since the renderer thread already carries a long queue.

## 17A. Alex's rulings, 25 Nov 2026

12:15 UTC, in his words: "aids should be toggleable. uninterrupted view is better... but it would effectively be a HUD option enabled at planet level (and remembered) - so the GM will use it sometimes - the aids line at the top works fine - fold under hud. Will we get transitions as we drop lower - we will see ourselves get lower rather than jump? Any audio uploaded to the presentation/routing layer should be available under Your own..." and then "otherwise yup to your questions".

- **Yes to all 14 questions**, so each recommendation in section 17 is a ruling.
- **Aids**: off by default, folded under the HUD chip, remembered (section 10). This refines question 1.
- **Going down is seen, not jumped** (section 6).
- **"Your own…" lists the sounds loaded for the Playbook and Switchboard** (section 9).
- Earlier the same day, 12:10 UTC: the clip ruling (section 9, "Where the shipped clips come from").

## 18. Picking this up without the chat

- This note and `handover/status/from-here.md` are the whole brief. The design is accepted; nothing is built. Section 17A wins where the rest differs.
- The mockups are in project files `sse4/handover-screens/FH-screens/` (on Alex's PC under `C:\Development\sse4-project-files\` once copied). `from-here-mockup.html` opens in a browser; the address takes `#scene=venus|mars|titan|moon|dwarf|j05|j4|j20|j200|j1000`, `&expo=eye|lit|long`, `&card=0`, `&marks=0`, `&meter=0`, `&listen=1`, `&lamp=1`, `&theme=dark`, `&phone=1`, `&depth=science`, or `#sheet=1`.
- SSE3's version is public at `FrunkQ/star-system-generator`, files named in section 2. Read its comments for the reasoning; do not port its code.
- The existing code this builds on: `packages/renderer/src/gpu-surface.ts`, `surface-sky.ts`, `surface-patch.ts` (the standing view), `atmosphere.ts` and `crates/sse-light/src/air.rs`, `gas.rs` (air optics and light through gas), `crates/sse-weather/src/hooks.rs` (the column and sound cues), `packages/sound/` (the N1 engine).
