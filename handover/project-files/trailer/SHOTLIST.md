# SSE4 trailer shot list

**Mode (Alex, 21 Nov): gather clips as features are finished; no trailer gets cut until the app is more feature complete.**
Every clip now carries a caption card saying what is on screen, plus a matching `<id>.srt` (film with `--no-captions` for clean plates).

Rig: `scripts/trailer/` in the repo (README there). Clips: `clips/<id>.mp4` with a still and a JSON sidecar.
Status: **captured** (usable now), **weak** (filmed, but not trailer quality yet; reason given), **needs GPU**
(must be filmed on a real GPU), **not built** (no footage until the feature lands; never faked).

Last full pass: 23 Nov 2026 22:00 UTC, main at b1581f38 (all 25 shots pass after fixing 07 and 24 for the one set of view controls: Scale and Band; 09 stays on hold)

## Filmed

| Shot | What it shows | Status | Notes |
|---|---|---|---|
| 00-start-sheet | The start sheet (U21), its verbs, then Open the Solar System: the catalogued Sun and planets on the chart | captured | Filmed 22 Nov. Good intro opener |
| 01-chart-open | Chart room, hover peek, card grows from Tarn | captured | Opens on the "Fixture data" stamp; re-shoot once the start sheet (U13+) lands |
| 02-edit-and-why | Type a new mass, ask why on surface temperature, Science tab, follow the trail | captured | Strong "why" shot; best when crops on the Why panel |
| 03-drag-orbit | Drag Tarn outward, temperature drops, undo | captured | |
| 04-time-play | Ruler to a year, step by the week, play: worlds move | captured | |
| 05-look-ahead | Age Kestrel on a branch to its death | weak | Panel is small and low on screen; the chart changes little. Better once 12.2 "one clock" lands, or with an aged Sun (Solar System) |
| 06-system-3d | 2D to 3D, into Vault's moons | captured (GPU) | Filmed 21 Nov on AC-ProArt, live site, WebGPU: orbits, rings and labels drawn, 19.5 fps. Take in clips/gpu/ |
| 07-sizes | True-scale lineup, add Jupiter and a neutron star, then Scale: Readable (log) | captured | Re-shot 23 Nov: log scale is now Readable in the view controls |
| 08-cutaway | Tarn cut in half, shells with reasons | captured | |
| 09-globe-photo | Globe of the V3 sample planet, turned by drag | hold | Alex 21 Nov: no clouds, air or ice caps; the V3 sample is not a modelled Earth (never caption it Earth); the 96% granite surface looks wrong (with the physics stream); cutaway missing; the drag turns it to open ocean. Re-shoot when these land |
| 10-star-xray | The Sun drawn from physics, then in X-rays | captured | Striking; a likely intro beat. GPU take only 8 fps, may look choppy |
| 11-galaxy | Up the place line to the galaxy: rings, supernova layer, ink an equal-mass merger at 8 Gyr | captured | Re-shot 22 Nov through the place line (U24). Selects change values without the pointer visiting them |
| 12-cosmic-web | Straight up the place line to the cosmic web, from 1 Gyr played to today | captured | Early frames are grey noise by nature; use the second half |
| 13-route | Three courses to Vault, pick the textbook transfer | captured | |
| 14-sky-from-here | Window view from Tarn | weak | Mostly an empty sky with one dot; wait for the Sky from here instrument and telescope views |
| 15-stories | Story finder hits with evidence | captured | |
| 16-grow-system | Grow a new system from seed "trailer-1" | captured | Result appears at once; no formation animation exists yet |
| 17-night-chart | Switch to Night chart, pick Vault | captured | |
| 19-change-the-physics | Build desk, Engine depth, why? into the flow diagram, a refused edit, a validated one with effects, adopt | captured | The core of the physics video (D). Filmed 21 Nov on main be607f0 |
| 20-climate-why | Climate Why? on the sample planet: six belts of temperature and rain, the kernel's reasons and assumptions | captured | U17, filmed 21 Nov evening on main fe602c0. Beat for the physics video (D). Globe in frame is blank in the cloud; re-film on a GPU when the globe is ready |
| 21-how-it-formed | Grow a system from seed trailer-1, open "How it formed": dated events from disc to white dwarfs | captured | For the generation explainer (F). A text list, not a picture. Caption card is hidden under the modal dialog (the .srt has it) |
| 22-formation-replay | Grow seed "first light", Watch it form: nine stages from the run's own record (disc and lines, embryos, the violent phase, what is left), then Play | captured | U19, filmed 22 Nov. The heart of the generation explainer (F) and an intro beat. Dialog is modal, so the caption card sits under it in places (the .srt has all captions) |
| 23-reference-elements | Workshop, Reference: the constant G with its source, then the periodic table coloured by origin, Gold and Iron cards | captured | MP10, filmed 22 Nov. Physics video (D) and features |
| 24-mag-earth | Solar System, catalogued Earth, Body screen, Mag: measured dipole, the wind's bubble, legend whys | captured (cloud) | MF14b, re-shot 23 Nov: Mag is now a Band in the view controls. The globe now has oceans and a cutaway, but its continents are generated, and the caption says so. Re-film on a GPU later; Mag replaces the picture, so the globe matters little |
| 18-players | Players' page, scanned data fills in | captured | Could pair with a split screen against the GM view |

Rough cut of the captured shots: `rough-cut-2026-11-21.mp4` (110 s, no music or titles), only to judge pace.

## Built but not yet filmed (next pass)

- Earth through time (PR 140, renderer): Body screen, Earth, "Through time"; layers land/sea, plates, volcanism, temperature, rain, ice, runoff; 100 Myr to today in 45 keyframes. Core of the generation explainer (F, chapter 7). Wait for the chart room's timeline (log scrubber, play, event marks, why card), then film on AC-ProArt's GPU. Check first whether this Earth is a modelled Earth before captioning it "Earth".

- Copy and paste (U20, Build desk), view controls on every diagram (U25), Voyages screen (V8 part 3), Units and Constants pages: filmable, next pass.
- Earth through time: the controls (U22) have landed, but the catalogued Earth's surface is generated, not modelled, so it is not filmed as "Earth". Wait for a modelled Earth, or film it on a grown world with an honest name (GPU take on AC-ProArt).

| Shot idea | Feature | Notes |
|---|---|---|
| Player themes: same system as green phosphor, glass cockpit, paper | V2 themes | Pitch line 8 and 42 |
| Physics flow editor: open a flow from a why, change a parameter, see the diff | U3 | The "change the physics" beat |
| Life screen: epochs, food web | U7 | Example worlds until chart worlds carry habitats |
| Measure on the chart | V14 | |
| Light switch on the chart (Eye, Camera, SHO) | O1 | |
| Sound on (orbit bed) | N1 | Needs audio capture; the rig films picture only today |
| Ask Tars, Orac draft | A2 to A4 | Needs a key, or Tars from passages with no key |
| Share with players: two screens, one universe | V4, V10 | Two browser contexts side by side |
| Traveller hex map and world codes | X3 | Views written, not mounted in the app yet |
| Weather report and storms | W1 to W6 | No screen yet |

- Land anywhere (SL1, 23 Nov; Alex 23 Nov: wait for an honest world, then film on the GPU): map with landmarks, descent verdict and dangers, flown descent with the kernel's log, walk on the ground. Held: it only works on worlds with a kernel surface (the V3 sample or the catalogued Earth), and both show invented continent and landmark names under the name "Earth"; the catalogued Earth's Land screen also says it has no air (fixed by the renderer's PR 328; Land also opens on the catalogued Mars and Venus, so re-check next run). In the cloud the ground draws on WebGL as a flat dark plane. Film on AC-ProArt's GPU once it runs on an honestly named world. Core of the scale journey (C).
- True on the system chart with a living star (V10a, V10c), bands through True (V10d), Mag at system scale (V10e), galaxy in 3D and True with collision marks (V9t, GH9): GPU takes for AC-ProArt.

## Not built yet (wait, never fake)

Start sheet and rotating lines, tours, desks and depth (top-level redesign after U13); "why is this here" on a
crater, coal seam or lava tube (S14 to S16 surface detail); spectrum of any object; survey from afar; descent and
landing; programs and autopilots; nebulae on the charts; exotic objects in bands (R5b);
Explorers library in the app; one universe clock with "make this system this old".

Known on screen in every take filmed 21 Nov: the Explore / Build / Run switch shows "uild" (clipped B). Reported; re-shoot after the fix.

New videos (21 Nov): scale journey (C), physics system (D) and RPG series (E); their shots and status are in STORYBOARDS.md.

New video (22 Nov): "Teach it new physics" (G), Orac brings in a paper on coloured white dwarfs and saves it
as a pack. Film near the end; almost every shot waits on track ND. Shots and status are in STORYBOARDS.md
section G; the plan is `/mnt/project-files/sse4/SSE4-new-physics-demo.md`. Only shot 1 (a plain white dwarf
in an aged system, with its Why?) is filmable today.

Video G (Teach it new physics) accepted by Alex 22 Nov; filming it is ND10, owned by this thread, after ND1 to ND9 land. On-screen claim: "no new code for this paper" (blocks are not code).
