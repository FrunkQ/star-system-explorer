# SSE4 star census and "one body" plan (19 Nov 2026)

Asked by Alex 14:37 and 14:39: every known kind of star (plus hypotheticals, marked) with the physics to run it;
every kind should **appear by itself** when conditions and time allow (rare or hypothetical ones may need switching
on); and stars and planets must be **one kind of data object**, with star-like or planet-like physics applied
wherever the conditions hold.

Status marks: **Have** (built and tested), **Sketch** (present but rough), **Missing**.
Evidence tags for the catalogue: **observed**, **inferred** (strong indirect evidence), **hypothetical** (allowed by
physics, never seen; shown with that tag and off by default unless noted).

## 1. What exists today
- **Life line** (`sse-formation/src/lifeline.rs`, Hurley, Pols and Tout 2000 fits, 0.08 to 100 Msun, brown dwarfs
  from 0.0124 Msun). Phases: pre-main-sequence, main sequence, Hertzsprung gap, giant, brown dwarf, white dwarf,
  neutron star, black hole. Luminosity, radius, T_eff at any age; metallicity-dependent.
- **Ageing star** (`sse-time/src/star.rs`): Reimers winds over the giant phase, core mass, planetary nebula as the
  envelope thrown off, colour from the blackbody.
- **Remnants**: white dwarf mass from the initial-final mass relation, white dwarf cooling, neutron star or black
  hole by initial mass. No spin, field or kick.
- **Binaries (F7)** in `sse-time/src/binary.rs`: Roche-lobe overflow, common envelope, mergers, stable transfer.
  Reported as outcomes, **not fed back** into either star's life line.
- **Population III**: only as the "first stars" epoch in `sse-cosmos`. No Pop III star can be made.
- **Class atlas** (`packs/base-atlas`, 69 classes): planet and world schemes only, plus `brown-dwarf`. **No stellar
  scheme**: no spectral type, luminosity class or variability.

## 2. The catalogue against what exists

### Birth
| Type | Evidence | Status | Physics to add |
|---|---|---|---|
| Molecular cloud core, prestellar core | observed | Missing (cosmos clouds only) | Jeans/Bonnor-Ebert collapse from the cloud's T and density |
| Protostar (Class 0/I) | observed | Missing | accretion luminosity L = G M Ṁ / R, envelope; Hayashi track start |
| T Tauri (incl. classical, weak-line), FU Orionis outbursts | observed | Sketch (pre-MS phase, no type) | pre-MS tracks with disc accretion; FUor as accretion bursts |
| Herbig Ae/Be | observed | Missing | pre-MS of 2 to 8 Msun from the same tracks |
| Herbig-Haro objects and jets | observed | Missing | track O jets (already designed) |

### Substellar and the boundary
| Type | Evidence | Status | Physics to add |
|---|---|---|---|
| Brown dwarfs M, L, T, Y | observed | Sketch (phase only, no spectral class) | cooling tracks (Burrows et al. 1997 type fits); class from T_eff |
| Deuterium-burning planets and lithium test (13 and 65 Mjup) | observed | Missing | burning thresholds as conditions on any body |
| Sub-brown dwarfs, rogue planets | observed | Sketch (rogues exist as bodies) | same cooling physics as brown dwarfs |

### Main sequence
| Type | Evidence | Status | Physics to add |
|---|---|---|---|
| O, B, A, F, G, K, M dwarfs | observed | Have (physics), Missing (class) | spectral class and subclass from T_eff in the atlas |
| Red dwarf flare stars (UV Ceti) | observed | Missing | track O stellar activity |
| Subdwarfs (sdO, sdB, metal-poor sd) | observed | Sketch (low Z works, no class) | luminosity class VI; sdB from stripped helium cores (needs F7 feedback) |
| Chemically peculiar Ap/Bp, Am, HgMn, Be stars | observed | Missing | tags from rotation and field; Be decretion disc |
| Blue stragglers | observed | Missing | F7 merger or mass gain fed back into the life line |
| Population III (zero metal, 10 to 1000 Msun) | inferred | Missing | Z = 0 fits beyond Hurley's range (Marigo et al. 2001 type tracks), pair-instability fates |
| Very massive stars (over 150 Msun, R136a1) | observed | Missing (cap at 100) | extend tracks, strong winds |

### Evolved
| Type | Evidence | Status | Physics to add |
|---|---|---|---|
| Subgiants | observed | Have (Hertzsprung gap) | class IV |
| Red giant branch, red clump, horizontal branch (incl. blue HB, EHB) | observed | Sketch (one "giant" phase) | split core-helium burning out of "giant" (Hurley phases 3 to 6 exist in the paper) |
| AGB, thermal pulses, post-AGB, planetary nebula nuclei | observed | Sketch | AGB phase with dredge-up and superwind |
| Carbon stars, S stars, MS stars, barium stars | observed | Missing | C/O from dredge-up (AGB) or transfer (F7) |
| Supergiants: red, yellow, blue (Ia, Iab, Ib, Ia+ hypergiants) | observed | Sketch (giant phase) | luminosity class from L and log g |
| Luminous blue variables | observed | Missing | Humphreys-Davidson limit, eruptions as events |
| Wolf-Rayet WN, WC, WO | observed | Missing | stripped by winds or F7; surface abundances |
| R Coronae Borealis, extreme helium stars | observed | Missing | white dwarf mergers (F7 feedback) |

### Variables (as tags on any star, from its state)
| Type | Evidence | Status | Physics to add |
|---|---|---|---|
| Classical Cepheids, Type II Cepheids, RR Lyrae, delta Scuti, beta Cephei, SPB | observed | Missing | instability strip in the HR plane, period-luminosity and period-density relations |
| Miras, semiregulars, long-period variables | observed | Missing | AGB pulsation, P from radius and mass |
| Rotational and spotted (BY Dra, RS CVn) | observed | Missing | track O activity |
| Eclipsing binaries | observed | Missing | geometry from orbit and line of sight (sensing) |

### Remnants and endings
| Type | Evidence | Status | Physics to add |
|---|---|---|---|
| White dwarfs DA, DB, DO, DQ, DZ, DC; ZZ Ceti pulsators; magnetic WDs | observed | Sketch (mass, cooling, no type) | spectral type from T_eff and atmosphere; crystallisation |
| Neutron stars | observed | Sketch (mass only) | radius from an equation of state table, spin, field, kick |
| Pulsars: radio, millisecond (recycled), X-ray | observed | Missing | spin-down P Ṗ, death line, recycling by F7 accretion |
| Magnetars (SGR, AXP) | observed | Missing | field decay, giant flares as events |
| Stellar black holes | observed | Sketch (mass only) | spin, accretion disc when fed |
| Intermediate black holes | inferred | Missing | dense-cluster runaway or Pop III remnants (needs cluster stage) |
| Supernovae: II, Ib/c, Ia, pair-instability, electron-capture; hypernovae, GRBs, kilonovae | observed | Sketch (core collapse as a fate) | events with energy, ejecta, nucleosynthesis yields (feeds galaxy chemistry) |
| Supernova remnants, pulsar wind nebulae | observed | Missing | track O nebulae |

### Interacting binaries (F7 outcomes made visible)
| Type | Evidence | Status | Physics to add |
|---|---|---|---|
| Cataclysmic variables, classical and dwarf novae, recurrent novae, polars | observed | Missing | WD accretion rate, nova ignition mass |
| Low- and high-mass X-ray binaries, ultraluminous X-ray sources | observed | Missing | Eddington-limited accretion luminosity |
| Microquasars (SS 433) | observed | Missing | jets (track O) on accreting compact objects |
| Symbiotic stars, Algols, contact binaries (W UMa), luminous red novae | observed | Missing | F7 states fed back |
| Double degenerates, LISA sources, mergers | observed | Missing | gravitational-wave inspiral time (Peters 1964) |

### Galaxy scale (owner: kernel thread, G track)
| Type | Evidence | Status | Notes |
|---|---|---|---|
| Supermassive black holes, AGN, quasars, blazars, Seyferts | observed | Missing | grow with the galaxy (seed, accretion, mergers as G6 events); jets and brightness from accretion |
| Direct-collapse black holes | inferred | Missing | early-universe seed channel |
| Tidal disruption events, hypervelocity stars | observed | Missing | events near the SMBH |

### Hypothetical (tagged hypothetical; switch on to allow)
| Type | Evidence | How it would arise |
|---|---|---|
| Black dwarfs | hypothetical (universe too young) | white dwarf cooling past ~10^15 yr; appears with deep-time ageing, default on |
| Iron stars | hypothetical (far future) | cold-fusion/tunnelling over ~10^1500 yr; deep-time only |
| Frozen stars, blue dwarfs (red dwarfs' future) | hypothetical (simulated) | red dwarf evolution past 10^12 yr (Laughlin et al. 1997); default on in deep time |
| Quasi-stars | hypothetical | early-universe very massive envelope around a growing black hole |
| Dark stars (dark-matter annihilation powered) | hypothetical | Pop III in dense dark matter cusps |
| Supermassive stars | hypothetical | 10^4 to 10^5 Msun primordial collapse |
| Thorne-Zytkow objects | hypothetical (candidates) | neutron star engulfed by a supergiant via F7 common envelope |
| Quark stars, strange stars, hybrid stars | hypothetical | neutron star equation of state switch |
| Boson stars, gravastars, Planck stars, black stars | hypothetical | exotic physics packs, author-placed or switched on |
| Blitzars | hypothetical | supramassive neutron star collapsing as spin-down ends (FRB link) |
| Electroweak stars, preon stars | hypothetical | exotic; placeholder classes only |
| Q-stars, Population III "dark" remnants | hypothetical | exotic EOS packs |

## 3. Stars and bodies today: how far from one object
- **Facts and the chronicle: already one object.** Every thing is an address with a row of facts. Stars carry
  `body.mass`, `body.radius` like any body, plus a `star.*` family (T_eff, luminosity, phase, metallicity). Nothing in
  the fact store forbids `star.luminosity` on a planet. `body.kind` is a stored text fact ("star", "planet").
- **Addresses: role baked into the name.** Stars are `.../star:0`, worlds `.../body:N`. Addresses are identity and
  sit in saves and hashes, so they should **not** change; they just must stop meaning anything about physics.
- **Engines: split.** Formation makes stars in `stars.rs` from the life line (refused below 0.0124 Msun) and worlds
  from protoplanets. `SystemSpec` is one `star_mass` plus a `planets` list. Surface `Body` has no self-luminosity,
  and `Climate::Star` assumes one host. NAV assumes one central star. So a 20 Mjup object is either a "star" with no
  surface or a "planet" with no glow; neither gets deuterium burning or cooling heat. This is the V3 problem again,
  one layer down.
- **Importer**: V3's `roleHint == "star"` decides which facts a node gets.
- **Atlas**: classes exist only for worlds (plus `brown-dwarf`). Kind is not derived from classes.
- **UI**: about 15 places in `ui-contract` (`light.ts`, `scene.ts`, `derivation.ts`, `make.ts`, lenses) test
  `kind === "star"` to find the light source or the system's root.

### Proposal: one body, physics by condition
1. **A body is a body.** Every object has mass, composition (Z, C/O, water, heat elements), age and orbit.
2. **Physics modules attach by condition**, read from the body's own state, not its kind:
   - *self-luminosity*: fusion (H, D, Li, He...) when central conditions allow, else contraction and cooling (any
     mass: Jupiter's internal heat, brown dwarfs, white dwarfs, neutron stars all use it);
   - *surface, air, interior, weather*: whenever there is a reachable surface or an envelope cool enough to have one;
   - *degenerate remnant*, *compact object*, *accretion*: by density and mass.
   One mass axis from dust to 1000 Msun, with the boundaries (13 Mjup deuterium, 0.075 Msun hydrogen, 1.4 Msun
   Chandrasekhar, ~2.3 Msun neutron star limit) as pack data with sources.
3. **Kind is derived**, not stored: the class atlas gets stellar schemes (spectral type, luminosity class, remnant,
   variability, evidence) and "star", "brown dwarf", "planet" are classes it assigns. `body.kind` stays readable as a
   derived fact so old saves and the UI keep working.
4. **Light comes from any luminous body.** Climate sums the flux from every luminous body (binaries and a glowing
   giant planet's moons come free). The UI finds light sources by `luminosity > 0`, not by kind.
5. **Every type appears by itself** from formation, ageing and F7 feedback; hypotheticals sit behind a pack switch
   with the "hypothetical" evidence tag on GM views.

### What every body publishes for the renderer and lenses (Alex, 14:41)
Every body, star or planet, publishes from its own physics:
- **Emission by band**: a spectrum from radio to gamma (thermal blackbody or cooling, plus non-thermal parts:
  accretion discs as multi-temperature discs (Shakura and Sunyaev 1973), coronae and jets as power laws, pulsar and
  magnetar beams, aurorae, lightning), in the bands of the nebulae-and-light design, so a black hole's disc can be
  seen in X-ray or gamma through the light lenses.
- **Magnetic field**: dipole moment, tilt and offset to the spin axis, plus a geometry kind (dipole, multipole,
  open wind field as a Parker spiral, magnetar or pulsar). Planets from a dynamo rule (core size, rotation, heat
  flow), stars from rotation and convection, remnants from flux conservation. This feeds an improved version of V3's
  magnetic field overlay (field lines, magnetopause, belts) and NAV's radiation belts.
These land in B1 as body-level facts and ops; the renderer thread is told the ops when they exist.

### Multiples and what binaries leave behind (Alex, 14:50 and 14:54)
- **Binaries change type over time**: contact binaries, mass transfer, stars changing class, all fed back into each
  star's life line (B4). Alex confirmed one body should come early, or be accommodated until made: nothing new is
  built on the star and planet split.
- **Hierarchy is the native structure**: a close pair is one object (a barycentre node with its own local orbit),
  which can itself be a member of a wider pair, nested as deep as needed. This matches the nested-frames rule
  (local orbits per parent in f64). Generated and **hand-authored** multiples use the same structure.
- **Stability check that tags, never forbids**: each inner and outer pair is tested with the Mardling and Aarseth
  2001 criterion (outer periapsis against the critical ratio from mass ratio, eccentricity and inclination);
  authored configurations that fail are tagged "unstable" (with the time scale) on GM views, like any impossible
  edit. Circumbinary planets use Holman and Wiegert 1999 limits.
- **Binaries leave nebulae**: nova shells, common-envelope and bipolar planetary nebulae, luminous red nova
  remnants, supernova remnants from binary channels. Each is a dated object for track O (nebulae and light), with
  mass, expansion speed and composition from the event that made it.

### Size and saved-format impact
- **Physics (me)**: one life line across the mass axis (merge brown dwarf and giant planet cooling into it), F7
  feedback, the evolved and remnant phases above, multi-source climate. About 6 to 8 packages.
- **Kernel thread** (agreed 14:43): owns SMBHs, AGN, quasars and direct-collapse seeds (G3 grows the hole, G6
  dates mergers and active episodes; quasar luminosity and spectrum, seed channels and black-hole mergers are one
  G package after J4 to J7, publishing the nucleus as a body at the halo centre). For derived kind:
  - derive `kind` first in its own scheme pass, and no stellar scheme may read `kind` (the atlas refuses the cycle);
  - publish under the same `body.kind` field, winning over a stored derive-sourced value; a GM-authored kind stays
    authored and is tagged where physics disagrees, so a save's state hash does not move on load;
  - keep the field name so observe packs that list `kind` as obvious keep working;
  - keep today's words ("star", "planet", "moon", ...) as class labels, so goldens stay put;
  - "moon" is a role from the parent (orbit host is a non-star body), not physics.
  The kernel thread takes the atlas half once Alex has seen the plan.
- **UI thread** (agreed 14:42): one ui-contract helper giving "luminous" (colour, glow, light), "has a surface" (Body globe, Life) and "root" predicates, replacing the `kind === "star"` tests in ui-contract and in Body, Compare and Biosphere views; the UI thread takes the view side when the scene facts land; stellar classes in lenses
  and symbols. About 1 to 2 packages.
- **Formats and hashes**: addresses unchanged; facts only gain keys. Goldens **will** change where new physics adds
  facts (giant planets gaining luminosity, stars gaining classes). No saved format breaks if `body.kind` stays as a
  derived fact. Nothing that changes saved formats or hashes starts before Alex has seen this.

## 4. Proposed order
1. **Census to Alex** (this file) and agreement with the kernel and UI threads.
2. **B1 One body**: shared mass axis, self-luminosity by condition, multi-source light; atlas stellar schemes.
3. **B2 Evolved stars**: split HB/AGB/supergiants, carbon and S stars, Wolf-Rayet, LBV, variables by instability strip.
4. **B3 Remnants**: white dwarf types, neutron star EOS, pulsars, magnetars, black hole spin; supernova and nova events.
5. **B4 Binaries fed back**: blue stragglers, CVs, X-ray binaries, microquasars, TZOs, sdB, R CrB.
6. **B5 Birth and the first stars**: protostars, T Tauri and Herbig, Pop III and very massive stars, dark stars,
   quasi-stars, supermassive stars (with the G track's early universe).
7. **B6 Deep time and exotics**: black dwarfs, frozen stars, iron stars; exotic EOS pack (quark, boson, Planck...).

Interleaving: S5 landing sites and weather W1 to W3, W5 go first as already ordered (short and nearly done); B1
starts once the threads agree, since it touches everyone.

## 5. Sources (to verify, written from memory)
- Hurley, Pols and Tout 2000, MNRAS 315, 543, doi:10.1046/j.1365-8711.2000.03426.x
- Burrows et al. 1997, ApJ 491, 856, doi:10.1086/305002 (brown dwarf and giant planet cooling)
- Laughlin, Bodenheimer and Adams 1997, ApJ 482, 420, doi:10.1086/304125 (red dwarf futures)
- Peters 1964, Phys. Rev. 136, B1224, doi:10.1103/PhysRev.136.B1224 (inspiral)
- Thorne and Zytkow 1977, ApJ 212, 832, doi:10.1086/155109
- Begelman, Rossi and Armitage 2008, MNRAS 387, 1649, doi:10.1111/j.1365-2966.2008.13344.x (quasi-stars)
- Spolyar, Freese and Gondolo 2008, PRL 100, 051101, doi:10.1103/PhysRevLett.100.051101 (dark stars)
- Falcke and Rezzolla 2014, A&A 562, A137, doi:10.1051/0004-6361/201321996 (blitzars)
- Mardling and Aarseth 2001, MNRAS 321, 398, doi:10.1046/j.1365-8711.2001.03974.x (triple stability)
- Holman and Wiegert 1999, AJ 117, 621, doi:10.1086/300695 (planets in binaries)
- Shakura and Sunyaev 1973, A&A 24, 337 (accretion discs)
- Dyson 1979, Rev. Mod. Phys. 51, 447, doi:10.1103/RevModPhys.51.447 (iron stars)
